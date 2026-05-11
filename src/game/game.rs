use std::sync::{Arc, Mutex};
use easy_gpu::frame::Frame;
use hecs::World;
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::game::physics::collider::Collider;
use crate::game::physics::transform::Transform;
use crate::game::player::player::Player;
use crate::game::terrain::chunk_manager::ChunkManager;
use crate::game::terrain::terrain_generator::TerrainGenerator;

pub struct Game{
    pub world: World,
    pub chunk_manager: ChunkManager,
    terrain_generator: Arc<Mutex<TerrainGenerator>>,
    pub player_position: [f32;2],
}

impl Game{
    pub fn new()->Self{
        Self{
            world: World::new(),
            chunk_manager: ChunkManager::new(),
            terrain_generator: Arc::new(Mutex::new(TerrainGenerator::new())),
            player_position: [0.,0.],
        }
    }

    pub fn spawn_player(&mut self){
        self.world.spawn((
            Player::new(),
            Collider::new(1.8,2.8,[0.,-1.9],true),
            Transform::new([0.,20.]),
          //  Sprite::new(asset_registry.player, 2.0, 0, [1.,1.,1.,1.0]),
        ));
    }

    pub fn update(&mut self,egpu: &mut easy_gpu::Renderer, file_manager: &Arc<FileManager>,input_manager: &InputManager,dt: f32){
        self.chunk_manager.handle_input(&input_manager);
        self.chunk_manager.unload_chunks(self.player_position);
        self.chunk_manager.update_data_queue(self.player_position);
        self.chunk_manager.load_chunks_data(file_manager,&self.terrain_generator);
        self.chunk_manager.update_mesh_queue(self.player_position);
        self.chunk_manager.generate_chunk_meshes(egpu);
        self.chunk_manager.save_chunks(file_manager);

        self.update_colliders(dt);
        self.update_player(input_manager,dt);
    }

    fn update_player(&mut self,input_manager: &InputManager,dt: f32) {
        for (_, (player, transform,collider)) in self.world.query::<(&Player, &Transform,&mut Collider)>().iter() {
            self.player_position = transform.translation;
            player.update(input_manager,collider,dt);
        }
    }

    pub fn update_colliders(&mut self,dt:f32){
        for (_, (transform, collider)) in self.world.query::<(&mut Transform, &mut Collider)>().iter() {
            collider.handle_collider(transform,&self.chunk_manager,dt);
        }
    }

    pub fn draw(&self, frame: &mut Frame){
        self.chunk_manager.draw(frame);
    }

    pub fn extract_tiles(&self) -> Vec<u8>{
        self.chunk_manager.tiles(self.player_position)
    }
}