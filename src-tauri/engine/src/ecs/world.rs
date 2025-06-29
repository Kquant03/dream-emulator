// src-tauri/engine/src/ecs/world.rs
use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::{Component, ComponentStorage, EntityId, Query};

pub struct World {
    entities: Vec<EntityId>,
    components: ComponentStorage,
    next_entity_id: EntityId,
    entity_generation: HashMap<EntityId, u32>,
    free_entities: Vec<EntityId>,
}

impl World {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            components: ComponentStorage::new(),
            next_entity_id: 0,
            entity_generation: HashMap::with_capacity(capacity),
            free_entities: Vec::new(),
        }
    }
    
    pub fn create_entity(&mut self) -> EntityId {
        if let Some(id) = self.free_entities.pop() {
            // Reuse entity ID with new generation
            let gen = self.entity_generation.get(&id).copied().unwrap_or(0) + 1;
            self.entity_generation.insert(id, gen);
            self.entities.push(id);
            id
        } else {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            self.entities.push(id);
            self.entity_generation.insert(id, 0);
            id
        }
    }
    
    pub fn destroy_entity(&mut self, entity: EntityId) -> bool {
        if let Some(idx) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(idx);
            self.components.remove_all(entity);
            self.free_entities.push(entity);
            true
        } else {
            false
        }
    }
    
    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) {
        self.components.insert(entity, component);
    }
    
    pub fn remove_component<T: Component>(&mut self, entity: EntityId) -> Option<T> {
        self.components.remove::<T>(entity)
    }
    
    pub fn get_component<T: Component>(&self, entity: EntityId) -> Option<&T> {
        self.components.get::<T>(entity)
    }
    
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        self.components.get_mut::<T>(entity)
    }
    
    pub fn query<Q: Query>(&self) -> Q::Iter<'_> {
        Q::query(&self.components)
    }
    
    pub fn query_mut<Q: Query>(&mut self) -> Q::IterMut<'_> {
        Q::query_mut(&mut self.components)
    }
    
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
    
    pub fn clear(&mut self) {
        self.entities.clear();
        self.components.clear();
        self.free_entities.clear();
    }
}