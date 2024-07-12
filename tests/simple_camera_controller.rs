use egde::scene::script::Script;
use glam::Vec3;
use sdl2::keyboard::Scancode;
use wgpu::Queue;

pub struct CameraController {
    
}

impl Script for CameraController {
    fn setup(&mut self, scene: &mut egde::scene::Scene) {
        
    }

    fn update(&mut self, chunks: &mut std::collections::HashMap<uuid::Uuid, egde::scene::chunk::Chunk>, camera: &mut egde::scene::camera::Camera, delta_time: f32, event_pump: &sdl2::EventPump, queue: &Queue) {
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

        camera.move_towards(move_direction);
        camera.update_uniform_buffer(queue);
    }
}