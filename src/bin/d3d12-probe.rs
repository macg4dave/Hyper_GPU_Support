//! Hardware-only D3D12 offscreen correctness probe.

use std::mem::ManuallyDrop;
use std::process::ExitCode;
use std::time::Instant;

use hyper_gpu_support::probe::{
    EXPECTED_IMAGE_SHA256, ExitClass, HEIGHT, ProbeReport, WIDTH, sha256_hex,
};
use hyper_gpu_support::windows_probe::{AdapterSelectionError, select_rtx_5060};
use windows::Win32::Foundation::{CloseHandle, E_INVALIDARG, RECT, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D::{
    D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_12_0,
    D3D_FEATURE_LEVEL_12_1, D3D_FEATURE_LEVEL_12_2, D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST, ID3DBlob,
};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGIAdapter;
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::Interface;

const VERTEX_SHADER: &[u8] = include_bytes!("../../probes/shaders/compiled/d3d12-vs.dxil");
const PIXEL_SHADER: &[u8] = include_bytes!("../../probes/shaders/compiled/d3d12-ps.dxil");

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShaderModelQueryError {
    InvalidArgument,
    Failed,
    Unsupported,
}

impl ProbeFailure {
    const fn new(class: ExitClass, message: &'static str) -> Self {
        Self { class, message }
    }
}

#[allow(unsafe_code)]
fn run() -> Result<ProbeReport, ProbeFailure> {
    let started = Instant::now();
    let selected = select_rtx_5060().map_err(|error| match error {
        AdapterSelectionError::Runtime | AdapterSelectionError::Native(_) => {
            ProbeFailure::new(ExitClass::Runtime, "DXGI/D3DKMT enumeration failed")
        }
        AdapterSelectionError::Missing | AdapterSelectionError::Ambiguous => {
            ProbeFailure::new(ExitClass::Adapter, "exact RTX 5060 adapter unavailable")
        }
    })?;
    let adapter: IDXGIAdapter = selected
        .adapter()
        .cast()
        .map_err(|_| ProbeFailure::new(ExitClass::Runtime, "DXGI adapter cast failed"))?;
    let mut device = None;
    // SAFETY: the adapter is a live hardware DXGI adapter; the out pointer targets
    // owned `Option` storage and requests the documented minimum feature level.
    unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }
        .map_err(|_| ProbeFailure::new(ExitClass::Runtime, "D3D12 device creation failed"))?;
    let device: ID3D12Device = device.ok_or(ProbeFailure::new(
        ExitClass::Runtime,
        "D3D12 device was not returned",
    ))?;
    let (feature_level, shader_model) = query_features(&device)?;
    let root_signature = create_root_signature(&device)?;
    let pipeline = create_pipeline(&device, &root_signature)?;
    let queue_description = D3D12_COMMAND_QUEUE_DESC {
        Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
        Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
        Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
        NodeMask: 0,
    };
    // SAFETY: descriptor is initialized and the returned COM object is owned.
    let queue: ID3D12CommandQueue = unsafe { device.CreateCommandQueue(&queue_description) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "command queue creation failed"))?;
    // SAFETY: creates an owned allocator for direct command lists.
    let allocator: ID3D12CommandAllocator = unsafe {
        device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
    }
    .map_err(|_| ProbeFailure::new(ExitClass::Execution, "command allocator creation failed"))?;
    // SAFETY: allocator and pipeline are live and compatible with a direct list.
    let command_list: ID3D12GraphicsCommandList = unsafe {
        device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, &pipeline)
    }
    .map_err(|_| ProbeFailure::new(ExitClass::Execution, "command list creation failed"))?;

    let texture_description = texture_description();
    let clear_value = D3D12_CLEAR_VALUE {
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        Anonymous: D3D12_CLEAR_VALUE_0 {
            Color: [0.0, 1.0, 1.0, 1.0],
        },
    };
    let render_target = create_resource(
        &device,
        &default_heap(),
        &texture_description,
        D3D12_RESOURCE_STATE_RENDER_TARGET,
        Some(&clear_value),
    )?;
    let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
    let mut row_count = 0;
    let mut row_size = 0;
    let mut total_bytes = 0;
    // SAFETY: all output pointers target initialized storage and the resource
    // descriptor remains valid for this synchronous call.
    unsafe {
        device.GetCopyableFootprints(
            &texture_description,
            0,
            1,
            0,
            Some(&mut footprint),
            Some(&mut row_count),
            Some(&mut row_size),
            Some(&mut total_bytes),
        )
    };
    if row_count != HEIGHT || row_size != (WIDTH * 4) as u64 || total_bytes == 0 {
        return Err(ProbeFailure::new(
            ExitClass::Internal,
            "unexpected D3D12 copy footprint",
        ));
    }
    let readback_description = buffer_description(total_bytes);
    let readback = create_resource(
        &device,
        &readback_heap(),
        &readback_description,
        D3D12_RESOURCE_STATE_COPY_DEST,
        None,
    )?;
    let heap_description = D3D12_DESCRIPTOR_HEAP_DESC {
        Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        NumDescriptors: 1,
        Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        NodeMask: 0,
    };
    // SAFETY: descriptor is initialized and the returned heap is owned.
    let descriptor_heap: ID3D12DescriptorHeap =
        unsafe { device.CreateDescriptorHeap(&heap_description) }
            .map_err(|_| ProbeFailure::new(ExitClass::Execution, "RTV heap creation failed"))?;
    // SAFETY: the heap is live and contains one CPU-visible RTV descriptor.
    let render_target_view = unsafe { descriptor_heap.GetCPUDescriptorHandleForHeapStart() };
    // SAFETY: resource/descriptor handle are live and default view inference is
    // valid for this single-mip RGBA render target.
    unsafe { device.CreateRenderTargetView(&render_target, None, render_target_view) };
    record_commands(
        &command_list,
        &root_signature,
        render_target_view,
        &render_target,
        &readback,
        footprint,
    )?;
    // SAFETY: closes a live recording list exactly once.
    unsafe { command_list.Close() }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "command list close failed"))?;
    let base_list: ID3D12CommandList = command_list
        .cast()
        .map_err(|_| ProbeFailure::new(ExitClass::Internal, "command list cast failed"))?;
    // SAFETY: queue and command list are live; the slice remains valid for the call.
    unsafe { queue.ExecuteCommandLists(&[Some(base_list)]) };
    wait_for_completion(&device, &queue)?;
    let bytes = readback_bytes(&readback, footprint, total_bytes)?;
    if !bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [255, 0, 255, 255])
    {
        return Err(ProbeFailure::new(
            ExitClass::IncorrectOutput,
            "D3D12 pixel bytes differ from oracle",
        ));
    }
    let output_sha256 = sha256_hex(&bytes);
    if output_sha256 != EXPECTED_IMAGE_SHA256 {
        return Err(ProbeFailure::new(
            ExitClass::IncorrectOutput,
            "D3D12 output hash differs from oracle",
        ));
    }
    Ok(ProbeReport {
        schema: 1,
        probe: "d3d12-offscreen".into(),
        status: "pass".into(),
        adapter: selected.identity().clone(),
        feature_level: feature_level_name(feature_level).into(),
        shader_profiles: format!("vs_6_0,ps_6_0,max_{shader_model}"),
        width: WIDTH,
        height: HEIGHT,
        output_sha256,
        duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
    })
}

#[allow(unsafe_code)]
fn query_features(
    device: &ID3D12Device,
) -> Result<(D3D_FEATURE_LEVEL, &'static str), ProbeFailure> {
    let requested = [
        D3D_FEATURE_LEVEL_12_2,
        D3D_FEATURE_LEVEL_12_1,
        D3D_FEATURE_LEVEL_12_0,
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
    ];
    let mut levels = D3D12_FEATURE_DATA_FEATURE_LEVELS {
        NumFeatureLevels: requested.len() as u32,
        pFeatureLevelsRequested: requested.as_ptr(),
        MaxSupportedFeatureLevel: D3D_FEATURE_LEVEL_11_0,
    };
    // SAFETY: the feature data pointer/size exactly match the selected feature.
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_FEATURE_LEVELS,
            std::ptr::addr_of_mut!(levels).cast(),
            std::mem::size_of_val(&levels) as u32,
        )
    }
    .map_err(|_| ProbeFailure::new(ExitClass::Runtime, "feature-level query failed"))?;
    let shader_model = negotiate_shader_model(|requested| {
        let mut shader = D3D12_FEATURE_DATA_SHADER_MODEL {
            HighestShaderModel: requested,
        };
        // SAFETY: the feature data pointer/size exactly match the selected
        // feature and remain live for the synchronous query.
        match unsafe {
            device.CheckFeatureSupport(
                D3D12_FEATURE_SHADER_MODEL,
                std::ptr::addr_of_mut!(shader).cast(),
                std::mem::size_of_val(&shader) as u32,
            )
        } {
            Ok(()) => Ok(shader.HighestShaderModel),
            Err(error) if error.code() == E_INVALIDARG => {
                Err(ShaderModelQueryError::InvalidArgument)
            }
            Err(_) => Err(ShaderModelQueryError::Failed),
        }
    })
    .map_err(|error| match error {
        ShaderModelQueryError::Unsupported => {
            ProbeFailure::new(ExitClass::Runtime, "shader model 6.0 is unavailable")
        }
        ShaderModelQueryError::InvalidArgument | ShaderModelQueryError::Failed => {
            ProbeFailure::new(ExitClass::Runtime, "shader-model query failed")
        }
    })?;
    Ok((
        levels.MaxSupportedFeatureLevel,
        shader_model_name(shader_model),
    ))
}

fn negotiate_shader_model(
    mut query: impl FnMut(D3D_SHADER_MODEL) -> Result<D3D_SHADER_MODEL, ShaderModelQueryError>,
) -> Result<D3D_SHADER_MODEL, ShaderModelQueryError> {
    let candidates = [
        D3D_SHADER_MODEL_6_9,
        D3D_SHADER_MODEL_6_8,
        D3D_SHADER_MODEL_6_7,
        D3D_SHADER_MODEL_6_6,
        D3D_SHADER_MODEL_6_5,
        D3D_SHADER_MODEL_6_4,
        D3D_SHADER_MODEL_6_3,
        D3D_SHADER_MODEL_6_2,
        D3D_SHADER_MODEL_6_1,
        D3D_SHADER_MODEL_6_0,
    ];
    for candidate in candidates {
        match query(candidate) {
            Ok(supported) if supported.0 >= D3D_SHADER_MODEL_6_0.0 => return Ok(supported),
            Ok(_) => return Err(ShaderModelQueryError::Unsupported),
            Err(ShaderModelQueryError::InvalidArgument) => {}
            Err(error) => return Err(error),
        }
    }
    Err(ShaderModelQueryError::Unsupported)
}

#[allow(unsafe_code)]
fn create_root_signature(device: &ID3D12Device) -> Result<ID3D12RootSignature, ProbeFailure> {
    let description = D3D12_ROOT_SIGNATURE_DESC {
        Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        ..Default::default()
    };
    let mut blob: Option<ID3DBlob> = None;
    let mut errors: Option<ID3DBlob> = None;
    // SAFETY: descriptor/out pointers are valid for the synchronous serializer.
    unsafe {
        D3D12SerializeRootSignature(
            &description,
            D3D_ROOT_SIGNATURE_VERSION_1,
            &mut blob,
            Some(&mut errors),
        )
    }
    .map_err(|_| ProbeFailure::new(ExitClass::Internal, "root signature serialization failed"))?;
    let blob = blob.ok_or(ProbeFailure::new(
        ExitClass::Internal,
        "root signature blob was not returned",
    ))?;
    // SAFETY: blob pointer/length are owned by the live `ID3DBlob` for the slice.
    let bytes =
        unsafe { std::slice::from_raw_parts(blob.GetBufferPointer().cast(), blob.GetBufferSize()) };
    // SAFETY: serialized root signature bytes are valid and the returned COM
    // interface is owned.
    unsafe { device.CreateRootSignature(0, bytes) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "root signature creation failed"))
}

#[allow(unsafe_code)]
fn create_pipeline(
    device: &ID3D12Device,
    root_signature: &ID3D12RootSignature,
) -> Result<ID3D12PipelineState, ProbeFailure> {
    let mut description = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        pRootSignature: ManuallyDrop::new(Some(root_signature.clone())),
        VS: shader_bytecode(VERTEX_SHADER),
        PS: shader_bytecode(PIXEL_SHADER),
        ..Default::default()
    };
    description.BlendState.RenderTarget[0] = D3D12_RENDER_TARGET_BLEND_DESC {
        SrcBlend: D3D12_BLEND_ONE,
        DestBlend: D3D12_BLEND_ZERO,
        BlendOp: D3D12_BLEND_OP_ADD,
        SrcBlendAlpha: D3D12_BLEND_ONE,
        DestBlendAlpha: D3D12_BLEND_ZERO,
        BlendOpAlpha: D3D12_BLEND_OP_ADD,
        LogicOp: D3D12_LOGIC_OP_NOOP,
        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
        ..Default::default()
    };
    description.SampleMask = u32::MAX;
    description.RasterizerState = D3D12_RASTERIZER_DESC {
        FillMode: D3D12_FILL_MODE_SOLID,
        CullMode: D3D12_CULL_MODE_NONE,
        DepthClipEnable: true.into(),
        ..Default::default()
    };
    description.DepthStencilState = D3D12_DEPTH_STENCIL_DESC::default();
    description.PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;
    description.NumRenderTargets = 1;
    description.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
    description.SampleDesc = DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
    };
    // SAFETY: all descriptor pointers reference embedded shader slices that remain
    // live for the call; the returned pipeline object is owned.
    let result = unsafe { device.CreateGraphicsPipelineState(&description) };
    // SAFETY: the generated descriptor suppresses automatic destruction of its
    // optional COM interface. This releases exactly the clone placed above after
    // the native call has finished reading the descriptor.
    unsafe { ManuallyDrop::drop(&mut description.pRootSignature) };
    result.map_err(|_| ProbeFailure::new(ExitClass::Execution, "pipeline creation failed"))
}

fn shader_bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}

fn texture_description() -> D3D12_RESOURCE_DESC {
    D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: WIDTH as u64,
        Height: HEIGHT,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
    }
}

fn buffer_description(size: u64) -> D3D12_RESOURCE_DESC {
    D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width: size,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: Default::default(),
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: D3D12_RESOURCE_FLAG_NONE,
    }
}

fn default_heap() -> D3D12_HEAP_PROPERTIES {
    D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
        MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
        CreationNodeMask: 1,
        VisibleNodeMask: 1,
    }
}

fn readback_heap() -> D3D12_HEAP_PROPERTIES {
    D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_READBACK,
        CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
        MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
        CreationNodeMask: 1,
        VisibleNodeMask: 1,
    }
}

#[allow(unsafe_code)]
fn create_resource(
    device: &ID3D12Device,
    heap: &D3D12_HEAP_PROPERTIES,
    description: &D3D12_RESOURCE_DESC,
    state: D3D12_RESOURCE_STATES,
    clear_value: Option<&D3D12_CLEAR_VALUE>,
) -> Result<ID3D12Resource, ProbeFailure> {
    let mut resource = None;
    // SAFETY: all descriptors are initialized; optional clear value matches the
    // render format when present; the out pointer owns the returned interface.
    unsafe {
        device.CreateCommittedResource(
            heap,
            D3D12_HEAP_FLAG_NONE,
            description,
            state,
            clear_value.map(std::ptr::from_ref),
            &mut resource,
        )
    }
    .map_err(|_| ProbeFailure::new(ExitClass::Execution, "resource creation failed"))?;
    resource.ok_or(ProbeFailure::new(
        ExitClass::Execution,
        "resource was not returned",
    ))
}

#[allow(unsafe_code)]
fn record_commands(
    list: &ID3D12GraphicsCommandList,
    root_signature: &ID3D12RootSignature,
    rtv: D3D12_CPU_DESCRIPTOR_HANDLE,
    render_target: &ID3D12Resource,
    readback: &ID3D12Resource,
    footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
) -> Result<(), ProbeFailure> {
    let viewport = D3D12_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: WIDTH as f32,
        Height: HEIGHT as f32,
        MinDepth: 0.0,
        MaxDepth: 1.0,
    };
    let scissor = RECT {
        left: 0,
        top: 0,
        right: WIDTH as i32,
        bottom: HEIGHT as i32,
    };
    // SAFETY: command list and all referenced resources/descriptors remain live
    // until GPU completion; slice pointers are consumed during recording.
    unsafe {
        list.SetGraphicsRootSignature(root_signature);
        list.RSSetViewports(&[viewport]);
        list.RSSetScissorRects(&[scissor]);
        list.OMSetRenderTargets(1, Some(&rtv), false, None);
        list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        list.ClearRenderTargetView(rtv, &[0.0, 1.0, 1.0, 1.0], None);
        list.DrawInstanced(3, 1, 0, 0);
    }
    let transition = D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: ManuallyDrop::new(Some(render_target.clone())),
        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
        StateAfter: D3D12_RESOURCE_STATE_COPY_SOURCE,
    };
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(transition),
        },
    };
    let mut destination = D3D12_TEXTURE_COPY_LOCATION {
        pResource: ManuallyDrop::new(Some(readback.clone())),
        Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
            PlacedFootprint: footprint,
        },
    };
    let mut source = D3D12_TEXTURE_COPY_LOCATION {
        pResource: ManuallyDrop::new(Some(render_target.clone())),
        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
            SubresourceIndex: 0,
        },
    };
    // SAFETY: barrier transition matches the current/required resource states;
    // source/destination locations describe live compatible resources.
    unsafe {
        list.ResourceBarrier(std::slice::from_ref(&barrier));
        list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
        // SAFETY: these generated descriptor fields suppress automatic
        // destruction. Native recording has finished reading them, so release
        // exactly the interface clones placed into the descriptors above.
        ManuallyDrop::drop(&mut (*barrier.Anonymous.Transition).pResource);
        ManuallyDrop::drop(&mut destination.pResource);
        ManuallyDrop::drop(&mut source.pResource);
    }
    Ok(())
}

#[allow(unsafe_code)]
fn wait_for_completion(
    device: &ID3D12Device,
    queue: &ID3D12CommandQueue,
) -> Result<(), ProbeFailure> {
    // SAFETY: creates an owned fence object initialized at zero.
    let fence: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "fence creation failed"))?;
    // SAFETY: queue/fence are live and value one has not previously been signaled.
    unsafe { queue.Signal(&fence, 1) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "queue signal failed"))?;
    // SAFETY: creates a process-owned auto-reset event with no name.
    let event = unsafe { CreateEventW(None, false, false, None) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "event creation failed"))?;
    // SAFETY: event/fence handles are live for the wait registration.
    if unsafe { fence.SetEventOnCompletion(1, event) }.is_err() {
        // SAFETY: closes the event created above exactly once.
        let _ = unsafe { CloseHandle(event) };
        return Err(ProbeFailure::new(
            ExitClass::Execution,
            "fence event registration failed",
        ));
    }
    // SAFETY: waits at most 15 seconds on the live event handle.
    let wait = unsafe { WaitForSingleObject(event, 15_000) };
    // SAFETY: closes the event after the bounded wait exactly once.
    let close = unsafe { CloseHandle(event) };
    if close.is_err() {
        return Err(ProbeFailure::new(
            ExitClass::Execution,
            "event close failed",
        ));
    }
    if wait != WAIT_OBJECT_0 {
        return Err(ProbeFailure::new(
            ExitClass::Timeout,
            "D3D12 fence timed out",
        ));
    }
    Ok(())
}

#[allow(unsafe_code)]
fn readback_bytes(
    resource: &ID3D12Resource,
    footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    total_bytes: u64,
) -> Result<Vec<u8>, ProbeFailure> {
    let read_range = D3D12_RANGE {
        Begin: 0,
        End: total_bytes as usize,
    };
    let mut data = std::ptr::null_mut();
    // SAFETY: readback heap is CPU-readable; range is bounded by allocation size
    // and the out pointer targets initialized storage.
    unsafe { resource.Map(0, Some(&read_range), Some(&mut data)) }
        .map_err(|_| ProbeFailure::new(ExitClass::Execution, "readback map failed"))?;
    let row_bytes = WIDTH as usize * 4;
    if data.is_null() || footprint.Footprint.RowPitch < WIDTH * 4 {
        // SAFETY: balances the successful map without declaring CPU writes.
        unsafe { resource.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 })) };
        return Err(ProbeFailure::new(
            ExitClass::Execution,
            "readback layout is invalid",
        ));
    }
    let mut bytes = Vec::with_capacity(row_bytes * HEIGHT as usize);
    for row in 0..HEIGHT as usize {
        // SAFETY: `data` covers `total_bytes`; the footprint was returned by the
        // device and each copied row is no wider than its reported row pitch.
        let source = unsafe {
            std::slice::from_raw_parts(
                data.cast::<u8>()
                    .add(footprint.Offset as usize + row * footprint.Footprint.RowPitch as usize),
                row_bytes,
            )
        };
        bytes.extend_from_slice(source);
    }
    // SAFETY: balances the successful read-only map after row borrows expire.
    unsafe { resource.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 })) };
    Ok(bytes)
}

const fn feature_level_name(level: D3D_FEATURE_LEVEL) -> &'static str {
    match level {
        D3D_FEATURE_LEVEL_12_2 => "12_2",
        D3D_FEATURE_LEVEL_12_1 => "12_1",
        D3D_FEATURE_LEVEL_12_0 => "12_0",
        D3D_FEATURE_LEVEL_11_1 => "11_1",
        D3D_FEATURE_LEVEL_11_0 => "11_0",
        _ => "unknown",
    }
}

const fn shader_model_name(model: D3D_SHADER_MODEL) -> &'static str {
    match model.0 {
        0x60 => "6_0",
        0x61 => "6_1",
        0x62 => "6_2",
        0x63 => "6_3",
        0x64 => "6_4",
        0x65 => "6_5",
        0x66 => "6_6",
        0x67 => "6_7",
        0x68 => "6_8",
        0x69 => "6_9",
        0x6a => "6_10",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{ShaderModelQueryError, negotiate_shader_model};
    use windows::Win32::Graphics::Direct3D12::{
        D3D_SHADER_MODEL_6_5, D3D_SHADER_MODEL_6_6, D3D_SHADER_MODEL_6_7, D3D_SHADER_MODEL_6_8,
        D3D_SHADER_MODEL_6_9,
    };

    #[test]
    fn retries_runtime_unknown_shader_models_in_descending_order() {
        let mut requested = Vec::new();
        let model = negotiate_shader_model(|candidate| {
            requested.push(candidate);
            if candidate.0 > D3D_SHADER_MODEL_6_6.0 {
                Err(ShaderModelQueryError::InvalidArgument)
            } else {
                Ok(D3D_SHADER_MODEL_6_5)
            }
        });
        assert_eq!(model, Ok(D3D_SHADER_MODEL_6_5));
        assert_eq!(
            requested,
            [
                D3D_SHADER_MODEL_6_9,
                D3D_SHADER_MODEL_6_8,
                D3D_SHADER_MODEL_6_7,
                D3D_SHADER_MODEL_6_6,
            ]
        );
    }

    #[test]
    fn preserves_non_compatibility_query_failures() {
        let mut calls = 0;
        let result = negotiate_shader_model(|_| {
            calls += 1;
            Err(ShaderModelQueryError::Failed)
        });
        assert_eq!(result, Err(ShaderModelQueryError::Failed));
        assert_eq!(calls, 1);
    }
}
