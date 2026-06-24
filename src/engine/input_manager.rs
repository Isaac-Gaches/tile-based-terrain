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
    pub nums: [bool;10]
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
                Key::Character("0") => self.nums[0] = true,
                Key::Character("1") => self.nums[1] = true,
                Key::Character("2") => self.nums[2] = true,
                Key::Character("3") => self.nums[3] = true,
                Key::Character("4") => self.nums[4] = true,
                Key::Character("5") => self.nums[5] = true,
                Key::Character("6") => self.nums[6] = true,
                Key::Character("7") => self.nums[7] = true,
                Key::Character("8") => self.nums[8] = true,
                Key::Character("9") => self.nums[9] = true,
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
                Key::Character("0") => self.nums[0] = false,
                Key::Character("1") => self.nums[1] = false,
                Key::Character("2") => self.nums[2] = false,
                Key::Character("3") => self.nums[3] = false,
                Key::Character("4") => self.nums[4] = false,
                Key::Character("5") => self.nums[5] = false,
                Key::Character("6") => self.nums[6] = false,
                Key::Character("7") => self.nums[7] = false,
                Key::Character("8") => self.nums[8] = false,
                Key::Character("9") => self.nums[9] = false,
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