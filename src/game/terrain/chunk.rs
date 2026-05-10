use easy_gpu::assets::{Material, Mesh};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use serde::{Deserialize, Serialize};
use crate::engine::render::Vertex;
use crate::game::terrain::chunk_manager::ChunkBorders;
use crate::game::terrain::terrain_generator::TerrainGenerator;
use crate::game::terrain::region::{RegionPosition, REGION_WIDTH};
use crate::game::terrain::tile::{Tile};

pub const CHUNK_SIZE: usize = 32;

const TEX_ATLAS_DIV: [f32;2] = [1./4.,1./3.];
const MARCH_SQR_DIV: f32 = 1./7.;

const TILE_TEX_COORDS: [[[f32;2];9];2] = [[ //bg
    [0.,0.],//1
    [1.,0.],//2
    [2.,0.],//3
    [3.,2.],//4
    [2.,1.],//5
    [2.,2.],//6
    [1.,2.],//7
    [0.,2.],//8
    [0.,2.],//9
],[ //fg
    [0.,0.],//1
    [1.,0.],//2
    [2.,0.],//3
    [3.,2.],//4
    [2.,1.],//5
    [2.,2.],//6
    [1.,2.],//7
    [0.,2.],//8
    [0.,2.],//9
]];

#[derive(Serialize,Deserialize)]
pub struct ChunkData {
    tiles: Vec<Vec<Tile>>,
}

impl ChunkData {
    pub fn new(position: &ChunkPosition,generator: &mut TerrainGenerator) -> Self{
        let tiles = generator.chunk_tiles(position);

        Self{
            tiles
        }
    }
}

pub struct Chunk{
    data: ChunkData,
    meshes: Vec<Option<Handle<Mesh>>>,
    materials: Vec<Option<Handle<Material>>>,
    pub dirty: Vec<bool>,
    pub save: bool,
}

impl Chunk{
    pub fn new(data:ChunkData)->Self{
        Self{
            data,
            meshes: vec![None,None],
            materials: vec![None,None],
            dirty: vec![true, true],
            save: false,
        }
    }

    pub fn get_tile(&self, x: usize, y: usize, layer: usize) -> &Tile{
        &self.data.tiles[layer][x * CHUNK_SIZE + y]
    }

    pub fn set_tile(&mut self, x: usize, y: usize,id:u8, layer: usize){
        self.data.tiles[layer][x * CHUNK_SIZE + y].id = id;
        self.dirty[layer] = true;

        if !self.save{
            self.save = true;
        }
    }

    pub fn has_mesh(&self)-> bool{
        self.meshes[0].is_some() || self.meshes[1].is_some()
    }

    pub fn dirty(&self)-> bool{
        self.dirty[0] || self.dirty[1]
    }

    pub fn data(&self) -> &ChunkData{
        &self.data
    }

    pub fn destroy_mesh(&mut self){
        self.dirty = vec![true,true];
        self.meshes = vec![None,None];
    }

    pub fn draw(&self,frame: &mut Frame,materials: &Vec<Handle<Material>>){
        for (mesh,material) in self.meshes.iter().zip(materials.iter()){
            if let Some(mesh_handle) = mesh{
                frame.draw_mesh(
                    material.clone(),
                    mesh_handle.clone(),
                );
            }
        }
    }

    pub fn build_mesh(&mut self,egpu: &mut easy_gpu::Renderer,layer: usize, position: &ChunkPosition,borders: ChunkBorders){
        let mut vertices = vec![];
        let mut indices = vec![];

        let mut sides: u16 = 0;

        for x in 0..CHUNK_SIZE{
            for y in 0..CHUNK_SIZE{
                if self.get_tile(x,y,layer).id > 0{
                    let neighbours = self.tile_neighbours(x as i32,y as i32,layer,&borders);
                    let marching_squares_coord = Self::map_marching_squares(neighbours);

                    let world_x = x as f32 + position.x as f32 * CHUNK_SIZE as f32;
                    let world_y = y as f32 + position.y as f32 * CHUNK_SIZE as f32;

                    let tex_index = TILE_TEX_COORDS[layer][self.get_tile(x,y,layer).id as usize-1];

                    vertices.push (Vertex::new(
                        [world_x - 0.5, world_y - 0.5,0.8 -layer as f32*0.5 ],
                        [(tex_index[0] + MARCH_SQR_DIV * marching_squares_coord[0]) * TEX_ATLAS_DIV[0],(tex_index[1] + MARCH_SQR_DIV * marching_squares_coord[1] + MARCH_SQR_DIV)* TEX_ATLAS_DIV[1]]
                    ));
                    vertices.push (Vertex::new(
                        [world_x + 0.5, world_y - 0.5,0.8 -layer as f32*0.5],
                        [(tex_index[0]  + MARCH_SQR_DIV * marching_squares_coord[0] + MARCH_SQR_DIV)* TEX_ATLAS_DIV[0],(tex_index[1]+ MARCH_SQR_DIV * marching_squares_coord[1] + MARCH_SQR_DIV)* TEX_ATLAS_DIV[1] ],
                    ));
                    vertices.push (Vertex::new(
                        [world_x + 0.5, world_y + 0.5,0.8 -layer as f32*0.5],
                        [(tex_index[0] + MARCH_SQR_DIV * marching_squares_coord[0] + MARCH_SQR_DIV)* TEX_ATLAS_DIV[0],(tex_index[1]  + MARCH_SQR_DIV * marching_squares_coord[1])* TEX_ATLAS_DIV[1]]
                    ));
                    vertices.push (Vertex::new(
                        [world_x - 0.5, world_y + 0.5,0.8 -layer as f32*0.5],
                        [(tex_index[0] + MARCH_SQR_DIV * marching_squares_coord[0])* TEX_ATLAS_DIV[0],(tex_index[1]  + MARCH_SQR_DIV * marching_squares_coord[1])* TEX_ATLAS_DIV[1]]
                    ));

                    indices.push(sides + 1);
                    indices.push(sides + 3);
                    indices.push(sides);

                    indices.push(sides + 1);
                    indices.push(sides + 2);
                    indices.push(sides + 3);

                    sides += 4;
                }
            }
        }

        self.dirty[layer] = false;

        let fg_index_count = indices.len() as u32;
        if fg_index_count == 0 {
            return;
        }

        let handle = egpu.create_mesh(vertices.as_slice(),indices.as_slice());
        self.meshes[layer] = Some(handle);
    }

    fn tile_neighbours(&self,x: i32, y: i32, layer: usize, borders: &ChunkBorders) -> Vec<i32>{
        let mut neighbours = Vec::new();

        for i in -1..=1{
            for j in -1..=1{
                if i == 0 && j == 0 {
                    continue;
                }
                if self.get_tile(x as usize,y as usize,layer).id > 0{
                    let neighbour = if x+i < 0 || x+i == CHUNK_SIZE as i32 || y+j < 0 || y+j == CHUNK_SIZE as i32 {
                        if if j == 1 && y == CHUNK_SIZE as i32 - 1{
                            borders.top[(x+i) as usize+1]
                        }
                        else if j == -1 && y == 0{
                            borders.bottom[(x+i) as usize+1]
                        }
                        else if i == 1{
                            borders.right[(y+j) as usize]
                        }
                        else{
                            borders.left[(y+j) as usize]
                        }{ 1 } else { 0 }
                    }
                    else if self.get_tile((x+i) as usize,(y+j) as usize, layer).id > 0
                    { 1 } else { 0 };

                    neighbours.push(neighbour);
                }
            }
        }

        neighbours
    }

    fn map_marching_squares(neighbours: Vec<i32>) -> [f32;2]{
        if neighbours == vec![1; 8] { [1.,2.] } // no sides
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[4] == 1 && neighbours[3] == 1 { //all +
            if neighbours[0] == 0 && neighbours[2] == 0 && neighbours[5] == 0 && neighbours[7] == 0 {[4.,4.]}//+
            else if neighbours[0] == 1 && neighbours[7] == 1 && neighbours[2] == 0 && neighbours[5] == 0{[5.,6.]}
            else if neighbours[2] == 1 && neighbours[5] == 1 && neighbours[0] == 0 && neighbours[7] == 0 {[4.,6.]}
            else if neighbours[0] == 1 && neighbours[2] == 0 && neighbours[5] == 0{[1.,5.]} //top right and right top ledge
            else if neighbours[0] == 0 && neighbours[2] == 1 && neighbours[7] == 0{[1.,6.]} //bottom right and right bottom ledge
            else if neighbours[5] == 0 && neighbours[7] == 1 && neighbours[2] == 0 {[3.,5.]} //bottom left and left bottom ledge
            else if neighbours[5] == 1 && neighbours[7] == 0 && neighbours[0] == 0{[3.,6.]} //top left and left top ledge
            else if neighbours[0] == 0 && neighbours[5] == 0 {[0.,6.]} //bottom prong
            else if neighbours[2] == 0 && neighbours[7] == 0 {[2.,5.]} //top prong
            else if neighbours[0] == 0 && neighbours[2] == 0 {[2.,6.]} //left prong
            else if neighbours[5] == 0 && neighbours[7] == 0 {[0.,5.]} //right prong
            else if neighbours[0] == 0 {[4.,2.]} //bottom left corner open
            else if neighbours[2] == 0 {[4.,3.]} //top left corner open
            else if neighbours[7] == 0 {[2.,4.]} //top right corner open
            else if neighbours[5] == 0 {[3.,4.]} //bottom right corner open
            else{ [4.,4.] } // +
        }
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[3] == 1 && neighbours[0] == 1 && neighbours[5] == 1 { [1.,1.] } // top open
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[4] == 1 && neighbours[2] == 1 && neighbours[7] == 1 { [1.,3.] } // bottom open
        else if neighbours[6] == 1 && neighbours[3] == 1 && neighbours[4] == 1 && neighbours[5] == 1 && neighbours[7] == 1 { [0.,2.] } // left open
        else if neighbours[1] == 1 && neighbours[3] == 1 && neighbours[4] == 1 && neighbours[2] == 1 && neighbours[0] == 1 { [2.,2.]  } // right open
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[3] == 1  && neighbours[5] == 1{ [5.,2.] } // left top ledge
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[3] == 1  && neighbours[0] == 1{ [6.,2.] } // right top ledge
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[4] == 1  && neighbours[2] == 1{ [6.,3.] } // left bottom ledge
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[4] == 1  && neighbours[7] == 1{ [5.,3.] } // right bottom ledge
        else if neighbours[3] == 1 && neighbours[4] == 1 && neighbours[6] == 1  && neighbours[5] == 1{ [5.,4.] } // top left ledge
        else if neighbours[3] == 1 && neighbours[4] == 1 && neighbours[1] == 1  && neighbours[0] == 1{ [6.,4.] } // top right ledge
        else if neighbours[3] == 1 && neighbours[4] == 1 && neighbours[6] == 1  && neighbours[7] == 1{ [4.,5.] } // bottom left ledge
        else if neighbours[3] == 1 && neighbours[4] == 1 && neighbours[1] == 1  && neighbours[2] == 1{ [5.,5.] } // bottom right ledge
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[4] == 1 { [5.,1.] } // T
        else if neighbours[1] == 1 && neighbours[6] == 1 && neighbours[3] == 1 { [6.,1.] } // upside down T
        else if neighbours[1] == 1 && neighbours[4] == 1 && neighbours[3] == 1 { [6.,0.] } // side T
        else if neighbours[6] == 1 && neighbours[4] == 1 && neighbours[3] == 1 { [5.,0.] } // side upside down T
        else if neighbours[1] == 1 && neighbours[6] == 1 { [2.,0.] } // =
        else if neighbours[1] == 0 && neighbours[4] == 1 && neighbours[6] == 1 && neighbours[7] == 1{ [0.,3.]} // solid L
        else if neighbours[1] == 1 && neighbours[4] == 1 && neighbours[6] == 0 && neighbours[2] == 1{ [2.,3.]} // back solid L
        else if neighbours[1] == 0 && neighbours[3] == 1 && neighbours[6] == 1 && neighbours[5] == 1{ [0.,1.]} // up down solid L
        else if neighbours[1] == 1 && neighbours[3] == 1 && neighbours[6] == 0 && neighbours[0] == 1{ [2.,1.]} // back up down solid L
        else if neighbours[1] == 1 && neighbours[4] == 1 && neighbours[6] == 0{ [4.,0.] } // backwards L
        else if neighbours[1] == 0 && neighbours[4] == 1 && neighbours[6] == 1{  [0.,4.]} //  L
        else if neighbours[1] == 0 && neighbours[3] == 1 && neighbours[6] == 1{  [1.,4.]} // up sown L
        else if neighbours[1] == 1 && neighbours[3] == 1 && neighbours[6] == 0{ [4.,1.] } // up sown back L
        else if neighbours[1] == 1{ [3.,0.] } // right cap
        else if neighbours[6] == 1{ [1.,0.] } //left cap
        else if neighbours[3] == 1 && neighbours[4] == 1 { [3.,2.] } // side =
        else if neighbours[3] == 1{ [3.,1.] } // top cap
        else if neighbours[4] == 1{ [3.,3.] } // bottom cap
        else{[0.,0.]}
    }

    pub fn mesh(&self,layer: usize) -> &Option<Handle<Mesh>>{
        &self.meshes[layer]
    }

}

#[derive(Hash, PartialEq,Clone,Debug,Eq)]
pub struct ChunkPosition{
    pub x: i32,
    pub y: i32,
}

impl ChunkPosition{
    pub fn new(x:i32,y:i32)->Self{
        Self{
            x,
            y,
        }
    }
    pub fn to_region_space(&self) -> RegionPosition {
        RegionPosition {
            x: self.x.div_euclid(REGION_WIDTH),
            y: self.y.div_euclid(REGION_WIDTH),
        }
    }

    pub fn to_world_space(&self) -> (i32,i32){
        (self.x * CHUNK_SIZE as i32, self.y * CHUNK_SIZE as i32)
    }

    pub fn from_world_space(x:i32,y:i32) -> Self{
        Self{
            x: x.div_euclid(CHUNK_SIZE as i32),
            y: y.div_euclid(CHUNK_SIZE as i32),
        }
    }
}




