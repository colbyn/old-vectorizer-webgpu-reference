use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuVertex {
    pub position: [f32; 2],
    pub prim_id: u32,
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuPrimitive {
    pub color: [f32; 4],
}

impl GpuPrimitive {
    pub fn from_u8_rgba(color: super::RGBA<u8>) -> Self {
        let super::RGBA{red, green, blue, alpha} = color.to_f32();
        GpuPrimitive {
            color: [red, green, blue, alpha],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuGlobals {
    pub picture_resolution: [f32; 2],
    pub _pad: f32,
}

impl FillVertexConstructor<GpuVertex> for super::VertexConstructor {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position().to_array(),
            prim_id: self.prim_id,
        }
    }
}

impl StrokeVertexConstructor<GpuVertex> for super::VertexConstructor {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position().to_array(),
            prim_id: self.prim_id,
        }
    }
}

unsafe impl bytemuck::Pod for GpuGlobals {}
unsafe impl bytemuck::Zeroable for GpuGlobals {}
unsafe impl bytemuck::Pod for GpuVertex {}
unsafe impl bytemuck::Zeroable for GpuVertex {}
unsafe impl bytemuck::Pod for GpuPrimitive {}
unsafe impl bytemuck::Zeroable for GpuPrimitive {}

