//! Win32 dashboard over the shared runner; observed state and staged intent stay separate.
use hyper_gpu_support::{
    credentials,
    gui_model::{Inventory, View, allocation_text, save_configuration},
    model::{Configuration, Power, Target},
    runner::{self, Operation},
    workflow::Plan,
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{self, HFONT, LOGFONTW},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::*, HiDpi::*, Input::KeyboardAndMouse::EnableWindow, WindowsAndMessaging::*,
        },
    },
    core::{PCWSTR, PWSTR, w},
};
const LIST: usize = 101;
const TOGGLE: usize = 103;
const APPLY: usize = 104;
const REFRESH: usize = 105;
const DETAILS: usize = 106;
const DISCARD: usize = 107;
const VERIFY: usize = 108;
const REAPPLY: usize = 112;
const SAVE: usize = 109;
const STORE: usize = 110;
const FORGET: usize = 111;
const PAGES: [&str; 4] = [
    "Virtual Machines",
    "System Information",
    "Settings",
    "About",
];
thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; static RENDERING: Cell<bool> = const { Cell::new(false) }; }
#[derive(Clone)]
enum Work {
    Discover,
    Preview(Target),
    Apply(Target),
    Verify,
}
struct State {
    view: View,
    path: PathBuf,
    source: String,
    window: HWND,
    nav: Vec<HWND>,
    title: HWND,
    subtitle: HWND,
    refresh: HWND,
    list: HWND,
    info: HWND,
    toggle: HWND,
    details: HWND,
    verify: HWND,
    reapply: HWND,
    discard: HWND,
    apply: HWND,
    save: HWND,
    store: HWND,
    forget: HWND,
    page_text: HWND,
    status: HWND,
    progress: HWND,
    page: usize,
    fonts: Vec<HFONT>,
    work: Option<Work>,
    receiver: Option<Receiver<Result<serde_json::Value, String>>>,
    modal: bool,
    close_requested: bool,
    notice: String,
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
#[allow(unsafe_code)]
fn send(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: GUI-owned HWNDs and message-specific buffers valid through each synchronous call.
    unsafe { SendMessageW(h, m, Some(w), Some(l)) }
}
#[allow(unsafe_code)]
pub(super) fn error(text: &str) {
    let text = wide(text);
    // SAFETY: live terminated text and no borrowed window owner.
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Hyper GPU Support"),
            MB_OK | MB_ICONERROR,
        );
    }
}
#[allow(unsafe_code)]
fn caption(h: HWND, text: &str) {
    let text = wide(text);
    // SAFETY: live child HWND; Windows copies the terminated text.
    unsafe {
        let _ = SetWindowTextW(h, PCWSTR(text.as_ptr()));
    }
}
#[allow(unsafe_code)]
fn control(
    parent: HWND,
    class: PCWSTR,
    text: &str,
    id: usize,
    style: WINDOW_STYLE,
) -> Result<HWND, String> {
    let text = wide(text);
    // SAFETY: fixed native class, live parent/string, integral child-control ID.
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            PCWSTR(text.as_ptr()),
            WS_CHILD | WS_VISIBLE | style,
            0,
            0,
            1,
            1,
            Some(parent),
            Some(HMENU(id as *mut _)),
            None,
            None,
        )
    }
    .map_err(|e| e.to_string())
}
fn busy(s: &State) -> bool {
    s.work.is_some() || s.modal
}
#[allow(unsafe_code)]
pub(super) fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "--config" {
        return Err("Usage: hyper-gpu-gui --config FILE\nUse runtime schema 2 and administrator hyper-gpu-support install --config FILE for enrollment.".into());
    }
    let path = PathBuf::from(&args[1]);
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config = Configuration::parse(&source)?;
    // SAFETY: DPI configuration precedes all HWNDs; fixed common-control initialization structure.
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        if !InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_LISTVIEW_CLASSES | ICC_PROGRESS_CLASS,
        })
        .as_bool()
        {
            return Err("Native controls initialization failed".into());
        }
    }
    // SAFETY: borrowed process module, GUI-thread callback and static window class strings.
    let instance = HINSTANCE(
        unsafe { GetModuleHandleW(None) }
            .map_err(|e| e.to_string())?
            .0,
    );
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: w!("HyperGpuSupportProduct"),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.map_err(|e| e.to_string())?,
        hbrBackground: Gdi::HBRUSH((Gdi::COLOR_WINDOW.0 + 1) as *mut _),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err("Native window registration failed".into());
    }
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class.lpszClassName,
            w!("Hyper GPU Support"),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1360,
            900,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .map_err(|e| e.to_string())?;
    let mut nav = Vec::new();
    for (i, title) in PAGES.iter().enumerate() {
        nav.push(control(
            window,
            w!("BUTTON"),
            title,
            200 + i,
            WS_TABSTOP
                | WINDOW_STYLE(BS_AUTORADIOBUTTON as u32 | BS_PUSHLIKE as u32)
                | if i == 0 { WS_GROUP } else { WINDOW_STYLE(0) },
        )?);
    }
    let title = control(window, w!("STATIC"), PAGES[0], 0, WINDOW_STYLE(0))?;
    let subtitle = control(
        window,
        w!("STATIC"),
        "Manage GPU support and related settings for your Hyper-V virtual machines.",
        0,
        WINDOW_STYLE(0),
    )?;
    let refresh = control(window, w!("BUTTON"), "Refresh", REFRESH, WS_TABSTOP)?;
    let list = control(
        window,
        WC_LISTVIEWW,
        "Virtual machines and observed GPU support",
        LIST,
        WS_TABSTOP | WS_BORDER | WINDOW_STYLE(LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS),
    )?;
    send(
        list,
        LVM_SETEXTENDEDLISTVIEWSTYLE,
        WPARAM(0),
        LPARAM((LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER | LVS_EX_GRIDLINES) as isize),
    );
    for (i, name) in [
        "Virtual Machine",
        "Status",
        "GPU Support",
        "GPU Memory Allocation",
        "Details",
    ]
    .iter()
    .enumerate()
    {
        let mut text = wide(name);
        let column = LVCOLUMNW {
            mask: LVCF_TEXT | LVCF_WIDTH,
            cx: 180,
            pszText: PWSTR(text.as_mut_ptr()),
            ..Default::default()
        };
        send(
            list,
            LVM_INSERTCOLUMNW,
            WPARAM(i),
            LPARAM((&column as *const LVCOLUMNW) as isize),
        );
    }
    let info = control(
        window,
        w!("STATIC"),
        "GPU Support\nEnable prepares the current signed NVIDIA driver, attaches the enrolled GPU and checks graphics. Disable retains driver files. Attachment and the last graphics check are separate. Concurrent sharing is not yet qualified.",
        0,
        WS_BORDER,
    )?;
    let toggle = control(
        window,
        w!("BUTTON"),
        "Enable GPU support for selected VM",
        TOGGLE,
        WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
    )?;
    let details = control(window, w!("BUTTON"), "View Details", DETAILS, WS_TABSTOP)?;
    let verify = control(window, w!("BUTTON"), "Verify Graphics", VERIFY, WS_TABSTOP)?;
    let reapply = control(
        window,
        w!("BUTTON"),
        "Reapply / Update",
        REAPPLY,
        WS_TABSTOP,
    )?;
    let discard = control(window, w!("BUTTON"), "Discard Changes", DISCARD, WS_TABSTOP)?;
    let apply = control(
        window,
        w!("BUTTON"),
        "Apply Changes",
        APPLY,
        WS_TABSTOP | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
    )?;
    let save = control(
        window,
        w!("BUTTON"),
        "Retry Configuration Save",
        SAVE,
        WS_TABSTOP,
    )?;
    let store = control(
        window,
        w!("BUTTON"),
        "Store Guest Credentials",
        STORE,
        WS_TABSTOP,
    )?;
    let forget = control(
        window,
        w!("BUTTON"),
        "Forget Guest Credentials",
        FORGET,
        WS_TABSTOP,
    )?;
    let page_text = control(
        window,
        w!("EDIT"),
        "",
        0,
        WS_TABSTOP
            | WS_BORDER
            | WS_VSCROLL
            | WINDOW_STYLE(ES_MULTILINE as u32 | ES_READONLY as u32),
    )?;
    let status = control(
        window,
        w!("STATIC"),
        "Reading provider state...",
        0,
        WINDOW_STYLE(0),
    )?;
    let progress = control(
        window,
        PROGRESS_CLASSW,
        "Operation progress",
        0,
        WINDOW_STYLE(PBS_MARQUEE),
    )?;
    STATE.with(|c| {
        *c.borrow_mut() = Some(State {
            view: View::new(config),
            path,
            source,
            window,
            nav,
            title,
            subtitle,
            refresh,
            list,
            info,
            toggle,
            details,
            verify,
            reapply,
            discard,
            apply,
            save,
            store,
            forget,
            page_text,
            status,
            progress,
            page: 0,
            fonts: Vec::new(),
            work: None,
            receiver: None,
            modal: false,
            close_requested: false,
            notice: "Reading current provider state...".into(),
        })
    });
    fonts();
    layout();
    render();
    discover();
    // SAFETY: GUI-owned window, bounded result-pump timer, owning thread's message loop.
    unsafe {
        SetTimer(Some(window), 1, 150, None);
        let _ = ShowWindow(window, SW_SHOW);
    }
    let mut msg = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) }.0;
        if result == 0 {
            break;
        }
        if result == -1 {
            return Err("Windows message retrieval failed".into());
        }
        if !unsafe { IsDialogMessageW(window, &msg) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
    STATE.with(|c| {
        if let Some(s) = c.borrow_mut().take() {
            for font in s.fonts {
                unsafe {
                    let _ = Gdi::DeleteObject(font.into());
                }
            }
        }
    });
    Ok(())
}
#[allow(unsafe_code)]
fn fonts() {
    STATE.with(|c| {
        let mut b = c.borrow_mut();
        let Some(s) = b.as_mut() else {
            return;
        };
        // SAFETY: GUI-owned HWND and font descriptor; owned fonts outlive their control use.
        let dpi = unsafe { GetDpiForWindow(s.window) }.max(96) as i32;
        let mut next: Vec<HFONT> = Vec::new();
        for (size, weight) in [(16, 400), (27, 600)] {
            let mut spec = LOGFONTW {
                lfHeight: -(size * dpi / 96),
                lfWeight: weight,
                ..Default::default()
            };
            for (d, v) in spec.lfFaceName.iter_mut().zip(wide("Segoe UI")) {
                *d = v;
            }
            let font = unsafe { Gdi::CreateFontIndirectW(&spec) };
            if font.0.is_null() {
                for f in next {
                    unsafe {
                        let _ = Gdi::DeleteObject(f.into());
                    }
                }
                return;
            }
            next.push(font);
        }
        for h in [
            s.subtitle,
            s.refresh,
            s.list,
            s.info,
            s.toggle,
            s.details,
            s.verify,
            s.reapply,
            s.discard,
            s.apply,
            s.save,
            s.store,
            s.forget,
            s.page_text,
            s.status,
        ]
        .into_iter()
        .chain(s.nav.iter().copied())
        {
            send(h, WM_SETFONT, WPARAM(next[0].0 as usize), LPARAM(1));
        }
        send(s.title, WM_SETFONT, WPARAM(next[1].0 as usize), LPARAM(1));
        for old in s.fonts.drain(..) {
            unsafe {
                let _ = Gdi::DeleteObject(old.into());
            }
        }
        s.fonts = next;
    });
}
#[allow(unsafe_code)]
fn layout() {
    STATE.with(|c| {
        let b = c.borrow();
        let Some(s) = b.as_ref() else {
            return;
        };
        let mut r = RECT::default();
        // SAFETY: GUI-owned HWNDs, output rectangle and dimensions derived from client/DPI.
        if unsafe { GetClientRect(s.window, &mut r) }.is_err() {
            return;
        }
        let dpi = unsafe { GetDpiForWindow(s.window) }.max(96) as i32;
        let scale = |v: i32| v * dpi / 96;
        let w = r.right * 96 / dpi;
        let h = r.bottom * 96 / dpi;
        let side = if w >= 1300 { 280 } else { 220 };
        let x = side + 28;
        let width = (w - x - 28).max(400);
        let place = |hwnd, x, y, width: i32, height: i32| unsafe {
            let _ = MoveWindow(
                hwnd,
                scale(x),
                scale(y),
                scale(width.max(1)),
                scale(height.max(1)),
                true,
            );
        };
        for (i, nav) in s.nav.iter().enumerate() {
            place(*nav, 18, 92 + i as i32 * 55, side - 36, 42);
        }
        place(s.title, x, 26, width - 140, 40);
        place(s.subtitle, x, 74, width, 50);
        place(s.refresh, x + width - 118, 28, 118, 36);
        let table_height = (h - 360).max(160);
        place(s.list, x, 132, width, table_height);
        for (i, portion) in [28, 12, 26, 22, 12].iter().enumerate() {
            send(
                s.list,
                LVM_SETCOLUMNWIDTH,
                WPARAM(i),
                LPARAM(scale(width * portion / 100) as isize),
            );
        }
        let bottom = 132 + table_height;
        place(s.toggle, x, bottom + 16, width - 490, 36);
        place(s.reapply, x + width - 475, bottom + 16, 165, 36);
        place(s.details, x + width - 300, bottom + 16, 142, 36);
        place(s.verify, x + width - 148, bottom + 16, 148, 36);
        place(s.info, x, bottom + 64, width, 96);
        place(s.discard, x + width - 344, h - 74, 164, 40);
        place(s.apply, x + width - 164, h - 74, 164, 40);
        place(s.status, x, h - 28, width, 24);
        place(s.progress, x, h - 82, (width - 374).max(40), 6);
        place(s.page_text, x, 132, width, (h - 260).max(160));
        place(s.store, x, h - 114, 218, 36);
        place(s.forget, x + 230, h - 114, 218, 36);
        place(s.save, x + width - 238, h - 74, 238, 40);
    });
}
fn selected(s: &State) -> Option<Target> {
    let id = s.view.selected.as_ref()?;
    s.view
        .configuration
        .targets
        .iter()
        .find(|t| &t.vm_id == id)
        .cloned()
}
fn page_text(s: &State) -> String {
    match s.page {
        1 => s.view.inventory.as_ref().map_or_else(|| "GPU inventory unavailable. Refresh to read provider state.".into(), |i| {
            let mut text = "Partitionable GPUs (inventory, not a guest graphics check)\r\n\r\n".to_string();
            for g in &i.discovery.gpus { text.push_str(&format!("{}\r\nDriver: {}\r\nPreparation supported: {}\r\nProvider VRAM range: {} / {} / {} (raw units)\r\n\r\n", g.name, g.driver_version, g.preparation_supported, g.vram.minimum, g.vram.optimal, g.vram.maximum)); }
            if i.discovery.gpus.is_empty() { text.push_str("Successful discovery returned no partitionable GPUs."); } text
        }),
        2 => format!("Configuration: {}\r\n\r\nRuntime schema 2 expresses desired pairs. Protected administrator enrollment independently authorizes them.\r\n\r\nSetup in an administrator console:\r\nhyper-gpu-support inventory\r\nhyper-gpu-support install --config \"{}\"\r\n\r\nChange GPU pairs in configuration and re-enroll. Ordinary dashboard launch needs no elevation.\r\n\r\nCredential buttons use the selected VM and this user's Windows vault. Storage is explicit; passwords never enter configuration.\r\n\r\nConfiguration save pending: {}\r\nAfter a successful VM operation, fix the file and retry SAVE; do not replay Apply.", s.path.display(), s.path.display(), s.view.unsaved),
        3 => format!("Hyper GPU Support {}\r\n\r\nNative Windows GUI and Rust core for existing Hyper-V Generation 2 VMs. NVIDIA preparation first.\r\n\r\nThe CLI and GUI use the same protected runner/workflow. Allocation is in provider units; no GiB or enforcement claim. Simultaneous sharing is unqualified.\r\n\r\nDocumentation: README.md, docs/ROADMAP.md\r\nSee repository third-party notices for dependency provenance.", env!("CARGO_PKG_VERSION")),
        _ => String::new(),
    }
}
#[allow(unsafe_code)]
fn render() {
    RENDERING.with(|f| f.set(true));
    STATE.with(|c| {
        let b = c.borrow();
        let Some(s) = b.as_ref() else {
            return;
        };
        let active = busy(s);
        let dashboard = s.page == 0;
        caption(s.title, PAGES[s.page]);
        caption(
            s.subtitle,
            if dashboard {
                "Manage GPU support and related settings for your Hyper-V virtual machines."
            } else {
                "Current product facts, configuration and supported actions."
            },
        );
        for (i, nav) in s.nav.iter().enumerate() {
            send(
                *nav,
                BM_SETCHECK,
                WPARAM(usize::from(s.page == i)),
                LPARAM(0),
            );
        }
        // SAFETY: all HWNDs belong to this UI thread; visibility/enabled state is presentation only.
        unsafe {
            for h in [
                s.list, s.toggle, s.details, s.verify, s.reapply, s.info, s.discard, s.apply,
            ] {
                let _ = ShowWindow(h, if dashboard { SW_SHOW } else { SW_HIDE });
            }
            let _ = ShowWindow(s.page_text, if dashboard { SW_HIDE } else { SW_SHOW });
            for h in [s.save, s.store, s.forget] {
                let _ = ShowWindow(h, if s.page == 2 { SW_SHOW } else { SW_HIDE });
            }
            let _ = ShowWindow(s.progress, if active { SW_SHOW } else { SW_HIDE });
        }
        send(
            s.progress,
            PBM_SETMARQUEE,
            WPARAM(usize::from(active)),
            LPARAM(35),
        );
        caption(s.page_text, &page_text(s));
        caption(s.status, &s.notice);
        send(s.list, LVM_DELETEALLITEMS, WPARAM(0), LPARAM(0));
        if let Some(i) = &s.view.inventory {
            for (row, vm) in i.discovery.vms.iter().enumerate() {
                let power = match vm.power {
                    Power::Off => "Off",
                    Power::Running => "Running",
                    Power::Other(_) => "Saved / transition",
                };
                for (column, value) in [
                    format!("{} (Gen {})", vm.name, vm.generation),
                    power.into(),
                    s.view.support_text(vm),
                    allocation_text(vm).into(),
                    "View Details".into(),
                ]
                .iter()
                .enumerate()
                {
                    let mut text = wide(value);
                    let item = LVITEMW {
                        mask: LVIF_TEXT,
                        iItem: row as i32,
                        iSubItem: column as i32,
                        pszText: PWSTR(text.as_mut_ptr()),
                        ..Default::default()
                    };
                    send(
                        s.list,
                        if column == 0 {
                            LVM_INSERTITEMW
                        } else {
                            LVM_SETITEMW
                        },
                        WPARAM(0),
                        LPARAM((&item as *const LVITEMW) as isize),
                    );
                }
                if s.view.selected.as_ref() == Some(&vm.vm_id) {
                    let item = LVITEMW {
                        stateMask: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
                        state: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
                        ..Default::default()
                    };
                    send(
                        s.list,
                        LVM_SETITEMSTATE,
                        WPARAM(row),
                        LPARAM((&item as *const LVITEMW) as isize),
                    );
                }
            }
        }
        let target = selected(s);
        let enabled = s
            .view
            .draft
            .as_ref()
            .filter(|t| Some(&t.vm_id) == s.view.selected.as_ref())
            .map(|t| t.enabled)
            .or_else(|| {
                target.as_ref().and_then(|t| {
                    s.view
                        .vm(&t.vm_id)
                        .map(|v| v.gpus == vec![t.gpu_interface.clone()])
                })
            })
            .unwrap_or(false);
        send(
            s.toggle,
            BM_SETCHECK,
            WPARAM(usize::from(enabled)),
            LPARAM(0),
        );
        let editable = target.as_ref().is_some_and(|t| {
            let mut t = t.clone();
            t.enabled = !enabled;
            s.view.eligibility(&t).is_ok()
        }) && !s.view.needs_readback
            && !s.view.unsaved;
        let verifiable = target
            .as_ref()
            .is_some_and(|t| s.view.verification(t).is_ok());
        unsafe {
            for h in [s.list, s.refresh].into_iter().chain(s.nav.iter().copied()) {
                let _ = EnableWindow(h, !active);
            }
            for (h, enabled) in [
                (s.toggle, editable),
                (s.reapply, editable),
                (s.details, s.view.selected.is_some()),
                (s.verify, verifiable && !s.view.needs_readback),
                (s.discard, s.view.draft.is_some()),
                (s.apply, s.view.apply_target().is_ok()),
                (s.save, s.view.unsaved),
                (s.store, target.is_some()),
                (s.forget, target.is_some()),
            ] {
                let _ = EnableWindow(h, !active && enabled);
            }
        }
    });
    RENDERING.with(|f| f.set(false));
}
fn background(
    work: Work,
    task: impl FnOnce() -> Result<serde_json::Value, String> + Send + 'static,
) {
    STATE.with(|c| {
        let mut b = c.borrow_mut();
        let Some(s) = b.as_mut() else {
            return;
        };
        if busy(s) {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        s.receiver = Some(receiver);
        s.work = Some(work);
        s.notice =
            "Operation in progress; wait for the final response. No streamed stage is available."
                .into();
        std::thread::spawn(move || {
            let _ = sender.send(task());
        });
    });
    render();
}
fn discover() {
    background(Work::Discover, || {
        runner::submit(runner::request(Operation::Discover, None, None))
    });
}
fn notice(text: impl Into<String>) {
    STATE.with(|c| {
        if let Some(s) = c.borrow_mut().as_mut() {
            s.notice = text.into();
        }
    });
    render();
}
fn modal(value: bool) {
    STATE.with(|c| {
        if let Some(s) = c.borrow_mut().as_mut() {
            s.modal = value;
        }
    });
    render();
}
#[allow(unsafe_code)]
fn confirm(parent: HWND, title: &str, text: &str) -> bool {
    let title = wide(title);
    let text = wide(text);
    // SAFETY: GUI-owned modal parent and live terminated strings; rejection is the default.
    (unsafe {
        MessageBoxW(
            Some(parent),
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_YESNO | MB_ICONINFORMATION | MB_DEFBUTTON2,
        )
    }) == IDYES
}
fn credential(t: &Target, parent: HWND) -> Result<credentials::Credential, String> {
    credentials::read(&t.vm_id)?.map_or_else(|| credentials::prompt(&t.vm_id, parent), Ok)
}
fn poll() {
    let completed = STATE.with(|c| {
        let mut b = c.borrow_mut(); let s = b.as_mut()?;
        let result = match s.receiver.as_ref()?.try_recv() { Ok(v) => v, Err(TryRecvError::Empty) => return None, Err(TryRecvError::Disconnected) => Err("Lost background response; native work may have completed. Read back before retrying.".into()) };
        s.receiver = None; Some((s.work.take()?, result, s.window))
    });
    let Some((work, result, parent)) = completed else {
        return;
    };
    match work {
        Work::Discover => match result.and_then(|v| serde_json::from_value::<Inventory>(v).map_err(|e| e.to_string())) {
            Ok(i) => STATE.with(|c| { if let Some(s) = c.borrow_mut().as_mut() { s.view.refresh(i); s.notice = if s.view.unsaved { "Operation succeeded. Configuration save pending; use Settings to retry save." } else { "Observed provider state; graphics is not freshly checked. Stage a toggle or open Details." }.into(); } }),
            Err(e) => { STATE.with(|c| { if let Some(s) = c.borrow_mut().as_mut() { s.view.needs_readback = true; } }); notice(format!("Historical inventory; actions disabled. Refresh failed: {e}. See Settings for runner guidance.")); },
        },
        Work::Preview(t) => {
            modal(true);
            match result.and_then(|v| serde_json::from_value::<Plan>(v).map_err(|e| e.to_string())) {
                Ok(p) if p.desired == t => {
                    if confirm(parent, "Preview GPU changes", &format!("VM: {}\n\n{}\n\nApply this staged target?", p.observed.name, p.preview.summary.join("\n"))) {
                        let supplied = if t.enabled { credential(&t, parent).map(Some) } else { Ok(None) }; modal(false);
                        match supplied { Ok(credential) => { let copy = t.clone(); background(Work::Apply(t), move || runner::submit(runner::request(Operation::Apply, Some(copy), credential))); }, Err(e) => notice(format!("Credential entry failed/cancelled; no apply requested: {e}")) }
                    } else { modal(false); notice("Preview cancelled; staged intent retained."); }
                }
                Ok(_) => { modal(false); notice("Preview target mismatch; no apply requested. Refresh before retrying."); }
                Err(e) => { modal(false); notice(format!("Preview failed; no apply requested: {e}")); }
            }
        }
        Work::Apply(t) => {
            match result {
                Ok(_) => {
                    let saved = STATE.with(|c| {
                        let mut b = c.borrow_mut(); let s = b.as_mut().ok_or("View closed")?; s.view.applied(&t)?;
                        let text = save_configuration(&s.path, &s.source, &s.view.configuration)?; s.source = text; s.view.unsaved = false; Ok::<_, String>(())
                    });
                    if let Err(e) = saved { error(&format!("VM operation succeeded; configuration save failed: {e}")); }
                }
                Err(e) => { STATE.with(|c| { if let Some(s) = c.borrow_mut().as_mut() { s.view.needs_readback = true; } }); error(&format!("Apply failed or is uncertain: {e}\nReadback follows; inspect recovery before retrying.")); }
            }
            discover();
        }
        Work::Verify => { if let Err(e) = result { error(&format!("Graphics verification failed or is uncertain: {e}")); } STATE.with(|c| { if let Some(s) = c.borrow_mut().as_mut() { s.view.needs_readback = true; } }); discover(); }
    }
    render();
    if STATE.with(|c| {
        c.borrow()
            .as_ref()
            .is_some_and(|s| s.close_requested && !busy(s))
    }) {
        close_window();
    }
}
fn stage() {
    let result = STATE.with(|c| {
        let mut b = c.borrow_mut();
        let s = b.as_mut().ok_or("View closed")?;
        if busy(s) {
            return Err("Wait for the active operation.".into());
        }
        let id = s.view.selected.clone().ok_or("Select a VM")?;
        let enabled = send(s.toggle, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 != 0;
        s.view.stage(&id, enabled)
    });
    match result {
        Err(e) => notice(e),
        Ok(()) => {
            notice("Pending intent only; Apply fetches a fresh preview. Discard has no VM effects.")
        }
    }
}
fn apply() {
    let target = STATE.with(|c| {
        c.borrow()
            .as_ref()
            .ok_or("View closed")?
            .view
            .apply_target()
    });
    match target {
        Err(e) => notice(e),
        Ok(t) => {
            let copy = t.clone();
            background(Work::Preview(t), move || {
                runner::submit(runner::request(Operation::Plan, Some(copy), None))
            });
        }
    }
}
#[allow(unsafe_code)]
fn details() {
    let text = STATE.with(|c| {
        let b = c.borrow(); let s = b.as_ref()?; let id = s.view.selected.as_ref()?; let vm = s.view.vm(id)?; let t = selected(s); let i = s.view.inventory.as_ref()?;
        let gpu = t.as_ref().and_then(|t| i.discovery.gpus.iter().find(|g| g.interface == t.gpu_interface)); let j = i.managed.get(id).and_then(Option::as_ref);
        let eligibility = t.as_ref().map_or_else(|| "Not configured/enrolled".into(), |t| s.view.eligibility(t).err().unwrap_or_else(|| "Eligible enrolled pair".into()));
        Some(format!("{}\nVM: {}\nGeneration: {}\nPower: {:?}\n\n{}\nRequested configuration: {}\nObserved support: {}\nSelected GPU: {}\nAttached interfaces: {:?}\nAllocation: {} {:?}\n\nPending recovery: {}\nRecorded preparation: {}\nLast successful graphics check (UTC Unix seconds): {}\nA recorded check is not fresh health. Guest OS is unobserved.\n\nGPU selection is constrained to the protected pair. Update runtime configuration and re-run administrator install to change GPU; the dashboard cannot grant enrollment.", vm.name, id, vm.generation, vm.power, eligibility, t.as_ref().map_or("Unconfigured", |t| if t.enabled { "Enabled" } else { "Disabled" }), s.view.support_text(vm), gpu.map_or("Unavailable", |g| g.name.as_str()), vm.gpus, allocation_text(vm), vm.vram, j.is_some_and(|j| j.pending), j.and_then(|j| j.prepared.as_deref()).unwrap_or("Not recorded"), j.and_then(|j| j.last_verified).map_or_else(|| "Not recorded".into(), |t| t.to_string())))
    }).unwrap_or_else(|| "Select an observed VM.".into());
    let parent = STATE.with(|c| c.borrow().as_ref().map(|s| s.window));
    modal(true);
    let text = wide(&text);
    // SAFETY: GUI-owned optional parent and live terminated text.
    unsafe {
        MessageBoxW(parent, PCWSTR(text.as_ptr()), w!("VM / GPU Details"), MB_OK);
    }
    modal(false);
}
fn verify() {
    let data = STATE.with(|c| {
        let b = c.borrow();
        let s = b.as_ref()?;
        let t = selected(s)?;
        let power = s.view.vm(&t.vm_id)?.power.clone();
        let restoration = s.view.verification(&t).ok()?;
        Some((t, s.window, power, restoration))
    });
    let Some((t, parent, power, restoration)) = data else {
        return;
    };
    modal(true);
    if !confirm(
        parent,
        "Verify graphics",
        &format!(
            "Check PnP and hardware D3D11 rendering on the enrolled GPU.\nObserved power: {power:?}. Final recovery power: {restoration:?}. An Off guest will be started for the check. Restoring Off includes graceful guest shutdown. The runner revalidates current state before execution.\nContinue?"
        ),
    ) {
        modal(false);
        return;
    }
    let supplied = credential(&t, parent);
    modal(false);
    match supplied {
        Ok(c) => background(Work::Verify, move || {
            runner::submit(runner::request(Operation::Verify, Some(t), Some(c)))
        }),
        Err(e) => notice(e),
    }
}
fn save() {
    let result = STATE.with(|c| {
        let mut b = c.borrow_mut();
        let s = b.as_mut().ok_or("View closed")?;
        let text = save_configuration(&s.path, &s.source, &s.view.configuration)?;
        s.source = text;
        s.view.unsaved = false;
        Ok::<_, String>(())
    });
    match result {
        Ok(()) => notice("Configuration saved; no VM operation repeated."),
        Err(e) => notice(format!("Configuration is still unsaved: {e}")),
    }
}
fn vault(store: bool) {
    let data = STATE.with(|c| {
        let b = c.borrow();
        let s = b.as_ref()?;
        Some((selected(s)?, s.window))
    });
    let Some((t, parent)) = data else {
        return;
    };
    modal(true);
    let result = if store {
        credentials::prompt(&t.vm_id, parent).and_then(|c| credentials::store(&t.vm_id, &c))
    } else {
        credentials::forget(&t.vm_id)
    };
    modal(false);
    match result {
        Ok(()) => notice(if store {
            "Credential explicitly stored in this user's Windows vault."
        } else {
            "Stored guest credential removed."
        }),
        Err(e) => notice(e),
    }
}
#[allow(unsafe_code)]
fn close_window() {
    let data = STATE.with(|c| {
        let mut b = c.borrow_mut();
        let s = b.as_mut()?;
        if busy(s) {
            s.close_requested = true;
            s.notice =
                "Close deferred until bounded work finishes; closing would not cancel native work."
                    .into();
            return None;
        }
        Some((s.window, s.view.draft.is_some() || s.view.unsaved))
    });
    if let Some((window, dirty)) = data {
        if dirty {
            modal(true);
            let yes = confirm(
                window,
                "Close Hyper GPU Support",
                "Close with staged edits or unsaved configuration? This does not undo a completed VM operation.",
            );
            modal(false);
            if !yes {
                STATE.with(|c| {
                    if let Some(s) = c.borrow_mut().as_mut() {
                        s.close_requested = false;
                    }
                });
                return;
            }
        }
        // SAFETY: owning GUI thread, no active request is represented as cancelled.
        unsafe {
            let _ = DestroyWindow(window);
        }
    } else {
        render();
    }
}
#[allow(unsafe_code)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: Windows supplies live message-specific structures on the owning UI thread.
    unsafe {
        match message {
            WM_SIZE => {
                if !RENDERING.with(Cell::get) {
                    layout();
                }
                LRESULT(0)
            }
            WM_DPICHANGED => {
                let r = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                fonts();
                layout();
                render();
                LRESULT(0)
            }
            WM_GETMINMAXINFO => {
                let info = &mut *(lparam.0 as *mut MINMAXINFO);
                let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                info.ptMinTrackSize.x = 980 * dpi / 96;
                info.ptMinTrackSize.y = 660 * dpi / 96;
                LRESULT(0)
            }
            WM_NOTIFY if !RENDERING.with(Cell::get) => {
                let hdr = &*(lparam.0 as *const NMHDR);
                if hdr.idFrom == LIST {
                    if hdr.code == LVN_ITEMCHANGED {
                        let changed = &*(lparam.0 as *const NMLISTVIEW);
                        if changed.uNewState & LVIS_SELECTED.0 != 0 {
                            STATE.with(|c| {
                                if let Some(s) = c.borrow_mut().as_mut()
                                    && let Some(vm) =
                                        s.view.inventory.as_ref().and_then(|i| {
                                            i.discovery.vms.get(changed.iItem as usize)
                                        })
                                {
                                    s.view.selected = Some(vm.vm_id.clone());
                                }
                            });
                            render();
                        }
                    } else if hdr.code == NM_DBLCLK || hdr.code == NM_CLICK {
                        let click = &*(lparam.0 as *const NMITEMACTIVATE);
                        if click.iItem >= 0 && click.iSubItem == 4 {
                            details();
                        }
                    }
                }
                LRESULT(0)
            }
            WM_COMMAND if !RENDERING.with(Cell::get) => {
                let id = wparam.0 & 0xffff;
                let active = STATE.with(|c| c.borrow().as_ref().is_some_and(busy));
                if !active {
                    match id {
                        200..=203 => {
                            STATE.with(|c| {
                                if let Some(s) = c.borrow_mut().as_mut() {
                                    s.page = id - 200;
                                }
                            });
                            render();
                        }
                        REFRESH => discover(),
                        TOGGLE => stage(),
                        APPLY => apply(),
                        DETAILS => details(),
                        VERIFY => verify(),
                        REAPPLY => {
                            let result = STATE.with(|c| {
                                let mut b = c.borrow_mut();
                                let s = b.as_mut().ok_or("View closed")?;
                                let id = s.view.selected.clone().ok_or("Select a VM")?;
                                s.view.reapply(&id)
                            });
                            notice(result.map_or_else(|e| e, |()| "Reapply staged; Apply previews current driver preparation and any effects.".into()));
                        }
                        SAVE => save(),
                        STORE => vault(true),
                        FORGET => vault(false),
                        DISCARD => {
                            STATE.with(|c| {
                                if let Some(s) = c.borrow_mut().as_mut() {
                                    s.view.discard();
                                }
                            });
                            notice("Staged edits discarded; no VM effects.");
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_TIMER => {
                poll();
                LRESULT(0)
            }
            WM_CLOSE => {
                close_window();
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
