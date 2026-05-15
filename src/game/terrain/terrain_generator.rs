use fast_noise_lite_rs::{FastNoiseLite, FractalType, NoiseType};
use crate::game::terrain::chunk::{CHUNK_SIZE, ChunkPosition};
use crate::game::terrain::tile::Tile;

pub struct TerrainGenerator{
    seed: u32,
    pub big_noise: FastNoiseLite,
    pub med_noise: FastNoiseLite,
    pub small_noise: FastNoiseLite,
}

impl TerrainGenerator{
    pub fn new()->Self{
        let seed = 1000;

        let mut big_noise = FastNoiseLite::new(seed as i32);
        big_noise.set_noise_type(NoiseType::OpenSimplex2);
        big_noise.set_frequency(0.005);
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
        if position.y > 30{
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
            let height = self.big_noise.get_noise_2d(tile_x,0.) * 30.;

            for y in 0..CHUNK_SIZE {
                let tile_y = (position.y * CHUNK_SIZE as i32 +y as i32) as f32;

                if tile_y > height {
                    tiles[1].push(Tile::new(0));
                    tiles[0].push(Tile::new(0));
                }
                else {
                    let dirt_level = (height - 100. + self.small_noise.get_noise_2d(tile_x,tile_y) *200.).min(height);

                    if (-0.1..-0.0).contains(&self.big_noise.get_noise_2d(tile_x,tile_y)) || self.med_noise.get_noise_2d(tile_x,tile_y) < -0.4 {
                        tiles[1].push(Tile::new(0));
                        if tile_y <= dirt_level {
                            tiles[0].push(Tile::new(3));
                        } else if tile_y < height-2. {
                            tiles[0].push(Tile::new(2));
                        } else {
                            tiles[0].push(Tile::new(0));
                        }
                    } else {
                        let ore = self.small_noise.get_noise_2d(tile_x,tile_y);

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
                        } else if tile_y < height - 2. {
                            tiles[1].push(Tile::new(2));
                            tiles[0].push(Tile::new(2));
                        } else {
                            tiles[1].push(Tile::new(1));
                            tiles[0].push(Tile::new(0));
                        }
                    }
                }
            }
        }

        tiles
    }
}