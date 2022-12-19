use super::{RendererContext, pipeline};
use super::{GpuHandle, state::RendererState, pipeline::RendererPipeline};
use super::UpdateHint;
use crate::{GpuBackend, GpuContext, ViewResolution, PictureResolution};
use crate::data::{Resolution, ViewInfo, View};
use crate::frontend::DrawableObject;
// use crate::frontend::{DrawCmd, DrawCmdRef};

use either::{Either, Either::Left, Either::Right};
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
    // pub fn draw_hot<Hot>(&mut self, items: T) where Hot: DrawableObject {
        
    // }
}

impl RendererContext {
    pub fn draw_view(&mut self, latest_view: View) {
        self.update(latest_view);
        assert!(self.all_resources_exist());
        let state = self.state
            .as_ref()
            .unwrap()
            .write_all_buffers_to_queue(&self.gpu_handle);
        let frame = match self.gpu_handle.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                println!("Swap-chain error: {:?}", e);
                return;
            }
        };
        let frame_view_descriptor = wgpu::TextureViewDescriptor{
            format: Some(GpuHandle::TEXTURE_FORMAT),
            ..wgpu::TextureViewDescriptor::default()
        };
        let frame_view = frame.texture.create_view(&frame_view_descriptor);
        let mut encoder = self.gpu_handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });
        self.execute_render_pass(&frame_view, &mut encoder);
        self.gpu_handle.queue.submit(Some(encoder.finish()));
        frame.present();
        assert!(self.all_resources_exist());
    }
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// RENDER PASS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererContext {
    pub fn execute_render_pass(
        &self,
        frame_view: &wgpu::TextureView,
        // frame_view: &smaa::SmaaFrame,
        encoder: &mut wgpu::CommandEncoder
    ) {
        assert!(self.all_resources_exist());
        let has_msaa_texture = self.gpu_handle.gpu_view_info
            .as_ref()
            .map(|x| self.msaa_texture.is_some())
            .unwrap_or(false);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: {
                    self.msaa_texture.as_ref().unwrap_or(&frame_view)
                },
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: if has_msaa_texture {
                    Some(&frame_view)
                } else {
                    None
                },
            })],
            depth_stencil_attachment: None,
        });
        let pipeline = self.pipeline.as_ref().unwrap();
        let state = self.state.as_ref().unwrap();
        if self.gpu_handle.wireframe {
            pass.set_pipeline(&pipeline.wireframe_render_pipeline);
        } else {
            pass.set_pipeline(&pipeline.render_pipeline);
        }
        pass.set_bind_group(0, &pipeline.bind_group, &[]);
        pass.set_index_buffer(state.ibo.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, state.vbo.slice(..));
        pass.set_blend_constant(wgpu::Color::TRANSPARENT);
        // ----------------------------------------------------------------
        // NOTE: DRAW
        // ----------------------------------------------------------------
        pass.draw_indexed(0..(state.tessellated_picture.mesh.indices.len() as u32), 0, 0..1);
    }
}


