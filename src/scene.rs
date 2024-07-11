use std::collections::LinkedList;

use camera::{Camera, CameraData};
use chunk::Chunk;
use glam::Vec3;
use sdl2::{keyboard::Scancode, EventPump};
use wgpu::{CommandEncoder, Device, Queue};

use crate::render::{chunk_renderer::{self, ChunkRenderer}, g_buffer::GBuffer};

pub mod chunk;
pub mod camera;

pub const VOXEL_SIZE: f32 = 0.1; 

pub struct Scene {
    camera: Camera,
    chunks: LinkedList<Chunk>,
}

impl Scene {
    pub fn new(device: &Device, camera_data: CameraData) -> Self {
        Self {
            chunks: LinkedList::new(),
            camera: Camera::new(device, camera_data)
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push_back(chunk);
    }

    pub fn render(&self, chunk_renderer: &ChunkRenderer, device: &Device, g_buffer: &GBuffer, encoder: &mut CommandEncoder) {
        for chunk in self.chunks.iter() {
            chunk_renderer.render(encoder, device, g_buffer, chunk, &self.camera);
        }
    }

    pub fn update(& mut self, queue: &Queue, delta_time: f32, event_pump: &EventPump) {
        let mut move_direction = Vec3::ZERO;
        let speed: f32 = 5.;
        let dash_speed: f32 = 8.;

        let pressed_keys: Vec<Scancode> = event_pump
        .keyboard_state()
        .scancodes()
        .into_iter()
        .filter(|(_, pressed)| *pressed)
        .map(|(scan_code, _)| scan_code)
        .collect();
    

        for key in pressed_keys {
            match key {
                Scancode::W => move_direction += Vec3::new(0., 0., 1.),
                Scancode::S => move_direction += Vec3::new(0., 0., -1.),
                Scancode::A => move_direction += Vec3::new(-1., 0., 0.),
                Scancode::D => move_direction += Vec3::new(1., 0., 0.),
                Scancode::Space => move_direction += Vec3::new(0., 1., 0.),
                Scancode::LCtrl => move_direction += Vec3::new(0., -1., 0.),
                _ => {}
            }
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::LShift) {
            move_direction *= dash_speed;
        } else {
            move_direction *= speed;
        }

        move_direction *= delta_time;

        self.camera.move_towards(move_direction);
        self.camera.update_uniform_buffer(queue);
    }
}