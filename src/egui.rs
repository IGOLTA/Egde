use std::time::SystemTime;

use egui::{ahash::HashMapExt, viewport, DroppedFile, Event, HoveredFile, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2, ViewportId, ViewportIdMap, ViewportInfo};
use sdl2::{event::WindowEvent, keyboard::{Keycode, Scancode}, libc::SOCKET, mouse::{MouseButton, MouseWheelDirection}, video::FullscreenType};

use crate::Game;

//Fournit à egui le "raw input" = l'état des entrées de l'utilisateur et de la fenêtre à un instant donné. 
pub fn collect_raw_input(game: & Game, delta_time: f32, frame_events: &[sdl2::event::Event]) -> RawInput {
    let viewport_id = ViewportId::ROOT;
    
    let mut viewports = ViewportIdMap::<ViewportInfo>::new();
    viewports.insert(viewport_id, ViewportInfo{
        title: Some(game.config.game_name.clone()),
        minimized: Some(game.window.is_minimized()),
        maximized: Some(game.window.is_maximized()),
        fullscreen: Some(game.window.fullscreen_state() == FullscreenType::True || game.window.fullscreen_state() == FullscreenType::Desktop),
        focused: Some(game.window.has_mouse_focus() || game.window.has_mouse_focus()),
        ..Default::default()
    });

    let screen_rect = Some(Rect::from_min_size(Default::default(), Vec2::new(game.surface_config.width as f32, game.surface_config.height as f32)));
    let time = Some(game.game_start.elapsed().as_secs_f64());
    let predicted_dt = delta_time;

    let modifiers = Modifiers {
        alt: game.event_pump.keyboard_state().is_scancode_pressed(Scancode::LAlt) || game.event_pump.keyboard_state().is_scancode_pressed(Scancode::RAlt) ,
        ctrl: game.event_pump.keyboard_state().is_scancode_pressed(Scancode::LCtrl) || game.event_pump.keyboard_state().is_scancode_pressed(Scancode::RCtrl),
        shift: game.event_pump.keyboard_state().is_scancode_pressed(Scancode::LShift) || game.event_pump.keyboard_state().is_scancode_pressed(Scancode::RShift), 
        mac_cmd: false, //TODO: implement mac compatibility
        command: game.event_pump.keyboard_state().is_scancode_pressed(Scancode::LCtrl) || game.event_pump.keyboard_state().is_scancode_pressed(Scancode::RCtrl) 
    };

    let events: Vec<egui::Event> = frame_events
        .iter()
        .filter_map(|sdl_event| {
            match sdl_event {
                sdl2::event::Event::TextInput  { text, .. } => Some(egui::Event::Text(text.clone())),
                sdl2::event::Event::KeyUp { keycode: Some(keycode), repeat, .. } => if let Some(key) = sdl_key_code_to_egui(keycode) {
                    Some(egui::Event::Key { 
                        key,
                        physical_key: None, 
                        pressed: false, 
                        repeat: *repeat, 
                        modifiers: modifiers
                    })
                } else {
                    None
                },
                sdl2::event::Event::KeyDown { keycode: Some(keycode), repeat, .. } => if let Some(key) = sdl_key_code_to_egui(keycode) {
                    Some(egui::Event::Key { 
                        key,
                        physical_key: None, 
                        pressed: true, 
                        repeat: *repeat, 
                        modifiers: modifiers
                    })
                } else {
                    None
                },
                sdl2::event::Event::MouseMotion { x, y, xrel, yrel , ..} => Some(egui::Event::PointerMoved(Pos2::new(*x as f32, *y as f32))),
                sdl2::event::Event::MouseButtonUp { mouse_btn, x, y,..} => {
                    match mouse_button_to_pointer_button(mouse_btn) {
                        Some(pointer_button) => Some(egui::Event::PointerButton { pos: Pos2::new(*x as f32, *y as f32), button: pointer_button, pressed: false, modifiers: modifiers }),
                        None => None,
                    }
                },
                sdl2::event::Event::MouseButtonDown { mouse_btn, x, y,..} => {
                    match mouse_button_to_pointer_button(mouse_btn) {
                        Some(pointer_button) => Some(egui::Event::PointerButton { pos: Pos2::new(*x as f32, *y as f32), button: pointer_button, pressed: true, modifiers: modifiers }),
                        None => None,
                    }
                },
                sdl2::event::Event::MouseWheel { direction, precise_x, precise_y, .. } => {
                    Some(egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(*precise_x, *precise_y), modifiers: modifiers })
                },
                sdl2::event::Event::Window { win_event: WindowEvent::FocusGained, .. } => Some(egui::Event::WindowFocused(true)),
                sdl2::event::Event::Window { win_event: WindowEvent::FocusLost, .. } => Some(egui::Event::WindowFocused(false)),
                _ => None
            }
        })
        .collect();

    let hovered_files: Vec<HoveredFile> = Vec::new();
    let dropped_files: Vec<DroppedFile> = Vec::new();
    let focused = game.window.has_input_focus() || game.window.has_input_focus();

    RawInput {
        viewport_id,
        viewports,
        screen_rect,
        time,
        predicted_dt,
        modifiers,
        events,
        hovered_files,
        dropped_files,
        focused,
        ..Default::default()
    }
}

//Diverses fonctions servant d'interfance entre les structures de données SDL2 et les structures EGUI
fn sdl_key_code_to_egui(keycode: &Keycode) -> Option<Key> {
    Key::from_name(&keycode.name())
}

fn mouse_button_to_pointer_button(mouse_btn: &MouseButton) -> Option<PointerButton> {
    match mouse_btn {
        MouseButton::Unknown => None,
        MouseButton::Left => Some(PointerButton::Primary),
        MouseButton::Middle => Some(PointerButton::Middle),
        MouseButton::Right => Some(PointerButton::Secondary),
        MouseButton::X1 => Some(PointerButton::Extra1),
        MouseButton::X2 => Some(PointerButton::Extra2),
    }
}