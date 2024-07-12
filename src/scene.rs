use std::collections::{HashMap, HashSet, LinkedList};

use camera::{Camera, CameraData};
use chunk::{chunk_content::ChunkContentLoadingError, Chunk, UnloadedChunk};
use glam::Vec3;
use script::Script;
use sdl2::{keyboard::Scancode, EventPump};
use uuid::Uuid;
use wgpu::{core::device::queue, CommandEncoder, Device, Queue};

use crate::{render::{chunk_renderer::{self, ChunkRenderer}, g_buffer::GBuffer}};

pub mod script;
pub mod chunk;
pub mod camera;

pub const VOXEL_SIZE: f32 = 0.1; 

pub struct Scene {
    camera: Camera,
    chunks: HashMap<Uuid, Chunk>,
    scripts: HashMap<(), Box<dyn Script>>
}

impl Scene {
    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(Uuid::new_v4(), chunk);
    }

    pub fn add_script(&mut self, script: Box<dyn Script>) {
        self.scripts.insert((), script);
    }

    pub fn render(&self, chunk_renderer: &ChunkRenderer, device: &Device, g_buffer: &GBuffer, encoder: &mut CommandEncoder) {
        for (_uuid, chunk) in self.chunks.iter() {
            chunk_renderer.render(encoder, device, g_buffer, chunk, &self.camera);
        }
    }

    pub fn update(& mut self, queue: &Queue, delta_time: f32, event_pump: &EventPump) {
        for script in self.scripts.values_mut().into_iter() {
            script.update(&mut self.chunks, &mut self.camera, delta_time, event_pump, queue);
        }
    }
}


pub struct UnloadedScene {
    chunks: HashMap<Uuid, UnloadedChunk>,
    camera_data: CameraData,
    scripts: HashMap<(), Box<dyn Script>>
}

impl UnloadedScene {
    pub fn new(camera_data: CameraData) -> Self {
        Self{
            chunks: HashMap::new(),
            camera_data: camera_data,
            scripts: HashMap::new()
        }
    }

    pub fn load(self, device: &Device, queue: &Queue, aspect_ratio:f32) -> Result<Scene, ChunkContentLoadingError> {
        let mut chunks = HashMap::<Uuid, Chunk>::new();
        for (uuid, chunk) in self.chunks {
            chunks.insert(uuid, chunk.load(device, queue)?);
        }

        Ok(Scene {
            chunks,
            camera: Camera::new(device, self.camera_data, aspect_ratio),
            scripts: self.scripts
        })
    }

    pub fn add_chunk(&mut self, chunk: UnloadedChunk) {
        self.chunks.insert(Uuid::new_v4(), chunk);
    }

    pub fn add_script(&mut self, script: Box<dyn Script>) {
        self.scripts.insert((), script);
    }
}