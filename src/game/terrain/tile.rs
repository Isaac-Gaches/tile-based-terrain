use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct Tile{
    pub id: u8,
}

impl Tile{
    #[inline(always)]
    pub fn new(id: u8) -> Self{
        Self{
            id
        }
    }
    #[inline(always)]
    pub fn solid(&self) -> bool{
        self.id > 0
    }
}

#[derive(Serialize,Deserialize)]
pub struct Deco {
    pub x: u8,
    pub y: u8,
    id: u8,
}

impl Deco{
    #[inline(always)]
    pub fn new(id: u8, x: u8, y: u8) -> Self{
        Self{
            x,
            y,
            id
        }
    }
}

pub const TILE_LIGHT_SOURCES: [Option<[f32;3]>;9] = [
    None, //1
    None, //2
    None, //3
    None, //4
    None, //5
    Some([0.1,0.4,0.7]), //6
    None, //7
    None, //8
    None, //9
];