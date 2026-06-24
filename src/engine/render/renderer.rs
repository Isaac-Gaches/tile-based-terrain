use std::sync::Arc;
use easy_gpu::assets::{BufferLayout, BufferUsages, GpuVertex, Material, Mesh, Texture};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use easy_gpu::wgpu::VertexFormat;
use hecs::{Entity, World};
use winit::window::Window;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{Camera, LightSource, Sprite};
use crate::engine::render::gui::GuiEngine;
use crate::engine::render::lighting::LightingEngine;
use crate::engine::render::mesh::MeshEngine;
use crate::engine::render::sky::Sky;
use crate::engine::render::sprite_batch::{Atlas, SpriteBatchEngine};
use crate::game::physics::transform::Transform;

pub struct Renderer{
    pub egpu: easy_gpu::Renderer,
    pub lighting_engine: LightingEngine,
    pub camera: Camera,
    pub mesh_engine: MeshEngine,
    pub sprite_batch_engine: SpriteBatchEngine,
    pub gui_engine: GuiEngine,
    pub sky: Sky,
}

impl Renderer{
    pub fn new(window: Arc<Window>)->Self{
        let mut egpu = pollster::block_on(easy_gpu::Renderer::new(window))
            .clear_colour(0.3,0.6,1.0,1.0);

        let lighting_engine = LightingEngine::new(&mut egpu);
        let camera = Camera::new(&mut egpu);
        let mesh_engine = MeshEngine::new(&mut egpu,&camera,&lighting_engine);
        let sprite_batch_engine = SpriteBatchEngine::new(&mut egpu);
        let gui_engine = GuiEngine::new(&mut egpu);
        let sky = Sky::new(&mut egpu);

        Self{
            egpu,
            lighting_engine,
            camera,
            mesh_engine,
            sprite_batch_engine,
            gui_engine,
            sky,
        }
    }

    pub fn update_camera(&mut self,input_manager: &InputManager,player_pos: [f32;2],dt:f32){
        self.camera.update(player_pos,input_manager,&mut self.egpu,dt);
        self.gui_engine.update(&self.egpu);
    }
    pub fn update_sky(&mut self,dt: f32){
        self.sky.update(&mut self.egpu,self.lighting_engine.sky_light,dt);
    }
    pub fn update_light_buffers(&mut self,lights: Vec<LightSource>,player_pos: [f32;2]) {
        self.lighting_engine.update_light_buffers(&mut self.egpu, player_pos,lights);
    }
    pub fn update_tile_buffers(&mut self,tiles: Vec<u8>) {
        self.lighting_engine.update_tile_buffer(&mut self.egpu, tiles);
    }

    pub fn create_atlas(&mut self ) -> Atlas{
        Atlas{
            buffer: self.egpu.create_buffer(BufferUsages::STORAGE | BufferUsages::COPY_DST,1024),
            frames: vec![],
        }
    }

    pub fn create_sprite_material(&mut self,texture: Handle<Texture>, atlas: &Atlas) -> Handle<Material>{
        self.egpu.write_array_buffer(atlas.buffer,atlas.frames.as_slice());

        self.sprite_batch_engine.create_sprite_material(
            &mut self.egpu,
            &self.camera,
            &self.lighting_engine,
            texture,
            atlas.buffer
        )
    }

    pub fn create_gui_material(&mut self,texture: Handle<Texture>, atlas: &Atlas) -> Handle<Material>{
        self.egpu.write_array_buffer(atlas.buffer,atlas.frames.as_slice());

        self.gui_engine.create_gui_material(
            &mut self.egpu,
            &self.sprite_batch_engine,
            texture,
            atlas.buffer
        )
    }

    pub fn draw_gui(&mut self){
        self.gui_engine.draw(self.egpu.current_frame(), &self.sprite_batch_engine);
    }
    pub fn draw_sky(&mut self){
        self.sky.draw(self.egpu.current_frame());
    }
    pub fn draw_ecs_sprites(&mut self,world: &World){
        self.sprite_batch_engine.draw_sprites(self.egpu.current_frame(), world);
    }
    pub fn compute_lightmap(&mut self){
        self.lighting_engine.compute(self.egpu.current_frame());
    }
    pub fn new_frame(&mut self){
        self.egpu.begin_frame();
    }
    pub fn finish(&mut self){
        self.egpu.current_frame().sort_by_material();
        self.egpu.render();
    }
}

