use godot::classes::{AnimationPlayer, CharacterBody3D, CollisionShape3D, ICharacterBody3D, Input};
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(base=CharacterBody3D)]
struct Player {
    #[base]
    base: Base<CharacterBody3D>,
    #[export]
    speed: f32,
    #[export]
    jump: f32,
    #[export]
    gravity: f32,
}

#[godot_api]
impl Player {
    #[func]
    fn init(&mut self) {
        godot_print!("player init");
    }

    #[func]
    fn ready(&mut self) {
        godot_print!("player ready");
    }
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        let s = Self {
            base: base,
            speed: 1.0,
            jump: 1.0,
            gravity: 0.5,
        };

        godot_print!("player init - {:?}", s);

        s
    }

    fn physics_process(&mut self, delta: f64) {
        let input_dir =
            Input::singleton().get_vector("move_left", "move_right", "move_forward", "move_back");

        let direction =
            self.base().get_global_transform().basis * Vector3::new(input_dir.x, 0.0, input_dir.y);
        let mut velocity = self.base().get_velocity();

        godot_print!("dir - {}", direction);

        if self.base().is_on_floor() {
            if Input::singleton().is_action_just_pressed("jump") {
                velocity.y = self.jump; // FIX 1
            }
        } else {
            velocity.y -= self.gravity * delta as f32; // FIX 2
        }

        if input_dir != Vector2::ZERO {
            velocity.x = direction.x * self.speed;
            velocity.z = direction.z * self.speed;
        } else {
            velocity.x *= 0.8; // Friction
            velocity.z *= 0.8;
        }

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }
}
