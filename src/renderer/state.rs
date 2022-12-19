use crate::data::{self, Resolution, ViewInfo, View, gpu_types};
use crate::data::{Picture, TesselationSettings, TessellatedPicture};
use crate::{GpuBackend, GpuContext};
use crate::renderer::{RendererContext, GpuHandle};
use super::UpdateHint;

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
// RENDERER-STATE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub(crate) struct RendererState {
    pub ibo: wgpu::Buffer,
    pub vbo: wgpu::Buffer,
    pub prim_buffer_byte_size: u64,
    pub prims_ssbo: wgpu::Buffer,
    pub globals_buffer_byte_size: u64,
    pub globals_ubo: wgpu::Buffer,
    pub tessellated_picture: TessellatedPicture,
    // pub smaa_target: smaa::SmaaTarget,
}

// pub(crate) struct RendererBuffers {
//     pub ibo: wgpu::Buffer,
//     pub vbo: wgpu::Buffer,
//     pub prim_buffer_byte_size: u64,
//     pub prims_ssbo: wgpu::Buffer,
//     pub globals_buffer_byte_size: u64,
//     pub globals_ubo: wgpu::Buffer,
//     pub tessellated_picture: TessellatedPicture,
// }


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// INIT RENDERER-STATE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererState {
    pub fn new(handle: &GpuHandle, view: View) -> RendererState {
        let tessellated_picture = view.picture.tessellate();
        let vbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&tessellated_picture.mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ibo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&tessellated_picture.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let prim_buffer_byte_size = (tessellated_picture.primitives.len() * std::mem::size_of::<data::gpu_types::GpuPrimitive>()) as u64;
        let prims_ssbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Prims ssbo"),
            contents: bytemuck::cast_slice(&tessellated_picture.primitives),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });
        let picture_viewport = tessellated_picture.picture_resolution;
        let globals_buffer_byte_size = std::mem::size_of::<data::gpu_types::GpuGlobals>() as u64;
        let globals_ubo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals ubo"),
            contents: bytemuck::cast_slice(&[gpu_types::GpuGlobals {
                picture_resolution: [picture_viewport.width(), picture_viewport.height()],
                _pad: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        RendererState {
            globals_ubo,
            ibo,
            vbo,
            prims_ssbo,
            prim_buffer_byte_size,
            globals_buffer_byte_size,
            tessellated_picture,
        }
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// UPDATE RENDERER-STATE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererState {
    pub fn update(&mut self, handle: &GpuHandle, view: &View) {
        self.tessellated_picture = view.picture.tessellate();
        self.vbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.tessellated_picture.mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.ibo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.tessellated_picture.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        // self.prim_buffer_byte_size = (self.tessellated_picture.primitives.len() * std::mem::size_of::<data::gpu_types::GpuPrimitive>()) as u64;
        // self.prims_ssbo = handle.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Prims ssbo"),
        //     size: self.prim_buffer_byte_size,
        //     usage: wgpu::BufferUsages::VERTEX
        //         | wgpu::BufferUsages::STORAGE
        //         | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// RENDERER-STATE HELPERS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


impl RendererState {
    pub fn needs_update(&self, view: &View) -> UpdateHint {
        UpdateHint::MaybeNeedsUpdate
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// WRITE RENDERER-STATE BUFFERS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererState {
    pub fn write_all_buffers_to_queue(&self, handle: &GpuHandle) {
        // handle.queue.write_buffer(&self.prims_ssbo, 0, bytemuck::cast_slice(&self.tessellated_picture.primitives));
        let picture_viewport = self.tessellated_picture.picture_resolution;
        // handle.queue.write_buffer(
        //     &self.globals_ubo,
        //     0,
        //     bytemuck::cast_slice(&[gpu_types::GpuGlobals {
        //         picture_resolution: [picture_viewport.width, picture_viewport.height],
        //         _pad: 0.0,
        //     }]),
        // );
    }
}


