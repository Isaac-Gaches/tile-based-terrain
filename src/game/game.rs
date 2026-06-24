use std::sync::{Arc};
use std::time::Instant;
use ahash::{AHashMap};
use easy_gpu::frame::Frame;
use hecs::World;
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{Light, LightSource, Renderer, Sprite};
use crate::engine::render::gui::GuiEngine;
use crate::game::entities::bomb::{update_bombs};
use crate::game::entities::Despawn;
use crate::game::entities::grass::{update_grass, update_vine};
use crate::game::entities::particle::update_particles;
use crate::game::items::inventory::Inventory;
use crate::game::items::item_registry::{ItemID, ItemRegistry};
use crate::game::physics::collider::{Collider, update_colliders};
use crate::game::physics::transform::Transform;
use crate::game::player::player::{Player, update_player, spawn_player};
use crate::game::terrain::chunk::CHUNK_SIZE;
use crate::game::terrain::chunk_manager::{ChunkManager, HORIZONTAL_CHUNK_LOAD_DISTANCE, VERTICAL_CHUNK_LOAD_DISTANCE};
use crate::game::terrain::terrain_generator::{TerrainGenerator, MAX_VINE_LENGTH};

pub struct Game{
    pub world: World,
    pub chunk_manager: ChunkManager,
    terrain_generator: Arc<TerrainGenerator>,
    pub player_position: [f32;2],
    unload_timer: Instant,
    pub item_registry: ItemRegistry,
    inventory: Inventory,
}

impl Game{
    pub fn new()->Self{
        Self{
            world: World::new(),
            chunk_manager: ChunkManager::new(),
            terrain_generator: Arc::new(TerrainGenerator::new()),
            player_position: [0.,0.],
            unload_timer: Instant::now(),
            item_registry: ItemRegistry::new(),
            inventory: Inventory::new(),
        }
    }

    pub fn begin_world(&mut self,renderer: &mut Renderer, file_manager: &Arc<FileManager>,asset_registry: &AssetRegistry){
        for _ in 0..50 {
            self.chunk_manager.update_data_queue(self.player_position);
            self.chunk_manager.load_chunks_data(file_manager, &self.terrain_generator);
            self.chunk_manager.update_mesh_queue(self.player_position);
            self.chunk_manager.generate_chunk_meshes(&mut renderer.egpu,&mut self.world,asset_registry);
        }
        spawn_player(&mut self.world,renderer);
        self.inventory.add_item("bomb".to_string(),1,0);
        self.inventory.add_item("potion".to_string(),1,1);
        self.inventory.add_item("glow_stick".to_string(),1,2);
        self.inventory.add_item("big_bomb".to_string(),1,3);
        self.inventory.add_item("dirt".to_string(),1,4);
        self.inventory.add_item("red_light".to_string(),1,5);
    }

    #[profiling::function]
    pub fn update(&mut self,egpu: &mut easy_gpu::Renderer, file_manager: &Arc<FileManager>,input_manager: &InputManager, asset_registry: &AssetRegistry,dt: f32){
        self.inventory.handle_input(input_manager);
        profiling::scope!("update");
        self.chunk_manager.update_data_queue(self.player_position);
        self.chunk_manager.load_chunks_data(file_manager,&self.terrain_generator);
        self.chunk_manager.update_mesh_queue(self.player_position);
        self.chunk_manager.generate_chunk_meshes(egpu,&mut self.world,asset_registry);

        if self.chunk_manager.dirty{
            update_grass(&mut self.world,&mut self.chunk_manager);
            for _ in 0..MAX_VINE_LENGTH{
                update_vine(&mut self.world,&mut self.chunk_manager);
            }
        }

        if self.unload_timer.elapsed().as_secs() > 20{
            self.chunk_manager.save_chunks(file_manager);
        self.chunk_manager.unload_chunks(self.player_position,&mut self.world);
            self.unload_timer = Instant::now();
        }

        update_colliders(&mut self.world,&mut self.chunk_manager,dt);
        update_bombs(&mut self.world,dt,&mut self.chunk_manager,asset_registry);
        update_particles(&mut self.world,dt);
        self.player_position = update_player(&mut self.world,input_manager,&mut self.inventory,&self.item_registry,dt,&mut self.chunk_manager);

        self.unload_entities();
    }

    pub fn add_gui(&self,gui:&mut GuiEngine,asset_registry: &AssetRegistry){
        self.inventory.draw_hotbar(gui,asset_registry,&self.item_registry);
    }

    fn unload_entities(&mut self){
        let mut unload = Vec::new();
        for (entity,(transform,_)) in self.world.query::<(&Transform,&Despawn)>().iter(){
            if (transform.translation[0] - self.player_position[0]).abs() > (HORIZONTAL_CHUNK_LOAD_DISTANCE+1) as f32 * CHUNK_SIZE as f32
                || (transform.translation[1] - self.player_position[1]).abs() > (VERTICAL_CHUNK_LOAD_DISTANCE+1) as f32 * CHUNK_SIZE as f32{
                unload.push(entity);
            }
        }
        for entity in unload{
            let _ = self.world.despawn(entity);
        }
    }

    pub fn draw_terrain(&self, frame: &mut Frame,asset_registry: &AssetRegistry){
        self.chunk_manager.draw(frame,asset_registry);
    }

    pub fn extract_tiles(&self) -> (Vec<u8>,Vec<LightSource>){
        self.chunk_manager.extract_tiles(self.player_position)
    }

    pub fn extract_lights(&self) -> Vec<LightSource>{
        let mut lights= AHashMap::new();

        for (_,(light,transform)) in self.world.query::<(&Light,&Transform)>().iter() {
            let pos = [(transform.translation[0]).round() as i32, (transform.translation[1] + 0.5).round() as i32];
            lights
                .entry(pos)
                .and_modify(|existing: &mut LightSource| {
                    existing.colour[0] = existing.colour[0].max(light.colour[0]);
                    existing.colour[1] = existing.colour[1].max(light.colour[1]);
                    existing.colour[2] = existing.colour[2].max(light.colour[2]);
                })
                .or_insert(LightSource::new([pos[0] as f32,pos[1] as f32],light.colour));
        }

        lights.into_values().collect()
    }
}

