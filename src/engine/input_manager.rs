use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::Key;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use crate::engine::render::Camera;

#[derive(Default)]
pub struct InputManager{
    pub mouse_pos: [f32;2],
    pub mouse_world_pos: [f32;2],
    pub left_mouse: bool,
    pub right_mouse: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub plus: bool,
    pub minus: bool,
}

impl InputManager {
    pub fn handle_keyboard(&mut self, event: &KeyEvent){
        if event.state == ElementState::Pressed && !event.repeat {
            match event.key_without_modifiers().as_ref() {
                Key::Character("w") => self.up = true,
                Key::Character("s") => self.down = true,
                Key::Character("a") => self.left = true,
                Key::Character("d") => self.right = true,
                Key::Character("=") => self.plus = true,
                Key::Character("-") => self.minus = true,
                _ => (),
            }
        }
        else if event.state == ElementState::Released {
            match event.key_without_modifiers().as_ref() {
                Key::Character("w") => self.up = false,
                Key::Character("s") => self.down = false,
                Key::Character("a") => self.left = false,
                Key::Character("d") => self.right = false,
                Key::Character("=") => self.plus = false,
                Key::Character("-") => self.minus = false,
                _ => (),
            }
        }
    }

    pub fn handle_mouse_buttons(&mut self, button: &MouseButton, state: &ElementState){
        match button {
            MouseButton::Left =>{
                self.left_mouse = state.is_pressed();
            },
            MouseButton::Right =>{
                self.right_mouse = state.is_pressed();
            },
            _ => (),
        }
    }

    pub fn handle_mouse_move(&mut self, position: &PhysicalPosition<f64>,width:u32,height:u32){
        self.mouse_pos =
            [(position.x as f32/width as f32 - 0.5)*2.0,
                ((height as f32- position.y as f32)/height as f32 - 0.5)*2.0];
    }

    pub fn update_mouse_world_pos(&mut self, camera: &Camera){
        self.mouse_world_pos = camera.screen_to_world(self.mouse_pos);
    }
}