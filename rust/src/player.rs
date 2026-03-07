#[allow(unused_imports)]
use godot::classes::{
    input::MouseMode, AnimationPlayer, Area3D, Camera3D, CanvasLayer, CharacterBody3D,
    CollisionShape3D, ColorRect, ICharacterBody3D, Input, MeshInstance3D, Node3D,
    PhysicsDirectSpaceState3D, PhysicsRayQueryParameters3D, RigidBody3D,
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
    sprint_speed: f32,
    #[export]
    jump: f32,
    #[export]
    crouch_speed: f32,
    #[export]
    crouch_height_scale: f32,
    #[export]
    stand_height_scale: f32,
    #[export]
    crouch_transition: f32,
    #[export]
    is_crouching: bool,
    #[export]
    crouch_toggle_mode: bool,
    target_scale: f32,
    last_crouch_offset: f32,

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
    hands_horizontal_sway: f32,
    #[export]
    hands_vertical_sway: f32,

    #[export]
    raycast_distance: f32,
    crosshair_world_pos: Vector3,
    hit_something: bool,

    // head & rotation
    #[export]
    head_max_degrees: f32,
    head_rotation_x: f32,
    head_rotation_y: f32,

    #[export]
    yaw_sensitivity: f32,
    #[export]
    pitch_sensitivity: f32,

    #[export]
    body_start_degrees: f32,
    body_rotation_y: f32,
    head: Option<Gd<Node3D>>,

    // climb and vault
    climb_hitbox: Option<Gd<Area3D>>,
    mantle_target: Vector3,
}

#[godot_api]
impl Player {}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        Self {
            base,
            speed: 5.0,
            sprint_speed: 10.0,
            jump: 6.0,
            crouch_speed: 2.5,
            crouch_height_scale: 0.3,
            stand_height_scale: 1.0,
            crouch_transition: 8.0,

            is_crouching: false,
            crouch_toggle_mode: true,
            target_scale: 1.0,
            last_crouch_offset: 0.0,

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

            deadzone_percent: 0.255555,
            last_mouse_pos: Vector2::ZERO,

            // point hands
            hands_horizontal_sway: 0.3,
            hands_vertical_sway: 0.2,

            raycast_distance: 100.0,
            crosshair_world_pos: Vector3::ZERO,
            hit_something: false,

            // head rotation
            head_max_degrees: 15.0,
            head_rotation_x: 0.0,
            head_rotation_y: 0.0,

            yaw_sensitivity: 0.3,
            pitch_sensitivity: 0.3,

            body_start_degrees: 10.0,
            body_rotation_y: 0.0,
            head: None,

            // climb & vault
            climb_hitbox: None,
            mantle_target: Vector3::ZERO,
        }
    }

    fn ready(&mut self) {
        Input::singleton().set_mouse_mode(MouseMode::HIDDEN);

        let head = self.base().get_node_as::<Node3D>("Head");

        self.head_base_pos = head.get_position();

        let hands_node = self.base_mut().get_node_as::<Node3D>("Head/Hands");
        self.hands_base_pos = hands_node.get_position();
        self.hands = Some(hands_node);

        let camera_node = self.base().get_node_as::<Camera3D>("Head/Camera");
        self.camera = Some(camera_node);

        let crosshair_node = self
            .base()
            .get_node_as::<ColorRect>("CrosshairLayer/CrosshairDot");
        let crosshair_size = crosshair_node.get_size();
        self.crosshair_offset = crosshair_size / 2.0;
        self.crosshair_node = Some(crosshair_node);

        let mut viewport = self.base().get_viewport().unwrap();
        let viewport_size = viewport.get_visible_rect().size;
        let deadzone_size = viewport_size * self.deadzone_percent;
        let padding = (viewport_size - deadzone_size) / 2.0;
        let viewport_center = padding + (deadzone_size / 2.0);

        // CRITICAL: Initialize at EXACT deadzone/viewport center
        self.crosshair_pos = Vector2::ZERO;
        viewport.warp_mouse(viewport_center);
        self.last_mouse_pos = viewport_center;

        let head_node = self.base().get_node_as::<Node3D>("Head");
        self.head = Some(head_node);

        self.climb_hitbox = Some(self.base().get_node_as("ClimbHitbox"));

        godot_print!("Crosshair node: {:?}", self.crosshair_node.is_some());
        godot_print!("Head base pos: {:?}", self.head_base_pos);
    }

    fn process(&mut self, delta: f64) {
        let mut viewport = self.base().get_viewport().unwrap();
        let viewport_size = viewport.get_visible_rect().size;
        let screen_center = viewport_size / 2.0;

        let deadzone_size = viewport_size * self.deadzone_percent;

        let mouse_screen_pos = viewport.get_mouse_position();

        let mouse_velocity = (mouse_screen_pos - self.last_mouse_pos) * self.mouse_sensitivity;
        self.crosshair_pos += mouse_velocity;

        let max_offset = deadzone_size / 2.0;
        self.crosshair_pos = self.crosshair_pos.clamp(-max_offset, max_offset);

        viewport.warp_mouse(screen_center);
        self.last_mouse_pos = screen_center;

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
                // Virtual wall at max distance
                self.crosshair_world_pos = ray_origin + ray_dir * self.raycast_distance;
                self.hit_something = true; // Treat as hit for hands pointing
            }
        }

        // *** NEW ORDER: BODY FIRST ***
        let crosshair_outside_x = self.crosshair_pos.x.abs() > max_offset.x * 0.99;
        let crosshair_outside_y = self.crosshair_pos.y.abs() > max_offset.y * 0.99;

        let yaw_input = if crosshair_outside_x && mouse_velocity.x != 0.0 {
            -mouse_velocity.x * self.yaw_sensitivity
        } else {
            0.0
        };

        let pitch_input = if crosshair_outside_y && mouse_velocity.y != 0.0 {
            -mouse_velocity.y * self.pitch_sensitivity
        } else {
            0.0
        };

        if let Some(mut head) = self.head.take() {
            let head_rotation = head.get_rotation_degrees();

            let target_pitch = self.head_rotation_x.lerp(pitch_input, 12.0 * delta as f32);

            // Predict next rotation BEFORE applying
            let next_pitch_degrees = head_rotation.x + target_pitch;
            let clamped_pitch_degrees = next_pitch_degrees.clamp(-90.0, 90.0);

            // Apply clamped amount (difference from current)
            let pitch_delta = clamped_pitch_degrees - head_rotation.x;
            head.rotate_x(pitch_delta.to_radians());

            self.head_rotation_x = clamped_pitch_degrees - head_rotation.x; // Keep state consistent

            // yaw
            // TODO: rotate head within -60 & 60 before rotating the body
            let target_yaw = self.body_rotation_y.lerp(yaw_input, 12.0 * delta as f32);
            self.body_rotation_y = target_yaw;
            self.base_mut().rotate_y(target_yaw.to_radians());

            self.head = Some(head);
        }

        if let Some(mut hands) = self.hands.take() {
            let camera_transform = self.camera.as_ref().unwrap().get_global_transform();
            let camera_pos = self.camera.as_ref().unwrap().get_global_position();
            let basis = camera_transform.basis;

            let sway_x =
                (self.crosshair_pos.x / max_offset.x).clamp(-1.0, 1.0) * self.hands_horizontal_sway;
            let sway_y =
                -(self.crosshair_pos.y / max_offset.y).clamp(-1.0, 1.0) * self.hands_vertical_sway;

            let z_distance = if self.hit_something {
                self.crosshair_world_pos
                    .distance_to(camera_pos)
                    .clamp(0.2, 0.8)
            } else {
                0.5
            };

            // LOCAL POSITION: Right biased + good sway range
            let hands_local_pos = Vector3::new(
                sway_x + 0.25, // RIGHT bias + left/right sway
                sway_y - 0.35, // LOWER bias + up/down sway
                -z_distance,   // Forward from camera (negative Z local)
            );

            let new_pos = hands
                .get_position()
                .lerp(hands_local_pos, 6.0 * delta as f32);
            hands.set_position(new_pos);

            if self.hit_something {
                hands.look_at(self.crosshair_world_pos);
            } else {
                let forward_dir = basis.rows[2].normalized();
                hands.look_at(camera_pos + forward_dir * self.raycast_distance);
            }

            self.hands = Some(hands);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        let mut head = self.base().get_node_as::<Node3D>("Head");

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
        let input_dir = input.get_vector("move_left", "move_right", "move_forward", "move_back");

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

        // *******************************
        // -------------------------------- start wip chunk
        // *******************************

        // *******************************
        // -------------------------------- end wip chunk
        // *******************************

        // Crouch & crouch toggle
        self.is_crouching = if self.crouch_toggle_mode {
            if input.is_action_just_pressed("crouch") {
                !self.is_crouching
            } else {
                self.is_crouching
            }
        } else {
            input.is_action_pressed("crouch")
        };

        self.target_scale = if self.is_crouching {
            self.crouch_height_scale
        } else {
            self.stand_height_scale
        };

        // Scale collision shape directly
        let mut collision_shape = self
            .base()
            .get_node_as::<CollisionShape3D>("CollisionShape3D");
        let current_scale = collision_shape.get_scale().y;
        let scale_diff = self.target_scale - current_scale;

        let t = self.crouch_transition * delta as f32;
        let eased_t = if scale_diff < 0.0 {
            1.0 - (1.0 - t).powf(2.0)
        } else {
            t
        };

        let new_scale = current_scale + scale_diff * eased_t.min(1.0);
        collision_shape.set_scale(Vector3::new(1.0, new_scale, 1.0));

        let frame_offset = (self.stand_height_scale - new_scale) - self.last_crouch_offset;
        self.last_crouch_offset = self.stand_height_scale - new_scale;

        let mut transform = self.base_mut().get_global_transform();
        transform.origin.y -= frame_offset;
        self.base_mut().set_global_transform(transform);

        // Speed selection
        let self_speed = if self.is_crouching {
            self.crouch_speed
        } else if input.is_action_pressed("sprint") {
            self.sprint_speed
        } else {
            self.speed
        };

        if input_dir != Vector2::ZERO {
            velocity.x = direction.x * self_speed;
            velocity.z = direction.z * self_speed;
        } else {
            velocity.x *= 0.8;
            velocity.z *= 0.8;
        }

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        // point and teleport
        if Input::singleton().is_action_just_pressed("shoot") {
            if let Some(hitbox) = &self.climb_hitbox {
                let player_pos = self.base().get_global_position();
                let bodies = hitbox.get_overlapping_bodies();

                godot_print!("bodies: {}", bodies);

                for body_rid in bodies.iter_shared() {
                    let body_node: Option<Gd<Node3D>> = body_rid.try_into().ok();
                    if let Some(body_node) = body_node {
                        let body_pos = body_node.get_global_position();

                        // STEP 1: Step toward object (X/Z direction)
                        let step_dir = (body_pos - player_pos).normalized();
                        let step_pos = player_pos
                            + Vector3::new(step_dir.x * 0.8, player_pos.y, step_dir.z * 0.8);

                        // STEP 2: Ray UP from step position to find top surface
                        let ray_from = step_pos;
                        let ray_to = step_pos + Vector3::new(0.0, 3.0, 0.0);

                        let mut space_state = self
                            .base_mut()
                            .get_world_3d()
                            .unwrap()
                            .get_direct_space_state()
                            .unwrap();
                        let mut exclude = Array::new();
                        exclude.push(self.base().get_rid());
                        // exclude.push(body_rid);

                        let mut ray_query =
                            PhysicsRayQueryParameters3D::create(ray_from, ray_to).unwrap();
                        ray_query.set_exclude(&exclude);
                        let ray_result = space_state.intersect_ray(&ray_query);

                        if !ray_result.is_empty() {
                            let top_pos = ray_result
                                .get("position")
                                .unwrap()
                                .try_to::<Vector3>()
                                .unwrap();

                            // STEP 3: Teleport to top surface
                            let target_pos = Vector3::new(top_pos.x, top_pos.y + 1.0, top_pos.z);
                            self.base_mut().set_global_position(target_pos);
                            break;
                        }
                    }
                }
            }
        }

        // Shooting/pushing (left click)
        // if input.is_action_just_pressed("shoot") {
        //     let camera = self.base().get_node_as::<Camera3D>("Head/Camera");
        //     let from = camera.get_global_position();
        //     let to = from - camera.get_global_transform().basis.col_c() * 100.0;
        //
        //     let mut space_state = self
        //         .base_mut()
        //         .get_world_3d()
        //         .unwrap()
        //         .get_direct_space_state()
        //         .unwrap();
        //
        //     if let Some(query) = PhysicsRayQueryParameters3D::create(from, to) {
        //         let result = space_state.intersect_ray(&query);
        //         if !result.is_empty() {
        //             godot_print!("Hit: {:?}", result);
        //
        //             if let Some(mut body) = result
        //                 .get("collider")
        //                 .and_then(|n| n.try_to::<Gd<RigidBody3D>>().ok())
        //             {
        //                 let hit_position = result
        //                     .get("position")
        //                     .and_then(|v| v.try_to::<Vector3>().ok())
        //                     .unwrap_or(from);
        //                 body.upcast_mut::<RigidBody3D>()
        //                     .apply_impulse((from - hit_position) * -3.0);
        //             }
        //         }
        //     }
        // }
    }
}
