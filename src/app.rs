use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{Renderer, Sprite};
use crate::game::game::Game;
use crate::game::physics::transform::Transform;
use crate::game::player::player::spawn_player;

pub struct App{
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    asset_registry: Option<AssetRegistry>,
    file_manager: Arc<FileManager>,
    input_manager: InputManager,
    game: Game,
    last_update_time: Instant,
    light_update_timer: Instant,
}

impl App{
    pub fn new()->Self{
        Self{
            window: None,
            renderer: None,
            asset_registry: None,
            file_manager: Arc::new(FileManager::new()),
            input_manager: Default::default(),
            game: Game::new(),
            last_update_time: Instant::now(),
            light_update_timer: Instant::now(),
        }
    }
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window"));

        let mut renderer = Renderer::new(window.clone());
        let assets_registry = AssetRegistry::new(&mut renderer);
        self.game.item_registry.load(&assets_registry);

        self.game.begin_world(&mut renderer, &self.file_manager,&assets_registry);

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.asset_registry = Some(assets_registry);
    }

    #[profiling::function]
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent)  {
        let renderer = self.renderer.as_mut().unwrap();
        let asset_registry = self.asset_registry.as_mut().unwrap();

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
                self.game.chunk_manager.save_chunks(&self.file_manager);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.egpu.resize_surface(size);
            }

            WindowEvent::RedrawRequested => {
                profiling::scope!("frame");

                let dt = self.last_update_time.elapsed().as_secs_f32();
                self.last_update_time = Instant::now();

                self.game.update(&mut renderer.egpu, &self.file_manager, &mut self.input_manager, asset_registry, dt);

                renderer.update_camera(&self.input_manager, self.game.player_position,dt);
                renderer.update_sky(dt);

                renderer.new_frame();

                if self.game.chunk_manager.dirty || self.light_update_timer.elapsed().as_secs_f32() >= 0.06 {
                    self.light_update_timer = Instant::now();
                    self.game.chunk_manager.dirty = false;

                    let mut lights = self.game.extract_lights();
                    let (tiles,lit_tiles) = self.game.extract_tiles();
                    lights.extend(lit_tiles);
                    
                    renderer.update_tile_buffers(tiles);
                    renderer.update_light_buffers(lights,self.game.player_position);
                    renderer.compute_lightmap();
                }
                
                renderer.draw_sky();
                renderer.draw_ecs_sprites(&self.game.world);
                self.game.draw_terrain(renderer.egpu.current_frame(),asset_registry);
                self.game.add_gui(&mut renderer.gui_engine,asset_registry);
                renderer.draw_gui();

                renderer.finish();

                profiling::finish_frame!();
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