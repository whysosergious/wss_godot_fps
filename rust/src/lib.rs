use godot::prelude::*;

struct MyExtension;

#[derive(GodotClass)]
#[class(base=Node)]
struct TestNode {}

#[godot_api]
impl INode for TestNode {
    fn init(_base: Base<Node>) -> Self {
        godot_print!("Rust TestNode created!");
        Self {}
    }
}

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
