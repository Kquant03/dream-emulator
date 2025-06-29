// src-tauri/engine/src/lib.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

pub mod ecs;
pub mod math;
pub mod renderer;
pub mod physics;
pub mod compiler;
pub mod assets;

// Re-export commonly used types
pub use ecs::{Component, World, System, SystemSchedule, EntityId};
pub use math::{Vec2, Vec3, Quat, Transform};
pub use renderer::{Renderer, Sprite, create_renderer, RendererBackend};
pub use physics::{PhysicsWorld, RigidBody, Collider, BodyType};

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
    physics: PhysicsWorld,
    systems: SystemSchedule,
    config: EngineConfig,
    accumulator: f32,
    time: f32,
}

impl DreamEngine {
    pub fn new(config: EngineConfig) -> Result<Self, EngineError> {
        let world = World::with_capacity(config.max_entities);
        let renderer = create_renderer(RendererBackend::Canvas)?;
        let physics = PhysicsWorld::new();
        let systems = SystemSchedule::new();
        
        Ok(Self {
            world,
            renderer,
            physics,
            systems,
            config,
            accumulator: 0.0,
            time: 0.0,
        })
    }
    
    pub fn world(&self) -> &World {
        &self.world
    }
    
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    
    pub fn physics(&self) -> &PhysicsWorld {
        &self.physics
    }
    
    pub fn physics_mut(&mut self) -> &mut PhysicsWorld {
        &mut self.physics
    }
    
    pub fn systems_mut(&mut self) -> &mut SystemSchedule {
        &mut self.systems
    }
    
    pub fn update(&mut self, dt: f32) {
        // Fixed timestep with interpolation
        self.accumulator += dt;
        
        while self.accumulator >= self.config.fixed_timestep {
            self.fixed_update(self.config.fixed_timestep);
            self.accumulator -= self.config.fixed_timestep;
            self.time += self.config.fixed_timestep;
        }
        
        // Interpolate rendering
        let alpha = self.accumulator / self.config.fixed_timestep;
        self.render(alpha);
    }
    
    fn fixed_update(&mut self, dt: f32) {
        // Update physics
        self.physics.step(dt);
        
        // Run systems
        self.systems.execute(&mut self.world, &mut self.physics, dt);
    }
    
    fn render(&mut self, interpolation: f32) {
        self.renderer.begin_frame();
        self.renderer.clear([0.1, 0.1, 0.2, 1.0]);
        
        // Render all entities with sprite components
        for (entity, (transform, sprite)) in self.world.query::<(&Transform, &Sprite)>().iter() {
            self.renderer.draw_sprite(sprite, transform, interpolation);
        }
        
        self.renderer.end_frame();
    }
    
    pub fn get_render_frame(&self) -> Option<Vec<u8>> {
        self.renderer.get_frame_data()
    }
    
    pub fn load_compiled_game(&mut self, data: &[u8]) -> Result<(), EngineError> {
        let game: CompiledGame = bincode::deserialize(data)?;
        
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
        
        if let Some(body) = data.rigid_body {
            self.world.add_component(entity, body);
            self.physics.add_rigid_body(entity, body);
        }
        
        if let Some(collider) = data.collider {
            self.world.add_component(entity, collider);
            self.physics.add_collider(entity, collider);
        }
        
        Ok(entity)
    }
    
    pub fn create_test_scene(&mut self) {
        // Create a test entity with a sprite
        let entity = self.world.create_entity();
        
        self.world.add_component(entity, Transform::from_position(Vec3::new(400.0, 300.0, 0.0)));
        self.world.add_component(entity, Sprite {
            texture_id: "test_sprite".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        });
        
        // Add physics
        let body = RigidBody::new(Vec2::new(400.0, 300.0), BodyType::Dynamic)
            .with_mass(1.0)
            .with_velocity(Vec2::new(50.0, 0.0));
        self.world.add_component(entity, body.clone());
        self.physics.add_rigid_body(entity, body);
        
        let collider = Collider::circle(32.0);
        self.world.add_component(entity, collider.clone());
        self.physics.add_collider(entity, collider);
    }
}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Renderer initialization failed: {0}")]
    RendererInit(String),
    
    #[error("Failed to deserialize game data: {0}")]
    Deserialization(#[from] bincode::Error),
    
    #[error("Component not found")]
    ComponentNotFound,
    
    #[error("Entity not found")]
    EntityNotFound,
    
    #[error("System error: {0}")]
    SystemError(String),
}

// Compiled game format
#[derive(Serialize, Deserialize)]
pub struct CompiledGame {
    pub entities: Vec<EntityData>,
    pub assets: HashMap<String, Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct EntityData {
    pub name: String,
    pub transform: Option<Transform>,
    pub sprite: Option<Sprite>,
    pub rigid_body: Option<RigidBody>,
    pub collider: Option<Collider>,
}

// Visual script types (shared with TypeScript)
#[derive(Serialize, Deserialize, Clone)]
pub struct VisualScript {
    pub id: String,
    pub name: String,
    pub nodes: Vec<VisualScriptNode>,
    pub connections: Vec<VisualScriptConnection>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VisualScriptNode {
    pub id: String,
    pub node_type: String,
    #[serde(rename = "type")]
    pub node_type_alt: Option<String>, // Handle both 'node_type' and 'type' from TypeScript
    pub position: (f32, f32),
    pub data: HashMap<String, serde_json::Value>,
}

impl VisualScriptNode {
    pub fn get_type(&self) -> &str {
        self.node_type_alt.as_ref().unwrap_or(&self.node_type)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VisualScriptConnection {
    pub id: String,
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub scenes: Vec<Scene>,
    pub scripts: Vec<VisualScript>,
    pub assets: Vec<AssetInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub objects: Vec<GameObject>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameObject {
    pub id: String,
    pub name: String,
    pub position: math::Vec2,
    pub rotation: f32,
    pub scale: math::Vec2,
    pub components: Vec<ComponentData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ComponentData {
    pub component_type: String,
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub asset_type: String,
}

// Integration with Tauri
#[cfg(feature = "tauri-integration")]
pub mod tauri_integration {
    use super::*;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    // Global storage for preview engines
    static PREVIEW_ENGINES: Lazy<Mutex<HashMap<String, Arc<Mutex<DreamEngine>>>>> = 
        Lazy::new(|| Mutex::new(HashMap::new()));
    
    pub fn create_preview_engine(project_id: String) -> Result<String, String> {
        let engine = DreamEngine::new(EngineConfig::default())
            .map_err(|e| e.to_string())?;
        
        // Add a test scene for now
        let mut engine = engine;
        engine.create_test_scene();
        
        let engine_id = format!("engine_{}", project_id);
        let engine_arc = Arc::new(Mutex::new(engine));
        
        PREVIEW_ENGINES.lock().unwrap()
            .insert(engine_id.clone(), engine_arc);
        
        Ok(engine_id)
    }
    
    pub fn update_preview_scene(engine_id: String, scene_data: Vec<u8>) -> Result<(), String> {
        let engines = PREVIEW_ENGINES.lock().unwrap();
        let engine = engines.get(&engine_id)
            .ok_or_else(|| "Engine not found".to_string())?;
        
        let mut engine = engine.lock().unwrap();
        
        // Clear current scene
        engine.world_mut().clear();
        
        // Load new scene data
        // This would deserialize the scene_data and create entities
        
        Ok(())
    }
    
    pub fn render_preview_frame(engine_id: String, dt: f32) -> Result<Vec<u8>, String> {
        let engines = PREVIEW_ENGINES.lock().unwrap();
        let engine = engines.get(&engine_id)
            .ok_or_else(|| "Engine not found".to_string())?;
        
        let mut engine = engine.lock().unwrap();
        
        // Update engine
        engine.update(dt);
        
        // Get render data
        engine.get_render_frame()
            .ok_or_else(|| "No frame data available".to_string())
    }
    
    pub fn destroy_preview_engine(engine_id: String) -> Result<(), String> {
        PREVIEW_ENGINES.lock().unwrap()
            .remove(&engine_id)
            .ok_or_else(|| "Engine not found".to_string())?;
        
        Ok(())
    }
    
    pub fn compile_visual_script(script_json: String) -> Result<String, String> {
        let script: VisualScript = serde_json::from_str(&script_json)
            .map_err(|e| format!("Failed to parse script: {}", e))?;
        
        compiler::compile_visual_script(&script)
            .map(|compiled| compiled.code)
            .map_err(|e| e.to_string())
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
    
    #[test]
    fn test_physics_integration() {
        let mut engine = DreamEngine::new(EngineConfig::default()).unwrap();
        
        // Create entity with physics
        let entity = engine.world_mut().create_entity();
        
        let body = RigidBody::new(Vec2::ZERO, BodyType::Dynamic);
        engine.world_mut().add_component(entity, body.clone());
        engine.physics_mut().add_rigid_body(entity, body);
        
        // Run a physics step
        engine.update(1.0 / 60.0);
        
        // Check that physics ran
        assert!(engine.physics().get_body(entity).is_some());
    }
}