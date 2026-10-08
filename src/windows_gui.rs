//! Native Win32 view; no web frontend, local server or duplicated GPU management.
use hyper_gpu_support::{
    credentials::{self},
    model::{Configuration, Discovery, Power, Target},
    runner::{self, Operation},
};
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
};
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
    core::{PCWSTR, w},
};
const LIST: usize = 101;
const GPU: usize = 102;
const TOGGLE: usize = 103;
const APPLY: usize = 104;
const REFRESH: usize = 105;
const DETAILS: usize = 106;
const DISCARD: usize = 107;
thread_local! {static STATE:RefCell<Option<State>>=const{RefCell::new(None)};}
struct State {
    config: Configuration,
    path: PathBuf,
    discovery: Option<Discovery>,
    window: HWND,
    list: HWND,
    gpu: HWND,
    toggle: HWND,
    status: HWND,
    apply: HWND,
    receiver: Option<Receiver<Result<serde_json::Value, String>>>,
    pending_target: Option<Target>,
    managed: serde_json::Value,
    preview_target: Option<Target>,
}
#[allow(unsafe_code)]
fn send(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: callers use GUI-owned controls and valid message-specific buffers.
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::SendMessageW(hwnd, message, Some(w), Some(l))
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
#[allow(unsafe_code)]
pub(super) fn error(text: &str) {
    let text = wide(text);
    // SAFETY: live terminated strings and no borrowed window handles.
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
fn caption(hwnd: HWND, text: &str) {
    let text = wide(text);
    // SAFETY: live GUI-owned HWND, terminated text copied by Windows.
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(text.as_ptr()));
    }
}
#[allow(unsafe_code)]
// This narrow adapter mirrors the native control creation arguments.
#[allow(clippy::too_many_arguments)]
fn create(
    parent: HWND,
    class: PCWSTR,
    text: &str,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    style: WINDOW_STYLE,
) -> Result<HWND, String> {
    let text = wide(text);
    // SAFETY: GUI thread owns parent; fixed class, live title buffer, control ID.
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            PCWSTR(text.as_ptr()),
            WS_CHILD | WS_VISIBLE | style,
            x,
            y,
            width,
            height,
            Some(parent),
            Some(HMENU(id as *mut _)),
            None,
            None,
        )
    }
    .map_err(|e| e.to_string())?;
    // SAFETY: borrowed Windows stock font; HWND is a live GUI-owned control.
    let font = unsafe {
        windows::Win32::Graphics::Gdi::GetStockObject(
            windows::Win32::Graphics::Gdi::DEFAULT_GUI_FONT,
        )
    };
    send(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    Ok(hwnd)
}
#[allow(unsafe_code)]
pub(super) fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "--config" {
        return Err("Usage: hyper-gpu-gui --config FILE\nInstall/enroll the product runner first using the CLI.".into());
    }
    let path = PathBuf::from(&args[1]);
    let config = Configuration::parse(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)?;
    // SAFETY: read-only current module lookup; module handle is borrowed, not closed.
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
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
            (windows::Win32::Graphics::Gdi::COLOR_WINDOW.0 + 1) as *mut _,
        ),
        ..Default::default()
    };
    // SAFETY: class structure and fixed static class name remain live for registration.
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err("native window registration failed".into());
    }
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class.lpszClassName,
            w!("Hyper GPU Support"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1120,
            720,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .map_err(|e| e.to_string())?;
    create(
        window,
        w!("STATIC"),
        "Virtual Machines",
        0,
        24,
        18,
        600,
        32,
        WINDOW_STYLE(0),
    )?;
    create(
        window,
        w!("STATIC"),
        "Manage GPU support for enrolled, existing Hyper-V virtual machines.",
        0,
        24,
        56,
        900,
        26,
        WINDOW_STYLE(0),
    )?;
    create(
        window,
        w!("BUTTON"),
        "Refresh",
        REFRESH,
        932,
        20,
        140,
        34,
        WS_TABSTOP,
    )?;
    let list = create(
        window,
        w!("LISTBOX"),
        "",
        LIST,
        24,
        102,
        1048,
        285,
        WS_TABSTOP | WS_BORDER | WS_VSCROLL | WINDOW_STYLE(LBS_NOTIFY as u32),
    )?;
    create(
        window,
        w!("STATIC"),
        "Selected VM GPU",
        0,
        24,
        412,
        180,
        24,
        WINDOW_STYLE(0),
    )?;
    let gpu = create(
        window,
        w!("COMBOBOX"),
        "",
        GPU,
        210,
        406,
        862,
        240,
        WS_TABSTOP | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
    )?;
    let toggle = create(
        window,
        w!("BUTTON"),
        "Enable GPU support",
        TOGGLE,
        24,
        452,
        240,
        32,
        WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
    )?;
    create(
        window,
        w!("STATIC"),
        "GPU memory uses Hyper-V provider defaults in this milestone.\nNo hard VRAM limit or GiB mapping is claimed. Graceful guest restarts are automatic when required.",
        0,
        300,
        450,
        772,
        58,
        WINDOW_STYLE(0),
    )?;
    create(
        window,
        w!("BUTTON"),
        "View Details",
        DETAILS,
        24,
        522,
        180,
        36,
        WS_TABSTOP,
    )?;
    create(
        window,
        w!("BUTTON"),
        "Discard Changes",
        DISCARD,
        680,
        580,
        180,
        40,
        WS_TABSTOP,
    )?;
    let apply = create(
        window,
        w!("BUTTON"),
        "Apply Changes",
        APPLY,
        880,
        580,
        192,
        40,
        WS_TABSTOP | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
    )?;
    let status = create(
        window,
        w!("STATIC"),
        "Discovering current Hyper-V state...",
        0,
        24,
        638,
        1048,
        26,
        WINDOW_STYLE(0),
    )?;
    STATE.with(|s| {
        *s.borrow_mut() = Some(State {
            config,
            path,
            discovery: None,
            window,
            list,
            gpu,
            toggle,
            status,
            apply,
            receiver: None,
            pending_target: None,
            managed: serde_json::Value::Null,
            preview_target: None,
        })
    });
    start_discovery();
    // SAFETY: GUI-owned window; timer pumps bounded background-operation replies.
    unsafe {
        SetTimer(Some(window), 1, 200, None);
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
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    STATE.with(|s| *s.borrow_mut() = None);
    Ok(())
}
fn background(
    work: impl FnOnce() -> Result<serde_json::Value, String> + Send + 'static,
    target: Option<Target>,
) {
    STATE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let Some(s) = borrow.as_mut() else {
            return;
        };
        if s.receiver.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        s.receiver = Some(receiver);
        s.pending_target = target;
        caption(
            s.status,
            "Operation in progress. Guest downtime may be required.",
        );
        #[allow(unsafe_code)]
        // SAFETY: controls are GUI-owned; disable changes until the operation completes.
        unsafe {
            let _ = EnableWindow(s.apply, false);
        }
        std::thread::spawn(move || {
            let _ = sender.send(work());
        });
    });
}
fn start_discovery() {
    background(
        || runner::submit(runner::request(Operation::Discover, None, None)),
        None,
    );
}
#[allow(unsafe_code)]
fn poll() {
    let result = STATE.with(|cell| {
        let mut b = cell.borrow_mut();
        let s = b.as_mut()?;
        let result = s.receiver.as_ref()?.try_recv().ok()?;
        s.receiver = None;
        Some(result)
    });
    if let Some(result) = result {
        let preview = STATE.with(|c| {
            c.borrow_mut()
                .as_mut()
                .and_then(|s| s.preview_target.take().map(|target| (target, s.window)))
        });
        if let Some((target, parent)) = preview {
            let message = match result.and_then(|value| {
                serde_json::from_value::<hyper_gpu_support::workflow::Plan>(value)
                    .map_err(|e| e.to_string())
            }) {
                Err(e) => {
                    error(&e);
                    "Preview failed; no changes requested."
                }
                Ok(plan) => {
                    let text = wide(&format!(
                        "VM: {}\nGPU: {}\n\n{}\n\nApply these changes?",
                        plan.observed.name,
                        plan.desired.gpu_interface,
                        plan.preview.summary.join("\n")
                    ));
                    // SAFETY: GUI-owned parent and live terminated buffers for a modal preview.
                    if unsafe {
                        MessageBoxW(
                            Some(parent),
                            PCWSTR(text.as_ptr()),
                            w!("Preview GPU changes"),
                            MB_YESNO | MB_ICONINFORMATION | MB_DEFBUTTON2,
                        )
                    } == IDYES
                    {
                        if execute_apply(target, parent) {
                            "Applying previewed changes..."
                        } else {
                            "Credential entry failed or was cancelled; no changes requested."
                        }
                    } else {
                        "Preview cancelled; no changes requested."
                    }
                }
            };
            STATE.with(|c| {
                if let Some(s) = c.borrow().as_ref() {
                    caption(s.status, message);
                    // SAFETY: GUI-owned control, re-enabled only when no background work remains.
                    unsafe {
                        let _ = EnableWindow(s.apply, s.receiver.is_none());
                    }
                }
            });
            return;
        }
        match result {
            Err(e) => {
                error(&e);
                STATE.with(|c| {
                    if let Some(s) = c.borrow_mut().as_mut() {
                        s.pending_target = None;
                        caption(
                            s.status,
                            "Operation failed. Inspect details and reconcile before retry.",
                        );
                    }
                });
            }
            Ok(value) => {
                let updated = STATE.with(|c| {
                    let mut b = c.borrow_mut();
                    let s = b.as_mut().ok_or("view is closed")?;
                    if let Some(target) = s.pending_target.take() {
                        if let Some(old) = s
                            .config
                            .targets
                            .iter_mut()
                            .find(|t| t.vm_id == target.vm_id)
                        {
                            *old = target;
                        }
                        let text = toml::to_string_pretty(&s.config).map_err(|e| e.to_string())?;
                        std::fs::write(&s.path, text).map_err(|e| {
                            format!("operation succeeded but configuration save failed: {e}")
                        })?;
                        caption(
                            s.status,
                            "GPU operation completed; effective state will refresh.",
                        );
                        Ok::<_, String>(true)
                    } else {
                        s.managed = value
                            .get("managed")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        let d: Discovery =
                            serde_json::from_value(value).map_err(|e| e.to_string())?;
                        send(s.list, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
                        send(s.gpu, CB_RESETCONTENT, WPARAM(0), LPARAM(0));
                        for v in &d.vms {
                            let status = match v.power {
                                Power::Off => "Off",
                                Power::Running => "Running",
                                Power::Other(_) => "Transitional/saved",
                            };
                            let gpu = if v.gpus.is_empty() {
                                "Disabled"
                            } else {
                                "Assigned"
                            };
                            let text = wide(&format!(
                                "{}    |    {status}    |    GPU: {gpu}    |    Generation {}",
                                v.name, v.generation
                            ));
                            send(
                                s.list,
                                LB_ADDSTRING,
                                WPARAM(0),
                                LPARAM(text.as_ptr() as isize),
                            );
                        }
                        for g in &d.gpus {
                            let text = wide(&format!(
                                "{} ({}){}",
                                g.name,
                                g.driver_version,
                                if g.preparation_supported {
                                    ""
                                } else {
                                    " — preparation unsupported"
                                }
                            ));
                            send(
                                s.gpu,
                                CB_ADDSTRING,
                                WPARAM(0),
                                LPARAM(text.as_ptr() as isize),
                            );
                        }
                        s.discovery = Some(d);
                        caption(
                            s.status,
                            "Current provider state. Select an enrolled VM to edit GPU support.",
                        );
                        Ok(false)
                    }
                });
                match updated {
                    Ok(true) => start_discovery(),
                    Ok(false) => {}
                    Err(e) => error(&e),
                }
            }
        }
        STATE.with(|c| {
            if let Some(s) = c.borrow().as_ref() {
                unsafe {
                    let _ = EnableWindow(s.apply, s.receiver.is_none());
                }
            }
        });
    }
}
#[allow(unsafe_code)]
fn select() {
    STATE.with(|cell|{let mut b=cell.borrow_mut();let Some(s)=b.as_mut()else{return;};let Some(d)=s.discovery.as_ref()else{return;};
    // SAFETY: control messages return indexes into the displayed discovery vectors.
    let index=send(s.list,LB_GETCURSEL,WPARAM(0),LPARAM(0)).0;
    if let Some(v)=usize::try_from(index).ok().and_then(|i|d.vms.get(i)){
        let target=s.config.targets.iter().find(|t|t.vm_id==v.vm_id);
        let gpu=target.map(|t|t.gpu_interface.as_str()).or_else(||v.gpus.first().map(String::as_str));
        let gpu_index=gpu.and_then(|id|d.gpus.iter().position(|g|g.interface==id)).map_or(-1,|i|i as isize);
        send(s.gpu,CB_SETCURSEL,WPARAM(gpu_index as usize),LPARAM(0));send(s.toggle,BM_SETCHECK,WPARAM(usize::from(target.map_or(!v.gpus.is_empty(),|t|t.enabled))),LPARAM(0));
        caption(s.status,if target.is_some(){"Edit the selected VM; Apply Changes runs the shared product workflow."}else{"This VM is not enrolled. Add it to configuration and run administrator install before applying."});
    }});
}
#[allow(unsafe_code)]
fn apply() {
    let selection = STATE.with(|cell| {
        let b = cell.borrow();
        let s = b.as_ref().ok_or("view is closed")?;
        if s.receiver.is_some() {
            return Err("an operation is already active");
        }
        let d = s.discovery.as_ref().ok_or("refresh inventory first")?;
        let index = send(s.list, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
        let v = usize::try_from(index)
            .ok()
            .and_then(|i| d.vms.get(i))
            .ok_or("select a VM")?;
        let mut target = s
            .config
            .targets
            .iter()
            .find(|t| t.vm_id == v.vm_id)
            .cloned()
            .ok_or("enroll this VM using administrator install first")?;
        let index = send(s.gpu, CB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
        let gpu = usize::try_from(index)
            .ok()
            .and_then(|i| d.gpus.get(i))
            .ok_or("select a GPU")?;
        target.gpu_interface = gpu.interface.clone();
        target.enabled = send(s.toggle, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 != 0;
        Ok((target, s.window))
    });
    match selection {
        Err(e) => error(e),
        Ok((target, _)) => {
            STATE.with(|c| {
                if let Some(s) = c.borrow_mut().as_mut() {
                    s.preview_target = Some(target.clone());
                }
            });
            background(
                move || runner::submit(runner::request(Operation::Plan, Some(target), None)),
                None,
            );
        }
    }
}
fn execute_apply(target: Target, parent: HWND) -> bool {
    let credential = if target.enabled {
        match credentials::read(&target.vm_id).and_then(|stored| {
            stored.map_or_else(|| credentials::prompt(&target.vm_id, parent), Ok)
        }) {
            Ok(c) => Some(c),
            Err(e) => {
                error(&e);
                return false;
            }
        }
    } else {
        None
    };
    let copy = target.clone();
    background(
        move || runner::submit(runner::request(Operation::Apply, Some(copy), credential)),
        Some(target),
    );
    true
}
#[allow(unsafe_code)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: Win32 invokes this callback on the owning GUI thread with native message arguments.
    unsafe {
        match message {
            WM_COMMAND => {
                let id = wparam.0 & 0xffff;
                let code = (wparam.0 >> 16) & 0xffff;
                match id {
                    REFRESH => start_discovery(),
                    APPLY => apply(),
                    DISCARD => select(),
                    LIST if code == LBN_SELCHANGE as usize => select(),
                    DETAILS => {
                        let text = STATE
                            .with(|c| {
                                let b = c.borrow();
                                let s = b.as_ref()?;
                                let d = s.discovery.as_ref()?;
                                let index = send(s.list, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
                                let v = usize::try_from(index).ok().and_then(|i| d.vms.get(i))?;
                                serde_json::to_string_pretty(&serde_json::json!({"observed": v, "managed": s.managed.get(&v.vm_id)})).ok()
                            })
                            .unwrap_or_else(|| "Select a VM to inspect.".into());
                        let text = wide(&text);
                        MessageBoxW(
                            Some(hwnd),
                            PCWSTR(text.as_ptr()),
                            w!("Effective VM / GPU configuration"),
                            MB_OK,
                        );
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_TIMER => {
                poll();
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
