use super::UpdateHint;
use crate::data::{self, Resolution, ViewInfo, View};
use crate::data::{Picture, TesselationSettings, TessellatedPicture};
use crate::{GpuBackend, GpuContext};
use crate::renderer::{RendererContext, GpuHandle};
use crate::renderer::state::RendererState;

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
// RENDERER-PIPELINE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


pub(crate) struct Shaders {

}

pub(crate) struct RendererPipeline {
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: wgpu::ShaderModule,
    pub wireframe_render_pipeline: wgpu::RenderPipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// INIT RENDERER-PIPELINE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererPipeline {
    pub fn new(handle: &GpuHandle, state: &RendererState, msaa_samples: u32) -> RendererPipeline {
        let vs_module = handle.device.create_shader_module(include_wgsl!("./../../shaders/geometry.vs.wgsl"));
        let fs_module = handle.device.create_shader_module(include_wgsl!("./../../shaders/geometry.fs.wgsl"));
        let bind_group_layout = handle.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        // min_binding_size: wgpu::BufferSize::new(state.globals_buffer_byte_size),
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        // min_binding_size: wgpu::BufferSize::new(state.prim_buffer_byte_size),
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = handle.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(state.globals_ubo.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(state.prims_ssbo.as_entire_buffer_binding()),
                },
            ],
        });
        let pipeline_layout = handle.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
            label: None,
        });
        let fragment_targets = &[Some(wgpu::ColorTargetState {
            format: GpuHandle::TEXTURE_FORMAT,
            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let mut render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<data::gpu_types::GpuVertex>() as wgpu::BufferAddress,
                    // array_stride: std::mem::size_of::<data::gpu_types::GpuVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x2,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            format: wgpu::VertexFormat::Uint32,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: fragment_targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                strip_index_format: None,
                cull_mode: None,
                // cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                // alpha_to_coverage_enabled: false,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        };
        let render_pipeline = handle.device.create_render_pipeline(&render_pipeline_descriptor);
        // TODO: this isn't what we want: we'd need the equivalent of VK_POLYGON_MODE_LINE,
        // but it doesn't seem to be exposed by wgpu?
        render_pipeline_descriptor.primitive.polygon_mode = wgpu::PolygonMode::Line;
        let wireframe_render_pipeline = handle.device.create_render_pipeline(&render_pipeline_descriptor);
        RendererPipeline {
            vs_module,
            fs_module,
            wireframe_render_pipeline,
            render_pipeline,
            bind_group_layout,
            bind_group,
        }
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// UPDATE RENDERER-PIPELINE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl RendererPipeline {
    pub fn update(&mut self, handle: &GpuHandle, state: &RendererState) {

    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// HELPERS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


impl RendererPipeline {
    pub fn needs_update(&self, view: &View) -> UpdateHint {
        UpdateHint::MaybeNeedsUpdate
    }
}





