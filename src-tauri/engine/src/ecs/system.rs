// src-tauri/engine/src/ecs/system.rs
use super::{World, EntityId};
use crate::physics::PhysicsWorld;

pub trait System: Send + Sync {
    fn execute(&mut self, world: &mut World, physics: &mut PhysicsWorld, dt: f32);
    
    // Optional methods for system lifecycle
    fn initialize(&mut self, _world: &mut World) {}
    fn cleanup(&mut self, _world: &mut World) {}
}

pub struct SystemSchedule {
    systems: Vec<Box<dyn System>>,
    parallel_systems: Vec<Vec<Box<dyn System>>>,
}

impl SystemSchedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            parallel_systems: Vec::new(),
        }
    }
    
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }
    
    pub fn add_parallel_systems(&mut self, systems: Vec<Box<dyn System>>) {
        self.parallel_systems.push(systems);
    }
    
    pub fn execute(&mut self, world: &mut World, physics: &mut PhysicsWorld, dt: f32) {
        // Execute sequential systems
        for system in &mut self.systems {
            system.execute(world, physics, dt);
        }
        
        // Execute parallel system groups
        // In production, you'd use rayon or similar for actual parallelism
        for group in &mut self.parallel_systems {
            for system in group {
                system.execute(world, physics, dt);
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.systems.clear();
        self.parallel_systems.clear();
    }
}