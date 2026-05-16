use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ahash::{AHashMap, AHashSet};
use easy_gpu::assets::Material;
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use rayon::iter::{IntoParallelIterator, ParallelDrainFull, ParallelDrainRange};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator, ParallelSlice};
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::game::terrain::chunk::{CHUNK_SIZE, ChunkPosition, ChunkData, Chunk};
use crate::game::terrain::terrain_generator::TerrainGenerator;
use crate::game::terrain::tile::Tile;

pub struct ChunkManager{
    pub dirty: bool,
    chunks: AHashMap<ChunkPosition,Chunk>,
    data_load_queue: AHashSet<ChunkPosition>,
    mesh_load_queue: AHashSet<ChunkPosition>,
    mesh_materials: Vec<Handle<Material>>
}
pub const HORIZONTAL_CHUNK_LOAD_DISTANCE: i32 = 3;
pub const VERTICAL_CHUNK_LOAD_DISTANCE: i32 = 2;


impl ChunkManager{
    pub fn new()->Self{
        Self{
            dirty: false,
            chunks: AHashMap::new(),
            data_load_queue: AHashSet::new(),
            mesh_load_queue: AHashSet::new(),
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

            if x_dist >= (HORIZONTAL_CHUNK_LOAD_DISTANCE + 2) as f32 * CHUNK_SIZE as f32 || y_dist >= (VERTICAL_CHUNK_LOAD_DISTANCE + 2) as f32 * CHUNK_SIZE as f32{
                unload.push(chunk_pos.clone());
            }
        }
        
        for key in &unload{
            self.chunks.remove(key);
        }
    }

    pub fn update_data_queue(&mut self, player_pos: [f32;2]){
        let player_chunk = ChunkPosition::new(
            player_pos[0].div_euclid(CHUNK_SIZE as f32) as i32,
            player_pos[1].div_euclid(CHUNK_SIZE as f32) as i32
        );
        for x in -(HORIZONTAL_CHUNK_LOAD_DISTANCE+1)..=(HORIZONTAL_CHUNK_LOAD_DISTANCE+1) {
            for y in -(VERTICAL_CHUNK_LOAD_DISTANCE+1)..=(VERTICAL_CHUNK_LOAD_DISTANCE+1) {
                let chunk_pos = ChunkPosition::new(
                    player_chunk.x + x,
                    player_chunk.y + y
                );

                if !self.chunks.contains_key(&chunk_pos){
                    self.data_load_queue.insert(chunk_pos);
                }
            }
        }
    }

    pub fn update_mesh_queue(&mut self,player_pos: [f32;2]){
        for (chunk_pos,chunk) in &mut self.chunks{
            let x_dist = ((chunk_pos.x * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[0]).abs();
            let y_dist = ((chunk_pos.y * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[1]).abs();

            if x_dist >= (HORIZONTAL_CHUNK_LOAD_DISTANCE) as f32 * CHUNK_SIZE as f32 || y_dist >= (VERTICAL_CHUNK_LOAD_DISTANCE) as f32 * CHUNK_SIZE as f32{
                if chunk.has_mesh(){
                    chunk.destroy_mesh();
                }
            }
            else if chunk.dirty(){
                self.mesh_load_queue.insert(chunk_pos.clone());
                self.dirty = true;
            }
        }
    }

    pub fn load_chunks_data(
        &mut self,
        file_manager: &Arc<FileManager>,
        terrain_generator: &Arc<TerrainGenerator>,
    ) {
        if self.data_load_queue.len() == 0{
            return;
        }
        let loaded_chunks: Vec<(ChunkPosition, Chunk)> = self
            .data_load_queue
            .par_drain()
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
        if self.mesh_load_queue.is_empty() {
            return;
        }

        struct MeshJob<'a> {
            chunk_pos: ChunkPosition,
            layer: usize,
            borders: ChunkBorders,
            chunk: &'a Chunk,
        }

        let queued: Vec<_> = self.mesh_load_queue.drain().collect();

        // Store dirty flags BEFORE removing marks
        let mut dirty_chunks = Vec::new();

        for chunk_pos in &queued {
            if let Some(chunk) = self.chunks.get(chunk_pos) {
                dirty_chunks.push((chunk_pos.clone(), chunk.dirty));
            }
        }

        // Now clear dirty flags
        for chunk_pos in &queued {
            if let Some(chunk) = self.chunks.get_mut(chunk_pos) {
                chunk.remove_mark();
            }
        }

        // Build jobs
        let mut jobs = Vec::<MeshJob>::new();

        for (chunk_pos, dirty) in dirty_chunks {
            let chunk = match self.chunks.get(&chunk_pos) {
                Some(chunk) => chunk,
                None => continue,
            };

            if dirty[1] {
                jobs.push(MeshJob {
                    chunk_pos: chunk_pos.clone(),
                    layer: 1,
                    borders: self.chunk_borders(&chunk_pos, 1),
                    chunk,
                });
            }

            if dirty[0] {
                jobs.push(MeshJob {
                    chunk_pos: chunk_pos.clone(),
                    layer: 0,
                    borders: self.chunk_borders(&chunk_pos, 0),
                    chunk,
                });
            }
        }

        if jobs.is_empty() {
            return;
        }

        // Parallel mesh generation
        let generated_meshes: Vec<_> = jobs
            .into_par_iter()
            .filter_map(|job| {
                let mesh_data = job.chunk.build_mesh(
                    job.layer,
                    &job.chunk_pos,
                    &job.borders,
                )?;

                Some((job.chunk_pos, job.layer, mesh_data))
            })
            .collect();

        // GPU upload
        for (chunk_pos, layer, mesh_data) in generated_meshes {
            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                let mesh = egpu.create_mesh(&mesh_data.0, &mesh_data.1);

                chunk.set_mesh(layer, mesh);
            }
        }
    }

    pub fn chunk_borders(
        &self,
        chunk_pos: &ChunkPosition,
        layer: usize,
    ) -> ChunkBorders {

        let north_pos = ChunkPosition::new(chunk_pos.x, chunk_pos.y + 1);
        let south_pos = ChunkPosition::new(chunk_pos.x, chunk_pos.y - 1);
        let west_pos  = ChunkPosition::new(chunk_pos.x - 1, chunk_pos.y);
        let east_pos  = ChunkPosition::new(chunk_pos.x + 1, chunk_pos.y);

        let north = self.chunks.get(&north_pos);
        let south = self.chunks.get(&south_pos);
        let west  = self.chunks.get(&west_pos);
        let east  = self.chunks.get(&east_pos);

        let mut top = [true; CHUNK_SIZE + 2];
        let mut bottom = [true; CHUNK_SIZE + 2];
        let mut left = [true; CHUNK_SIZE];
        let mut right = [true; CHUNK_SIZE];

        // =========================
        // TOP
        // =========================

        if let Some(chunk) = north {
            for x in 0..CHUNK_SIZE {
                top[x + 1] = chunk
                    .get_tile(x, 0, layer)
                    .solid();
            }

            // corners
            top[0] = west
                .map(|c| c.get_tile(CHUNK_SIZE - 1, CHUNK_SIZE - 1, layer).solid())
                .unwrap_or(true);

            top[CHUNK_SIZE + 1] = east
                .map(|c| c.get_tile(0, CHUNK_SIZE - 1, layer).solid())
                .unwrap_or(true);
        }

        // =========================
        // BOTTOM
        // =========================

        if let Some(chunk) = south {
            for x in 0..CHUNK_SIZE {
                bottom[x + 1] = chunk
                    .get_tile(x, CHUNK_SIZE - 1, layer)
                    .solid();
            }

            // corners
            bottom[0] = west
                .map(|c| c.get_tile(CHUNK_SIZE - 1, 0, layer).solid())
                .unwrap_or(true);

            bottom[CHUNK_SIZE + 1] = east
                .map(|c| c.get_tile(0, 0, layer).solid())
                .unwrap_or(true);
        }

        // =========================
        // LEFT
        // =========================

        if let Some(chunk) = west {
            for y in 0..CHUNK_SIZE {
                left[y] = chunk
                    .get_tile(CHUNK_SIZE - 1, y, layer)
                    .solid();
            }
        }

        // =========================
        // RIGHT
        // =========================

        if let Some(chunk) = east {
            for y in 0..CHUNK_SIZE {
                right[y] = chunk
                    .get_tile(0, y, layer)
                    .solid();
            }
        }

        ChunkBorders {
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

    pub fn extract_tiles(&self, player_pos: [f32;2]) -> Vec<u8>{
        let horizontal_load_distance = HORIZONTAL_CHUNK_LOAD_DISTANCE*CHUNK_SIZE as i32;
        let vertical_load_distance = VERTICAL_CHUNK_LOAD_DISTANCE*CHUNK_SIZE as i32;

        let mut tiles = vec![1;(horizontal_load_distance as usize *2 + CHUNK_SIZE)*(vertical_load_distance as usize *2 + CHUNK_SIZE)];//solid

        let player_chunk = [
            player_pos[0].div_euclid(CHUNK_SIZE as f32) as i32* CHUNK_SIZE as i32,
            player_pos[1].div_euclid(CHUNK_SIZE as f32) as i32* CHUNK_SIZE as i32
        ];

        for x in -horizontal_load_distance..horizontal_load_distance + CHUNK_SIZE as i32{
            for y in -vertical_load_distance..vertical_load_distance + CHUNK_SIZE as i32{
                let fg_tile = self.get_tile(player_chunk[0] + x,player_chunk[1] + y,1).expect("no tile");
                let bg_tile = self.get_tile(player_chunk[0] + x,player_chunk[1]+ y,0).expect("no tile");

                let width = horizontal_load_distance * 2 + CHUNK_SIZE as i32;

                if fg_tile.id == 0{
                    if bg_tile.id == 0{
                        tiles[((y+vertical_load_distance) * width + (x+horizontal_load_distance )) as usize] = 0;//empty
                    }
                    else{
                        tiles[((y+vertical_load_distance ) * width + (x+horizontal_load_distance)) as usize] = 2;//wall
                    }
                }
                else if fg_tile.id == 4{//lights
                    tiles[((y+vertical_load_distance) * width + (x+horizontal_load_distance)) as usize] = 4;
                }
                else if fg_tile.id == 6{
                    tiles[((y+vertical_load_distance ) * width + (x+horizontal_load_distance)) as usize] = 6;
                }
                else if fg_tile.id == 9{
                    tiles[((y+vertical_load_distance ) * width + (x+horizontal_load_distance )) as usize] = 9;
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
            self.set_tile(x,y,0,1);
        }
        else if input.left_mouse{
            self.set_tile(x,y,4,1);
        }
    }
}

pub struct ChunkBorders {
    pub top: [bool; CHUNK_SIZE + 2],
    pub bottom: [bool; CHUNK_SIZE + 2],
    pub left: [bool; CHUNK_SIZE],
    pub right: [bool; CHUNK_SIZE],
}

