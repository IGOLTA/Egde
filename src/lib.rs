#[deny(deprecated)]

pub mod script;
pub mod scene;
pub mod render;
pub mod memory;

use core::default::Default;
use std::path::Path;
use std::time::Instant;
use glam::{Quat, UVec3, Vec3};
use scene::camera::{Camera, CameraData};
use scene::chunk::{self, Chunk};
use scene::Scene;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

use crate::render::g_buffer::GBuffer;
use crate::render::chunk_renderer::ChunkRenderer;
use crate::render::render_plane::RenderPlane;

#[derive(Debug, Clone, Copy)]
struct GameConfig {
    render_width: u32,
    render_height: u32,
}

struct Game<'a> {
    config: GameConfig,
    event_pump: EventPump,
    surface: Surface<'a>,
    window: Window,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,

    render_plane: RenderPlane,
    chunk_renderer: ChunkRenderer,
    g_buffer: GBuffer,

    last_frame: Instant,

    scene: Scene,
}

impl Game<'_> {
    pub async fn new(config: GameConfig) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        
        let window = video_subsystem.window("Edge", 800, 600)
            .position_centered()
            .borderless()
            .build()
            .unwrap();
        
        sdl_context.mouse().show_cursor(false);
        sdl_context.mouse().set_relative_mouse_mode(true);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
           ..Default::default()
        });
    
        let surface = unsafe { 
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle { 
                raw_display_handle: window.raw_display_handle().unwrap(), 
                raw_window_handle: window.raw_window_handle().unwrap()
            }).unwrap() 
        };
    
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
    
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits:  wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();
    
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.size().0,
            height: window.size().1,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let event_pump = sdl_context.event_pump().unwrap();
        
        let g_buffer = GBuffer::new(&device, config);

        let render_plane = RenderPlane::new(&device, &surface_config);

        let chunk_renderer = ChunkRenderer::new(&device);

        let mut scene = Scene::new(&device, CameraData {
            position: Vec3::new(0., 0., 0.),
            near: 0.001,
            far: 100.,
            fov: std::f32::consts::PI * 100. / 180.,
            aspect_ratio: config.render_width as f32 / config.render_height as f32,
        });

        scene.add_chunk(Chunk::from_file(&device,&queue, chunk::ChunkData{
            position: Vec3::new(0., 0., 0.),
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0., 0., 0.),
        }, Path::new("C:/Users/igolt/Desktop/T-Rex.zip")).unwrap());

        Game {
            config,
            event_pump,
            surface,
            window,
            device,
            queue,
            surface_config,
            g_buffer,
            render_plane,
            chunk_renderer,
            scene: scene,
            last_frame: Instant::now(),
        }
    }

    fn update(& mut self) -> bool {
        let delta_time = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();

        //Poll events
        while let Some(event) = self.event_pump.poll_event() {
            match event {
                Event::Quit {..} => return false,
                Event::Window { win_event: WindowEvent::SizeChanged(width, height), .. } => {
                    self.resize(width, height); 
                }
                Event::KeyDown { scancode: Some(code), .. } => {
                    match code {
                        Scancode::Escape => return false,
                        _ => {}
                    }
                }
                Event::MouseMotion { xrel, yrel, ..} => {
                    let rotation_speed:f32 = 0.005;
                }
                _ => {}
            }

        }
        
        self.scene.update(&self.queue, delta_time, &self.event_pump);

        true
    }

    fn render(& self) -> Result<(), wgpu::SurfaceError>{
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.scene.render(&self.chunk_renderer, &self.device, &self.g_buffer, &mut encoder);

        self.render_plane.render(&mut encoder, &self.device, &view, &self.g_buffer);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }

    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        if new_width > 0 && new_height > 0 {
            self.surface_config.width = new_width as u32;
            self.surface_config.height = new_height as u32;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

pub async fn run() {    
    let mut game = Game::new(GameConfig { 
        render_width: 800, 
        render_height: 600, 
    }).await;
    while game.update() {
        match game.render() {
            Ok(_) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => game.resize(game.surface_config.width as i32, game.surface_config.height as i32),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) =>  {
                eprintln!("{:?}", wgpu::SurfaceError::OutOfMemory);
                break;
            },
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }
 
    }
}