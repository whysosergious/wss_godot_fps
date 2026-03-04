use godot::classes::{
    input::MouseMode, AnimationPlayer, Camera3D, CharacterBody3D, CollisionShape3D,
    ICharacterBody3D, Input, Node3D, PhysicsRayQueryParameters3D, RigidBody3D,
};
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
    #[export]
    mouse_sensitivity: f32,
    #[export]
    pitch: f32,
    #[export]
    fov: f32,
    #[export]
    ads_fov: f32,
    #[export]
    bob_time: f32,
    #[export]
    bob_amount: f32,
    #[export]
    weapon_bob_amount: f32,
    #[export]
    head_base_pos: Vector3,
    #[export]
    head_target_pos: Vector3,
    // camera: Option<Gd<Camera3D>>,
}

#[godot_api]
impl Player {}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        Self {
            base,
            speed: 5.0,
            jump: 4.5,
            gravity: 10.0,
            mouse_sensitivity: 0.00002,
            pitch: 0.0,
            fov: 90.0,
            ads_fov: 90.0,
            bob_time: 0.0,
            bob_amount: 0.1,
            weapon_bob_amount: 0.02,
            head_base_pos: Vector3::ZERO,
            head_target_pos: Vector3::ZERO,
            // camera: base.get_node_as::<Camera3D>("Camera"),
        }
    }

    fn ready(&mut self) {
        let head = self.base().get_node_as::<Node3D>("HeadPivot");
        self.head_base_pos = head.get_position();
        godot_print!("Head base pos: {:?}", self.head_base_pos);
    }

    fn physics_process(&mut self, delta: f64) {
        let mut head = self.base().get_node_as::<Node3D>("HeadPivot");

        // Mouse look
        if Input::singleton().is_action_pressed("ui_cancel") {
            // Escape
            Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
        } else {
            Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        }

        let mouse_delta = Input::singleton().get_last_mouse_velocity();
        let yaw = mouse_delta.x * self.mouse_sensitivity;
        let pitch_input = mouse_delta.y * self.mouse_sensitivity;

        self.pitch = pitch_input;
        self.pitch = self.pitch.clamp(-1.55, 1.55);

        self.base_mut().rotate_y(-yaw); // Yaw whole body

        let current_rot = head.get_rotation();
        let new_rot = Vector3::new(
            (current_rot.x - pitch_input).clamp(-1.55, 1.55),
            current_rot.y,
            current_rot.z,
        );
        head.set_rotation(new_rot);

        // head bob
        let velocity = self.base().get_velocity();
        let velocity_h = Vector3::new(velocity.x, 0.0, velocity.z);
        let speed = velocity_h.length();

        if speed > 0.1 {
            self.bob_time += delta as f32 * speed * 2.0;
            let bob_offset = Vector3::new(
                (self.bob_time.sin() * 0.5).abs() * self.bob_amount,
                self.bob_time.sin() * self.bob_amount,
                0.0,
            );
            self.head_target_pos = self.head_base_pos + bob_offset; // USE STORED BASE

            let current_pos = head.get_position();
            let new_pos = current_pos.lerp(self.head_target_pos, 10.0 * delta as f32);
            head.set_position(new_pos);
        } else {
            self.bob_time = 0.0;
            let new_pos = head
                .get_position()
                .lerp(self.head_base_pos, 2.0 * delta as f32);
            head.set_position(new_pos);
        }

        // input
        let input_dir =
            Input::singleton().get_vector("move_left", "move_right", "move_forward", "move_back");

        let direction =
            self.base().get_global_transform().basis * Vector3::new(input_dir.x, 0.0, input_dir.y);
        let mut velocity = self.base().get_velocity();

        if self.base().is_on_floor() {
            if Input::singleton().is_action_just_pressed("jump") {
                velocity.y = self.jump;
            }
        } else {
            velocity.y -= self.gravity * delta as f32;
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

        // Shooting (left click)

        if Input::singleton().is_action_just_pressed("shoot") {
            let camera = self.base().get_node_as::<Camera3D>("HeadPivot/Camera3D");
            let from = camera.get_global_position();
            let to = from - camera.get_global_transform().basis.col_c() * 100.0;

            let mut space_state = self
                .base_mut()
                .get_world_3d()
                .unwrap()
                .get_direct_space_state()
                .unwrap();

            if let Some(query) = PhysicsRayQueryParameters3D::create(from, to) {
                let result = space_state.intersect_ray(&query);
                if !result.is_empty() {
                    godot_print!("Hit: {:?}", result);

                    if let Some(mut body) = result
                        .get("collider")
                        .and_then(|n| n.try_to::<Gd<RigidBody3D>>().ok())
                    {
                        let hit_position = result
                            .get("position")
                            .and_then(|v| v.try_to::<Vector3>().ok())
                            .unwrap_or(from);
                        body.upcast_mut::<RigidBody3D>()
                            .apply_impulse((from - hit_position) * -3.0);
                    }
                }
            }
        }
    }
}
