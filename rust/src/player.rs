#[allow(unused_imports)]
use godot::classes::{
    input::MouseMode, AnimationPlayer, Camera3D, CanvasLayer, CharacterBody3D, CollisionShape3D,
    ColorRect, ICharacterBody3D, Input, MeshInstance3D, Node3D, PhysicsDirectSpaceState3D,
    PhysicsRayQueryParameters3D, RigidBody3D,
};
use godot::obj::WithBaseField;
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
    bob_time: f32,
    #[export]
    bob_amount: f32,
    head_base_pos: Vector3,
    head_target_pos: Vector3,
    camera: Option<Gd<Camera3D>>,
    hands: Option<Gd<Node3D>>,
    hands_base_pos: Vector3,

    // crosshair
    #[export]
    crosshair_visible: bool,
    crosshair_pos: Vector2,
    crosshair_node: Option<Gd<ColorRect>>,
    crosshair_offset: Vector2,

    deadzone_percent: f32,
    last_mouse_pos: Vector2,

    // point hands
    #[export]
    raycast_distance: f32,
    crosshair_world_pos: Vector3,
    hit_something: bool,

    crosshair_offset_world: Vector3,

    #[export]
    hand_pitch_limit: f32, // Less up/down
    #[export]
    hand_yaw_limit: f32, // Left/right rotation
    hand_center_offset: Vector3,
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
            mouse_sensitivity: 1.0,
            pitch: 0.0,
            fov: 90.0,
            ads_fov: 90.0,
            bob_time: 0.0,
            bob_amount: 0.1,
            head_base_pos: Vector3::ZERO,
            head_target_pos: Vector3::ZERO,
            camera: None,
            hands: None,
            hands_base_pos: Vector3::ZERO,

            // crosshair
            crosshair_visible: true,
            crosshair_pos: Vector2::ZERO,
            crosshair_node: None,
            crosshair_offset: Vector2::ZERO,

            deadzone_percent: 0.4,
            last_mouse_pos: Vector2::ZERO,

            // point hands
            raycast_distance: 100.0,
            crosshair_world_pos: Vector3::ZERO,
            hit_something: false,

            crosshair_offset_world: Vector3::ZERO,

            hand_pitch_limit: 0.1,                            // Less up/down
            hand_yaw_limit: 0.3,                              // Left/right rotation
            hand_center_offset: Vector3::new(0.0, -0.2, 0.5), // Below center
        }
    }

    fn ready(&mut self) {
        Input::singleton().set_mouse_mode(MouseMode::HIDDEN);

        let head = self.base().get_node_as::<Node3D>("HeadPivot");

        self.head_base_pos = head.get_position();

        let hands_node = self.base_mut().get_node_as::<Node3D>("HeadPivot/Hands");
        self.hands_base_pos = hands_node.get_position();
        self.hands = Some(hands_node);

        let camera_node = self.base().get_node_as::<Camera3D>("HeadPivot/Camera");
        self.camera = Some(camera_node);

        let crosshair_node = self
            .base()
            .get_node_as::<ColorRect>("CrosshairLayer/CrosshairDot");
        let crosshair_size = crosshair_node.get_size();
        self.crosshair_offset = crosshair_size / 2.0;
        self.crosshair_node = Some(crosshair_node);

        let viewport = self.base().get_viewport().unwrap();
        let viewport_size = viewport.get_visible_rect().size;
        let deadzone_size = viewport_size * self.deadzone_percent;
        let padding = (viewport_size - deadzone_size) / 2.0;

        // CRITICAL: Initialize at EXACT deadzone center
        self.crosshair_pos = Vector2::ZERO;
        self.last_mouse_pos = padding + (deadzone_size / 2.0);

        godot_print!("Crosshair node: {:?}", self.crosshair_node.is_some());
        godot_print!("Head base pos: {:?}", self.head_base_pos);
    }

    fn process(&mut self, delta: f64) {
        let viewport = self.base().get_viewport().unwrap();
        let viewport_size = viewport.get_visible_rect().size;
        let screen_center = viewport_size / 2.0;

        let deadzone_size = viewport_size * self.deadzone_percent;

        let mouse_screen_pos = viewport.get_mouse_position();

        let mouse_velocity = (mouse_screen_pos - self.last_mouse_pos) * self.mouse_sensitivity;
        self.crosshair_pos += mouse_velocity;

        let max_offset = deadzone_size / 2.0;
        self.crosshair_pos = self.crosshair_pos.clamp(-max_offset, max_offset);

        self.last_mouse_pos = mouse_screen_pos;

        if let Some(mut crosshair) = self.crosshair_node.take() {
            if self.crosshair_visible {
                let screen_pos = screen_center + self.crosshair_pos - self.crosshair_offset;
                crosshair.set_position(screen_pos);
                crosshair.show();
            } else {
                crosshair.hide();
            }
            self.crosshair_node = Some(crosshair);
        }

        // *************************************
        // ---------------------------------------- start wip chunk
        // *************************************

        // Raycast from camera through crosshair - FULL VERSION
        if let Some(camera) = &self.camera {
            let crosshair_screen = screen_center + self.crosshair_pos;

            // ✅ DEFINE ray_origin/ray_end HERE
            let ray_origin = camera.project_ray_origin(crosshair_screen);
            let ray_dir = camera.project_ray_normal(crosshair_screen);
            let ray_end = ray_origin + ray_dir * self.raycast_distance;

            // Exclude player
            let mut exclude = Array::new();
            exclude.push(self.base().get_rid());

            let mut query = PhysicsRayQueryParameters3D::create(ray_origin, ray_end).unwrap();
            query.set_exclude(&exclude);

            let mut space_state = self
                .base_mut()
                .get_world_3d()
                .unwrap()
                .get_direct_space_state()
                .unwrap();
            let result = space_state.intersect_ray(&query);

            if !result.is_empty() {
                self.crosshair_world_pos =
                    result.get("position").unwrap().try_to::<Vector3>().unwrap();
                self.hit_something = true;
            } else {
                self.crosshair_world_pos = ray_end;
                self.hit_something = false;
            }
        }

        // hand point
        if let Some(mut hands) = self.hands.take() {
            let basis = self.camera.as_ref().unwrap().get_global_transform().basis;
            let camera_pos = self.camera.as_ref().unwrap().get_global_position();

            let screen_offset = self.crosshair_pos.normalized();

            // YOUR FIXES + slower movement
            let sway_x = -screen_offset.x * 0.12; // SLOWER left/right + your -
            let sway_y = screen_offset.y * 0.06; // Shallow up/down

            // Dynamic Z closer when hit near
            let z_distance = if self.hit_something {
                let hit_dist = self.crosshair_world_pos.distance_to(camera_pos);
                (hit_dist / self.raycast_distance).clamp(0.4, 0.8)
            } else {
                0.5
            };

            let hands_target_pos = camera_pos
                + basis.rows[2].normalized() * z_distance
                + basis.rows[0] * (sway_x - 0.12)
                + basis.rows[1] * (sway_y - 0.3);

            // EVEN SLOWER lerp = no fast snapping across center
            let new_pos = hands
                .get_global_position()
                .lerp(hands_target_pos, 4.0 * delta as f32);
            hands.set_global_position(new_pos);

            // look_at unchanged
            if self.hit_something {
                hands.look_at(self.crosshair_world_pos);
            } else {
                let forward_dir = basis.rows[2].normalized();
                let forward_target = camera_pos + forward_dir * self.raycast_distance;
                hands.look_at(forward_target);
            }

            self.hands = Some(hands);
        }

        // *************************************
        // ---------------------------------------- end wip chunk
        // *************************************
    }

    fn physics_process(&mut self, delta: f64) {
        let mut head = self.base().get_node_as::<Node3D>("HeadPivot");

        // // AUTO-CAPTURE mouse when window focused
        // if Input::singleton().get_mouse_mode() != MouseMode::CAPTURED {
        //     Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        // }
        //
        // // Mouse look
        // if Input::singleton().is_action_pressed("ui_cancel") {
        //     // Escape
        //     Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
        // } else {
        //     Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        // }

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
