use hecs::EntityBuilder;
use crate::game::items::projectile::LightConfig;

pub enum Placeable {
    Entity(EntityConfig),
    Tile(TileConfig)
}

pub struct TileConfig {
    pub id: u8,
}

pub struct EntityConfig{
    pub light: Option<LightConfig>
}

impl EntityConfig {
    pub fn add_component(&self,builder: &mut EntityBuilder){
        if let Some(light) = &self.light {
        builder.add(light.build());
        }
    }
}
