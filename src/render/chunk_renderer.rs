use std::mem;

use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BufferUsages, CommandEncoder, Device, PipelineLayoutDescriptor, RenderPipelineDescriptor};

use crate::scene::{camera::Camera, chunk::Chunk};

use super::g_buffer::{self, GBuffer};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32;3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        }
    }
}

const CHUNK_VERTICES: &[Vertex] = &[
    Vertex{position: [0., 0., 0.]},
    Vertex{position: [1., 0., 0.]},
    Vertex{position: [1., 1., 0.]},
    Vertex{position: [0., 1., 0.]},
    Vertex{position: [0., 0., 1.]},
    Vertex{position: [1., 0., 1.]},
    Vertex{position: [1., 1., 1.]},
    Vertex{position: [0., 1., 1.]},
];    

const CHUNK_INDICES: &[u16] = &[
    0, 1, 3, 3, 1, 2,
    1, 5, 2, 2, 5, 6,
    5, 4, 6, 6, 4, 7,
    4, 0, 7, 7, 0, 3,
    3, 2, 7, 7, 2, 6,
    4, 5, 0, 0, 5, 1
];

pub struct ChunkRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    chunk_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl ChunkRenderer {

    pub fn new(device: &Device) -> Self {
        let chunk_layout = Chunk::generate_bind_group_layout(device);
        let camera_layout = Camera::generate_bind_group_layout(device);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Chunk Renderer pipeline layout"),
            bind_group_layouts: &[&chunk_layout, &camera_layout],
            push_constant_ranges: &[],
        });

        Self {
            render_pipeline:  Self::generate_render_plane_pipeline(device, render_pipeline_layout),
            vertex_buffer: device.create_buffer_init(
                &BufferInitDescriptor {
                    label: Some("Chunk renderer vertex buffer"),
                    contents: bytemuck::cast_slice(CHUNK_VERTICES),
                    usage: BufferUsages::VERTEX
                }
            ),
            index_buffer: device.create_buffer_init(
                &BufferInitDescriptor{
                    label: Some("Chunk renderer index buffer"),
                    contents: bytemuck::cast_slice(CHUNK_INDICES),
                    usage: wgpu::BufferUsages::INDEX
                }
            ),
            chunk_bind_group_layout: chunk_layout,
            camera_bind_group_layout: camera_layout,
        }
    }


    fn generate_render_plane_pipeline(device: &Device, layout: wgpu::PipelineLayout) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/chunk_shader.wgsl"));
    
        device.create_render_pipeline(&RenderPipelineDescriptor{
            label: Some("Chunk renderer pipeline"),
            layout: Some(&layout),
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
                    format: g_buffer::FORMAT,
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

    pub fn render(&self, encoder: &mut CommandEncoder, device: &Device, g_buffer: &GBuffer, chunk: &Chunk, camera: &Camera) {
        let chunk_bind_group =  chunk.generate_bind_group(device, &self.chunk_bind_group_layout);
        let camera_bind_group = camera.generate_bind_group(device, &self.camera_bind_group_layout);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &g_buffer.albedo_texture_view,
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

        render_pass.set_bind_group(0, &chunk_bind_group, &[]);
        render_pass.set_bind_group(1, &camera_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(CHUNK_INDICES.len() as u32), 0, 0..1);        
    }
}

