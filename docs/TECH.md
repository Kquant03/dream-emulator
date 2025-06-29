# Dream Emulator - Complete Technical Documentation

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture Evolution](#architecture-evolution)
3. [The Hybrid Engine Architecture](#the-hybrid-engine-architecture)
4. [Visual Script Compilation Pipeline](#visual-script-compilation-pipeline)
5. [Engine Implementation Details](#engine-implementation-details)
6. [Build and Distribution System](#build-and-distribution-system)
7. [Performance Optimization Strategy](#performance-optimization-strategy)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Code Examples and Patterns](#code-examples-and-patterns)
10. [Future Enhancements](#future-enhancements)

## Project Overview

Dream Emulator is a revolutionary game creation tool that combines visual, node-based programming with the performance of native compilation. Unlike traditional game engines that interpret scripts at runtime, Dream Emulator compiles visual node graphs directly to optimized native code, producing standalone executables of 2-5MB instead of 150MB+ Electron applications.

### Core Innovation

The key insight is treating visual scripting not as a runtime interpretation layer, but as a **visual programming language** that compiles to efficient native code. This is achieved through a three-tier architecture:

1. **TypeScript/React Editor**: Provides intuitive visual editing
2. **Rust Compilation Engine**: Transforms visual scripts to optimized systems
3. **Zig Performance Modules**: Handles performance-critical operations

## Architecture Evolution

### Current State (Implemented)

```
┌─────────────────────────────────────────┐
│          TypeScript/React UI             │
│  • Main Menu with engine selection       │
│  • Top-down game editor interface        │
│  • Asset management panel                │
│  • PIXI.js game canvas                   │
│  • Zustand state management              │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│            Tauri Backend                 │
│  • File system access                    │
│  • Native window management              │
│  • Basic IPC commands                    │
└─────────────────────────────────────────┘
```

### Target Architecture (In Development)

```
┌─────────────────────────────────────────┐
│          Visual Editor (TS/React)        │
│  • Node-based visual scripting           │
│  • Real-time preview via engine          │
│  • Asset pipeline integration            │
└─────────────────┬───────────────────────┘
                  │ Compiles to
┌─────────────────▼───────────────────────┐
│      Dream Engine Runtime (Rust)         │
│  • Compile-time optimized ECS           │
│  • Zero-cost abstractions               │
│  • Hot-reload support                    │
│  • Native & WASM targets                │
└─────────────────┬───────────────────────┘
                  │ Optional FFI
┌─────────────────▼───────────────────────┐
│    Performance Modules (Zig)             │
│  • SIMD physics                          │
│  • Compile-time spatial hashing          │
│  • Perfect hash functions                │
└─────────────────────────────────────────┘
```

## The Hybrid Engine Architecture

### Why This Architecture?

1. **TypeScript/React for Editor**
   - Rapid UI development
   - Rich ecosystem (React Flow for nodes, PIXI.js for preview)
   - Cross-platform with Tauri

2. **Rust for Engine Core**
   - Memory safety without GC pauses
   - Excellent FFI for both TypeScript and Zig
   - Strong type system for compile-time guarantees
   - WASM support for web deployment

3. **Zig for Performance Hotspots**
   - Compile-time code generation (`comptime`)
   - Direct SIMD intrinsics
   - No hidden allocations
   - C ABI compatibility

### Component Interaction

```rust
// Rust engine calls Zig physics
extern "C" {
    fn dream_physics_create() -> *mut PhysicsWorld;
    fn dream_physics_step(world: *mut PhysicsWorld, dt: f32, count: u32);
}

// TypeScript editor calls Rust via Tauri
const engineId = await invoke('create_preview_engine', { projectId });
const frame = await invoke('render_frame', { engineId, dt: 1/60 });
```

## Visual Script Compilation Pipeline

### Overview

Visual node graphs are not interpreted at runtime. Instead, they go through a sophisticated compilation pipeline:

```
Visual Nodes → TypeScript Analyzer → Rust Code Generation → LLVM → Native Binary
```

### Node Types and Compilation

#### Event Nodes
```typescript
// Visual node definition
{
  type: 'event/collision',
  data: { filterTag: 'enemy' }
}

// Compiles to Rust
for (entity_a, entity_b) in physics.get_collision_pairs() {
    if entity_a.has_tag("enemy") || entity_b.has_tag("enemy") {
        // Connected nodes execute here
    }
}
```

#### Component Nodes
```typescript
// Visual node for getting transform
{
  type: 'component/get',
  data: { component: 'Transform' }
}

// Compiles to Rust
let transform = world.get_component::<Transform>(entity)?;
```

#### Logic Nodes
```typescript
// Visual if statement
{
  type: 'flow/if',
  inputs: { condition: 'health <= 0' }
}

// Compiles to Rust
if health <= 0 {
    // Then branch nodes
} else {
    // Else branch nodes
}
```

### Optimization Passes

1. **Dead Code Elimination**: Unconnected nodes are removed
2. **Constant Folding**: Compile-time math evaluation
3. **Loop Invariant Hoisting**: Move calculations outside loops
4. **System Batching**: Group similar operations

## Engine Implementation Details

### Entity Component System (ECS)

The ECS is designed for cache efficiency and compile-time optimization:

```rust
// Components are stored in cache-friendly arrays
pub struct ComponentStorage {
    // Structure of Arrays (SoA) for SIMD
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    healths: Vec<f32>,
}

// Systems are compiled from visual scripts
impl System for PlayerMovementSystem {
    fn execute(&mut self, world: &mut World, dt: f32) {
        // This code is generated from visual nodes
        for (entity, (pos, vel)) in world.query::<(&mut Position, &Velocity)>() {
            pos.x += vel.x * dt;
            pos.y += vel.y * dt;
        }
    }
}
```

### Memory Management

```rust
// Zero-copy asset loading via memory mapping
pub struct AssetManager {
    mapped_files: HashMap<AssetId, MmapAsset>,
}

// Object pooling for frequent allocations
pub struct ObjectPool<T> {
    available: Vec<T>,
    in_use: Vec<bool>,
}
```

### Hot Reload Architecture

```rust
// Systems can be reloaded without losing state
pub trait HotReloadable {
    fn serialize_state(&self) -> Vec<u8>;
    fn deserialize_state(&mut self, data: &[u8]);
}

// Reload process
1. Detect file change
2. Serialize current state
3. Unload old system
4. Load new system
5. Restore state
6. Continue execution
```

## Build and Distribution System

### Project Structure for Compiled Games

```
my_game/
├── src/
│   ├── main.rs           # Generated entry point
│   ├── systems.rs        # Compiled visual scripts
│   ├── entities.rs       # Scene data
│   └── components.rs     # Component definitions
├── assets/
│   ├── textures/         # Optimized images
│   ├── audio/            # Compressed sounds
│   └── manifest.bin      # Asset registry
├── Cargo.toml            # Generated build config
└── build.rs              # Asset embedding
```

### Build Pipeline

1. **Analysis Phase**
   ```typescript
   const compiler = new VisualScriptCompiler();
   const systems = project.scripts.map(s => compiler.compile(s));
   ```

2. **Code Generation**
   ```rust
   // Generated from visual scripts
   pub struct HealthSystem;
   impl System for HealthSystem {
       fn execute(&mut self, world: &mut World, dt: f32) {
           for (entity, health) in world.query::<&mut Health>() {
               if health.value <= 0.0 {
                   world.queue_destroy(entity);
               }
           }
       }
   }
   ```

3. **Optimization**
   ```toml
   [profile.release]
   lto = true           # Link-time optimization
   codegen-units = 1    # Single compilation unit
   opt-level = 3        # Maximum optimization
   strip = true         # Remove debug symbols
   ```

4. **Distribution**
   - Windows: Single `.exe` with embedded assets
   - Linux: Executable with launcher script
   - Web: WASM module with web worker support
   - Steam: Integration with Steamworks SDK

## Performance Optimization Strategy

### Compile-Time Optimizations

1. **Zig Comptime Physics**
   ```zig
   // Grid size known at compile time enables perfect hashing
   const SpatialHash = comptime generateSpatialHash(.{
       .grid_size = 64,
       .world_size = 4096,
   });
   ```

2. **Monomorphized Systems**
   ```rust
   // Each query combination generates specialized code
   world.query::<(&Position, &Velocity)>()  // Generates specific iterator
   ```

3. **Data-Oriented Design**
   ```rust
   // Components grouped by access pattern
   struct RenderData {
       positions: Vec<Vec3>,    // All positions together
       sprites: Vec<SpriteId>,  // All sprites together
       // CPU cache friendly iteration
   }
   ```

### Runtime Optimizations

1. **Parallel System Execution**
   ```rust
   // Systems without conflicts run in parallel
   Stage::Parallel(vec![
       PhysicsSystem,
       AnimationSystem,
       AudioSystem,
   ])
   ```

2. **SIMD Operations**
   ```zig
   // Zig enables explicit SIMD
   const positions = @Vector(8, f32){...};
   const velocities = @Vector(8, f32){...};
   positions += velocities * @splat(8, dt);
   ```

3. **Predictive Asset Loading**
   ```rust
   // Analyze scene graph to preload assets
   let load_plan = asset_manager.analyze_scene(&next_scene);
   load_plan.execute_async().await;
   ```

## Implementation Roadmap

### Phase 1: Core Engine (Current)
- [x] Basic ECS architecture
- [x] Tauri integration scaffolding
- [ ] Simple renderer for preview
- [ ] Basic visual script compiler
- [ ] Hot reload infrastructure

### Phase 2: Visual Scripting (Next)
- [ ] Node type definitions
- [ ] TypeScript → Rust compiler
- [ ] Live preview in editor
- [ ] Debugging support
- [ ] Node library UI

### Phase 3: Asset Pipeline
- [ ] Texture optimization
- [ ] Audio compression
- [ ] Asset bundling
- [ ] Streaming support
- [ ] Version control integration

### Phase 4: Performance
- [ ] Zig physics integration
- [ ] SIMD math operations
- [ ] GPU-driven rendering
- [ ] Multithreaded systems
- [ ] Memory pooling

### Phase 5: Distribution
- [ ] Standalone builds
- [ ] WASM compilation
- [ ] Steam integration
- [ ] Auto-updater
- [ ] Crash reporting

### Phase 6: Advanced Features
- [ ] Multiplayer support
- [ ] 3D engine variant
- [ ] Mobile targets
- [ ] Plugin system
- [ ] Cloud saves

## Code Examples and Patterns

### Creating a New System from Visual Nodes

```typescript
// Frontend: User connects nodes
const healingSystem = {
  name: "Healing System",
  nodes: [
    { id: '1', type: 'event/update' },
    { id: '2', type: 'query/get_entities', data: { components: ['Health', 'Healing'] }},
    { id: '3', type: 'flow/foreach', input: '2' },
    { id: '4', type: 'component/get', data: { component: 'Health' }},
    { id: '5', type: 'math/add', inputs: { a: '4.value', b: '3.healing_rate' }},
    { id: '6', type: 'component/set', data: { component: 'Health', value: '5' }}
  ]
};

// Compiles to this Rust code:
pub struct HealingSystem;
impl System for HealingSystem {
    fn execute(&mut self, world: &mut World, dt: f32) {
        for (entity, (health, healing)) in world.query::<(&mut Health, &Healing)>() {
            health.value += healing.rate * dt;
            health.value = health.value.min(health.max);
        }
    }
}
```

### Integrating Zig Physics

```rust
// In Rust
use dream_engine::physics::ZigPhysicsWorld;

let mut physics = ZigPhysicsWorld::new();
physics.add_body(entity_id, x, y, radius, mass);
physics.step(dt);

// Zig implementation
pub fn step(self: *PhysicsWorld, dt: f32) void {
    // Compile-time optimized spatial hash
    self.spatial_hash.clear();
    self.updateSpatialHash();
    
    // SIMD integration if available
    if (comptime hasSimd()) {
        self.integrateSimd(dt);
    } else {
        self.integrateScalar(dt);
    }
}
```

### Asset Processing Pipeline

```rust
// Texture optimization during build
async fn optimize_texture(input: &Path, output: &Path) -> Result<()> {
    let img = image::open(input)?;
    
    // Generate mipmaps
    let mipmaps = generate_mipmaps(&img);
    
    // Compress based on content
    let format = detect_best_format(&img);
    let compressed = compress_texture(&img, format)?;
    
    // Save with metadata
    save_with_metadata(output, compressed, TextureMetadata {
        format,
        has_alpha: img.color().has_alpha(),
        mipmap_count: mipmaps.len(),
    })?;
    
    Ok(())
}
```

## Future Enhancements

### 1. Advanced Visual Scripting
- Subgraphs and functions
- Generic node types
- Visual debugging with breakpoints
- Performance profiling overlay

### 2. Collaborative Editing
- Real-time collaboration via CRDTs
- Git integration for visual scripts
- Branching and merging support
- Cloud project hosting

### 3. Extended Platform Support
- iOS/Android via native APIs
- Console support (Switch, PlayStation, Xbox)
- VR/AR with OpenXR
- Apple Silicon optimization

### 4. AI Integration
- Code completion for visual nodes
- Automated optimization suggestions
- Bug detection and fixes
- Asset generation

### 5. Marketplace
- Share visual script templates
- Asset store integration
- Plugin marketplace
- Revenue sharing for creators

## Conclusion

Dream Emulator represents a paradigm shift in game development tools. By combining the accessibility of visual programming with the performance of native compilation, it enables creators to build games that are both easy to create and blazingly fast to run.

The hybrid architecture leverages the best of each technology:
- TypeScript for rapid UI development
- Rust for safe, fast systems
- Zig for extreme optimization

This is not just another game engine - it's a compiler that happens to have a visual interface. The result is games that run as fast as hand-written C++ while being as easy to create as connecting nodes.

The future of game development is visual, compiled, and incredibly fast. Dream Emulator is making that future a reality.