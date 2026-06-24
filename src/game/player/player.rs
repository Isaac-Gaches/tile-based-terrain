use easy_gpu::assets::Material;
use easy_gpu::assets_manager::Handle;
use hecs::{EntityBuilder, World};
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{Light, Renderer, Sprite};
use crate::game::entities::bomb::Bomb;
use crate::game::entities::Despawn;
use crate::game::items::definition::ItemDefinition;
use crate::game::items::inventory::Inventory;
use crate::game::items::item_registry::ItemRegistry;
use crate::game::items::placeable::{EntityConfig, Placeable};
use crate::game::physics::collider::Collider;
use crate::game::physics::transform::Transform;
use crate::game::terrain::chunk_manager::ChunkManager;

pub struct Player{
    acceleration:f32,
    speed: f32,
    jump_speed:f32,
    hit_delay: f32,
}

impl Player{
    pub fn new() -> Self{
        Self{
            acceleration: 20.0,
            speed: 20.0,
            jump_speed: 25.0,
            hit_delay: 0.0,
        }
    }

    fn move_player(&self,input: &InputManager,collider:&mut Collider,dt: f32){
        if input.up && collider.on_ground{
            collider.y_vel = self.jump_speed;
        }

        if input.left {
            collider.x_vel -= self.acceleration * dt;
            if collider.x_vel > 0. {collider.x_vel -= self.acceleration * dt} // if changing diretion, change faster to feel more responsive
            if collider.x_vel < -self.speed { collider.x_vel = -self.speed; }

        }

        if input.right {
            collider.x_vel += self.acceleration * dt;
            if collider.x_vel < 0. {collider.x_vel += self.acceleration * dt}  // if changing diretion, change faster to feel more responsive
            if collider.x_vel > self.speed { collider.x_vel = self.speed; }
        }

        if !input.right && !input.left{ // slow down if no input
            if collider.x_vel > 0.{
                collider.x_vel -= self.acceleration*2. * dt;//slow fast
                if collider.x_vel < 0.{ collider.x_vel = 0.; }
            }
            else if collider.x_vel < 0.{
                collider.x_vel += self.acceleration*2. * dt;
                if collider.x_vel > 0.{ collider.x_vel = 0.; }
            }
        }
    }
}

pub fn spawn_player(world: &mut World,renderer: &mut Renderer){
    let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../../textures/player.png").to_vec());
    let mut atlas = renderer.create_atlas();
    atlas.add_frame([0.,0.],[1.0,1.0]);
    let material = renderer.create_sprite_material(texture,&atlas);

    world.spawn((
        Player::new(),
        Collider::new(2.8, 2.8, [0., 0.], 0., 0.,0., true, 0., 0.0),
        Transform::new([0.,40.],3.0),
        Sprite::new(material, 0),
    ));
}

fn throw_projectile(
    pos: [f32; 2],
    target: [f32; 2],
    world: &mut World,
    item: &ItemDefinition,
) {
    let dx = target[0] - pos[0];
    let dy = target[1] - pos[1];

    let len = (dx * dx + dy * dy).sqrt();

    if len <= f32::EPSILON {
        return;
    }

    let dir_x = dx / len;
    let dir_y = dy / len;

    if let Some(throwable) = &item.projectile {
        let mut builder = EntityBuilder::new();

        throwable.add_components(&mut builder,dir_x*-20.,dir_x,dir_y,pos);
        builder.add(item.sprite);
        builder.add(Despawn);

        world.spawn(builder.build());
    }
}

fn place_entity(
    pos: [f32; 2],
    world: &mut World,
    item: &ItemDefinition,
){
    let mut builder = EntityBuilder::new();

    builder.add(item.sprite);
    builder.add(Transform::new([pos[0].floor(),pos[1].floor()],1.0));
    builder.add(Despawn);

    world.spawn(builder.build());
}


pub fn update_player(world: &mut World,input_manager: &InputManager,inventory:&mut Inventory,item_registry: &ItemRegistry,dt: f32,terrain: &mut ChunkManager) -> [f32;2]{
    let mut pos = [0.,0.];
    let mut can_hit = false;
    for (_, (player, transform,collider)) in world.query::<(&mut Player, &Transform,&mut Collider)>().iter() {
        player.move_player(input_manager,collider,dt);
        if player.hit_delay <= 0.{
            can_hit = true;
            if input_manager.left_mouse || input_manager.right_mouse{
                player.hit_delay = 0.2;
            }
        }
        else{
            player.hit_delay -= dt;
        }
        pos = transform.translation;
    }
    if let Some(item_id) = inventory.held_item() {
        if input_manager.left_mouse {
            let definition = item_registry.definitions.get(item_id).unwrap();
            if can_hit && definition.projectile.is_some() {
                throw_projectile(pos, input_manager.mouse_world_pos, world, definition);
            } else if let Some(placeable) = &definition.placeable {
                match placeable {
                    Placeable::Entity(config) => {
                        place_entity(input_manager.mouse_world_pos,world,definition);
                    }
                    Placeable::Tile(ids) => {
                        let x = input_manager.mouse_world_pos[0].floor() as i32;
                        let y = input_manager.mouse_world_pos[1].floor() as i32;
                        terrain.set_tile(x, y, ids.id,1);
                    }
                }
            }
        } else if input_manager.right_mouse {
            let definition = item_registry.definitions.get(item_id).unwrap();
            if let Some(placeable) = &definition.placeable {
                match placeable {
                    Placeable::Entity(config) => {}
                    Placeable::Tile(ids) => {
                        let x = input_manager.mouse_world_pos[0].floor() as i32;
                        let y = input_manager.mouse_world_pos[1].floor() as i32;
                        terrain.set_tile(x,y,ids.id,0);
                    }
                }
            }
            else{
                let x = input_manager.mouse_world_pos[0].floor() as i32;
                let y = input_manager.mouse_world_pos[1].floor() as i32;
                terrain.set_tile(x, y, 0,1);
            }
        }
    }

    pos
}