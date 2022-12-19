pub mod init;
pub mod update;
pub mod draw;
pub mod helpers;
pub mod gpu_target;

use std::marker::PhantomData;

use crate::data::{ViewResolution, PictureResolution, Picture, TessellatedPicture};
use crate::data::{Content, TessellatedContent};
use crate::data::draw_cmds::{DrawOp, FillOp, FillStrokeOp, StrokeOp};
use crate::frontend::DrawableObject;
use crate::frontend::SceneTessellator;

pub struct WgpuBackend<const N: usize> {
    pub gpu_handle: gpu_target::GpuHandle,
    pub layers: [Option<CanvasLayer>; N],
}

pub struct CanvasLayer  {
    pub scene_tessellator: SceneTessellator,
    pub ibo: wgpu::Buffer,
    pub vbo: wgpu::Buffer,
    pub prim_buffer_byte_size: u64,
    pub prims_ssbo: wgpu::Buffer,
    pub globals_buffer_byte_size: u64,
    pub globals_ubo: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub wireframe_render_pipeline: wgpu::RenderPipeline,
    pub msaa_texture: Option<wgpu::TextureView>,
    pub msaa_samples: u32,
    /// We need to write GPU buffers at least once.
    pub provisioned: bool,
}
