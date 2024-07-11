use sdl2::keyboard::Scancode;

use crate::scene::camera::CameraData;

pub struct GameContext {
    pub pressed_keys: Vec<Scancode>
}