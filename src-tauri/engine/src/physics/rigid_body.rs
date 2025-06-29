// src-tauri/engine/src/physics/rigid_body.rs
use crate::math::Vec2;
use crate::ecs::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BodyType {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    pub position: Vec2,
    pub rotation: f32,
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub force: Vec2,
    pub torque: f32,
    pub mass: f32,
    pub inertia: f32,
    pub restitution: f32,
    pub friction: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub body_type: BodyType,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            force: Vec2::ZERO,
            torque: 0.0,
            mass: 1.0,
            inertia: 1.0,
            restitution: 0.5,
            friction: 0.5,
            linear_damping: 0.1,
            angular_damping: 0.1,
            body_type: BodyType::Dynamic,
        }
    }
}

impl Component for RigidBody {}

impl RigidBody {
    pub fn new(position: Vec2, body_type: BodyType) -> Self {
        Self {
            position,
            body_type,
            ..Default::default()
        }
    }
    
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
    
    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }
    
    pub fn apply_force(&mut self, force: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.force += force;
        }
    }
    
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse / self.mass;
        }
    }
    
    pub fn apply_torque(&mut self, torque: f32) {
        if self.body_type == BodyType::Dynamic {
            self.torque += torque;
        }
    }
}