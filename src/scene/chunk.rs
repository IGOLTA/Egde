use glam::{DQuat, Mat4, Quat, UVec3, Vec3};
use wgpu::{ util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupLayout, Buffer, BufferUsages, Device, Queue};

pub struct Chunk {
    pub data: ChunkData,
    pub buffer: Buffer,
}

impl Chunk {
    pub fn new(device: &Device, data: ChunkData) -> Chunk {
        let buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Chunk buffer"),
            contents: unsafe { crate::memory::any_as_u8_slice(&ChunkUniform::from_data(data)) },
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self {
            data,
            buffer
        }
    }

    pub fn update_uniform_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, unsafe { crate::memory::any_as_u8_slice(&ChunkUniform::from_data(self.data)) })
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
                }
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
    pub size: UVec3,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct ChunkUniform {
    pub size: UVec3,
    pub transform: Mat4,
    pub invert_transform: Mat4,
}

impl ChunkUniform {
    fn from_data(data: ChunkData) -> Self {
        let scale = data.size.as_vec3() * super::VOXEL_SIZE;
        let transform =Mat4::from_scale_rotation_translation(scale, data.rotation, data.position); 
        ChunkUniform {
            size: data.size,
            transform,
            invert_transform: transform.inverse()
        }
    }

}