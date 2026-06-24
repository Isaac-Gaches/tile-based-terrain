use crate::engine::render::Sprite;
use crate::game::items::placeable::Placeable;
use crate::game::items::projectile::Projectile;

pub struct ItemDefinition{
    pub name: String,
    pub sprite: Sprite,
    pub icon_index: u32,

    pub projectile: Option<Projectile>,
    pub placeable: Option<Placeable>,
}