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