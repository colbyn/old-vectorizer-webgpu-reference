use super::{RendererContext, GpuHandle, state::RendererState, pipeline::RendererPipeline};
use super::UpdateHint;
use crate::{GpuBackend, GpuContext};

use std::borrow::Cow;
use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};
use wgpu::include_wgsl;
use futures::executor::block_on;
use wgpu::util::DeviceExt;
use std::f64::NAN;



impl RendererContext {
    pub fn new(backend: impl GpuBackend) -> Self {
        let gpu_handle = GpuHandle::new(backend);
        RendererContext {
            msaa_samples: gpu_handle.msaa_samples,
            gpu_handle,
            state: None,
            pipeline: None,
            msaa_texture: None,
        }
    }
}

