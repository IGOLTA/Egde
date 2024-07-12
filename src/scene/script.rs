use std::collections::HashMap;

use sdl2::EventPump;
use uuid::Uuid;
use wgpu::Queue;

use super::{camera::Camera, chunk::Chunk, Scene};

pub trait Script {
    fn setup(&mut self, scene: &mut Scene);
    fn update(&mut self, chunks: &mut HashMap<Uuid, Chunk>, camera: &mut Camera, delta_time: f32, event_pump: &EventPump, queue: &Queue);
}