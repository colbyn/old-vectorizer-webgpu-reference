use crate::canvas::{CanvasRenderer, UpdateStatus};
use super::CanvasLayer;
use crate::data::draw_cmds::DrawOp;
use super::gpu_target::{GpuBackend, GpuHandle};
use crate::frontend::DrawableObject;
use crate::data::{Content, TessellatedContent};
use crate::data::gpu_types;
use crate::data::collections::CowCollection;
use wgpu::util::DeviceExt;


#[derive(Debug, Clone)]
pub(crate) struct UpdateResult {
    hot_needs_update: bool,
    cold_needs_update: bool,
}

impl<const N: usize> CanvasRenderer<N> {
    
}


// impl HotLayer {
//     fn update_scene<T: DrawableObject>(&mut self, scene: impl IntoIterator<Item = T>) -> UpdateStatus {
//         unimplemented!()
//     }
// }

// impl ColdLayer {
//     fn update_scene<T: DrawableObject>(&mut self, scene: impl IntoIterator<Item = T>) -> UpdateStatus {
//         unimplemented!()
//     }
// }

impl CanvasLayer {
    pub fn update(
        &mut self,
        handle: &GpuHandle,
        scene: TessellatedContent,
    ) -> UpdateStatus {
        // self.tessellated_content = content.to_owned_content().tessellate();
        // self.vbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: None,
        //     contents: bytemuck::cast_slice(&self.tessellated_content.mesh.vertices),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });
        // self.ibo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: None,
        //     contents: bytemuck::cast_slice(&self.tessellated_content.mesh.indices),
        //     usage: wgpu::BufferUsages::INDEX,
        // });
        unimplemented!()
    }
    // pub fn update(
    //     &mut self,
    //     handle: &GpuHandle,
    //     content: ContentRef<'_, Item>
    // ) -> UpdateStatus where Item: ItemConstraints {
    //     if !self.needs_update(&content) {
    //         return UpdateStatus::Unchanged
    //     }
    //     self.tessellated_content = content.to_owned_content().tessellate();
    //     self.vbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: None,
    //         contents: bytemuck::cast_slice(&self.tessellated_content.mesh.vertices),
    //         usage: wgpu::BufferUsages::VERTEX,
    //     });
    //     self.ibo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: None,
    //         contents: bytemuck::cast_slice(&self.tessellated_content.mesh.indices),
    //         usage: wgpu::BufferUsages::INDEX,
    //     });
    //     unimplemented!()
    // }
}

