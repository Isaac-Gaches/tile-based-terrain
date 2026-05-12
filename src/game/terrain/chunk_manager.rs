use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use easy_gpu::assets::Material;
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use rayon::iter::ParallelDrainRange;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::game::terrain::chunk::{CHUNK_SIZE, ChunkPosition, ChunkData, Chunk};
use crate::game::terrain::terrain_generator::TerrainGenerator;
use crate::game::terrain::tile::Tile;

pub struct ChunkManager{
    pub dirty: bool,
    chunks: HashMap<ChunkPosition,Chunk>,
    data_load_queue: Vec<ChunkPosition>,
    mesh_load_queue: Vec<ChunkPosition>,
    mesh_materials: Vec<Handle<Material>>
}
pub const CHUNK_LOAD_DISTANCE: i32 = 3;

impl ChunkManager{
    pub fn new()->Self{
        Self{
            dirty: false,
            chunks: HashMap::new(),
            data_load_queue: Vec::new(),
            mesh_load_queue: Vec::new(),
            mesh_materials: vec![],
        }
    }
    pub fn set_mesh_materials(&mut self,materials: Vec<Handle<Material>>){
        self.mesh_materials = materials;
    }
    pub fn unload_chunks(&mut self,player_pos: [f32;2]){
        let mut unload = Vec::new();
        
        for chunk_pos in self.chunks.keys(){
            let x_dist = ((chunk_pos.x * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[0]).abs();
            let y_dist = ((chunk_pos.y * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[1]).abs();

            if x_dist >= (CHUNK_LOAD_DISTANCE + 2) as f32 * CHUNK_SIZE as f32 || y_dist >= (CHUNK_LOAD_DISTANCE + 2) as f32 * CHUNK_SIZE as f32{
                unload.push(chunk_pos.clone());
            }
        }
        
        for key in &unload{
            self.chunks.remove(key);
        }
    }

    pub fn update_data_queue(&mut self, player_pos: [f32;2]){
        for x in -(CHUNK_LOAD_DISTANCE+1)..=(CHUNK_LOAD_DISTANCE+1) {
            for y in -(CHUNK_LOAD_DISTANCE+1)..=(CHUNK_LOAD_DISTANCE+1) {
                let chunk_pos = ChunkPosition::new(
                    player_pos[0].div_euclid(CHUNK_SIZE as f32) as i32 + x,
                    player_pos[1].div_euclid(CHUNK_SIZE as f32) as i32 + y
                );

                if !self.chunks.contains_key(&chunk_pos){
                    self.data_load_queue.push(chunk_pos);
                }
            }
        }
    }

    pub fn update_mesh_queue(&mut self,player_pos: [f32;2]){
        for (chunk_pos,chunk) in &mut self.chunks{
            let x_dist = ((chunk_pos.x * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[0]).abs();
            let y_dist = ((chunk_pos.y * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[1]).abs();

            if x_dist >= (CHUNK_LOAD_DISTANCE) as f32 * CHUNK_SIZE as f32 || y_dist >= (CHUNK_LOAD_DISTANCE) as f32 * CHUNK_SIZE as f32{
                if chunk.has_mesh(){
                    chunk.destroy_mesh();
                }
            }
            else if chunk.dirty(){
                self.mesh_load_queue.push(chunk_pos.clone());
                self.dirty = true;
            }
        }
    }

    pub fn load_chunks_data(
        &mut self,
        file_manager: &Arc<FileManager>,
        terrain_generator: &Arc<TerrainGenerator>,
    ) {
        let loaded_chunks: Vec<(ChunkPosition, Chunk)> = self
            .data_load_queue
            .par_drain(..)
            .map(|chunk_pos| {
                let chunk_data = if let Some(chunk_data) = file_manager.load_chunk(&chunk_pos) {
                    chunk_data
                } else {
                    ChunkData::new(&chunk_pos, terrain_generator)
                };

                let chunk = Chunk::new(chunk_data);

                (chunk_pos, chunk)
            })
            .collect();

        for (chunk_pos, chunk) in loaded_chunks {
            self.chunks.insert(chunk_pos, chunk);
        }
    }

    pub fn generate_chunk_meshes(&mut self, egpu: &mut easy_gpu::Renderer) {
        if self.mesh_load_queue.len() == 0{
            return;
        }
        let jobs: Vec<_> = self
            .mesh_load_queue
            .drain(..self.mesh_load_queue.len().min(4))
            .flat_map(|chunk_pos| {
                let chunk = self.chunks.get_mut(&chunk_pos).expect("chunk doesn't exist");
                let mut dirty_layers = Vec::new();
                if chunk.dirty[1] {
                    dirty_layers.push((chunk_pos.clone(), 1));
                }
                if chunk.dirty[0] {
                    dirty_layers.push((chunk_pos, 0));
                }
                chunk.remove_mark();
                dirty_layers
            })
            .collect();


        let generated_meshes: Vec<_> = jobs
            .par_iter()
            .map(|(chunk_pos, layer)| {
                let borders = self.chunk_borders(chunk_pos, *layer);
                let chunk = self.chunks.get(chunk_pos).expect("chunk doesn't exist");
                let mesh_data = chunk.build_mesh(*layer, chunk_pos, borders);

                (chunk_pos, *layer, mesh_data)
            })
            .collect();

        for (chunk_pos, layer, mesh_data) in generated_meshes {
            let chunk = self.chunks.get_mut(&chunk_pos).expect("chunk doesn't exist");
            if let Some((vertices,indices)) = mesh_data {
                let mesh = egpu.create_mesh(vertices.as_slice(),indices.as_slice());
                chunk.set_mesh(layer,mesh);
            }
        }
    }
    
    pub fn chunk_borders(&self,chunk_pos: &ChunkPosition, layer: usize) -> ChunkBorders{
        let mut top = Vec::new();
        for x in -1..CHUNK_SIZE as i32+1{
            let tiles = self.get_tile(chunk_pos.x * CHUNK_SIZE as i32 + x , chunk_pos.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32,layer).unwrap().solid();
            top.push(tiles);
        }

        let mut right = Vec::new();
        for y in 0..CHUNK_SIZE as i32{
            let tiles = self.get_tile(chunk_pos.x * CHUNK_SIZE as i32+ CHUNK_SIZE as i32, chunk_pos.y * CHUNK_SIZE as i32 + y,layer).unwrap().solid();
            right.push(tiles);
        }

        let mut bottom = Vec::new();
        for x in -1..CHUNK_SIZE as i32+1{
            let tiles = self.get_tile(chunk_pos.x * CHUNK_SIZE as i32 + x,  chunk_pos.y * CHUNK_SIZE as i32-1,layer).unwrap().solid();
            bottom.push(tiles);
        }

        let mut left = Vec::new();
        for y in 0..CHUNK_SIZE as i32{
            let tiles = self.get_tile(chunk_pos.x * CHUNK_SIZE as i32 -1, chunk_pos.y * CHUNK_SIZE as i32 + y,layer).unwrap().solid();
            left.push(tiles);
        }

        ChunkBorders{
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn save_chunks(&self, file_manager: &Arc<FileManager>) {
        self.chunks
            .par_iter()
            .for_each(|(chunk_pos,chunk)| {
                if chunk.save {
                    if let Err(error) =
                        file_manager.save_chunk(chunk.data(), chunk_pos)
                    {
                        println!("{}", error);
                    }
                }
            });
    }

    pub fn draw(&self, frame: &mut Frame){
        for (_,chunk) in &self.chunks{
            chunk.draw(frame,&self.mesh_materials);
        }
    }

    pub fn get_tile(&self,x:i32,y:i32,layer:usize) -> Option<&Tile>{
        let chunk_pos = ChunkPosition::from_world_space(x,y);
        if let Some(chunk) = self.chunks.get(&chunk_pos){
            let (x,y) = (x.rem_euclid(CHUNK_SIZE as i32) as usize,y.rem_euclid(CHUNK_SIZE as i32) as usize);
            return Some(chunk.get_tile(x,y,layer));
        }
        None
    }

    pub fn tiles(&self, player_pos: [f32;2]) -> Vec<u8>{
        let load_distance = CHUNK_LOAD_DISTANCE*CHUNK_SIZE as i32;

        let mut tiles = vec![1;(load_distance as usize *2 + CHUNK_SIZE)*(load_distance as usize *2 + CHUNK_SIZE)];//solid

        let player_chunk = [player_pos[0].div_euclid(CHUNK_SIZE as f32) as i32,player_pos[1].div_euclid(CHUNK_SIZE as f32) as i32];

        for x in -load_distance..load_distance + CHUNK_SIZE as i32{
            for y in -load_distance..load_distance + CHUNK_SIZE as i32{
                let fg_tile = self.get_tile(player_chunk[0] * CHUNK_SIZE as i32 + x,player_chunk[1] * CHUNK_SIZE as i32 + y,1).expect("no tile");
                let bg_tile = self.get_tile(player_chunk[0] * CHUNK_SIZE as i32 + x,player_chunk[1] * CHUNK_SIZE as i32 + y,0).expect("no tile");

                let width = load_distance * 2 + CHUNK_SIZE as i32;

                if fg_tile.id == 0{
                    if bg_tile.id == 0{
                        tiles[((y+load_distance) * width + (x+load_distance )) as usize] = 0;//empty
                    }
                    else{
                        tiles[((y+load_distance ) * width + (x+load_distance)) as usize] = 2;//wall
                    }
                }
                else if fg_tile.id == 4{//lights
                    tiles[((y+load_distance) * width + (x+load_distance)) as usize] = 4;
                }
                else if fg_tile.id == 6{
                    tiles[((y+load_distance ) * width + (x+load_distance)) as usize] = 6;
                }
                else if fg_tile.id == 9{
                    tiles[((y+load_distance ) * width + (x+load_distance )) as usize] = 9;
                }

            }
        }

        tiles
    }

    fn set_tile(&mut self, x:i32, y:i32,id:u8,layer:usize){
        let mut key = ChunkPosition::new(
            (x as f32/CHUNK_SIZE as f32).floor() as i32,
            (y as f32/CHUNK_SIZE as f32).floor() as i32
        );

        let mut adj_chunks = [0,0];

        match self.chunks.get_mut(&key){
            Some(chunk) =>{
                let x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
                let y = y.rem_euclid(CHUNK_SIZE as i32) as usize;

                if x == 0{adj_chunks[0] = -1;}
                else if x == CHUNK_SIZE-1{adj_chunks[0] = 1;}
                if y == 0{adj_chunks[1] = -1;}
                else if y == CHUNK_SIZE-1{adj_chunks[1] = 1;}

                chunk.set_tile(x,y,id,layer);
            }
            None => {}
        }

        if adj_chunks[0] != 0{
            key.x += adj_chunks[0];
            match self.chunks.get_mut(&key){
                Some(chunk) =>{
                    chunk.dirty[layer] = true;
                }
                None => {}
            }
            key.x -= adj_chunks[0];
        }
        if adj_chunks[1] != 0{
            key.y += adj_chunks[1];
            match self.chunks.get_mut(&key){
                Some(chunk) =>{
                    chunk.dirty[layer] = true;
                }
                None => {}
            }
        }
        if adj_chunks[0] != 0 && adj_chunks[1] != 0{
            key.x += adj_chunks[0];
            match self.chunks.get_mut(&key){
                Some(chunk) =>{
                    chunk.dirty[layer] = true;
                }
                None => {}
            }
        }
    }

    pub fn handle_input(&mut self, input: &InputManager){
        let x = (input.mouse_world_pos[0]+0.5-16.).floor() as i32;
        let y = (input.mouse_world_pos[1]+0.5-16.).floor() as i32;

        if input.right_mouse{
            self.set_tile(x,y,9,1);
        }
        else if input.left_mouse{
            self.set_tile(x,y,4,1);
        }
    }
}

pub struct ChunkBorders{
    pub top: Vec<bool>,
    pub bottom: Vec<bool>,
    pub left: Vec<bool>,
    pub right: Vec<bool>,
}

