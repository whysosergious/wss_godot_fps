use godot::prelude::*;

mod helpers;
mod player;

struct WssFps;

#[gdextension]
unsafe impl ExtensionLibrary for WssFps {}
