use std::mem;

use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BufferUsages, CommandEncoder, Device, Extent3d, ImageSubresourceRange, PipelineLayoutDescriptor, RenderPipelineDescriptor, Sampler, SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureView};

use crate::memory;

use super::g_buffer::GBuffer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
pub const VERTICES: &[Vertex] = &[
    Vertex{position: [-1., -1.], uv:[0., 1.]},
    Vertex{position: [1., -1.], uv:[1., 1.]},
    Vertex{position: [-1., 1.], uv:[0., 0.]},
    Vertex{position: [-1., 1.], uv:[0., 0.]},
    Vertex{position: [1., -1.], uv:[1., 1.]},
    Vertex{position: [1., 1.], uv:[1., 0.]},
];    



pub struct RenderPlane {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    render_plane_bind_group_layout: wgpu::BindGroupLayout,
    sampler: Sampler,
}

impl RenderPlane {

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_plane_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("render_plane_bind_group_layout"),
        });         
        
        Self {
            render_pipeline:  Self::generate_render_plane_pipeline(device, config, &render_plane_bind_group_layout),
            vertex_buffer: device.create_buffer_init(
                &BufferInitDescriptor {
                    label: Some("Render plane vertex buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: BufferUsages::VERTEX
                }
            ),
            render_plane_bind_group_layout,
            sampler
        }
    }


    fn generate_render_plane_pipeline(device: &Device, config: &SurfaceConfiguration, layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render_plane.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Render plane pipeline layout"),
            bind_group_layouts: &[layout],
            push_constant_ranges: &[],
        });
    
        device.create_render_pipeline(&RenderPipelineDescriptor{
            label: Some("Render plane pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    pub fn render(&self, encoder: &mut CommandEncoder, device: &Device, render_view: &TextureView, g_buffer: &GBuffer) {
        let bind_group =  device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &self.render_plane_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&g_buffer.albedo_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    }
                ],
                label: Some("render_texture_bind_group"),
            }
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
        
    }
}

