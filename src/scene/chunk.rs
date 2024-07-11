mod chunk_content;

use std::path::Path;

use chunk_content::{ChunkContent, ChunkContentLoadingError};
use glam::{Mat4, Quat, UVec3, Vec3};
use wgpu::{ util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupLayout, Buffer, BufferUsages, Device, Queue, Sampler};

pub struct Chunk {
    pub data: ChunkData,

    chunk_content: ChunkContent,

    buffer: Buffer,

    sampler: Sampler,
}

impl Chunk {
    pub fn from_file(device: &Device, queue: &Queue, data: ChunkData, content_path: &Path) -> Result<Chunk, ChunkContentLoadingError> {
        let chunk_content = ChunkContent::from_chunk_file(device, queue, content_path)?;
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Chunk buffer"),
            contents: unsafe { crate::memory::any_as_u8_slice(&ChunkUniform::from_data_and_dimensions(data, chunk_content.dimensions)) },
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Ok(Self {
            data,
            buffer,
            sampler,
            chunk_content
        })
    }

    pub fn update_uniform_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, unsafe { crate::memory::any_as_u8_slice(&ChunkUniform::from_data_and_dimensions(self.data, self.chunk_content.dimensions)) })
    }

    pub fn generate_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0, //Chunk uniform
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("chunk_renderer_bind_group_layout"),
        })
    }

    pub fn generate_bind_group(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {

        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(self.buffer.as_entire_buffer_binding())
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.chunk_content.albedo_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    }
                ],
                label: Some("chunk_render_bind_group"),
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]

pub struct ChunkData {
    pub position: Vec3,
    pub rotation: Quat,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct ChunkUniform {
    pub size: UVec3,
    pub transform: Mat4,
    pub invert_rotation: Mat4,
}

impl ChunkUniform {
    fn from_data_and_dimensions(data: ChunkData, dimensions: UVec3) -> Self {
        let scale = dimensions.as_vec3() * super::VOXEL_SIZE;
        let transform =Mat4::from_scale_rotation_translation(scale, data.rotation, data.position); 
        ChunkUniform {
            size: dimensions,
            transform,
            invert_rotation: Mat4::from_quat(-data.rotation),
        }
    }

}