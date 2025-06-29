// src-tauri/engine/src/ecs/component.rs
use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::EntityId;

pub trait Component: Send + Sync + 'static {
    fn type_id() -> TypeId where Self: Sized {
        TypeId::of::<Self>()
    }
}

pub trait ComponentVec: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: EntityId);
    fn clear(&mut self);
}

pub struct TypedComponentVec<T: Component> {
    components: Vec<Option<T>>,
    entities: Vec<EntityId>,
    entity_indices: HashMap<EntityId, usize>,
}

impl<T: Component> TypedComponentVec<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            entity_indices: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, entity: EntityId, component: T) {
        if let Some(&idx) = self.entity_indices.get(&entity) {
            self.components[idx] = Some(component);
        } else {
            let idx = self.entities.len();
            self.entities.push(entity);
            self.components.push(Some(component));
            self.entity_indices.insert(entity, idx);
        }
    }
    
    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.entity_indices
            .get(&entity)
            .and_then(|&idx| self.components.get(idx))
            .and_then(|c| c.as_ref())
    }
    
    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        self.entity_indices
            .get(&entity)
            .and_then(|&idx| self.components.get_mut(idx))
            .and_then(|c| c.as_mut())
    }
    
    pub fn remove(&mut self, entity: EntityId) -> Option<T> {
        if let Some(idx) = self.entity_indices.remove(&entity) {
            // Swap remove for performance
            let last_idx = self.entities.len() - 1;
            if idx != last_idx {
                self.entities.swap(idx, last_idx);
                self.components.swap(idx, last_idx);
                
                // Update the swapped entity's index
                let swapped_entity = self.entities[idx];
                self.entity_indices.insert(swapped_entity, idx);
            }
            
            self.entities.pop();
            self.components.pop().unwrap()
        } else {
            None
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.entities.iter()
            .zip(self.components.iter())
            .filter_map(|(&e, c)| c.as_ref().map(|c| (e, c)))
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.entities.iter()
            .zip(self.components.iter_mut())
            .filter_map(|(&e, c)| c.as_mut().map(|c| (e, c)))
    }
}

impl<T: Component> ComponentVec for TypedComponentVec<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    fn remove(&mut self, entity: EntityId) {
        self.remove(entity);
    }
    
    fn clear(&mut self) {
        self.components.clear();
        self.entities.clear();
        self.entity_indices.clear();
    }
}

pub struct ComponentStorage {
    storages: HashMap<TypeId, Box<dyn ComponentVec>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }
    
    pub fn insert<T: Component>(&mut self, entity: EntityId, component: T) {
        let type_id = T::type_id();
        let storage = self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedComponentVec::<T>::new()));
        
        let typed_storage = storage
            .as_any_mut()
            .downcast_mut::<TypedComponentVec<T>>()
            .unwrap();
        
        typed_storage.insert(entity, component);
    }
    
    pub fn remove<T: Component>(&mut self, entity: EntityId) -> Option<T> {
        let type_id = T::type_id();
        self.storages.get_mut(&type_id)
            .and_then(|storage| {
                storage.as_any_mut()
                    .downcast_mut::<TypedComponentVec<T>>()
                    .unwrap()
                    .remove(entity)
            })
    }
    
    pub fn get<T: Component>(&self, entity: EntityId) -> Option<&T> {
        let type_id = T::type_id();
        self.storages.get(&type_id)
            .and_then(|storage| {
                storage.as_any()
                    .downcast_ref::<TypedComponentVec<T>>()
                    .unwrap()
                    .get(entity)
            })
    }
    
    pub fn get_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        let type_id = T::type_id();
        self.storages.get_mut(&type_id)
            .and_then(|storage| {
                storage.as_any_mut()
                    .downcast_mut::<TypedComponentVec<T>>()
                    .unwrap()
                    .get_mut(entity)
            })
    }
    
    pub fn get_storage<T: Component>(&self) -> Option<&TypedComponentVec<T>> {
        let type_id = T::type_id();
        self.storages.get(&type_id)
            .and_then(|storage| storage.as_any().downcast_ref())
    }
    
    pub fn get_storage_mut<T: Component>(&mut self) -> Option<&mut TypedComponentVec<T>> {
        let type_id = T::type_id();
        self.storages.get_mut(&type_id)
            .and_then(|storage| storage.as_any_mut().downcast_mut())
    }
    
    pub fn remove_all(&mut self, entity: EntityId) {
        for storage in self.storages.values_mut() {
            storage.remove(entity);
        }
    }
    
    pub fn clear(&mut self) {
        for storage in self.storages.values_mut() {
            storage.clear();
        }
    }
}