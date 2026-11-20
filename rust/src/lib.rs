//! Beastrider's game rules and state, exposed to Godot as a GDExtension library.

mod hero;
mod movement;
mod tuning;
mod tuning_store;

use godot::prelude::*;

/// Entry point that registers this library's classes with Godot.
struct BeastriderExtension;

// SAFETY: gdext requires this `unsafe impl` as the library's entry point; the
// `#[gdextension]` macro generates the C entry symbol Godot calls on load. It
// adds no unsafe code of our own.
#[gdextension]
unsafe impl ExtensionLibrary for BeastriderExtension {}
