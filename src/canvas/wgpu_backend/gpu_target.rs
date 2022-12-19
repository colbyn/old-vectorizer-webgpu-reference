use std::borrow::Cow;
use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};
use wgpu::{include_wgsl, SurfaceConfiguration};
use futures::executor::block_on;
use wgpu::util::DeviceExt;
use std::f64::NAN;

pub use crate::data::{ViewResolution, PictureResolution};



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// GPU BACKEND API
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub trait GpuBackend {
    fn into_context(self) -> GpuContext;
}

pub struct GpuContext {
    surface: wgpu::Surface,
    instance: wgpu::Instance,
}


pub struct MetalBackernd(GpuContext);

impl MetalBackernd {
    pub fn new(layer: *mut std::ffi::c_void) -> MetalBackernd {
        let instance = wgpu::Instance::new(wgpu::Backends::METAL);
        let surface = unsafe {
            instance.create_surface_from_core_animation_layer(layer)
        };
        MetalBackernd(GpuContext { surface, instance })
    }
}

impl GpuBackend for MetalBackernd {
    fn into_context(self) -> GpuContext {self.0}
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// GPU INSTANCE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug)]
pub struct GpuHandle {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub wireframe: bool,
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: wgpu::ShaderModule,
    pub gpu_view_info: Option<GpuViewInfo>,
    pub msaa_samples: u32,
}

#[derive(Debug)]
pub struct GpuViewInfo {
    pub surface_desc: wgpu::SurfaceConfiguration,
    pub view_resolution: ViewResolution,
}

impl GpuHandle {
    pub const DEFAULT_MSAA_SAMPLES: u32 = 4;
    // pub const DEFAULT_MSAA_SAMPLES: u32 = 1;
    // pub const DEFAULT_USE_ANTIALIASING: bool = true;
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    // pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// INIT GPU INSTANCE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl GpuHandle {
    pub fn new(backend: impl GpuBackend) -> GpuHandle {
        let GpuContext { surface, instance } = backend.into_context();
        // CREATE AN ADAPTER
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();
        // GPU DEVICE FEATURES
        let device_features = {
            wgpu::Features::default()
                | wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::CLEAR_TEXTURE
                | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
                | wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO
                | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER
        };
        // CREATE A DEVICE AND A QUEUE
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: device_features,
                limits: wgpu::Limits::default(),
            },
            // TRACE_PATH CAN BE USED FOR API CALL TRACING
            None,
        ))
        .unwrap();
        // device.features()
        let vs_module = device.create_shader_module(include_wgsl!("./../../../shaders/geometry.vs.wgsl"));
        let fs_module = device.create_shader_module(include_wgsl!("./../../../shaders/geometry.fs.wgsl"));
        GpuHandle {
            surface,
            device,
            queue,
            wireframe: false,
            msaa_samples: GpuHandle::DEFAULT_MSAA_SAMPLES,
            gpu_view_info: None,
            vs_module,
            fs_module
        }
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// UPDATE GPU INSTANCE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub enum GpuHandleUpdateInfo {
    UpdatedViewResolution,
    NoOp,
}

impl GpuHandleUpdateInfo {
    pub fn updated_view_resolution(&self) -> bool {
        match self {
            Self::UpdatedViewResolution => true,
            _ => false
        }
    }
}

impl GpuHandle {
    pub(crate) fn update(
        &mut self,
        latest_resolution: ViewResolution,
        msaa_samples: u32,
        msaa_texture: &mut Option<wgpu::TextureView>,
    ) -> GpuHandleUpdateInfo {
        let view_resolution_changed = self.gpu_view_info
            .as_ref()
            .map(|gpu_view_info| {
                gpu_view_info.view_resolution.resolution_changed(&latest_resolution)
            })
            .unwrap_or(true);
        let surface_needs_configure = view_resolution_changed || self.gpu_view_info.is_none();
        if view_resolution_changed {
            self.set_surface_config(latest_resolution, msaa_samples, msaa_texture);
            return GpuHandleUpdateInfo::UpdatedViewResolution;
        }
        GpuHandleUpdateInfo::NoOp
    }
    fn set_surface_config(
        &mut self,
        resolution: ViewResolution,
        msaa_samples: u32,
        msaa_texture: &mut Option<wgpu::TextureView>
    ) {
        assert!(resolution.width() > 0);
        assert!(resolution.height() > 0);
        let mut gpu_view_info = self.gpu_view_info
            .take()
            .unwrap_or_else(|| {
                let surface_desc = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: GpuHandle::TEXTURE_FORMAT,
                    width: resolution.width(),
                    height: resolution.height(),
                    present_mode: wgpu::PresentMode::AutoVsync,
                    // alpha_mode: wgpu::CompositeAlphaMode::PostMultiplied,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                };
                GpuViewInfo {
                    surface_desc,
                    view_resolution: resolution,
                }
            });
        gpu_view_info.surface_desc.format = GpuHandle::TEXTURE_FORMAT;
        gpu_view_info.surface_desc.width = resolution.width();
        gpu_view_info.surface_desc.height = resolution.height();
        self.surface.configure(&self.device, &gpu_view_info.surface_desc);
        if msaa_samples > 1 {
            *msaa_texture = Some(
                self.device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("Multisampled frame descriptor"),
                        size: wgpu::Extent3d {
                            width: resolution.width(),
                            height: resolution.height(),
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: msaa_samples,
                        dimension: wgpu::TextureDimension::D2,
                        format: GpuHandle::TEXTURE_FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    })
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
        }
        self.gpu_view_info = Some(gpu_view_info);
    }
}
