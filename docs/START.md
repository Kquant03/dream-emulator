# Dream Emulator - Getting Started Implementation Guide

## 🚀 Immediate Next Steps

### Step 1: Run Setup Script (2 minutes)
```bash
# From project root
chmod +x create-engine.sh
./create-engine.sh
```

### Step 2: Copy Artifact Contents (10 minutes)

Copy each artifact to its destination file:

| Artifact | Destination File |
|----------|-----------------|
| "Dream Engine Core Implementation" | `src-tauri/engine/src/lib.rs` |
| "Visual Script to Rust Compiler" | `src/compiler/visual-script-compiler.ts` |
| "Tauri Commands for Engine Integration" | Merge into `src-tauri/src/lib.rs` |
| "Zig Physics Performance Module" | `src-tauri/engine/zig/physics.zig` |
| "Integrating Dream Engine with Tauri" | `src-tauri/engine/Cargo.toml` |
| "Game Build Script" | `src-tauri/engine/src/compiler/mod.rs` |

### Step 3: Update Dependencies (5 minutes)

1. **Update `src-tauri/Cargo.toml`**:
```toml
[workspace]
members = [".", "engine"]

[dependencies]
dream-engine = { path = "./engine", features = ["tauri-integration"] }
# ... existing dependencies
```

2. **Verify engine builds**:
```bash
cd src-tauri/engine
cargo build
```

### Step 4: Test Basic Integration (10 minutes)

1. **Create a test file** `src-tauri/engine/tests/basic_test.rs`:
```rust
#[cfg(test)]
mod tests {
    use dream_engine::{DreamEngine, EngineConfig};
    
    #[test]
    fn test_engine_creation() {
        let engine = DreamEngine::new(EngineConfig::default());
        assert!(engine.is_ok());
    }
}
```

2. **Run the test**:
```bash
cd src-tauri/engine
cargo test
```

### Step 5: Wire Up Editor Preview (20 minutes)

1. **Update your React component** to use the engine:
```typescript
// src/components/GameCreator/TopDownGameCreator.tsx
import { invoke } from '@tauri-apps/api/tauri';

// In your component
useEffect(() => {
    let engineId: string;
    
    async function initEngine() {
        engineId = await invoke('create_preview_engine', { 
            projectId: currentProject.id 
        });
        
        // Start render loop
        const renderLoop = async () => {
            const frameData = await invoke('render_preview_frame', {
                engineId,
                dt: 1/60
            });
            // Update your PIXI canvas with frameData
            requestAnimationFrame(renderLoop);
        };
        renderLoop();
    }
    
    initEngine();
    
    return () => {
        // Cleanup engine
        if (engineId) {
            invoke('destroy_preview_engine', { engineId });
        }
    };
}, [currentProject]);
```

2. **Test the integration**:
```bash
npm run tauri dev
```

## 🧪 Verification Checklist

### ✓ Engine Setup is Correct When:

- [ ] `cargo build` succeeds in `src-tauri/engine/`
- [ ] `cargo test` passes all tests
- [ ] No compilation errors in `src-tauri/`

### ✓ TypeScript Integration Works When:

- [ ] `invoke('create_preview_engine')` returns an engine ID
- [ ] No TypeScript errors in the compiler module
- [ ] Visual script compiler can be imported

### ✓ Basic Functionality Works When:

- [ ] Editor opens without errors
- [ ] Can create a new project
- [ ] Preview canvas initializes
- [ ] No console errors in developer tools

## 🔧 Common Issues and Solutions

### Issue: "dream-engine not found"
**Solution**: Ensure workspace is configured in `src-tauri/Cargo.toml`:
```toml
[workspace]
members = [".", "engine"]
```

### Issue: "Module not found: compiler"
**Solution**: Add to `tsconfig.json`:
```json
{
  "compilerOptions": {
    "paths": {
      "@/compiler/*": ["./src/compiler/*"]
    }
  }
}
```

### Issue: Tauri commands not found
**Solution**: Regenerate handler in `src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    greet,
    create_preview_engine,
    update_preview_scene,
    render_preview_frame,
    // ... other commands
])
```

## 📊 First Milestone Goals

### Week 1 Targets
1. **Basic ECS**: Can create entities with transform components
2. **Simple Renderer**: Can draw colored rectangles
3. **Preview Works**: Editor shows live preview
4. **Hot Reload**: Can update without restart

### Success Metrics
- Create 100 entities without performance issues
- Maintain 60 FPS in preview
- Hot reload in < 200ms
- Zero memory leaks over 10 minutes

## 🎯 First Demo Goal

Create a simple demo where:
1. User places sprites in the editor
2. Sprites appear in the preview
3. Can add a "rotate" behavior via visual nodes
4. Can export and run as standalone executable

When this works, the core architecture is proven!

## 💡 Pro Tips

1. **Start Simple**: Get a rectangle rendering before complex sprites
2. **Log Everything**: Add debug logging to Tauri commands
3. **Test Often**: Run tests after each major change
4. **Profile Early**: Monitor performance from the start
5. **Document Decisions**: Keep notes on why you chose specific approaches

## 📞 Getting Help

If stuck, check:
1. **Tauri Discord**: https://discord.com/invite/tauri
2. **Rust GameDev**: https://discord.gg/yNtPTb2
3. **Zig Community**: https://discord.gg/gxsFFjE
4. **Project Issues**: Create detailed bug reports

## 🎉 You're Ready!

With these documents, you have everything needed to implement Dream Emulator's vision:
- A visual game creator that compiles to native code
- Games that are 50x smaller than Electron apps
- Performance that matches hand-written C++
- An editor that's actually fun to use

The future of game development is visual, compiled, and fast. Let's build it! 🚀