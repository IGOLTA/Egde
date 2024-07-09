use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureView};

use crate::GameConfig;

pub const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

pub struct GBuffer {
    albedo: Texture,
    pub albedo_texture_view: TextureView,
}

impl GBuffer {
    pub fn new(device: &Device, config: GameConfig) -> Self {
        let albedo = device.create_texture(&TextureDescriptor { 
            label: Some("GBuffer albedo"), 
            size: Extent3d {
                width: config.render_width,
                height: config.render_height,
                depth_or_array_layers: 1,
            }, 
            mip_level_count: 1, 
            sample_count: 1, 
            dimension: TextureDimension::D2, 
            format: FORMAT, 
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, 
            view_formats: &[]
        });

        let albedo_texture_view = albedo.create_view(&wgpu::TextureViewDescriptor::default());
    
        GBuffer {
            albedo,
            albedo_texture_view
        }
    }
}