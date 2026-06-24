use hecs::World;
use crate::game::physics::transform::Transform;
use crate::game::terrain::chunk_manager::ChunkManager;

pub struct Grass;

pub struct Vine;

pub fn update_grass(world: &mut World, terrain: &mut ChunkManager) {
    let mut despawn = Vec::new();
    for (entity, (transform,_)) in world.query::<(&Transform,&Grass)>().iter() {
        if let Some(tile) = terrain.get_tile(transform.translation[0] as i32,transform.translation[1] as i32, 1){
            if tile.id != 1{
                despawn.push(entity);
            }
        }
        else{
            despawn.push(entity);
        }
    }
    for entity in despawn{
        let _ = world.despawn(entity);
    }
}

pub fn update_vine(world: &mut World, terrain: &mut ChunkManager) {
    let mut despawn = Vec::new();
    for (entity, (transform,_)) in world.query::<(&Transform,&Vine)>().iter() {
        if terrain.get_deco(transform.translation[0] as i32,transform.translation[1] as i32+1).is_none(){
            if let Some(tile) = terrain.get_tile(transform.translation[0] as i32,transform.translation[1] as i32 + 1, 1){
                if tile.id != 1{
                    despawn.push(entity);
                    terrain.remove_deco(transform.translation[0] as i32,transform.translation[1] as i32)
                }
            }
            else{
                despawn.push(entity);
                terrain.remove_deco(transform.translation[0] as i32,transform.translation[1] as i32)
            }
        }
    }
    for entity in despawn{
        let _ = world.despawn(entity);
    }
}
