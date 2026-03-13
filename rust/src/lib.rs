use godot::prelude::*;

mod gun;
mod helpers;
mod player;

struct WssFps;

#[gdextension]
unsafe impl ExtensionLibrary for WssFps {}
