// src-tauri/engine/src/math/transform.rs
use super::{Vec3, Quat};
use serde::{Deserialize, Serialize};
use crate::ecs::Component;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Component for Transform {}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { position, rotation, scale }
    }
    
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }
    
    pub fn look_at(mut self, target: Vec3, up: Vec3) -> Self {
        let forward = (target - self.position).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        
        // Convert basis vectors to quaternion
        // This is a simplified version - in production you'd use a proper matrix to quaternion conversion
        self.rotation = Quat::IDENTITY; // TODO: Implement proper look_at quaternion
        self
    }
    
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.position + self.rotation.rotate_vec3(point * self.scale)
    }
    
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation.rotate_vec3(direction)
    }
    
    pub fn forward(&self) -> Vec3 {
        self.rotation.rotate_vec3(Vec3::FORWARD)
    }
    
    pub fn right(&self) -> Vec3 {
        self.rotation.rotate_vec3(Vec3::RIGHT)
    }
    
    pub fn up(&self) -> Vec3 {
        self.rotation.rotate_vec3(Vec3::UP)
    }
}