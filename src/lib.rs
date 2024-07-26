#[deny(deprecated)]

pub mod scene;
pub mod render;
pub mod memory;
pub mod egui;

use core::default::Default;
use std::path::Path;
use std::time::Instant;
use ::egui::{Context, FullOutput};
use egui_wgpu_backend::ScreenDescriptor;
use glam::{UVec3, Vec3};
use scene::camera::{Camera, CameraData};
use scene::chunk::chunk_content::ChunkContentLoadingError;
use scene::chunk::{self, Chunk};
use scene::{Scene, UnloadedScene};
use sdl2::video::Window;
use sdl2::{EventPump, VideoSubsystem};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

use crate::render::g_buffer::GBuffer;
use crate::render::chunk_renderer::ChunkRenderer;
use crate::render::render_plane::RenderPlane;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub game_name: String,
    pub window_width: u32,
    pub window_height: u32,
    pub render_scale: u32,
}

pub struct Game<'a> {
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

    game_start: Instant,
    last_frame: Instant,

    current_scene: Option<Scene>,

    egui_context: Context,
    egui_r_pass: egui_wgpu_backend::RenderPass,
    full_output: Option<FullOutput>,
}

impl Game<'_> {
    
    pub async fn new(config: GameConfig) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window(&config.game_name, config.window_width, config.window_height)
            .position_centered()
            .borderless()
            .build()
            .unwrap();
        
        sdl_context.mouse().show_cursor(true);
        sdl_context.mouse().set_relative_mouse_mode(false);

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
        
        let g_buffer = GBuffer::new(&device, config.window_width * config.render_scale, config.window_height * config.render_scale);

        let render_plane = RenderPlane::new(&device, &surface_config);

        let chunk_renderer = ChunkRenderer::new(&device);

        let egui_context = Context::default();

        let egui_r_pass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

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
            last_frame: Instant::now(),
            game_start: Instant::now(),
            current_scene: None,
            egui_context,
            egui_r_pass,
            full_output: None
        }
    }

    pub fn load_scene(& mut self, to_load: UnloadedScene) -> Result<(), ChunkContentLoadingError> {
        self.current_scene = Some(to_load.load(&self.device, &self.queue, self.config.window_width as f32 / self.config.window_height as f32)?);
        Ok(())
    }

    pub fn launch(& mut self) {
        while self.update() {
            match self.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => self.resize(self.surface_config.width as i32, self.surface_config.height as i32),
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

    fn update(& mut self) -> bool {
        let delta_time = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();

        //Poll events
        let mut events = Vec::<Event>::new();
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
                _ => {}
            }

            events.push(event);
        }

        let raw_input: ::egui::RawInput = crate::egui::collect_raw_input(&self, delta_time, &events);
        if let Some(ref mut scene) = self.current_scene {
            scene.update(&self.queue, delta_time, &self.event_pump);
        }

        self.full_output = Some(self.egui_context.run(raw_input, |ctx| {
            let frame =  ::egui::containers::Frame {
                ..Default::default()
            }; 
            ::egui::CentralPanel::default().frame(frame).show(&ctx, |ui| {
                ui.label("Hello world!");
                if ui.button("Click me").clicked() {
                    println!("Clicked");
                }
            
            });
        }));

        true
    }

    fn render(& mut self) -> Result<(), wgpu::SurfaceError>{
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        if let Some(ref scene) = self.current_scene {
            scene.render(&self.chunk_renderer, &self.device, &self.g_buffer, &mut encoder);
        }

        self.render_plane.render(&mut encoder, &self.device, &view, &self.g_buffer);
       
        if let Some(full_output) = &self.full_output {
            let clipped_primitives = self.egui_context.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

            self.egui_r_pass
                .add_textures(&self.device, &self.queue, &full_output.textures_delta)
                .expect("EGUI: Failed to add textures to render passs");
    
            let screen_descriptor = ScreenDescriptor {
                    physical_width: self.surface_config.width,
                    physical_height: self.surface_config.height,
                    scale_factor: 1.0,
            };
    
            self.egui_r_pass.update_buffers(&self.device, &self.queue, &clipped_primitives, &screen_descriptor);
            
            self.egui_r_pass
                .execute(
                    &mut encoder,
                    &view,
                    &clipped_primitives,
                    &screen_descriptor,
                    None,
                )
                .unwrap();
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }

    //TODO: Prendre en charge les différentes tailles de fenêrtre correctement (en redimentionant aussi le GBuffer)
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        if new_width > 0 && new_height > 0 {
            self.surface_config.width = new_width as u32;
            self.surface_config.height = new_height as u32;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}
