use godot::prelude::Vector3;

pub fn vec3_lerp(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a + (b - a) * t
}

pub fn f32_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
