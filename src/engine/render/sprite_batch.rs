use easy_gpu::assets::{BufferLayout, GpuInstance, GpuVertex, Mesh, render_texture, RenderPipeline, RenderPipelineBuilder, sampler, uniform};
use easy_gpu::assets_manager::Handle;
use easy_gpu::wgpu::VertexFormat;
use serde::{Deserialize, Serialize};
use crate::engine::render::{Camera, Vertex};
use crate::engine::render::lighting::LightingEngine;

pub struct SpriteBatchEngine{
    sprite_batch_pipeline: Handle<RenderPipeline>,
    quad_mesh: Handle<Mesh>,
}

impl SpriteBatchEngine{
    pub fn new(egpu: &mut easy_gpu::Renderer,camera: &Camera, lighting_engine: &LightingEngine) -> Self{
        let scale = 0.5;
        let vertices = [
            Vertex::new([-scale, -scale,2.0],[0.,0.]),
            Vertex::new([scale, -scale,2.0],[1.,0.]),
            Vertex::new([scale, scale,2.0],[1.,1.]),
            Vertex::new([-scale, scale,2.0],[0.,1.])
        ];

        let indices = [0, 1, 2, 0, 2, 3];

        let quad_mesh = egpu.create_mesh(&vertices, &indices);

        let shader = egpu.load_shader(include_str!("shaders/sprite_batch.wgsl"));

        let sprite_batch_pipeline = RenderPipelineBuilder::new(shader)
            .material_layout(&[
                uniform(0),
                render_texture(1),
                sampler(2),
                render_texture(3),
                sampler(4),
                uniform(5),
            ])
            .vertex_layout(Vertex::buffer_layout())
            .vertex_layout(Instance::buffer_layout())
            .build(egpu);

        Self{
            sprite_batch_pipeline,
            quad_mesh,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable,Serialize,Deserialize)]
pub struct Instance { //32
    pub position: [f32;2],
    pub rotation: f32,
    pub scale: f32,
    pub tex_index: u32,
    pub colour: [f32; 4],
}

impl GpuInstance for Instance{
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .attribute(2,0,VertexFormat::Float32x2)
            .attribute(3,8,VertexFormat::Float32)
            .attribute(4,12,VertexFormat::Float32)
            .attribute(5,16,VertexFormat::Uint32)
            .attribute(6,20,VertexFormat::Float32x4)
    }
}