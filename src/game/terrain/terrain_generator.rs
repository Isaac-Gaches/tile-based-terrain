use fast_noise_lite_rs::{FastNoiseLite, FractalType, NoiseType};
use rand::prelude::StdRng;
use rand::{random_range, Rng, SeedableRng};
use crate::game::terrain::chunk::{CHUNK_SIZE, ChunkPosition};
use crate::game::terrain::tile::{Deco, Tile};

pub struct TerrainGenerator{
    seed: u64,
    pub big_noise: FastNoiseLite,
    pub med_noise: FastNoiseLite,
    pub small_noise: FastNoiseLite,
}

impl TerrainGenerator{
    pub fn new()->Self{
        let seed = 70;

        let mut big_noise = FastNoiseLite::new(seed as i32);
        big_noise.set_noise_type(NoiseType::OpenSimplex2);
        big_noise.set_frequency(0.009);
        big_noise.set_fractal_type(FractalType::FBm);
        big_noise.set_fractal_octaves(4);
        big_noise.set_fractal_gain(0.5);
        big_noise.set_fractal_lacunarity(2.0);

        let mut med_noise = FastNoiseLite::new(seed as i32);
        med_noise.set_noise_type(NoiseType::OpenSimplex2);
        med_noise.set_frequency(0.02);
        med_noise.set_fractal_type(FractalType::FBm);
        med_noise.set_fractal_octaves(2);
        med_noise.set_fractal_gain(0.5);
        med_noise.set_fractal_lacunarity(2.2);

        let mut small_noise = FastNoiseLite::new(seed as i32);
        small_noise.set_noise_type(NoiseType::OpenSimplex2);
        small_noise.set_frequency(0.03);
        small_noise.set_fractal_type(FractalType::FBm);
        small_noise.set_fractal_octaves(3);
        small_noise.set_fractal_gain(0.5);
        small_noise.set_fractal_lacunarity(2.2);

        Self {
            seed,
            big_noise,
            med_noise,
            small_noise,
        }
    }
    pub fn chunk_tiles(&self, position: &ChunkPosition) -> [Vec<Tile>;2]{
        if position.y > 50{
            return [
                vec![Tile::new(0);CHUNK_SIZE*CHUNK_SIZE],
                vec![Tile::new(0);CHUNK_SIZE*CHUNK_SIZE],
            ];
        }

        let mut tiles = [
            Vec::with_capacity(CHUNK_SIZE*CHUNK_SIZE),
            Vec::with_capacity(CHUNK_SIZE*CHUNK_SIZE)
        ];

        for x in 0..CHUNK_SIZE {
            let tile_x = (position.x * CHUNK_SIZE as i32 + x as i32) as f32;
            let height = (self.big_noise.get_noise_2d(tile_x,0.)+1.0) * 25.;

            for y in 0..CHUNK_SIZE {
                let tile_y = (position.y * CHUNK_SIZE as i32 +y as i32) as f32;

                if tile_y > height {
                    tiles[1].push(Tile::new(0));
                    tiles[0].push(Tile::new(0));
                }
                else {
                    let dirt_level = (height - 100. + self.small_noise.get_noise_2d(tile_x,tile_y) *200.).min(height);

                    if (-0.25..-0.1).contains(&self.big_noise.get_noise_2d(tile_x,tile_y)) || self.med_noise.get_noise_2d(tile_x,tile_y) < -0.5 {
                        tiles[1].push(Tile::new(0));
                        if tile_y <= dirt_level {
                            tiles[0].push(Tile::new(3));
                        } else if tile_y < height-2. {
                            tiles[0].push(Tile::new(2));
                        } else {
                            tiles[0].push(Tile::new(0));
                        }
                    } else {
                        let ore = self.med_noise.get_noise_2d(-tile_x,-tile_y);

                        if tile_y <= dirt_level {
                            if ore < -0.6 {
                                tiles[1].push(Tile::new(6));
                                tiles[0].push(Tile::new(3));
                            }
                            else{
                                tiles[1].push(Tile::new(3));
                                tiles[0].push(Tile::new(3));
                            }
                        } else if tile_y < height - 20. {
                            if ore < -0.6 {
                                tiles[1].push(Tile::new(6));
                                tiles[0].push(Tile::new(3));
                            }
                            else{
                                tiles[1].push(Tile::new(3));
                                tiles[0].push(Tile::new(3));
                            }
                        } else if tile_y < height - 15. {
                            tiles[1].push(Tile::new(2));
                            tiles[0].push(Tile::new(2));
                        } else {
                            tiles[1].push(Tile::new(1));
                            if tile_y < height - 2. {
                                tiles[0].push(Tile::new(2));
                            }
                            else{
                                tiles[0].push(Tile::new(0));
                            }
                        }
                    }
                }
            }
        }

        tiles
    }

    pub fn generate_deco(&self, tiles: &Vec<Tile>) -> Vec<Deco>{
        let mut deco = Vec::with_capacity(64);
        let mut rng = StdRng::seed_from_u64(self.seed);
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if rng.random_bool(0.5) {
                    let index = x * CHUNK_SIZE + y;
                    if !tiles[index].solid() {
                        if y > 0 && tiles[index - 1].id == 1{
                            deco.push(Deco::new(rng.random_range(0..3), x as u8, y as u8));
                        }
                        else if y < CHUNK_SIZE - 1 && tiles[index + 1].id == 1 {
                            for i in 0..random_range(MIN_VINE_LENGTH..=MAX_VINE_LENGTH){
                                if i <= y as u8{
                                deco.push(Deco::new(3, x as u8, y as u8-i as u8));
                                }
                            }
                        }
                    }
                }
            }
        }
        deco
    }
}

pub const MAX_VINE_LENGTH: u8 = 10;
const MIN_VINE_LENGTH: u8 = 3;