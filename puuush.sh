#!/bin/bash

# Create Dream Engine Structure
# Run this from the root of your dream-emulator project

echo "🚀 Creating Dream Engine structure..."

# Create the engine crate
cd src-tauri
cargo new engine --lib
cd ..

# Create all necessary directories
echo "📁 Creating directories..."
mkdir -p src-tauri/engine/src/{ecs,renderer,physics,compiler,assets,math}
mkdir -p src-tauri/engine/zig
mkdir -p src/compiler

# Create engine module files
echo "📄 Creating engine module files..."

# Main engine file
cat > src-tauri/engine/src/lib.rs << 'EOF'
// Main engine implementation
// TODO: Copy content from "Dream Engine Core Implementation" artifact

pub mod ecs;
pub mod renderer;
pub mod physics;
pub mod compiler;
pub mod assets;
pub mod math;

// Re-export commonly used types
pub use ecs::*;
pub use math::*;
pub use renderer::*;

// Placeholder for main engine code
EOF

# ECS module files
cat > src-tauri/engine/src/ecs/mod.rs << 'EOF'
// Entity Component System module
// TODO: Move ECS code from lib.rs here

pub mod world;
pub mod component;
pub mod system;
pub mod query;

pub use world::*;
pub use component::*;
pub use system::*;
pub use query::*;
EOF

touch src-tauri/engine/src/ecs/world.rs
touch src-tauri/engine/src/ecs/component.rs
touch src-tauri/engine/src/ecs/system.rs
touch src-tauri/engine/src/ecs/query.rs

# Renderer module files
cat > src-tauri/engine/src/renderer/mod.rs << 'EOF'
// Renderer module
// TODO: Move renderer code from lib.rs here

pub mod wgpu_renderer;
pub mod canvas_renderer;

pub use wgpu_renderer::*;
EOF

touch src-tauri/engine/src/renderer/wgpu_renderer.rs
touch src-tauri/engine/src/renderer/canvas_renderer.rs

# Physics module files
cat > src-tauri/engine/src/physics/mod.rs << 'EOF'
// Physics module
// TODO: Move physics code from lib.rs here

pub mod world;
pub mod collision;

pub use world::*;
pub use collision::*;
EOF

touch src-tauri/engine/src/physics/world.rs
touch src-tauri/engine/src/physics/collision.rs

# Compiler module files
cat > src-tauri/engine/src/compiler/mod.rs << 'EOF'
// Compiler module
// TODO: Copy content from "Game Build Script" artifact

pub mod visual_script;
pub mod code_generator;
pub mod builder;

pub use visual_script::*;
pub use code_generator::*;
pub use builder::*;
EOF

touch src-tauri/engine/src/compiler/visual_script.rs
touch src-tauri/engine/src/compiler/code_generator.rs
touch src-tauri/engine/src/compiler/builder.rs

# Assets module files
cat > src-tauri/engine/src/assets/mod.rs << 'EOF'
// Asset management module

pub mod loader;
pub mod cache;

pub use loader::*;
pub use cache::*;
EOF

touch src-tauri/engine/src/assets/loader.rs
touch src-tauri/engine/src/assets/cache.rs

# Math module files
cat > src-tauri/engine/src/math/mod.rs << 'EOF'
// Math utilities module

pub mod vectors;

pub use vectors::*;
EOF

touch src-tauri/engine/src/math/vectors.rs

# Create Zig files
echo "📄 Creating Zig files..."

cat > src-tauri/engine/zig/physics.zig << 'EOF'
// High-performance physics module
// TODO: Copy content from "Zig Physics Performance Module" artifact

const std = @import("std");

// Placeholder for Zig physics code
EOF

cat > src-tauri/engine/zig/math.zig << 'EOF'
// SIMD math operations

const std = @import("std");

// TODO: Implement SIMD math operations
EOF

cat > src-tauri/engine/zig/build.zig << 'EOF'
// Zig build configuration

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    
    const physics_lib = b.addSharedLibrary(.{
        .name = "dream_physics",
        .root_source_file = .{ .path = "physics.zig" },
        .target = target,
        .optimize = optimize,
    });
    
    physics_lib.linkLibC();
    b.installArtifact(physics_lib);
}
EOF

# Create engine Cargo.toml
cat > src-tauri/engine/Cargo.toml << 'EOF'
# Engine dependencies
# TODO: Copy content from "Integrating Dream Engine with Tauri" artifact

[package]
name = "dream-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
# Placeholder - copy from artifact
EOF

# Create build.rs for engine
cat > src-tauri/engine/build.rs << 'EOF'
// Build script for Zig integration

use std::process::Command;

fn main() {
    // Build Zig modules if Zig is installed
    if has_zig() {
        println!("cargo:rerun-if-changed=zig/physics.zig");
        
        Command::new("zig")
            .args(&["build-lib", "zig/physics.zig", "-O", "ReleaseFast"])
            .status()
            .expect("Failed to build Zig physics module");
            
        println!("cargo:rustc-link-lib=static=physics");
    }
}

fn has_zig() -> bool {
    Command::new("zig").arg("version").output().is_ok()
}
EOF

# Create TypeScript compiler files
echo "📄 Creating TypeScript compiler files..."

cat > src/compiler/visual-script-compiler.ts << 'EOF'
// Visual Script to Rust Compiler
// TODO: Copy content from "Visual Script to Rust Compiler" artifact

export class VisualScriptCompiler {
  // Placeholder
}
EOF

cat > src/compiler/node-definitions.ts << 'EOF'
// Node type definitions for visual scripting

export interface NodeDefinition {
  type: string;
  category: string;
  inputs: PortDefinition[];
  outputs: PortDefinition[];
}

export interface PortDefinition {
  name: string;
  type: string;
}

// TODO: Define all node types
EOF

cat > src/compiler/index.ts << 'EOF'
// Compiler module exports

export { VisualScriptCompiler } from './visual-script-compiler';
export * from './node-definitions';
EOF

# Update workspace Cargo.toml
echo "📝 Updating workspace configuration..."

# Check if workspace section exists
if ! grep -q "\[workspace\]" src-tauri/Cargo.toml; then
    # Add workspace configuration at the beginning of the file
    cat > src-tauri/Cargo.toml.tmp << 'EOF'
[workspace]
members = [".", "engine"]

EOF
    cat src-tauri/Cargo.toml >> src-tauri/Cargo.toml.tmp
    mv src-tauri/Cargo.toml.tmp src-tauri/Cargo.toml
    echo "✅ Added workspace configuration to src-tauri/Cargo.toml"
else
    echo "⚠️  Workspace already configured in src-tauri/Cargo.toml"
fi

echo "
✨ Engine structure created successfully!

📋 Next steps:
1. Copy the artifact contents into the respective files:
   - 'Dream Engine Core Implementation' → src-tauri/engine/src/lib.rs
   - 'Visual Script to Rust Compiler' → src/compiler/visual-script-compiler.ts
   - 'Zig Physics Performance Module' → src-tauri/engine/zig/physics.zig
   - 'Integrating Dream Engine with Tauri' → src-tauri/engine/Cargo.toml
   - 'Game Build Script' → src-tauri/engine/src/compiler/mod.rs

2. Update src-tauri/src/lib.rs with the new Tauri commands from:
   - 'Tauri Commands for Engine Integration' artifact

3. Add the engine dependency to src-tauri/Cargo.toml:
   [dependencies]
   dream-engine = { path = \"./engine\", features = [\"tauri-integration\"] }

4. Build the engine:
   cd src-tauri/engine && cargo build

🎮 Happy game making!
"