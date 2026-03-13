use godot::classes::{MeshInstance3D, Node3D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
struct Gun {
    #[base]
    base: Base<Node3D>,
    mesh: Option<Gd<MeshInstance3D>>,
    recoil_offset: Vector3,
}

#[godot_api]
impl INode3D for Gun {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            mesh: None,
            recoil_offset: Vector3::ZERO,
        }
    }

    fn ready(&mut self) {
        self.mesh = Some(self.base().get_node_as("MeshInstance3D"));
    }

    fn process(&mut self, delta: f64) {
        // Recoil tween back to zero
        self.recoil_offset = self
            .recoil_offset
            .lerp(Vector3::ZERO, (delta * 10.0) as f32);

        // if let Some(mesh) = &self.mesh {
        //     let transform = Transform3D::from_translation(self.recoil_offset);
        //     mesh.set_transform(transform);
        // }
    }
}

#[godot_api]
impl Gun {
    #[func]
    fn fire(&mut self) {
        self.recoil_offset = Vector3::new(0.0, 0.02, -0.05); // Kick up/back
    }
}
