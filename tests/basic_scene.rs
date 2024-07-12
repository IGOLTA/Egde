mod simple_camera_controller;

use std::{f32::consts, path::{Path, PathBuf}};

use egde::{scene::{camera::CameraData, chunk::{ChunkData, UnloadedChunk}, Scene, UnloadedScene}, Game, GameConfig};
use glam::{EulerRot, Quat, Vec3};

#[test]
fn basic_scene_test() {
    env_logger::init();
    
    let camera_data = CameraData {
        position: Vec3::new(0., 0., 0.),
        near: 0.01,
        far: 100.0,
        fov: 100.0 * consts::PI / 180.0,
    };

    let mut scene = UnloadedScene::new(camera_data);

    scene.add_chunk(UnloadedChunk{
        content_path: PathBuf::from("C:/Users/igolt/Desktop/T-Rex.zip"),
        chunk_data: ChunkData {
            position: Vec3::new(0., 0., 0.),
            rotation: Quat::from_euler(EulerRot::XYZ, 0., 0., 0.),
        },
    });

    scene. add_script(Box::new(simple_camera_controller::CameraController{}));

    let game_config = GameConfig {
        game_name: "Basic scene".to_string(),
        window_width: 720,
        window_height: 480,
        render_scale: 1,
    };

    let mut game =pollster::block_on(Game::new(game_config));
    
    game.load_scene(scene).unwrap();
    game.launch();
}