// src-tauri/engine/src/lib.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use ecs::*;
pub use math::*;
pub use renderer::*;

mod ecs;
mod math;
mod renderer;
mod physics;
mod compiler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub target_fps: u32,
    pub fixed_timestep: f32,
    pub max_entities: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            fixed_timestep: 1.0 / 60.0,
            max_entities: 10000,
        }
    }
}

pub struct DreamEngine {
    world: World,
    renderer: Box<dyn Renderer>,
    physics: physics::PhysicsWorld,
    systems: SystemSchedule,
    config: EngineConfig,
    accumulator: f32,
}

impl DreamEngine {
    pub fn new(config: EngineConfig) -> Result<Self, EngineError> {
        let world = World::with_capacity(config.max_entities);
        let renderer = create_renderer()?;
        let physics = physics::PhysicsWorld::new();
        let systems = SystemSchedule::new();
        
        Ok(Self {
            world,
            renderer,
            physics,
            systems,
            config,
            accumulator: 0.0,
        })
    }
    
    pub fn update(&mut self, dt: f32) {
        // Fixed timestep with interpolation
        self.accumulator += dt;
        
        while self.accumulator >= self.config.fixed_timestep {
            self.fixed_update(self.config.fixed_timestep);
            self.accumulator -= self.config.fixed_timestep;
        }
        
        // Interpolate rendering
        let alpha = self.accumulator / self.config.fixed_timestep;
        self.render(alpha);
    }
    
    fn fixed_update(&mut self, dt: f32) {
        // Run systems in optimal order
        self.systems.execute(&mut self.world, &mut self.physics, dt);
    }
    
    fn render(&mut self, interpolation: f32) {
        self.renderer.begin_frame();
        
        // Render all entities with sprite components
        let query = self.world.query::<(&Transform, &Sprite)>();
        for (transform, sprite) in query.iter() {
            self.renderer.draw_sprite(sprite, transform, interpolation);
        }
        
        self.renderer.end_frame();
    }
    
    pub fn load_compiled_game(&mut self, data: &[u8]) -> Result<(), EngineError> {
        let game: CompiledGame = bincode::deserialize(data)?;
        
        // Load systems
        for system in game.systems {
            self.systems.add_system(system);
        }
        
        // Create entities
        for entity_data in game.entities {
            self.create_entity_from_data(entity_data)?;
        }
        
        Ok(())
    }
    
    fn create_entity_from_data(&mut self, data: EntityData) -> Result<EntityId, EngineError> {
        let entity = self.world.create_entity();
        
        // Add components based on data
        if let Some(transform) = data.transform {
            self.world.add_component(entity, transform);
        }
        
        if let Some(sprite) = data.sprite {
            self.world.add_component(entity, sprite);
        }
        
        // Add other components...
        
        Ok(entity)
    }
}

// ECS Module
mod ecs {
    use super::*;
    
    pub type EntityId = u32;
    pub type ComponentId = std::any::TypeId;
    
    #[derive(Default)]
    pub struct World {
        entities: Vec<EntityId>,
        components: ComponentStorage,
        next_entity_id: EntityId,
    }
    
    impl World {
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                entities: Vec::with_capacity(capacity),
                components: ComponentStorage::new(),
                next_entity_id: 0,
            }
        }
        
        pub fn create_entity(&mut self) -> EntityId {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            self.entities.push(id);
            id
        }
        
        pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) {
            self.components.insert(entity, component);
        }
        
        pub fn query<Q: Query>(&self) -> QueryIter<Q> {
            Q::query(&self.components)
        }
    }
    
    pub trait Component: Send + Sync + 'static {}
    
    pub trait System: Send + Sync {
        fn execute(&mut self, world: &mut World, physics: &mut physics::PhysicsWorld, dt: f32);
    }
    
    pub struct SystemSchedule {
        systems: Vec<Box<dyn System>>,
    }
    
    impl SystemSchedule {
        pub fn new() -> Self {
            Self {
                systems: Vec::new(),
            }
        }
        
        pub fn add_system(&mut self, system: Box<dyn System>) {
            self.systems.push(system);
        }
        
        pub fn execute(&mut self, world: &mut World, physics: &mut physics::PhysicsWorld, dt: f32) {
            for system in &mut self.systems {
                system.execute(world, physics, dt);
            }
        }
    }
    
    // Component storage with cache-friendly layout
    pub struct ComponentStorage {
        storages: HashMap<ComponentId, Box<dyn ComponentStorageBase>>,
    }
    
    impl ComponentStorage {
        pub fn new() -> Self {
            Self {
                storages: HashMap::new(),
            }
        }
        
        pub fn insert<T: Component>(&mut self, entity: EntityId, component: T) {
            let type_id = std::any::TypeId::of::<T>();
            let storage = self.storages
                .entry(type_id)
                .or_insert_with(|| Box::new(ComponentVec::<T>::new()));
            
            let typed_storage = storage.as_any_mut()
                .downcast_mut::<ComponentVec<T>>()
                .unwrap();
            
            typed_storage.insert(entity, component);
        }
    }
    
    trait ComponentStorageBase: Send + Sync {
        fn as_any(&self) -> &dyn std::any::Any;
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    }
    
    struct ComponentVec<T: Component> {
        components: Vec<Option<T>>,
        entities: Vec<EntityId>,
    }
    
    impl<T: Component> ComponentVec<T> {
        fn new() -> Self {
            Self {
                components: Vec::new(),
                entities: Vec::new(),
            }
        }
        
        fn insert(&mut self, entity: EntityId, component: T) {
            // Simple implementation - would use sparse set in production
            if entity as usize >= self.components.len() {
                self.components.resize_with(entity as usize + 1, || None);
            }
            self.components[entity as usize] = Some(component);
            self.entities.push(entity);
        }
    }
    
    impl<T: Component> ComponentStorageBase for ComponentVec<T> {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }
    
    // Query system for efficient iteration
    pub trait Query {
        type Item;
        fn query(storage: &ComponentStorage) -> QueryIter<Self>;
    }
    
    pub struct QueryIter<Q: Query> {
        _phantom: std::marker::PhantomData<Q>,
    }
    
    impl<Q: Query> QueryIter<Q> {
        pub fn iter(&self) -> impl Iterator<Item = Q::Item> {
            // Simplified - would implement actual iteration
            std::iter::empty()
        }
    }
}

// Math module
mod math {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Vec2 {
        pub x: f32,
        pub y: f32,
    }
    
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Vec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }
    
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Quat {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }
}

// Renderer module
mod renderer {
    use super::*;
    
    pub trait Renderer: Send + Sync {
        fn begin_frame(&mut self);
        fn end_frame(&mut self);
        fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform, interpolation: f32);
    }
    
    pub fn create_renderer() -> Result<Box<dyn Renderer>, EngineError> {
        // Would create appropriate renderer based on platform
        Ok(Box::new(NullRenderer))
    }
    
    struct NullRenderer;
    
    impl Renderer for NullRenderer {
        fn begin_frame(&mut self) {}
        fn end_frame(&mut self) {}
        fn draw_sprite(&mut self, _: &Sprite, _: &Transform, _: f32) {}
    }
}

// Physics module
mod physics {
    use super::*;
    
    pub struct PhysicsWorld {
        bodies: Vec<RigidBody>,
    }
    
    impl PhysicsWorld {
        pub fn new() -> Self {
            Self {
                bodies: Vec::new(),
            }
        }
        
        pub fn step(&mut self, dt: f32) {
            // Simple physics integration
            for body in &mut self.bodies {
                body.position += body.velocity * dt;
            }
        }
    }
    
    pub struct RigidBody {
        position: Vec2,
        velocity: Vec2,
        mass: f32,
    }
}

// Common components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Component for Transform {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub texture_id: String,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Component for Sprite {}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Renderer initialization failed: {0}")]
    RendererInit(String),
    
    #[error("Failed to deserialize game data: {0}")]
    Deserialization(#[from] bincode::Error),
    
    #[error("Component not found")]
    ComponentNotFound,
}

// Compiled game format
#[derive(Serialize, Deserialize)]
pub struct CompiledGame {
    pub systems: Vec<Box<dyn System>>,
    pub entities: Vec<EntityData>,
    pub assets: HashMap<String, Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct EntityData {
    pub name: String,
    pub transform: Option<Transform>,
    pub sprite: Option<Sprite>,
    // Other components...
}

// Module for visual script compilation
mod compiler {
    use super::*;
    
    pub struct CompiledSystem {
        pub name: String,
        pub code: String,
    }
    
    pub fn compile_visual_script(script: &VisualScript) -> Result<CompiledSystem, CompilerError> {
        let mut code = String::new();
        
        // Generate system struct
        code.push_str(&format!("pub struct {}System;\n\n", script.name));
        code.push_str(&format!("impl System for {}System {{\n", script.name));
        code.push_str("    fn execute(&mut self, world: &mut World, physics: &mut PhysicsWorld, dt: f32) {\n");
        
        // Compile nodes to Rust code
        for node in &script.nodes {
            code.push_str(&compile_node(node)?);
        }
        
        code.push_str("    }\n}\n");
        
        Ok(CompiledSystem {
            name: script.name.clone(),
            code,
        })
    }
    
    fn compile_node(node: &VisualScriptNode) -> Result<String, CompilerError> {
        match node.node_type.as_str() {
            "OnUpdate" => Ok("// Update logic\n".to_string()),
            "GetComponent" => Ok(format!(
                "let component = world.get_component::<{}>(entity);\n",
                node.data.get("component_type").unwrap()
            )),
            _ => Err(CompilerError::UnknownNode(node.node_type.clone())),
        }
    }
    
    #[derive(Debug, thiserror::Error)]
    pub enum CompilerError {
        #[error("Unknown node type: {0}")]
        UnknownNode(String),
    }
}

// Visual script types (shared with TypeScript)
#[derive(Serialize, Deserialize)]
pub struct VisualScript {
    pub name: String,
    pub nodes: Vec<VisualScriptNode>,
    pub connections: Vec<VisualScriptConnection>,
}

#[derive(Serialize, Deserialize)]
pub struct VisualScriptNode {
    pub id: String,
    pub node_type: String,
    pub position: (f32, f32),
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct VisualScriptConnection {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

// Integration with Tauri
#[cfg(feature = "tauri-integration")]
pub mod tauri_integration {
    use super::*;
    use tauri::command;
    
    #[command]
    pub async fn create_engine_preview() -> Result<String, String> {
        let engine = DreamEngine::new(EngineConfig::default())
            .map_err(|e| e.to_string())?;
        
        // Return handle to engine
        Ok("engine_handle_123".to_string())
    }
    
    #[command]
    pub async fn update_preview_scene(handle: String, scene_data: Vec<u8>) -> Result<(), String> {
        // Update the preview engine with new scene data
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_engine() {
        let engine = DreamEngine::new(EngineConfig::default());
        assert!(engine.is_ok());
    }
    
    #[test]
    fn test_ecs_basic() {
        let mut world = World::with_capacity(100);
        let entity = world.create_entity();
        
        world.add_component(entity, Transform {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        });
        
        assert_eq!(entity, 0);
    }
}