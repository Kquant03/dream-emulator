// src-tauri/engine/src/physics/collision.rs
use crate::math::Vec2;
use crate::ecs::{EntityId, Component};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Collider {
    Circle { radius: f32 },
    Box { half_extents: Vec2 },
    Polygon { vertices: Vec<Vec2> },
}

impl Component for Collider {}

impl Collider {
    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }
    
    pub fn box_collider(width: f32, height: f32) -> Self {
        Self::Box {
            half_extents: Vec2::new(width * 0.5, height * 0.5),
        }
    }
    
    pub fn get_aabb(&self, position: Vec2) -> (Vec2, Vec2) {
        match self {
            Collider::Circle { radius } => {
                let r = Vec2::splat(*radius);
                (position - r, position + r)
            }
            Collider::Box { half_extents } => {
                (position - *half_extents, position + *half_extents)
            }
            Collider::Polygon { vertices } => {
                let mut min = Vec2::new(f32::MAX, f32::MAX);
                let mut max = Vec2::new(f32::MIN, f32::MIN);
                
                for v in vertices {
                    let world_v = position + *v;
                    min.x = min.x.min(world_v.x);
                    min.y = min.y.min(world_v.y);
                    max.x = max.x.max(world_v.x);
                    max.y = max.y.max(world_v.y);
                }
                
                (min, max)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub point: Vec2,
    pub normal: Vec2,
    pub penetration: f32,
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: EntityId,
    pub entity_b: EntityId,
    pub contact: Contact,
}