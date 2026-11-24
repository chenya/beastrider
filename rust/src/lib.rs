//! Beastrider's game rules and state, exposed to Godot as a GDExtension library.

mod combat;
mod crowd;
mod enemy;
mod fighter;
mod fighters;
mod hero;
mod hit_stop;
mod movement;
mod run;
mod training_dummy;
mod tuning;
mod tuning_store;
mod wave_director;

use godot::prelude::*;

/// Entry point that registers this library's classes with Godot.
struct BeastriderExtension;

// SAFETY: gdext requires this `unsafe impl` as the library's entry point; the
// `#[gdextension]` macro generates the C entry symbol Godot calls on load. It
// adds no unsafe code of our own.
#[gdextension]
unsafe impl ExtensionLibrary for BeastriderExtension {}
