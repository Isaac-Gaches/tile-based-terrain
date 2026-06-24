use ahash::AHashMap;
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::render::gui::GuiElement;
use crate::engine::render::Sprite;
use crate::game::items::definition::ItemDefinition;
use crate::game::items::placeable::{TileConfig, Placeable, EntityConfig};
use crate::game::items::projectile::{BombConfig, ColliderConfig, ProjectileConfig, Projectile, LightConfig};
use crate::game::terrain::tile::Tile;

pub struct ItemRegistry{
    pub definitions: AHashMap<ItemID,ItemDefinition>,
}

pub type ItemID = String;

impl ItemRegistry{
    pub fn new() -> Self{
        Self{
            definitions: AHashMap::new(),
        }
    }
    pub fn load(&mut self,asset_registry: &AssetRegistry){
        self.definitions.insert("bomb".to_string(),ItemDefinition{
            name: "Bomb".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,0),
            icon_index: 0,
            projectile: Some(Projectile {
                speed: 40.0,
                projectile_config: ProjectileConfig{
                    bomb: Some(BombConfig{
                        timer: 3.0,
                        radius: 8,
                        num_particles: 40,
                        particle_lifespan: 1.0,
                    }),
                    light: Some(LightConfig{
                        colour: [0.5,0.1,0.0],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.9,
                    height: 0.9,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.8,
                    friction: 0.4,
                },
            }),
            placeable: None,
        });

        self.definitions.insert("glow_stick".to_string(),ItemDefinition{
            name: "Glow Stick".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,1),
            icon_index: 1,
            projectile: Some(Projectile {
                speed: 50.0,
                projectile_config: ProjectileConfig{
                    bomb: None,
                    light: Some(LightConfig{
                        colour: [0.7,1.0,0.1],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.2,
                    height: 0.2,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.7,
                    friction: 0.7,
                },
            }),
            placeable: None,
        });

        self.definitions.insert("big_bomb".to_string(),ItemDefinition{
            name: "Big Bomb".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,2),
            icon_index: 2,
            projectile: Some(Projectile {
                speed: 30.0,
                projectile_config: ProjectileConfig{
                    bomb: Some(BombConfig{
                        timer: 5.,
                        radius: 16,
                        num_particles: 80,
                        particle_lifespan: 1.5,
                    }),
                    light: Some(LightConfig{
                        colour: [0.7,0.05,0.0],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.8,
                    height: 0.8,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.7,
                    friction: 0.9,
                },
            }),
            placeable: None,
        });

        self.definitions.insert("potion".to_string(),ItemDefinition{
            name: "Potion".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,3),
            icon_index: 3,
            projectile: Some(Projectile {
                speed: 40.0,
                projectile_config: ProjectileConfig{
                    bomb: None,
                    light: Some(LightConfig{
                        colour: [0.5,0.1,1.0],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.8,
                    height: 0.8,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.1,
                    friction: 0.8,
                },
            }),
            placeable: None,
        });

        self.definitions.insert("dirt".to_string(),ItemDefinition{
            name: "Dirt".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,3),//needs to change
            icon_index: 5,
            projectile: None,
            placeable: Some(Placeable::Tile(TileConfig {
                id: 3,
            })),
        });
        self.definitions.insert("red_light".to_string(),ItemDefinition{
            name: "Red Light".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,3),//needs to change
            icon_index: 4,
            projectile: None,
            placeable: Some(Placeable::Tile(TileConfig {
                id: 4,
            })),
        });
    }
}