use crate::canvas::{CanvasRenderer, UpdateStatus};
use super::CanvasLayer;
use super::gpu_target::{GpuBackend, GpuHandle};
use crate::data::{Content, TessellatedContent};
use crate::data::gpu_types;
use crate::data::collections::CowCollection;
use wgpu::util::DeviceExt;



impl<const N: usize> CanvasRenderer<N> {
    fn draw(&mut self) {
        unimplemented!()
    }
}


// impl<Item> Layer<Item> {
//     pub fn draw(&mut self, handle: &GpuHandle, content: Content<'_, Item>) where Item: ItemConstraints {
//         let update_status = self.update(handle, content);
//         if update_status.changed() || !self.provisioned {
//             self.write_all_buffers_to_queue(handle);
//             self.provisioned = true;
//         }
//         let frame = match handle.surface.get_current_texture() {
//             Ok(frame) => frame,
//             Err(e) => {
//                 println!("Swap-chain error: {:?}", e);
//                 return;
//             }
//         };
//         let frame_view_descriptor = wgpu::TextureViewDescriptor{
//             format: Some(GpuHandle::TEXTURE_FORMAT),
//             ..wgpu::TextureViewDescriptor::default()
//         };
//         let frame_view = frame.texture.create_view(&frame_view_descriptor);
//         let mut encoder = handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//             label: Some("Encoder"),
//         });
//         self.execute_render_pass(&frame_view, &mut encoder, handle.wireframe);
//         handle.queue.submit(Some(encoder.finish()));
//         frame.present();
//         unimplemented!()
//     }
// }



// impl<Item> Layer<Item> {
//     pub fn execute_render_pass(
//         &self,
//         frame_view: &wgpu::TextureView,
//         encoder: &mut wgpu::CommandEncoder,
//         wireframe: bool,
//     ) where Item: ItemConstraints {
//         let has_msaa_texture = self.msaa_texture.is_some();
//         let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//             label: None,
//             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                 view: {
//                     self.msaa_texture.as_ref().unwrap_or(&frame_view)
//                 },
//                 ops: wgpu::Operations {
//                     load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
//                     store: true,
//                 },
//                 resolve_target: if has_msaa_texture {
//                     Some(&frame_view)
//                 } else {
//                     None
//                 },
//             })],
//             depth_stencil_attachment: None,
//         });
//         if wireframe {
//             pass.set_pipeline(&self.wireframe_render_pipeline);
//         } else {
//             pass.set_pipeline(&self.render_pipeline);
//         }
//         pass.set_bind_group(0, &self.bind_group, &[]);
//         pass.set_index_buffer(self.ibo.slice(..), wgpu::IndexFormat::Uint32);
//         pass.set_vertex_buffer(0, self.vbo.slice(..));
//         pass.set_blend_constant(wgpu::Color::TRANSPARENT);
//         // ----------------------------------------------------------------
//         // NOTE: DRAW
//         // ----------------------------------------------------------------
//         pass.draw_indexed(0..(self.tessellated_content.mesh.indices.len() as u32), 0, 0..1);
//     }
// }


// impl<Item> Layer<Item> {
//     pub fn write_all_buffers_to_queue(&self, handle: &GpuHandle) where Item: ItemConstraints {
//         handle.queue.write_buffer(&self.prims_ssbo, 0, bytemuck::cast_slice(&self.tessellated_content.primitives));
//         let picture_viewport = self.tessellated_content.picture.picture_resolution;
//         handle.queue.write_buffer(
//             &self.globals_ubo,
//             0,
//             bytemuck::cast_slice(&[gpu_types::GpuGlobals {
//                 picture_resolution: [picture_viewport.width(), picture_viewport.height()],
//                 _pad: 0.0,
//             }]),
//         );
//     }
// }
