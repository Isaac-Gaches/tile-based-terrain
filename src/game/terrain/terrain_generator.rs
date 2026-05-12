use fast_noise_lite_rs::{FastNoiseLite, FractalType, NoiseType};
use crate::game::terrain::chunk::{CHUNK_SIZE, ChunkPosition};
use crate::game::terrain::tile::Tile;

pub struct TerrainGenerator{
    seed: u32,
}

impl TerrainGenerator{
    pub fn new()->Self{
        Self {
            seed: 0,
        }
    }
    pub fn chunk_tiles(&self, position: &ChunkPosition) -> Vec<Vec<Tile>>{
        let mut tiles = vec![vec![],vec![]];

        let mut big_noise = FastNoiseLite::new(self.seed as i32);
        big_noise.set_noise_type(NoiseType::OpenSimplex2);
        big_noise.set_frequency(0.012);
        big_noise.set_fractal_type((FractalType::FBm));
        big_noise.set_fractal_octaves((4));
        big_noise.set_fractal_gain((0.5));
        big_noise.set_fractal_lacunarity((2.0));

        let mut med_noise = FastNoiseLite::new(self.seed as i32);
        med_noise.set_noise_type((NoiseType::OpenSimplex2));
        med_noise.set_frequency((0.025));
        med_noise.set_fractal_type((FractalType::FBm));
        med_noise.set_fractal_octaves((4));
        med_noise.set_fractal_gain((0.5));
        med_noise.set_fractal_lacunarity((2.2));

        let mut small_noise = FastNoiseLite::new(self.seed as i32);
        small_noise.set_noise_type((NoiseType::OpenSimplex2));
        small_noise.set_frequency((0.05));
        small_noise.set_fractal_type((FractalType::FBm));
        small_noise.set_fractal_octaves((3));
        small_noise.set_fractal_gain((0.5));
        small_noise.set_fractal_lacunarity((2.2));

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let tile_pos = [position.x * CHUNK_SIZE as i32 + x as i32,position.y * CHUNK_SIZE as i32 +y as i32];

                let height = (big_noise.get_noise_2d(tile_pos[0] as f32,0.) * 80.) as i32;
                let dirt_level = (height - 100 + (small_noise.get_noise_2d(tile_pos[0] as f32,tile_pos[1] as f32) *200.) as i32).min(height);
                let cave_system = big_noise.get_noise_2d(tile_pos[0] as f32, tile_pos[1] as f32);
                let cave = med_noise.get_noise_2d(tile_pos[0] as f32, tile_pos[1] as f32);
                let ore = small_noise.get_noise_2d(tile_pos[1] as f32,tile_pos[0] as f32);

                if tile_pos[1] > height {
                    tiles[1].push(Tile::new(0));
                    tiles[0].push(Tile::new(0));
                }
                else if cave_system > -0.16 && cave_system < -0.09 || cave < -0.22{
                    tiles[1].push(Tile::new(0));
                    if tile_pos[1] <= dirt_level {
                        tiles[0].push(Tile::new(3));
                    }
                    else if tile_pos[1] < height-3{
                        tiles[0].push(Tile::new(2));
                    }
                    else{
                        tiles[0].push(Tile::new(0));
                    }
                }
                else if ore < -0.35 {
                    tiles[1].push(Tile::new(6));
                    tiles[0].push(Tile::new(3));
                }
                else if tile_pos[1] <= dirt_level {
                    tiles[1].push(Tile::new(3));
                    tiles[0].push(Tile::new(3));
                }
                else if tile_pos[1] < height -20 {
                    tiles[1].push(Tile::new(3));
                    tiles[0].push(Tile::new(3));
                }
                else if tile_pos[1] < height-3 {
                    tiles[1].push(Tile::new(2));
                    tiles[0].push(Tile::new(2));
                }
                else {
                    tiles[1].push(Tile::new(1));
                    tiles[0].push(Tile::new(0));
                }
            }
        }

        tiles
    }
}