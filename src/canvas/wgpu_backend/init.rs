use crate::canvas::{CanvasRenderer, UpdateStatus};
use super::CanvasLayer;
use super::gpu_target::{GpuBackend, GpuHandle};
use crate::data::{Content, TessellatedContent};
use crate::data::gpu_types;
use crate::data::collections::CowCollection;
use crate::frontend::SceneTessellator;
use wgpu::util::DeviceExt;


macro_rules! define_canvas_renderer_for_layer {
    ($N:tt, [$($E:tt),*]) => {
        impl CanvasRenderer<$N> {
            fn new(backend: impl GpuBackend) -> CanvasRenderer<1> {
                let gpu_handle = GpuHandle::new(backend);
                let layers: [Option<CanvasLayer>; $N] = [$($E),*];
                unimplemented!();
                // CanvasRenderer {
                //     gpu_handle,
                //     layers: unimplemented!()
                // }
            }
        }
    };
}

define_canvas_renderer_for_layer!(1, [None]);
define_canvas_renderer_for_layer!(2, [None, None]);
define_canvas_renderer_for_layer!(3, [None, None, None]);
define_canvas_renderer_for_layer!(4, [None, None, None, None]);

impl<const N: usize> CanvasRenderer<N> {
    
}



impl CanvasLayer {
    pub fn init(
        handle: &GpuHandle,
        scene_tessellator: SceneTessellator,
    ) -> Self {
        //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
        // STATE
        //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
        let vbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&scene_tessellator.mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ibo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&scene_tessellator.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let prim_buffer_byte_size = (scene_tessellator.primitives.len() * std::mem::size_of::<gpu_types::GpuPrimitive>()) as u64;
        let prims_ssbo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Prims ssbo"),
            contents: bytemuck::cast_slice(&scene_tessellator.primitives),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });
        let picture_viewport = scene_tessellator.picture_resolution;
        let globals_buffer_byte_size = std::mem::size_of::<gpu_types::GpuGlobals>() as u64;
        let globals_ubo = handle.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals ubo"),
            contents: bytemuck::cast_slice(&[gpu_types::GpuGlobals {
                picture_resolution: [picture_viewport.width(), picture_viewport.height()],
                _pad: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
        // PIPELINE
        //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
        let msaa_samples = handle.msaa_samples;
        
        let bind_group_layout = handle.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
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
                    resource: wgpu::BindingResource::Buffer(globals_ubo.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(prims_ssbo.as_entire_buffer_binding()),
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
                module: &handle.vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<gpu_types::GpuVertex>() as wgpu::BufferAddress,
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
                module: &handle.fs_module,
                entry_point: "main",
                targets: fragment_targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                strip_index_format: None,
                cull_mode: None,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        };
        let render_pipeline = handle.device.create_render_pipeline(&render_pipeline_descriptor);
        // TODO: this isn't what we want: we'd need the equivalent of VK_POLYGON_MODE_LINE,
        // but it doesn't seem to be exposed by wgpu?
        render_pipeline_descriptor.primitive.polygon_mode = wgpu::PolygonMode::Line;
        let wireframe_render_pipeline = handle.device.create_render_pipeline(&render_pipeline_descriptor);
        //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
        // DONE
        //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
        let context: CanvasLayer = CanvasLayer {
            scene_tessellator,
            ibo, 
            vbo, 
            prim_buffer_byte_size, 
            prims_ssbo, 
            globals_buffer_byte_size, 
            globals_ubo, 
            bind_group_layout, 
            bind_group, 
            render_pipeline, 
            wireframe_render_pipeline,
            msaa_samples,
            msaa_texture: None,
            provisioned: false,
        };
        unimplemented!()
    }
}



