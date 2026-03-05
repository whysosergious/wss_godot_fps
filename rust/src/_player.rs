use godot::classes::{
    input::MouseMode, AnimationPlayer, Camera3D, CharacterBody3D, CollisionShape3D,
    ICharacterBody3D, Input, MeshInstance3D, Node3D, PhysicsRayQueryParameters3D, RigidBody3D,
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
    weapon_bob_time: f32,
    #[export]
    weapon_bob_amount: f32,
    #[export]
    head_base_pos: Vector3,
    head_target_pos: Vector3,
    camera: Option<Gd<Camera3D>>,
    hands: Option<Gd<MeshInstance3D>>,
    #[export]
    hands_base_pos: Vector3,
    #[export]
    deadzone_percent: f32,
    #[export]
    head_turn_threshold: f32,
    #[export]
    hand_sway_sensitivity: f32,
    hand_sway_offset: Vector2,
    last_mouse_pos: Vector2,
    #[export]
    turn_accel_mult: f32,
    #[export]
    turn_decay_rate: f32,
    #[export]
    hand_snap_rate: f32,
    head_turn_velocity: f32,
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
            mouse_sensitivity: 0.0006,
            pitch: 0.0,
            fov: 90.0,
            ads_fov: 90.0,
            bob_time: 0.0,
            bob_amount: 0.1,
            weapon_bob_time: 0.0,
            weapon_bob_amount: 0.03,
            head_base_pos: Vector3::ZERO,
            head_target_pos: Vector3::ZERO,
            camera: None,
            hands: None,
            hands_base_pos: Vector3::ZERO,
            deadzone_percent: 0.5,
            head_turn_threshold: 0.785,
            hand_sway_sensitivity: 1.2,
            hand_sway_offset: Vector2::ZERO,
            last_mouse_pos: Vector2::ZERO,
            turn_accel_mult: 1.8,
            turn_decay_rate: 8.0,
            hand_snap_rate: 20.0,
            head_turn_velocity: 0.0,
        }
    }

    fn ready(&mut self) {
        let head = self.base().get_node_as::<Node3D>("HeadPivot");

        self.head_base_pos = head.get_position();

        let hands_node = self
            .base_mut()
            .get_node_as::<MeshInstance3D>("HeadPivot/Hands");
        self.hands_base_pos = hands_node.get_position();
        self.hands = Some(hands_node);

        let camera_node = self.base().get_node_as::<Camera3D>("HeadPivot/Camera");
        self.camera = Some(camera_node);

        Input::singleton().set_mouse_mode(MouseMode::CAPTURED);

        godot_print!("Head base pos: {:?}", self.head_base_pos);
    }

    fn physics_process(&mut self, delta: f64) {
        let mut head = self.base().get_node_as::<Node3D>("HeadPivot");

        // AUTO-CAPTURE mouse when window focused
        if Input::singleton().get_mouse_mode() != MouseMode::CAPTURED {
            Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        }

        // Mouse look
        if Input::singleton().is_action_pressed("ui_cancel") {
            // Escape
            Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
        } else {
            Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        }

        // *************************************
        // ---------------------------------------- start wip chunk
        // *************************************

        // head bob
        let velocity = self.base().get_velocity();
        let velocity_h = Vector3::new(velocity.x, 0.0, velocity.z);
        let speed = velocity_h.length();

        let mouse_delta = Input::singleton().get_last_mouse_velocity();

        let viewport_size = self.base().get_viewport().unwrap().get_visible_rect().size;

        // Normalize to viewport percentage
        let viewport_mouse_x = mouse_delta.x / viewport_size.x;
        let viewport_mouse_y = mouse_delta.y / viewport_size.y;
        let normalized_mouse = Vector2::new(viewport_mouse_x, viewport_mouse_y);

        // 1. HANDS sway (ALL mouse movement)
        let current_mouse = normalized_mouse;

        // REPLACE hands accumulation:
        if normalized_mouse.length() > 0.001 {
            // Ignore noise/warping
            self.last_mouse_pos += current_mouse;
        }
        let max_radius = self.deadzone_percent;
        if self.last_mouse_pos.length() > max_radius {
            self.last_mouse_pos = self.last_mouse_pos.normalized() * max_radius;
        }
        // Use existing last_mouse_pos (stays put), don't reset to ZERO
        self.hand_sway_offset = self.last_mouse_pos * self.hand_sway_sensitivity; // self.last_mouse_pos = current_mouse;

        // self.hand_sway_offset =
        // (current_mouse + self.last_mouse_pos) * 0.5 * self.hand_sway_sensitivity;

        // 2. HEAD input (hands at edge AND real mouse input)
        let hands_radius = self.last_mouse_pos.length();
        let has_active_input = normalized_mouse.length() > 0.01; // Real mouse movement

        let head_input = if hands_radius >= self.deadzone_percent - 0.01 && has_active_input {
            self.last_mouse_pos.normalized() * 0.3
        } else {
            Vector2::ZERO
        };

        // 3. Apply hands sway to Hands node

        // 3. Apply COMBINED sway + bob to Hands node (SINGLE BLOCK)
        if let Some(mut hands) = self.hands.take() {
            // SWAY offset (mouse-driven, persistent)
            let sway_offset = Vector3::new(
                self.hand_sway_offset.x * 0.3,
                -self.hand_sway_offset.y * 0.3,
                0.0,
            );

            // BOB offset (movement-driven)
            let bob_offset = if speed > 0.1 {
                self.weapon_bob_time += delta as f32 * speed * 2.0; // Faster bob
                Vector3::new(
                    (self.weapon_bob_time.sin() * 0.5).abs() * self.weapon_bob_amount, // Side bob
                    self.weapon_bob_time.sin() * self.weapon_bob_amount,               // Up/down
                    0.0,
                )
            } else {
                self.weapon_bob_time *= 0.9; // Dampen when stopped
                Vector3::ZERO
            };

            // COMBINED target = base + sway + bob
            let combined_offset = sway_offset + bob_offset;
            let target_pos = self.hands_base_pos + combined_offset;

            // Single smooth lerp
            let current_pos = hands.get_position();
            let new_pos = current_pos.lerp(target_pos, 3.0 * delta as f32);
            hands.set_position(new_pos);

            self.hands = Some(hands);
        }

        // Yaw input from deadzone-filtered mouse (already computed)
        let target_yaw = head_input.x * self.mouse_sensitivity * 50.0;

        // Parameters (could be exported later)
        let yaw_response = 20.0; // how quickly we follow target
        let yaw_decay = 10.0; // how quickly we stop when target is 0

        // Move velocity toward target when there IS input
        if head_input.x.abs() > 0.0001 {
            // accelerate toward target_yaw
            let diff = target_yaw - self.head_turn_velocity;
            self.head_turn_velocity += diff * yaw_response * delta as f32;
        } else {
            // no input: decay velocity toward 0
            self.head_turn_velocity -= self.head_turn_velocity * yaw_decay * delta as f32;
        }

        // Apply yaw (body)
        let final_yaw = self.head_turn_velocity;
        self.base_mut().rotate_y(-final_yaw);

        // Pitch (as you had it)
        let pitch_input = head_input.y * self.mouse_sensitivity * 50.0;
        self.pitch = pitch_input.clamp(-1.55, 1.55);

        let current_rot = head.get_rotation();
        let new_rot = Vector3::new(
            (current_rot.x - pitch_input).clamp(-1.55, 1.55),
            current_rot.y,
            current_rot.z,
        );
        head.set_rotation(new_rot);

        // HAND SNAP during decay (negative lerp speed)
        // if head_input.x.abs() < 0.0001 {
        //     let snap_strength = 5.0; // tune
        //     self.last_mouse_pos = self
        //         .last_mouse_pos
        //         .lerp(Vector2::ZERO, snap_strength * delta as f32);
        //     self.hand_sway_offset = self.last_mouse_pos * self.hand_sway_sensitivity;
        // }

        // *************************************
        // ---------------------------------------- end wip chunk
        // *************************************

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

        // Shooting/pushing (left click)
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
