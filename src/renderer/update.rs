use super::RendererContext;
use super::{GpuHandle, state::RendererState, pipeline::RendererPipeline};
use super::UpdateHint;
use crate::GpuViewInfo;
use crate::{GpuBackend, GpuContext};
use crate::data::{Picture, TesselationSettings, TessellatedPicture, View, ViewResolution};
use crate::data::{self, Resolution, ViewInfo};

use std::borrow::Cow;
use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};
use wgpu::include_wgsl;
use futures::executor::block_on;
use wgpu::util::DeviceExt;
use std::f64::NAN;

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// UPDATE RENDERER-CONTEXT
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererContext {
    pub fn update(&mut self, latest_view: View) {
        let _ = self.gpu_handle.update(latest_view.view_resolution, self.msaa_samples, &mut self.msaa_texture);
        let (mut state_hint, mut renderer_state) = self.state.take()
            .map(|x| (x.needs_update(&latest_view), x))
            .unwrap_or_else(|| {
                let state = RendererState::new(&self.gpu_handle, latest_view.clone());
                (UpdateHint::NoUpdate, state)
            });
        let (mut pipeline_hint, mut renderer_pipeline) = self.pipeline
            .take()
            .map(|x| (x.needs_update(&latest_view), x))
            .unwrap_or_else(|| {
                let pipeline = RendererPipeline::new(&self.gpu_handle, &renderer_state, self.msaa_samples);
                (UpdateHint::NoUpdate, pipeline)
            });
        match (state_hint, pipeline_hint) {
            (UpdateHint::NoUpdate, UpdateHint::NoUpdate) => {}
            (UpdateHint::MaybeNeedsUpdate, _) => {
                // renderer_state.update(&self.gpu_handle, &latest_view);
                // renderer_pipeline.update(&self.gpu_handle, &renderer_state);
            }
            (UpdateHint::NoUpdate, UpdateHint::MaybeNeedsUpdate) => {
                unimplemented!("[RendererContext.update] Is this possible?")
            }
        }
        self.state = Some(renderer_state);
        self.pipeline = Some(renderer_pipeline);
        // self.current_view = Some(latest_view);
        // self.smaa_target = Some(smaa_target);
        assert!(self.all_resources_exist());
    }
}



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// INTERNAL HELEPRS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
