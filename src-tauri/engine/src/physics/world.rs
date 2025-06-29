// src-tauri/engine/src/physics/world.rs
use crate::math::{Vec2, Vec3};
use crate::ecs::{EntityId, Component};
use super::{RigidBody, Collider, CollisionEvent};
use std::collections::{HashMap, HashSet};

pub struct PhysicsWorld {
    bodies: HashMap<EntityId, RigidBody>,
    colliders: HashMap<EntityId, Collider>,
    collision_pairs: Vec<(EntityId, EntityId)>,
    collision_events: Vec<CollisionEvent>,
    gravity: Vec2,
    fixed_timestep: f32,
    accumulator: f32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            collision_pairs: Vec::new(),
            collision_events: Vec::new(),
            gravity: Vec2::new(0.0, -9.81),
            fixed_timestep: 1.0 / 60.0,
            accumulator: 0.0,
        }
    }
    
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }
    
    pub fn add_rigid_body(&mut self, entity: EntityId, body: RigidBody) {
        self.bodies.insert(entity, body);
    }
    
    pub fn add_collider(&mut self, entity: EntityId, collider: Collider) {
        self.colliders.insert(entity, collider);
    }
    
    pub fn remove_body(&mut self, entity: EntityId) {
        self.bodies.remove(&entity);
        self.colliders.remove(&entity);
    }
    
    pub fn get_body(&self, entity: EntityId) -> Option<&RigidBody> {
        self.bodies.get(&entity)
    }
    
    pub fn get_body_mut(&mut self, entity: EntityId) -> Option<&mut RigidBody> {
        self.bodies.get_mut(&entity)
    }
    
    pub fn step(&mut self, dt: f32) {
        self.accumulator += dt;
        
        // Fixed timestep for stable physics
        while self.accumulator >= self.fixed_timestep {
            self.fixed_update(self.fixed_timestep);
            self.accumulator -= self.fixed_timestep;
        }
    }
    
    fn fixed_update(&mut self, dt: f32) {
        // Clear previous frame's collision data
        self.collision_pairs.clear();
        self.collision_events.clear();
        
        // Apply forces and integrate velocities
        for (entity, body) in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                // Apply gravity
                body.apply_force(self.gravity * body.mass);
                
                // Integrate forces to velocity
                let acceleration = body.force / body.mass;
                body.velocity += acceleration * dt;
                
                // Apply damping
                body.velocity *= 1.0 - body.linear_damping * dt;
                body.angular_velocity *= 1.0 - body.angular_damping * dt;
                
                // Clear forces for next frame
                body.force = Vec2::ZERO;
            }
        }
        
        // Broad phase collision detection
        self.broad_phase();
        
        // Narrow phase collision detection
        self.narrow_phase();
        
        // Solve constraints
        self.solve_constraints();
        
        // Integrate positions
        for (entity, body) in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                body.position += body.velocity * dt;
                body.rotation += body.angular_velocity * dt;
            }
        }
    }
    
    fn broad_phase(&mut self) {
        // Simple O(n²) broad phase - in production, use spatial partitioning
        let entities: Vec<EntityId> = self.colliders.keys().copied().collect();
        
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let entity_a = entities[i];
                let entity_b = entities[j];
                
                // Skip if both are static
                let body_a = self.bodies.get(&entity_a);
                let body_b = self.bodies.get(&entity_b);
                
                if matches!((body_a, body_b), (Some(a), Some(b)) if a.body_type == BodyType::Static && b.body_type == BodyType::Static) {
                    continue;
                }
                
                // Check AABB overlap
                if let (Some(collider_a), Some(collider_b)) = (self.colliders.get(&entity_a), self.colliders.get(&entity_b)) {
                    if self.aabb_overlap(entity_a, collider_a, entity_b, collider_b) {
                        self.collision_pairs.push((entity_a, entity_b));
                    }
                }
            }
        }
    }
    
    fn aabb_overlap(&self, entity_a: EntityId, collider_a: &Collider, entity_b: EntityId, collider_b: &Collider) -> bool {
        let pos_a = self.bodies.get(&entity_a).map(|b| b.position).unwrap_or_default();
        let pos_b = self.bodies.get(&entity_b).map(|b| b.position).unwrap_or_default();
        
        let (min_a, max_a) = collider_a.get_aabb(pos_a);
        let (min_b, max_b) = collider_b.get_aabb(pos_b);
        
        min_a.x <= max_b.x && max_a.x >= min_b.x &&
        min_a.y <= max_b.y && max_a.y >= min_b.y
    }
    
    fn narrow_phase(&mut self) {
        for &(entity_a, entity_b) in &self.collision_pairs {
            if let Some(contact) = self.check_collision(entity_a, entity_b) {
                self.collision_events.push(CollisionEvent {
                    entity_a,
                    entity_b,
                    contact,
                });
            }
        }
    }
    
    fn check_collision(&self, entity_a: EntityId, entity_b: EntityId) -> Option<Contact> {
        let body_a = self.bodies.get(&entity_a)?;
        let body_b = self.bodies.get(&entity_b)?;
        let collider_a = self.colliders.get(&entity_a)?;
        let collider_b = self.colliders.get(&entity_b)?;
        
        // Simple circle-circle collision for now
        match (collider_a, collider_b) {
            (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) => {
                let distance = body_a.position.distance(body_b.position);
                let radius_sum = r1 + r2;
                
                if distance < radius_sum {
                    let normal = (body_b.position - body_a.position).normalize();
                    let penetration = radius_sum - distance;
                    
                    Some(Contact {
                        point: body_a.position + normal * r1,
                        normal,
                        penetration,
                    })
                } else {
                    None
                }
            }
            _ => None, // Other collision types not implemented yet
        }
    }
    
    fn solve_constraints(&mut self) {
        // Simple impulse-based constraint solver
        for event in &self.collision_events {
            let (body_a, body_b) = match (self.bodies.get(&event.entity_a), self.bodies.get(&event.entity_b)) {
                (Some(a), Some(b)) => (a.clone(), b.clone()),
                _ => continue,
            };
            
            // Skip if both static
            if body_a.body_type == BodyType::Static && body_b.body_type == BodyType::Static {
                continue;
            }
            
            // Calculate relative velocity
            let relative_velocity = body_b.velocity - body_a.velocity;
            let velocity_along_normal = relative_velocity.dot(event.contact.normal);
            
            // Don't resolve if velocities are separating
            if velocity_along_normal > 0.0 {
                continue;
            }
            
            // Calculate impulse scalar
            let inv_mass_a = if body_a.body_type == BodyType::Dynamic { 1.0 / body_a.mass } else { 0.0 };
            let inv_mass_b = if body_b.body_type == BodyType::Dynamic { 1.0 / body_b.mass } else { 0.0 };
            
            let restitution = (body_a.restitution + body_b.restitution) * 0.5;
            let j = -(1.0 + restitution) * velocity_along_normal / (inv_mass_a + inv_mass_b);
            
            let impulse = event.contact.normal * j;
            
            // Apply impulse
            if let Some(body) = self.bodies.get_mut(&event.entity_a) {
                if body.body_type == BodyType::Dynamic {
                    body.velocity -= impulse * inv_mass_a;
                }
            }
            
            if let Some(body) = self.bodies.get_mut(&event.entity_b) {
                if body.body_type == BodyType::Dynamic {
                    body.velocity += impulse * inv_mass_b;
                }
            }
            
            // Position correction to prevent sinking
            let percent = 0.2; // Penetration percentage to correct
            let slop = 0.01; // Penetration allowance
            let correction = event.contact.normal * 
                ((event.contact.penetration - slop).max(0.0) / (inv_mass_a + inv_mass_b)) * percent;
            
            if let Some(body) = self.bodies.get_mut(&event.entity_a) {
                if body.body_type == BodyType::Dynamic {
                    body.position -= correction * inv_mass_a;
                }
            }
            
            if let Some(body) = self.bodies.get_mut(&event.entity_b) {
                if body.body_type == BodyType::Dynamic {
                    body.position += correction * inv_mass_b;
                }
            }
        }
    }
    
    pub fn get_collision_pairs(&self) -> &[(EntityId, EntityId)] {
        &self.collision_pairs
    }
    
    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }
}