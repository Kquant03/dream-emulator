// src-tauri/engine/src/math/mod.rs
mod vectors;
mod quaternion;
mod transform;

pub use vectors::*;
pub use quaternion::*;
pub use transform::*;

// Common math utilities
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let normalized = (value - from_min) / (from_max - from_min);
    lerp(to_min, to_max, normalized)
}