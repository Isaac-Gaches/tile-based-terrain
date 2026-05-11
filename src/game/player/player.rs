use crate::engine::input_manager::InputManager;
use crate::game::physics::collider::Collider;

pub struct Player{
    acceleration:f32,
    speed: f32,
    jump_speed:f32,
}

impl Player{
    pub fn new() -> Self{
        Self{
            acceleration: 20.0,
            speed: 15.0,
            jump_speed: 20.0,
        }
    }
    pub fn update(&self,input: &InputManager,collider:&mut Collider,dt: f32){
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

