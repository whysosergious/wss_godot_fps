use godot::prelude::*;

mod player;

struct WssFps;

#[gdextension]
unsafe impl ExtensionLibrary for WssFps {}
