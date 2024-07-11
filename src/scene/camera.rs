use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupLayout, Buffer, BufferUsages, Device, Queue};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

pub struct Camera {
    pub data: CameraData,
    pub buffer: Buffer,
}

impl Camera {
    pub fn new(device: &Device, data: CameraData) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Camera buffer"),
            contents: unsafe { crate::memory::any_as_u8_slice(&CameraUniform::from_data(data)) },
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self {
            data,
            buffer
        }
    }

    pub fn update_uniform_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, unsafe { crate::memory::any_as_u8_slice(&CameraUniform::from_data(self.data)) })
    }

    pub fn generate_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0, //Camera uniform
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
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
                label: Some("camera_bind_group"),
            }
        )
    }

    pub fn move_towards(&mut self, move_direction: Vec3) {
        self.data.position +=  Vec4::new(move_direction.x, move_direction.y, move_direction.z, 1.0).xyz();
    }
}

#[derive(Debug, Copy, Clone)]

pub struct CameraData {
    pub position: Vec3,
    pub near: f32,
    pub far: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    pub position: Vec3,
    pub transform: Mat4,
}

impl CameraUniform {
    fn from_data(data: CameraData) -> Self {
        let translation = Mat4::from_translation(-data.position);
        let perspective = Mat4::perspective_lh(data.fov, data.aspect_ratio, data.near, data.far);

        let transform = OPENGL_TO_WGPU_MATRIX * perspective * translation;

        CameraUniform {
            position: data.position,
            transform,
        }
    }
}
