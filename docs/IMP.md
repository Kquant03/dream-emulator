# Dream Emulator - Quick Implementation Reference

## Current Status Summary

### ✅ Completed
- Main menu with engine selection
- Basic top-down game editor UI
- PIXI.js canvas integration
- Zustand state management
- Asset panel UI
- Tauri desktop app scaffolding

### 🚧 In Progress
- Dream Engine core (Rust)
- Visual script compiler (TypeScript → Rust)
- ECS implementation
- Tauri command integration

### 📋 Not Started
- Zig performance modules
- Hot reload system
- Build/export pipeline
- Multiplayer support
- 3D engine variant

## Key Files to Implement

### 1. Engine Core (`src-tauri/engine/src/lib.rs`)
```rust
// Core engine with ECS, renderer, physics
pub struct DreamEngine {
    world: World,
    systems: SystemSchedule,
    renderer: Box<dyn Renderer>,
}
```
**Status**: Scaffold created, needs implementation
**Dependencies**: `wgpu`, `rapier2d`, `bincode`

### 2. Visual Script Compiler (`src/compiler/visual-script-compiler.ts`)
```typescript
// Converts node graphs to Rust code
export class VisualScriptCompiler {
    compile(script: VisualScript): CompiledSystem
}
```
**Status**: Algorithm designed, needs implementation
**Key Feature**: Topological sort → code generation

### 3. Tauri Integration (`src-tauri/src/lib.rs`)
```rust
// Add these commands to existing file
#[tauri::command]
async fn create_preview_engine(...)
#[tauri::command] 
async fn compile_game(...)
```
**Status**: Commands defined, needs integration
**Action**: Merge with existing Tauri commands

### 4. Zig Physics (`src-tauri/engine/zig/physics.zig`)
```zig
// Compile-time optimized physics
pub fn PhysicsWorld(comptime config: PhysicsConfig) type
```
**Status**: Optional performance enhancement
**Benefit**: 2-3x physics performance

## Implementation Order

### Week 1: Core Engine
1. Implement basic ECS in `engine/src/ecs/`
2. Create preview renderer using `wgpu` or canvas
3. Add Tauri commands for engine creation
4. Test preview rendering in editor

### Week 2: Visual Scripting
1. Define node types in TypeScript
2. Implement compiler's code generation
3. Create test visual scripts
4. Verify generated Rust code compiles

### Week 3: Integration
1. Connect editor preview to engine
2. Implement hot reload for systems
3. Add basic physics (Rust first, Zig later)
4. Create first playable demo

### Week 4: Build Pipeline
1. Implement game export command
2. Asset optimization pipeline
3. Create standalone executables
4. Test on multiple platforms

## Critical Integration Points

### Editor → Engine Communication
```typescript
// In React component
const engineId = await invoke('create_preview_engine', { projectId });
// Update game state
await invoke('update_preview_scene', { engineId, sceneData });
// Get rendered frame
const frame = await invoke('render_frame', { engineId, dt: 1/60 });
```

### Visual Script → Rust Code
```
Node Graph → TypeScript Compiler → Rust Source → Cargo Build → Native Binary
```

### Asset Pipeline
```
Raw Asset → Optimization → Bundling → Embedding in Binary
```

## Performance Targets

- **Compiled Game Size**: < 5MB (vs 150MB Electron)
- **Startup Time**: < 1 second
- **Frame Rate**: 60 FPS with 1000+ entities
- **Compile Time**: < 10 seconds for average project
- **Hot Reload**: < 100ms

## Common Pitfalls to Avoid

1. **Don't interpret visual scripts** - Always compile to native code
2. **Don't copy assets multiple times** - Use memory mapping
3. **Don't block on compilation** - Use async Rust commands
4. **Don't rebuild unchanged systems** - Cache compiled code
5. **Don't forget WASM support** - Use conditional compilation

## Testing Strategy

### Unit Tests
- ECS operations
- Visual script compilation
- Asset processing

### Integration Tests
- Editor ↔ Engine communication
- Full compilation pipeline
- Cross-platform builds

### Performance Tests
- Entity count limits
- Frame time budgets
- Memory usage

## Next Developer Actions

1. **Run setup script**: `./create-engine.sh`
2. **Copy artifact contents** to created files
3. **Install Rust dependencies**: `cd src-tauri/engine && cargo build`
4. **Test basic engine**: `cargo test`
5. **Integrate with editor**: Update Tauri commands
6. **Create first visual script**: Test compilation

## Resources

- **Rust ECS**: Consider `hecs` or `bevy_ecs` for inspiration
- **WGPU Examples**: https://github.com/gfx-rs/wgpu/tree/master/examples
- **Zig Learning**: https://ziglearn.org/
- **Tauri IPC**: https://tauri.app/v1/guides/features/command/

## Success Criteria

You know the implementation is working when:
1. ✓ Can create visual node graphs in editor
2. ✓ Preview updates in real-time
3. ✓ Can export standalone 2-5MB executables
4. ✓ Games run at 60 FPS
5. ✓ Hot reload works during development

## Questions This Documentation Answers

- **Q**: Why Rust + Zig instead of just TypeScript?  
  **A**: Native performance, 50x smaller binaries

- **Q**: How do visual scripts become native code?  
  **A**: TypeScript analyzer → Rust code gen → LLVM → Binary

- **Q**: What makes this different from Godot/Unity?  
  **A**: Compile-time optimization, no runtime interpreter

- **Q**: Can I still use JavaScript for game logic?  
  **A**: No, everything compiles to native code for performance

- **Q**: Will this work on the web?  
  **A**: Yes, via WebAssembly compilation target