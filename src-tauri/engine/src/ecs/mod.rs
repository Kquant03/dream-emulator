// src-tauri/engine/src/ecs/mod.rs
mod world;
mod component;
mod system;
mod query;

pub use world::*;
pub use component::*;
pub use system::*;
pub use query::*;

pub type EntityId = u32;