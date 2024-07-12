use std::{error::Error, fs, io::{self, Read}, path::Path, fmt::Debug};
use glam::{UVec3, UVec4, Vec3, Vec4};
use wgpu::{Device, Extent3d, Origin3d, Queue, Texture, TextureDescriptor, TextureView};

pub const VOXEL_COMPONENTS: [&str; 1] = ["albedo"];

pub struct ChunkContent {
    pub dimensions: UVec3,

    pub albedo: Vec<u8>,

    pub albedo_texture: Texture,
    pub albedo_view: TextureView,
}

impl ChunkContent {
    pub fn from_raw_data(device: &Device, queue: &Queue, albedo: Vec<u8>, dimensions: UVec3) -> Result<Self, ChunkContentLoadingError> {
        if dimensions.x == 0 || dimensions.y == 0 || dimensions.z == 0 {
            return Err(ChunkContentLoadingError::InvalidDimensions);
        }

        let albedo_texture = device.create_texture(&TextureDescriptor {
                size: Extent3d { width: dimensions.x, height: dimensions.y, depth_or_array_layers: dimensions.z },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING |  wgpu::TextureUsages::COPY_DST,
                label: Some("Chunk albedo"),
                view_formats: &[] 
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &albedo_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &albedo.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(dimensions.x * 4),
                rows_per_image: Some(dimensions.y)
            },
            Extent3d { width: dimensions.x, height: dimensions.y, depth_or_array_layers: dimensions.z }
        );

        let albedo_view = albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());
        

        Ok(Self { 
            dimensions, 
            albedo, 
            albedo_texture,
            albedo_view
        })
    }

    pub fn from_chunk_file(device: &Device, queue: &Queue, path: &Path) -> Result<Self, ChunkContentLoadingError> {
        let chunk_content_file = match fs::File::open(path) {
            Ok(file) => file,
            Err(err) => return Err(ChunkContentLoadingError::FailedToReadChunkFile(err))
        };

        let mut chunk_content_zip = match zip::ZipArchive::new(chunk_content_file) {
            Ok(arch) => arch,
            Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile)
        };

        let first_image_bytes: Vec<u8> = match chunk_content_zip.by_name("0") {
            Ok(image0) => {
                let raw_bytes: Vec<Result<u8, io::Error>> = image0.bytes().collect();
                let mut bytes = Vec::<u8>::with_capacity(raw_bytes.len());
                for byte in raw_bytes {
                    match byte {
                        Ok(byte) => bytes.push(byte),
                        Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile),
                    }
                }

                bytes
            },
            Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile),
        };

        let first_image = match image::load_from_memory(first_image_bytes.as_slice()) {
            Ok(im) => im,
            Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile),
        };

        let dimensions = UVec3::new(first_image.width(), first_image.height(), chunk_content_zip.len() as u32);
        
        let mut albedo_bytes :Vec<u8> = Vec::with_capacity((dimensions.x * dimensions.y * dimensions.z * 4) as usize);
        
        albedo_bytes.extend(match first_image.as_rgba8() {
            Some(rgba) => rgba.iter(),
            None =>  return Err(ChunkContentLoadingError::InvalidChunkFile)
        });

        for i in 1..dimensions.z {
            let image_bytes: Vec<u8> = match chunk_content_zip.by_name(&format!("{}", i)) {
                Ok(image) => {
                    let raw_bytes: Vec<Result<u8, io::Error>> = image.bytes().collect();
                    let mut bytes = Vec::<u8>::with_capacity(raw_bytes.len());
                    for byte in raw_bytes {
                        match byte {
                            Ok(byte) => bytes.push(byte),
                            Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile),
                        }
                    }
    
                    bytes
                },
                Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile),
            };
    
            let image = match image::load_from_memory(image_bytes.as_slice()) {
                Ok(im) => im,
                Err(_) => return Err(ChunkContentLoadingError::InvalidChunkFile),
            };
    
            albedo_bytes.extend(match image.as_rgba8() {
                Some(rgba) => rgba.iter(),
                None =>  return Err(ChunkContentLoadingError::InvalidChunkFile)
            });
        }
        
        ChunkContent::from_raw_data(device, queue, albedo_bytes, dimensions)
    }
}

pub enum ChunkContentLoadingError {
    InvalidDimensions,
    InvalidChunkFile,
    FailedToReadChunkFile(std::io::Error),
}

impl Debug for ChunkContentLoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimensions => write!(f, "InvalidDimensions"),
            Self::InvalidChunkFile => write!(f, "InvalidChunkFile"),
            Self::FailedToReadChunkFile(arg0) => f.debug_tuple("FailedToReadChunkFile").field(arg0).finish(),
        }
    }
}
