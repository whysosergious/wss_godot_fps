#[allow(unused_imports)]
use godot::builtin::Aabb;
#[allow(unused_imports)]
use godot::classes::{
    input::MouseMode, AnimationPlayer, Area3D, Camera3D, CanvasLayer, CharacterBody3D,
    CollisionObject3D, CollisionShape3D, ColorRect, ICharacterBody3D, Input, MeshInstance3D, Node,
    Node3D, PhysicsDirectSpaceState3D, PhysicsRayQueryParameters3D, RayCast3D, RigidBody3D,
};
use godot::obj::WithBaseField;
use godot::prelude::*;

// use godot::global::lerp;
use crate::helpers::{f32_lerp, vec3_lerp};

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

    player_collision: Option<Gd<CollisionShape3D>>,

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
    mantle_target: Vector3,
    climb_ray: Option<Gd<RayCast3D>>,
    reach_ray: Option<Gd<RayCast3D>>,
    climb_ray_length: f32,
    is_mantling: bool,

    mantle_progress: f32, // 0.0 → 1.0
    mantle_start_pos: Vector3,
    mantle_end_pos: Vector3,
    look_for_ledge: bool,

    lefthand: Option<Gd<MeshInstance3D>>,
    lefthand_start_pos: Vector3,
    lefthand_target_pos: Vector3,
    lefthand_progress: f32,

    wall_touch_offset: Vector3,
    wall_lerp_t: f32,
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
            jump: 8.0,
            crouch_speed: 2.5,
            crouch_height_scale: 0.3,
            stand_height_scale: 1.0,
            crouch_transition: 8.0,

            is_crouching: false,
            crouch_toggle_mode: true,
            target_scale: 1.0,
            last_crouch_offset: 0.0,

            gravity: 20.0,
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

            player_collision: None,

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
            mantle_target: Vector3::ZERO,
            climb_ray: None,
            reach_ray: None,
            climb_ray_length: 3.3,
            is_mantling: false,

            mantle_progress: 0.0, // 0.0 → 1.0
            mantle_start_pos: Vector3::ZERO,
            mantle_end_pos: Vector3::ZERO,
            look_for_ledge: false,

            lefthand: None,
            lefthand_start_pos: Vector3::ZERO,
            lefthand_target_pos: Vector3::ZERO,
            lefthand_progress: 0.0,

            wall_touch_offset: Vector3::ZERO,
            wall_lerp_t: 0.0,
        }
    }

    fn ready(&mut self) {
        Input::singleton().set_mouse_mode(MouseMode::HIDDEN);

        self.player_collision = Some(
            self.base()
                .get_node_as::<CollisionShape3D>("CollisionShape3D"),
        );

        self.lefthand = Some(
            self.base()
                .get_node_as::<MeshInstance3D>("Head/Hands/LeftHand"),
        );

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

        self.climb_ray = Some(self.base().get_node_as::<RayCast3D>("ClimbRay"));
        self.reach_ray = Some(self.base().get_node_as::<RayCast3D>("ReachRay"));

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
            self.look_for_ledge = false;

            if Input::singleton().is_action_just_pressed("jump") {
                velocity.y = self.jump;
            }
        } else {
            velocity.y -= self.gravity * delta as f32;
        }

        if Input::singleton().is_action_just_pressed("jump") {
            self.look_for_ledge = true;
        }

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

        //-----------------------
        //
        //

        // Update ReachRay (bottom-left offset in inspector)

        if let Some(mut reach_ray) = self.reach_ray.take() {
            reach_ray.force_raycast_update();

            if let Some(mut hand) = self.lefthand.take() {
                // Add to struct: wall_lerp_t: f32 = 0.0;

                if reach_ray.is_colliding() {
                    let hit_pos = reach_ray.get_collision_point();
                    self.wall_lerp_t += delta as f32 * 8.0; // Speed 8
                    self.wall_lerp_t = self.wall_lerp_t.clamp(0.0, 1.0);
                    let target = self
                        .hands
                        .as_ref()
                        .unwrap()
                        .get_node_as::<Node3D>("LeftHandBase")
                        .get_global_position()
                        .lerp(hit_pos, self.wall_lerp_t);
                    hand.set_global_position(target);
                } else {
                    self.wall_lerp_t -= delta as f32 * 12.0;
                    self.wall_lerp_t = self.wall_lerp_t.clamp(0.0, 1.0);
                    let base_pos = self
                        .hands
                        .as_ref()
                        .unwrap()
                        .get_node_as::<Node3D>("LeftHandBase")
                        .get_global_position();
                    let target_pos = base_pos.lerp(hand.get_global_position(), self.wall_lerp_t);
                    hand.set_global_position(target_pos);
                }

                self.lefthand = Some(hand);
            }
            self.reach_ray = Some(reach_ray);
        }

        //
        //
        //---------------------------

        // *******************************
        // -------------------------------- start wip chunk
        // *******************************

        // point and teleport
        if self.look_for_ledge && !self.is_mantling {
            if let Some(mut ray) = self.climb_ray.take() {
                if self.is_crouching {
                    ray.set_target_position(Vector3::new(0.0, -(self.climb_ray_length * 0.7), 0.0));
                } else {
                    ray.set_target_position(Vector3::new(0.0, -self.climb_ray_length, 0.0));
                }

                ray.force_raycast_update(); // Immediate result

                // Check hit
                if ray.is_colliding() {
                    let hit_point = ray.get_collision_point();
                    let normal = ray.get_collision_normal();

                    if normal.dot(Vector3::new(0.0, 1.0, 0.0)) > 0.7 {
                        self.player_collision.as_mut().unwrap().set_disabled(true);

                        let target_pos = hit_point + Vector3::new(0.0, 0.3, 0.0);
                        self.mantle_target = target_pos;
                        godot_print!("Mantle target: {:?}", target_pos);

                        self.is_mantling = true;
                        self.mantle_start_pos = self.base().get_global_position();
                        self.mantle_end_pos = target_pos;
                        self.mantle_progress = 0.0;
                        self.lefthand_start_pos =
                            self.lefthand.as_ref().unwrap().get_global_position();
                        self.lefthand_target_pos = hit_point;
                        self.lefthand_progress = 0.0;
                        godot_print!("Mantle START");
                        // self.base_mut().set_global_position(target_pos);
                    }
                }

                self.climb_ray = Some(ray);
            }
        }

        if self.is_mantling {
            velocity = Vector3::ZERO;
            if self.mantle_progress < 1.0 {
                self.mantle_progress += delta as f32 * 2.0; // Fast 0.125s transition

                // Lerp: up first, then forward
                let lift_ratio = (self.mantle_progress * 2.0).min(1.0); // Up phase
                let forward_ratio = ((self.mantle_progress - 0.5).max(0.0) * 2.0); // Forward phase

                let current_y = f32_lerp(
                    self.mantle_start_pos.y,
                    self.mantle_end_pos.y, // Target ledge Y + clearance
                    lift_ratio,
                );
                let current_xz =
                    vec3_lerp(self.mantle_start_pos, self.mantle_end_pos, forward_ratio);

                self.base_mut().set_global_position(Vector3::new(
                    current_xz.x,
                    current_y,
                    current_xz.z,
                ));

                godot_print!(
                    "Mantle progress: {:.1} | Y: {:.2} | Lift: {:.2}",
                    self.mantle_progress,
                    current_y,
                    lift_ratio
                );

                if self.mantle_progress >= 1.0 {
                    self.player_collision.as_mut().unwrap().set_disabled(false);
                    self.is_mantling = false;

                    godot_print!("Mantle END");
                }
            }

            // if let (Some(mut collision), target) =
            //     (self.player_collision.take(), self.mantle_target)
            // {
            //     // 1. Disable collision
            //     collision.set_disabled(true);
            //
            //     let current_pos = self.base().get_global_position();
            //     let lift_pos = Vector3::new(current_pos.x, current_pos.y + 3.5, current_pos.z);
            //
            //     // 2. Lift up first
            //     self.base_mut().set_global_position(lift_pos);
            //
            //     // 3. Move to target
            //     self.base_mut().set_global_position(target);
            //
            //     // 4. Re-enable collision
            //     collision.set_disabled(false);
            //     self.player_collision = Some(collision);
            //
            //     self.is_mantling = false;
            //     self.mantle_target = Vector3::ZERO;
            //     godot_print!("Mantle COMPLETE");
            // }
        }

        // *******************************
        // -------------------------------- end wip chunk
        // *******************************

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

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }
}
