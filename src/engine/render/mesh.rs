use easy_gpu::assets::{BufferLayout, GpuVertex, Material, MaterialBuilder, render_texture, RenderPipelineBuilder, sampler, SamplerBuilder, uniform};
use easy_gpu::assets_manager::Handle;
use easy_gpu::wgpu::{BlendState, FilterMode, TextureFormat, VertexFormat};
use crate::engine::render::{Camera, Vertex};
use crate::engine::render::lighting::LightingEngine;

pub struct MeshEngine{
    pub fg_mesh_material: Handle<Material>,
    pub bg_mesh_material: Handle<Material>,
}

impl MeshEngine{
    pub fn new(egpu: &mut easy_gpu::Renderer,camera: &Camera, lighting_engine: &LightingEngine) -> Self{
        let mesh_shader = egpu.load_shader(include_str!("shaders/mesh.wgsl"));

        let fg_tile_texture = egpu.load_texture_from_file(include_bytes!("textures/fg_tiles.png").to_vec());
        let bg_tile_texture = egpu.load_texture_from_file(include_bytes!("textures/bg_tiles.png").to_vec());

        let tile_sampler = SamplerBuilder::new()
            .filter_mode(FilterMode::Nearest)
            .build(egpu);

        let fg_mesh_pipeline = RenderPipelineBuilder::new(mesh_shader.clone())
            .material_layout(&[
                uniform(0),
                render_texture(1),
                sampler(2),
                render_texture(3),
                sampler(4),
                uniform(5),
            ])
            .fs_entry_point("fs_fg_tiles")
            .vertex_layout(Vertex::buffer_layout())
            .depth_format(TextureFormat::Depth24Plus)
            .blend_mode(BlendState::REPLACE)
            .build(egpu);

        let fg_mesh_material = MaterialBuilder::new(fg_mesh_pipeline)
            .uniform(0,camera.buffer)
            .texture(1,fg_tile_texture)
            .sampler(2,tile_sampler)
            .texture(3,lighting_engine.smooth_texture_a)
            .sampler(4,lighting_engine.light_sampler)
            .uniform(5,lighting_engine.light_uniform)
            .build(egpu);

        let bg_mesh_pipeline = RenderPipelineBuilder::new(mesh_shader)
            .material_layout(&[
                uniform(0),
                render_texture(1),
                sampler(2),
                render_texture(3),
                sampler(4),
                uniform(5),
                render_texture(6),
            ])
            .fs_entry_point("fs_bg_tiles")
            .vertex_layout(Vertex::buffer_layout())
            .depth_format(TextureFormat::Depth24Plus)
            .blend_mode(BlendState::REPLACE)
            .build(egpu);

        let bg_mesh_material = MaterialBuilder::new(bg_mesh_pipeline)
            .uniform(0,camera.buffer)
            .texture(1,fg_tile_texture)
            .sampler(2,tile_sampler)
            .texture(3,lighting_engine.smooth_texture_a)
            .sampler(4,lighting_engine.light_sampler)
            .uniform(5,lighting_engine.light_uniform)
            .texture(6,lighting_engine.occlusion_texture)
            .build(egpu);

        Self{
            fg_mesh_material,
            bg_mesh_material,
        }
    }
}
