use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
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
    pub fn chunk_tiles(&mut self, position: &ChunkPosition) -> Vec<Vec<Tile>>{
        let mut tiles = vec![vec![],vec![]];

        let big_noise = Fbm::<OpenSimplex>::new(self.seed)
            .set_frequency(0.012)
            .set_octaves(4)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let med_noise = Fbm::<OpenSimplex>::new(self.seed)
            .set_frequency(0.025)
            .set_octaves(4)
            .set_persistence(0.5)
            .set_lacunarity(2.2);

        let small_noise = Fbm::<OpenSimplex>::new(self.seed)
            .set_frequency(0.05)
            .set_octaves(3)
            .set_persistence(0.5)
            .set_lacunarity(2.2);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let tile_pos = [position.x * CHUNK_SIZE as i32 + x as i32,position.y * CHUNK_SIZE as i32 +y as i32];

                let height = (big_noise.get([tile_pos[0] as f64,0.]) *80.) as i32;
                let dirt_level = (height - 100 + (small_noise.get([tile_pos[0] as f64,tile_pos[1] as f64]) *200.) as i32).min(height);
                let cave_system = big_noise.get([tile_pos[0] as f64,tile_pos[1] as f64]);
                let cave = med_noise.get([tile_pos[0] as f64,tile_pos[1] as f64]);
                let ore = small_noise.get([tile_pos[1] as f64,tile_pos[0] as f64]);

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