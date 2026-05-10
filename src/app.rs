use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::engine::render::Renderer;
use crate::game::game::Game;

pub struct App{
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    file_manager: FileManager,
    input_manager: InputManager,
    game: Game,
}

impl App{
    pub fn new()->Self{
        Self{
            window: None,
            renderer: None,
            file_manager: FileManager::new(),
            input_manager: Default::default(),
            game: Game::new(),
        }
    }
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window"));

        let renderer = Renderer::new(window.clone());

        self.game.chunk_manager.set_mesh_materials(vec![
            renderer.mesh_engine.bg_mesh_material.clone(),
            renderer.mesh_engine.fg_mesh_material.clone()
        ]);

        self.game.spawn_player();

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent)  {
        let renderer = self.renderer.as_mut().unwrap();

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_manager.handle_keyboard(&event);
            }
            WindowEvent::MouseInput { button,state,..} =>{
                self.input_manager.handle_mouse_buttons(&button,&state);
            }
            WindowEvent::CursorMoved {position,..} =>{
                self.input_manager.handle_mouse_move(&position,renderer.egpu.width(),renderer.egpu.height());
                self.input_manager.update_mouse_world_pos(&renderer.camera);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.egpu.resize_surface(size);
            }

            WindowEvent::RedrawRequested => {
                self.game.update(&mut renderer.egpu,&mut self.file_manager,&mut self.input_manager);
                renderer.update(self.game.tiles(),&self.input_manager,self.game.player_position);

                let frame = renderer.egpu.begin_frame();

                renderer.lighting_engine.compute(frame);
                self.game.draw(frame);

                frame.sort_by_material();
                renderer.egpu.render();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}