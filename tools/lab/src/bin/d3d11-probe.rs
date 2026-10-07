//! Hardware-only D3D11 offscreen correctness probe.

use std::process::ExitCode;
use std::time::Instant;

use hyper_gpu_support::probe::{
    EXPECTED_IMAGE_SHA256, ExitClass, HEIGHT, ProbeReport, WIDTH, sha256_hex,
};
use hyper_gpu_support::windows_probe::{AdapterSelectionError, select_configured_gpu};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
    D3D_FEATURE_LEVEL_12_0, D3D_FEATURE_LEVEL_12_1, D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_FLAG, D3D11_MAP_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_STAGING, D3D11_VIEWPORT, D3D11CreateDevice, ID3D11DepthStencilView, ID3D11Device,
    ID3D11DeviceContext, ID3D11PixelShader, ID3D11RenderTargetView, ID3D11Texture2D,
    ID3D11VertexShader,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGIAdapter;
use windows::core::Interface;

const VERTEX_SHADER: &[u8] = include_bytes!("../../../../probes/shaders/compiled/d3d11-vs.dxbc");
const PIXEL_SHADER: &[u8] = include_bytes!("../../../../probes/shaders/compiled/d3d11-ps.dxbc");

fn main() -> ExitCode {
    match run() {
        Ok(report) => match serde_json::to_string(&report) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(_) => fail(ExitClass::Internal, "cannot serialize probe result"),
        },
        Err(error) => fail(error.class, error.message),
    }
}

fn fail(class: ExitClass, message: &'static str) -> ExitCode {
    eprintln!("probe error: {message}");
    ExitCode::from(class.code())
}

struct ProbeFailure {
    class: ExitClass,
    message: &'static str,
}

impl ProbeFailure {
    const fn new(class: ExitClass, message: &'static str) -> Self {
        Self { class, message }
    }
}

#[allow(unsafe_code)]
fn run() -> Result<ProbeReport, ProbeFailure> {
    let started = Instant::now();
    let selected = select_configured_gpu().map_err(|error| match error {
        AdapterSelectionError::Runtime | AdapterSelectionError::Native(_) => {
            ProbeFailure::new(ExitClass::Runtime, "DXGI enumeration failed")
        }
        AdapterSelectionError::Missing | AdapterSelectionError::Ambiguous => {
            ProbeFailure::new(ExitClass::Adapter, "configured GPU adapter unavailable")
        }
    })?;
    let adapter: IDXGIAdapter = selected
        .adapter()
        .cast()
        .map_err(|_| ProbeFailure::new(ExitClass::Runtime, "DXGI adapter cast failed"))?;
    let levels = [
        D3D_FEATURE_LEVEL_12_1,
        D3D_FEATURE_LEVEL_12_0,
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
    ];
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let mut feature_level = D3D_FEATURE_LEVEL(0);
    // SAFETY: all out pointers target initialized `Option` storage; the adapter is
    // a live COM object and the feature-level slice remains valid for the call.
    unsafe {
        D3D11CreateDevice(
            &adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG(0),
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut feature_level),
            Some(&mut context),
        )
    }
    .map_err(|_| ProbeFailure::new(ExitClass::Runtime, "D3D11 device creation failed"))?;
    let device = device.ok_or(ProbeFailure::new(
        ExitClass::Runtime,
        "D3D11 device was not returned",
    ))?;
    let context = context.ok_or(ProbeFailure::new(
        ExitClass::Runtime,
        "D3D11 context was not returned",
    ))?;

    let render_description = D3D11_TEXTURE2D_DESC {
        Width: WIDTH,
        Height: HEIGHT,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };
    let staging_description = D3D11_TEXTURE2D_DESC {
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
        ..render_description
    };
    let render_texture = create_texture(&device, &render_description)?;
    let staging_texture = create_texture(&device, &staging_description)?;
    let mut render_target: Option<ID3D11RenderTargetView> = None;
    // SAFETY: the texture is live, the default descriptor matches its format and
    // the out pointer owns the returned COM reference.
    unsafe { device.CreateRenderTargetView(&render_texture, None, Some(&mut render_target)) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "render target creation failed"))?;
    let render_target = render_target.ok_or(ProbeFailure::new(
        ExitClass::Execution,
        "render target was not returned",
    ))?;
    let vertex_shader = create_vertex_shader(&device)?;
    let pixel_shader = create_pixel_shader(&device)?;
    let viewport = D3D11_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: WIDTH as f32,
        Height: HEIGHT as f32,
        MinDepth: 0.0,
        MaxDepth: 1.0,
    };
    // SAFETY: every COM interface is live for the duration of these immediate
    // context calls; slices contain owned interface clones valid during the call.
    unsafe {
        context.OMSetRenderTargets(
            Some(&[Some(render_target.clone())]),
            None::<&ID3D11DepthStencilView>,
        );
        context.RSSetViewports(Some(&[viewport]));
        context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        context.VSSetShader(&vertex_shader, None);
        context.PSSetShader(&pixel_shader, None);
        context.ClearRenderTargetView(&render_target, &[0.0, 1.0, 1.0, 1.0]);
        context.Draw(3, 0);
        context.CopyResource(&staging_texture, &render_texture);
    }

    let bytes = read_texture(&context, &staging_texture)?;
    if !bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [255, 0, 255, 255])
    {
        return Err(ProbeFailure::new(
            ExitClass::IncorrectOutput,
            "D3D11 pixel bytes differ from oracle",
        ));
    }
    let output_sha256 = sha256_hex(&bytes);
    if output_sha256 != EXPECTED_IMAGE_SHA256 {
        return Err(ProbeFailure::new(
            ExitClass::IncorrectOutput,
            "D3D11 output hash differs from oracle",
        ));
    }
    Ok(ProbeReport {
        schema: 1,
        probe: "d3d11-offscreen".into(),
        status: "pass".into(),
        adapter: selected.identity().clone(),
        feature_level: feature_level_name(feature_level).into(),
        shader_profiles: "vs_5_0,ps_5_0".into(),
        width: WIDTH,
        height: HEIGHT,
        output_sha256,
        duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
    })
}

#[allow(unsafe_code)]
fn create_texture(
    device: &ID3D11Device,
    description: &D3D11_TEXTURE2D_DESC,
) -> Result<ID3D11Texture2D, ProbeFailure> {
    let mut texture = None;
    // SAFETY: `description` is initialized and valid for the call; no initial data
    // is supplied and the out pointer targets owned `Option` storage.
    unsafe { device.CreateTexture2D(description, None, Some(&mut texture)) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "texture creation failed"))?;
    texture.ok_or(ProbeFailure::new(
        ExitClass::Execution,
        "texture was not returned",
    ))
}

#[allow(unsafe_code)]
fn create_vertex_shader(device: &ID3D11Device) -> Result<ID3D11VertexShader, ProbeFailure> {
    let mut shader = None;
    // SAFETY: embedded DXBC was produced by the pinned SDK compiler; the slice and
    // out pointer remain valid for the call and no class linkage is used.
    unsafe { device.CreateVertexShader(VERTEX_SHADER, None, Some(&mut shader)) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "vertex shader creation failed"))?;
    shader.ok_or(ProbeFailure::new(
        ExitClass::Execution,
        "vertex shader was not returned",
    ))
}

#[allow(unsafe_code)]
fn create_pixel_shader(device: &ID3D11Device) -> Result<ID3D11PixelShader, ProbeFailure> {
    let mut shader = None;
    // SAFETY: embedded DXBC was produced by the pinned SDK compiler; the slice and
    // out pointer remain valid for the call and no class linkage is used.
    unsafe { device.CreatePixelShader(PIXEL_SHADER, None, Some(&mut shader)) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "pixel shader creation failed"))?;
    shader.ok_or(ProbeFailure::new(
        ExitClass::Execution,
        "pixel shader was not returned",
    ))
}

#[allow(unsafe_code)]
fn read_texture(
    context: &ID3D11DeviceContext,
    texture: &ID3D11Texture2D,
) -> Result<Vec<u8>, ProbeFailure> {
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: the staging texture was created with CPU read access and the mapped
    // descriptor remains live until the matching `Unmap` below.
    unsafe { context.Map(texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped)) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "staging texture map failed"))?;
    let row_bytes = (WIDTH as usize) * 4;
    if mapped.pData.is_null() || (mapped.RowPitch as usize) < row_bytes {
        // SAFETY: `Map` succeeded for subresource zero, so it must be unmapped once.
        unsafe { context.Unmap(texture, 0) };
        return Err(ProbeFailure::new(
            ExitClass::Execution,
            "mapped texture layout is invalid",
        ));
    }
    let mut bytes = Vec::with_capacity(row_bytes * HEIGHT as usize);
    for row in 0..HEIGHT as usize {
        // SAFETY: DXGI guarantees `RowPitch` bytes for each of `HEIGHT` rows while
        // mapped; `row_bytes <= RowPitch` was checked above.
        let source = unsafe {
            std::slice::from_raw_parts(
                mapped
                    .pData
                    .cast::<u8>()
                    .add(row * mapped.RowPitch as usize),
                row_bytes,
            )
        };
        bytes.extend_from_slice(source);
    }
    // SAFETY: balances the successful map for subresource zero after all borrowed
    // row slices have expired.
    unsafe { context.Unmap(texture, 0) };
    Ok(bytes)
}

const fn feature_level_name(level: D3D_FEATURE_LEVEL) -> &'static str {
    match level {
        D3D_FEATURE_LEVEL_12_1 => "12_1",
        D3D_FEATURE_LEVEL_12_0 => "12_0",
        D3D_FEATURE_LEVEL_11_1 => "11_1",
        D3D_FEATURE_LEVEL_11_0 => "11_0",
        _ => "unknown",
    }
}
