use std::sync::Arc;
use easy_gpu::assets::{BufferLayout, GpuVertex};
use easy_gpu::frame::Frame;
use easy_gpu::wgpu::VertexFormat;
use winit::window::Window;
use crate::engine::input_manager::InputManager;
use crate::engine::render::Camera;
use crate::engine::render::lighting::LightingEngine;
use crate::engine::render::mesh::MeshEngine;
use crate::engine::render::sprite_batch::SpriteBatchEngine;

pub struct Renderer{
    pub egpu: easy_gpu::Renderer,
    pub lighting_engine: LightingEngine,
    pub camera: Camera,
    pub mesh_engine: MeshEngine,
    pub sprite_batch_engine: SpriteBatchEngine
}

impl Renderer{
    pub fn new(window: Arc<Window>)->Self{
        let mut egpu = pollster::block_on(easy_gpu::Renderer::new(window))
            .clear_colour(0.3,0.6,1.0,1.0);

        let lighting_engine = LightingEngine::new(&mut egpu);
        let camera = Camera::new(&mut egpu);
        let mesh_engine = MeshEngine::new(&mut egpu,&camera,&lighting_engine);
        let sprite_batch_engine = SpriteBatchEngine::new(&mut egpu,&camera,&lighting_engine);

        Self{
            egpu,
            lighting_engine,
            camera,
            mesh_engine,
            sprite_batch_engine,
        }
    }

    pub fn update(&mut self,input_manager: &InputManager,player_pos: [f32;2],dt:f32){
        self.camera.update(player_pos,input_manager,&mut self.egpu,dt);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex{
    position: [f32;3],
    pad: f32,
    uv: [f32;2]
}
impl Vertex {
    pub fn new(position: [f32;3],uv: [f32;2]) -> Self {
        Vertex{
            position,
            pad: 0.0,
            uv,
        }
    }
}
impl GpuVertex for Vertex {
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .attribute(0,0,VertexFormat::Float32x3)
            .attribute(1,16,VertexFormat::Float32x2)
    }
}