
use std::sync::{Arc};
use ahash::{AHashMap, AHashSet};
use easy_gpu::assets::Material;
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use hecs::{Entity, World};
use rayon::iter::{IntoParallelIterator};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{LightSource};
use crate::game::terrain::chunk::{CHUNK_SIZE, ChunkPosition, ChunkData, Chunk};
use crate::game::terrain::terrain_generator::TerrainGenerator;
use crate::game::terrain::tile::{Tile, TILE_LIGHT_SOURCES};

pub struct ChunkManager{
    pub dirty: bool,
    chunks: AHashMap<ChunkPosition,Chunk>,
    data_load_queue: AHashSet<ChunkPosition>,
    mesh_load_queue: AHashSet<ChunkPosition>,
}
pub const HORIZONTAL_CHUNK_LOAD_DISTANCE: i32 = 5;
pub const VERTICAL_CHUNK_LOAD_DISTANCE: i32 = 3;


impl ChunkManager{
    pub fn new()->Self{
        Self{
            dirty: false,
            chunks: AHashMap::new(),
            data_load_queue: AHashSet::new(),
            mesh_load_queue: AHashSet::new(),
        }
    }

    pub fn unload_chunks(&mut self,player_pos: [f32;2],world: &mut World){
        let mut unload = Vec::new();
        
        for chunk_pos in self.chunks.keys(){
            let x_dist = ((chunk_pos.x * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[0]).abs();
            let y_dist = ((chunk_pos.y * CHUNK_SIZE as i32) as f32 + (CHUNK_SIZE as f32/2.) - player_pos[1]).abs();

            if x_dist >= (HORIZONTAL_CHUNK_LOAD_DISTANCE + 2) as f32 * CHUNK_SIZE as f32 || y_dist >= (VERTICAL_CHUNK_LOAD_DISTANCE + 2) as f32 * CHUNK_SIZE as f32{
                unload.push(chunk_pos.clone());
            }
        }
        
        for key in &unload{
            let chunk = self.chunks.remove(key);
            if let Some(chunk) = chunk{
                chunk.despawn_deco(world);
            }
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

    pub fn update_mesh_queue(&mut self, player_pos: [f32; 2]) {
        let mut to_mesh = Vec::new();

        for (chunk_pos, chunk) in &mut self.chunks {
            let x_dist = (
                (chunk_pos.x * CHUNK_SIZE as i32) as f32
                    + (CHUNK_SIZE as f32 / 2.0)
                    - player_pos[0]
            ).abs();

            let y_dist = (
                (chunk_pos.y * CHUNK_SIZE as i32) as f32
                    + (CHUNK_SIZE as f32 / 2.0)
                    - player_pos[1]
            ).abs();

            if x_dist >= HORIZONTAL_CHUNK_LOAD_DISTANCE as f32 * CHUNK_SIZE as f32
                || y_dist >= VERTICAL_CHUNK_LOAD_DISTANCE as f32 * CHUNK_SIZE as f32
            {
                if x_dist >= (HORIZONTAL_CHUNK_LOAD_DISTANCE+1) as f32 * CHUNK_SIZE as f32
                    || y_dist >= (VERTICAL_CHUNK_LOAD_DISTANCE+1) as f32 * CHUNK_SIZE as f32
                {
                    if chunk.has_mesh() {
                        chunk.destroy_mesh();
                    }
                }
            }
            else if chunk.dirty() {
                to_mesh.push(chunk_pos.clone());
            }
        }

        for chunk_pos in to_mesh {
            if self.can_mesh_chunk(&chunk_pos) {
                self.mesh_load_queue.insert(chunk_pos);
                self.dirty = true;
            }
        }


    }

    pub fn can_mesh_chunk(&self, chunk_pos: &ChunkPosition) -> bool {
        const OFFSETS: [(i32, i32); 8] = [
            (-1, -1),
            ( 0, -1),
            ( 1, -1),
            (-1,  0),
            ( 1,  0),
            (-1,  1),
            ( 0,  1),
            ( 1,  1),
        ];

        let x = chunk_pos.x;
        let y = chunk_pos.y;

        OFFSETS.iter().all(|&(dx, dy)| {
            self.chunks.contains_key(&ChunkPosition::new(x + dx, y + dy))
        })
    }

    pub fn load_chunks_data(
        &mut self,
        file_manager: &Arc<FileManager>,
        terrain_generator: &Arc<TerrainGenerator>,
    ) {
        if self.data_load_queue.is_empty() {
            return;
        }

        const BUDGET: usize = 8;

        let queued: Vec<_> = self
            .data_load_queue
            .iter()
            .take(BUDGET)
            .cloned()
            .collect();

        if queued.is_empty() {
            return;
        }

        for item in queued.iter() {
            self.data_load_queue.take(item);
        }


        let loaded_chunks: Vec<_> = queued
            .into_par_iter()
            .map(|chunk_pos| {

                let chunk_data = file_manager
                    .load_chunk(&chunk_pos)
                    .unwrap_or_else(|| {
                        ChunkData::new(&chunk_pos, terrain_generator)
                    });

                (
                    chunk_pos,
                    Chunk::new(chunk_data),
                )
            })
            .collect();

        self.chunks.extend(loaded_chunks);
    }

    pub fn generate_chunk_meshes(
        &mut self, egpu: &mut easy_gpu::Renderer,
        world: &mut World,
        asset_registry: &AssetRegistry
    ) {
        if self.mesh_load_queue.is_empty() {
            return;
        }

        struct MeshJob<'a> {
            chunk_pos: ChunkPosition,
            layer: usize,
            borders: ChunkBorders,
            chunk: &'a Chunk,
        }

        const BUDGET: usize = 3;

        let queued: Vec<_> = self
            .mesh_load_queue
            .iter()
            .take(BUDGET)
            .cloned()
            .collect();

        for item in queued.iter() {
            self.mesh_load_queue.take(item);
        }

        let mut dirty_chunks = Vec::new();

        for chunk_pos in &queued {
            if let Some(chunk) = self.chunks.get_mut(chunk_pos) {
                dirty_chunks.push((chunk_pos.clone(), chunk.dirty));
                chunk.remove_mark();
            }
        }

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

        let generated_meshes: Vec<_> = jobs
            .into_par_iter()
            .map(|job| {
                let mesh_data = job.chunk.build_mesh(
                    job.layer,
                    &job.chunk_pos,
                    &job.borders,
                );

                (job.chunk_pos, job.layer, mesh_data)
            })
            .collect();

        for (chunk_pos, layer, mesh_data) in generated_meshes {
            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                if let Some(data) = mesh_data{
                    let mesh = egpu.create_mesh(&data.0, &data.1);
                    chunk.set_mesh(layer, mesh);
                    chunk.spawn_deco(world,&chunk_pos,asset_registry)
                }
                else{
                    chunk.meshes[layer] = None;
                }
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

        if let Some(chunk) = north {
            for x in 0..CHUNK_SIZE {
                top[x + 1] = chunk
                    .get_tile(x, 0, layer)
                    .solid();
            }

            top[0] = west
                .map(|c| c.get_tile(CHUNK_SIZE - 1, CHUNK_SIZE - 1, layer).solid())
                .unwrap_or(true);

            top[CHUNK_SIZE + 1] = east
                .map(|c| c.get_tile(0, CHUNK_SIZE - 1, layer).solid())
                .unwrap_or(true);
        }

        if let Some(chunk) = south {
            for x in 0..CHUNK_SIZE {
                bottom[x + 1] = chunk
                    .get_tile(x, CHUNK_SIZE - 1, layer)
                    .solid();
            }

            bottom[0] = west
                .map(|c| c.get_tile(CHUNK_SIZE - 1, 0, layer).solid())
                .unwrap_or(true);

            bottom[CHUNK_SIZE + 1] = east
                .map(|c| c.get_tile(0, 0, layer).solid())
                .unwrap_or(true);
        }

        if let Some(chunk) = west {
            for y in 0..CHUNK_SIZE {
                left[y] = chunk
                    .get_tile(CHUNK_SIZE - 1, y, layer)
                    .solid();
            }
        }

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

    pub fn draw(&self, frame: &mut Frame,asset_registry: &AssetRegistry){
        for (_,chunk) in &self.chunks{
            chunk.draw(frame,asset_registry);
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

    pub fn get_deco(&self,x:i32,y:i32) -> Option<&Entity>{
        let chunk_pos = ChunkPosition::from_world_space(x,y);
        if let Some(chunk) = self.chunks.get(&chunk_pos){
            return chunk.get_deco(x,y);
        }
        None
    }

    pub fn remove_deco(&mut self,x:i32,y:i32){
        let chunk_pos = ChunkPosition::from_world_space(x,y);
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos){
            chunk.remove_deco(x,y);
        }

    }

    pub fn extract_tiles(&self, player_pos: [f32; 2]) ->(Vec<u8>,Vec<LightSource>) {
        let chunk_size = CHUNK_SIZE as i32;

        let chunk_radius_x = HORIZONTAL_CHUNK_LOAD_DISTANCE;
        let chunk_radius_y = VERTICAL_CHUNK_LOAD_DISTANCE;

        let width_chunks = chunk_radius_x * 2 + 1;
        let height_chunks = chunk_radius_y * 2 + 1;

        let width_tiles = width_chunks * chunk_size;
        let height_tiles = height_chunks * chunk_size;

        let mut tiles = vec![1u8; (width_tiles * height_tiles) as usize];
        let mut lights = Vec::with_capacity(64);

        let player_chunk_x =
            player_pos[0].div_euclid(chunk_size as f32) as i32;

        let player_chunk_y =
            player_pos[1].div_euclid(chunk_size as f32) as i32;

        for cy in -chunk_radius_y..=chunk_radius_y {
            for cx in -chunk_radius_x..=chunk_radius_x {

                let chunk_pos = ChunkPosition::new(
                    player_chunk_x + cx,
                    player_chunk_y + cy,
                );

                let chunk = match self.chunks.get(&chunk_pos) {
                    Some(chunk) => chunk,
                    None => continue,
                };

                let base_x = (cx + chunk_radius_x) * chunk_size;
                let base_y = (cy + chunk_radius_y) * chunk_size;

                for y in 0..chunk_size as usize {
                    let row_start =
                        ((base_y as usize + y) * width_tiles as usize)
                            + base_x as usize;

                    for x in 0..chunk_size as usize {

                        let fg = chunk.get_tile(x, y, 1).id;
                        let bg = chunk.get_tile(x, y, 0).id;

                        if fg > 0 && let Some(light) = TILE_LIGHT_SOURCES[(fg-1) as usize]{
                            lights.push(LightSource::new([chunk_pos.x as f32 * CHUNK_SIZE as f32 + x as f32 ,chunk_pos.y as f32 * CHUNK_SIZE as f32  + y as f32],light))
                        }

                        let value = if fg == 0 {
                            if bg == 0 {
                                0
                            } else {
                                2
                            }
                        } else {
                           1
                        };

                        tiles[row_start + x] = value;
                    }
                }
            }
        }

        (tiles,lights)
    }

    pub fn set_tile(&mut self, x:i32, y:i32,id:u8,layer:usize){
        let mut key = ChunkPosition::new(
            (x as f32/CHUNK_SIZE as f32).floor() as i32,
            (y as f32/CHUNK_SIZE as f32).floor() as i32
        );

        let mut adj_chunks = [0,0];

        match self.chunks.get_mut(&key){
            Some(chunk) =>{
                let x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
                let y = y.rem_euclid(CHUNK_SIZE as i32) as usize;

                if x == 0 { adj_chunks[0] = -1; }
                else if x == CHUNK_SIZE-1 { adj_chunks[0] = 1; }
                if y == 0 { adj_chunks[1] = -1; }
                else if y == CHUNK_SIZE-1 { adj_chunks[1] = 1; }

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

    pub fn explode(&mut self, radius: i32,x: i32,y:i32){
        for i in -radius..radius{
            for j in -radius..radius{
                if i*i + j*j <= radius*radius{
                    self.set_tile(x+i,y+j,0,1);
                }
            }
        }
    }
}

pub struct ChunkBorders {
    pub top: [bool; CHUNK_SIZE + 2],
    pub bottom: [bool; CHUNK_SIZE + 2],
    pub left: [bool; CHUNK_SIZE],
    pub right: [bool; CHUNK_SIZE],
}

