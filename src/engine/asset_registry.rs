use ahash::AHashMap;
use easy_gpu::assets::{Material, Texture};
use easy_gpu::assets_manager::Handle;
use crate::engine::render::Renderer;

pub struct AssetRegistry{
    pub throwable_mat: Handle<Material>,
    pub particle_mat: Handle<Material>,
    pub natural_deco_mat: Handle<Material>,
    pub mesh_mats: Vec<Handle<Material>>,
    pub gui_mat: Handle<Material>,
    pub item_icon_mat: Handle<Material>,
}

impl AssetRegistry{
    pub fn new(renderer: &mut Renderer)-> Self{
        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/throwables.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.,0.],[0.25,1.0]);
        atlas.add_frame([0.25,0.],[0.5,1.0]);
        atlas.add_frame([0.5,0.],[0.75,1.0]);
        atlas.add_frame([0.75,0.],[1.0,1.0]);
        let throwable_mat = renderer.create_sprite_material(texture,&atlas);

        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/icons.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.,0.],[1./6.,1.]);
        atlas.add_frame([1./6.,0.],[2./6.,1.]);
        atlas.add_frame([2./6.,0.],[3./6.,1.]);
        atlas.add_frame([3./6.,0.],[4./6.,1.]);
        atlas.add_frame([4./6.,0.],[5./6.,1.]);
        atlas.add_frame([5./6.,0.],[6./6.,1.]);
        let item_icon_mat = renderer.create_gui_material(texture,&atlas);

        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/particles.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.25,0.],[0.5,1.0]);
        let particle_mat = renderer.create_sprite_material(texture,&atlas);

        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/deco.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.0,0.0],[0.2,0.5]);
        atlas.add_frame([0.2,0.0],[0.4,0.5]);
        atlas.add_frame([0.4,0.0],[0.6,0.5]);
        atlas.add_frame([0.4,0.5],[0.6,1.0]);
        atlas.add_frame([0.6,0.5],[0.8,1.0]);
        let natural_deco_mat = renderer.create_sprite_material(texture,&atlas);

        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/gui.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.0,0.0],[1.0,1.0]);
        let gui_mat = renderer.create_gui_material(texture,&atlas);

        Self{
            throwable_mat,
            particle_mat,
            natural_deco_mat,
            mesh_mats: vec![renderer.mesh_engine.bg_mesh_material,renderer.mesh_engine.fg_mesh_material],
            gui_mat,
            item_icon_mat,
        }
    }
}