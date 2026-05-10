use std::fs;
use crate::game::terrain::chunk::{ChunkPosition, ChunkData};

pub struct FileManager{
    world_path: std::path::PathBuf,
}

impl FileManager {
    pub fn new()->Self{
        Self{
            world_path: "worlds/world1/".into(),
        }
    }
    pub fn save_chunk(&self,chunk_data: &ChunkData, chunk_position: &ChunkPosition) -> std::io::Result<()> {
        let path = self.chunk_path(chunk_position);

        let bytes = bincode::serialize(&chunk_data)
            .expect("Failed to serialize chunk");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, bytes)
    }

    pub fn load_chunk(&self, chunk_position: &ChunkPosition) -> Option<ChunkData> {
        let path = self.chunk_path(chunk_position);

        let bytes = fs::read(path).ok()?;
        let data: ChunkData = bincode::deserialize(bytes.as_slice()).ok()?;

        Some(data)
    }

    fn chunk_path(&self,chunk_position: &ChunkPosition) -> std::path::PathBuf {
        let pos = [chunk_position.x.to_string().as_str(),chunk_position.y.to_string().as_str()].join("_");
        self.world_path.join(pos)
    }
}