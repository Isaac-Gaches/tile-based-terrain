use ahash::AHashMap;
use easy_gpu::assets::{render_storage, render_texture, render_uniform, sampler, Buffer, BufferUsages, GpuInstance, GpuVertex, Material, MaterialBuilder, RenderPipeline, RenderPipelineBuilder, Texture};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use easy_gpu::wgpu::{BlendState, TextureFormat};
use crate::engine::render::{Instance};
use crate::engine::render::sprite_batch::{SpriteBatchEngine, SpriteVertex};

pub struct GuiEngine{
    gui_pipeline: Handle<RenderPipeline>,
    batches: AHashMap<Handle<Material>,Vec<Instance>>,
    camera: Handle<Buffer>,
}

#[repr(C)]
#[derive(Copy,Clone,bytemuck::Pod, bytemuck::Zeroable)]
struct GuiCam{
    aspect: f32,
    _pad:[f32;3],
}

impl GuiEngine {
    pub fn new(egpu: &mut easy_gpu::Renderer) -> GuiEngine {
        let shader = egpu.load_shader(include_str!("shaders/gui.wgsl"));

        let cam_buf = egpu.create_buffer(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size_of::<GuiCam>() as u64
        );

        let gui_pipeline = RenderPipelineBuilder::new(shader)
            .material_layout(&[
                render_texture(0),
                sampler(1),
                render_storage(2,true),
                render_uniform(3)
            ])
            .vertex_layout(SpriteVertex::buffer_layout())
            .vertex_layout(Instance::buffer_layout())
            .depth_format(TextureFormat::Depth24Plus)
            .blend_mode(BlendState::REPLACE)
            .build(egpu);

        Self{
            gui_pipeline,
            batches: AHashMap::new(),
            camera: cam_buf,
        }
    }

    pub(super) fn create_gui_material(&self, egpu: &mut easy_gpu::Renderer,sprite_batch_engine: &SpriteBatchEngine, texture: Handle<Texture>,atlas_buffer: Handle<Buffer>) -> Handle<Material>{
        MaterialBuilder::new(self.gui_pipeline)
            .texture(0,texture)
            .sampler(1,sprite_batch_engine.sampler)
            .buffer(2,atlas_buffer)
            .buffer(3,self.camera)
            .build(egpu)
    }

    pub fn add_gui_element(&mut self, element: GuiElement) {
        let instance = Instance{
            position: [element.position[0],element.position[1],0.0],
            _pad: 0.0,
            rotation: 0.0,
            scale: element.scale,
            colour: [1.,1.,1.,1.],
            tex_index: element.texture_index,
        };

        if let Some(batch) = self.batches.get_mut(&element.material) {
            batch.push(instance);
        }
        else{
            self.batches.insert(element.material,vec![instance]);
        }
    }

    pub fn update(&mut self, egpu: &easy_gpu::Renderer) {
        egpu.write_buffer(self.camera,GuiCam{
            aspect: egpu.window_aspect(),
            _pad: [0.,0.,0.],
        });
    }

    pub(super) fn draw(&self, frame: &mut Frame,sprite_batch_engine: &SpriteBatchEngine) {
        for (material,instances) in &self.batches {
            frame.draw_batch(
                instances.as_slice(),
                *material,
                sprite_batch_engine.quad_mesh
            )
        }
    }
}

pub struct GuiElement{
    pub material: Handle<Material>,
    pub texture_index: u32,
    pub position: [f32; 3],
    pub scale: f32,
}