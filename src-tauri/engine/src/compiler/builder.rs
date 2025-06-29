// src-tauri/engine/src/compiler/builder.rs
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::{Project, VisualScript, compile_visual_script};
use super::CompilerError;

#[derive(Debug, Clone)]
pub enum BuildTarget {
    Native,
    WebAssembly,
    Windows,
    Linux,
    MacOS,
}

#[derive(Debug, Clone)]
pub enum OptimizeLevel {
    Debug,
    Release,
    ReleaseSmall,
}

#[derive(Debug)]
pub struct BuildResult {
    pub executable_path: String,
    pub assets_path: String,
    pub size_bytes: u64,
    pub warnings: Vec<String>,
}

pub struct GameCompiler {
    project: Project,
    target: BuildTarget,
    optimize_level: OptimizeLevel,
}

impl GameCompiler {
    pub fn new(project: Project, target: BuildTarget) -> Self {
        Self {
            project,
            target,
            optimize_level: OptimizeLevel::Release,
        }
    }
    
    pub fn with_optimization(mut self, level: OptimizeLevel) -> Self {
        self.optimize_level = level;
        self
    }
    
    pub async fn compile(&self) -> Result<BuildResult, CompilerError> {
        let build_dir = self.prepare_build_directory()?;
        
        // Step 1: Generate Rust project structure
        self.generate_cargo_toml(&build_dir)?;
        self.generate_main_file(&build_dir)?;
        
        // Step 2: Compile all visual scripts to Rust
        self.generate_systems_code(&build_dir).await?;
        
        // Step 3: Generate entity definitions from scenes
        self.generate_entities_code(&build_dir)?;
        
        // Step 4: Process and embed assets
        let asset_size = self.process_assets(&build_dir).await?;
        
        // Step 5: Build the Rust project
        let executable = self.build_executable(&build_dir).await?;
        
        // Step 6: Create final package
        let result = self.package_game(&build_dir, executable, asset_size).await?;
        
        Ok(result)
    }
    
    fn prepare_build_directory(&self) -> Result<PathBuf, CompilerError> {
        let build_dir = Path::new("target/game_builds").join(&self.project.id);
        
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
        }
        
        fs::create_dir_all(&build_dir)?;
        fs::create_dir_all(build_dir.join("src"))?;
        fs::create_dir_all(build_dir.join("assets"))?;
        
        Ok(build_dir)
    }
    
    fn generate_cargo_toml(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let project_name = self.project.name.to_lowercase().replace(' ', "_");
        
        let cargo_toml = format!(r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[dependencies]
dream-engine = {{ path = "../../../engine" }}
serde = {{ version = "1.0", features = ["derive"] }}
bincode = "1.3"

[profile.release]
opt-level = {}
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.release-small]
inherits = "release"
opt-level = "z"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
            project_name,
            match self.optimize_level {
                OptimizeLevel::Debug => "0",
                OptimizeLevel::Release => "3",
                OptimizeLevel::ReleaseSmall => "\"z\"",
            },
            project_name
        );
        
        fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;
        Ok(())
    }
    
    fn generate_main_file(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let main_code = format!(r#"use dream_engine::{{DreamEngine, EngineConfig}};

mod systems;
mod entities;

// Embedded asset data
const ASSET_DATA: &[u8] = include_bytes!("../assets/assets.pak");

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize engine with project configuration
    let config = EngineConfig {{
        target_fps: 60,
        fixed_timestep: 1.0 / 60.0,
        max_entities: 10000,
    }};
    
    let mut engine = DreamEngine::new(config)?;
    
    // Register all compiled systems
    systems::register_systems(engine.systems_mut());
    
    // Create initial entities from scenes
    entities::create_entities(engine.world_mut(), engine.physics_mut());
    
    // Load embedded assets
    // In production, this would deserialize ASSET_DATA
    
    // Run the game
    #[cfg(not(target_arch = "wasm32"))]
    {{
        // Native game loop
        use std::time::{{Duration, Instant}};
        
        let mut last_frame = Instant::now();
        let frame_time = Duration::from_secs_f32(1.0 / config.target_fps as f32);
        
        loop {{
            let now = Instant::now();
            let dt = now.duration_since(last_frame).as_secs_f32();
            last_frame = now;
            
            engine.update(dt);
            
            // Frame limiting
            let elapsed = Instant::now().duration_since(now);
            if elapsed < frame_time {{
                std::thread::sleep(frame_time - elapsed);
            }}
        }}
    }}
    
    #[cfg(target_arch = "wasm32")]
    {{
        // WASM game loop would be different
        // Using requestAnimationFrame
    }}
    
    Ok(())
}}
"#);
        
        fs::write(build_dir.join("src/main.rs"), main_code)?;
        Ok(())
    }
    
    async fn generate_systems_code(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let mut systems_code = String::new();
        let mut register_calls = Vec::new();
        
        // Add imports
        systems_code.push_str("use dream_engine::*;\n\n");
        
        // Compile each visual script
        for script in &self.project.scripts {
            let compiled = compile_visual_script(script)?;
            systems_code.push_str(&compiled.code);
            systems_code.push_str("\n\n");
            
            let system_name = to_rust_name(&script.name);
            register_calls.push(format!(
                "    schedule.add_system(Box::new({}System {{}}))",
                system_name
            ));
        }
        
        // Add register function
        systems_code.push_str("pub fn register_systems(schedule: &mut SystemSchedule) {\n");
        systems_code.push_str(&register_calls.join(";\n"));
        systems_code.push_str(";\n}\n");
        
        fs::write(build_dir.join("src/systems.rs"), systems_code)?;
        Ok(())
    }
    
    fn generate_entities_code(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let mut entities_code = String::new();
        
        entities_code.push_str("use dream_engine::*;\n\n");
        entities_code.push_str("pub fn create_entities(world: &mut World, physics: &mut PhysicsWorld) {\n");
        
        // Generate entity creation code for each scene
        for scene in &self.project.scenes {
            entities_code.push_str(&format!("    // Scene: {}\n", scene.name));
            
            for object in &scene.objects {
                entities_code.push_str(&format!(
                    r#"    {{
        let entity = world.create_entity();
        
        // Add transform
        world.add_component(entity, Transform {{
            position: Vec3::new({:.2}f32, {:.2}f32, 0.0),
            rotation: Quat::from_rotation_z({:.2}f32),
            scale: Vec3::new({:.2}f32, {:.2}f32, 1.0),
        }});
"#,
                    object.position.x, object.position.y,
                    object.rotation,
                    object.scale.x, object.scale.y
                ));
                
                // Add components based on object data
                for component in &object.components {
                    match component.component_type.as_str() {
                        "Sprite" => {
                            let texture_id = component.data.get("texture_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("default");
                            
                            entities_code.push_str(&format!(
                                r#"        
        // Add sprite
        world.add_component(entity, Sprite {{
            texture_id: "{}".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            source_rect: None,
            pivot: Vec2::new(0.5, 0.5),
        }});
"#,
                                texture_id
                            ));
                        }
                        
                        "RigidBody" => {
                            let body_type = component.data.get("body_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Dynamic");
                            let mass = component.data.get("mass")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(1.0) as f32;
                            
                            entities_code.push_str(&format!(
                                r#"        
        // Add rigid body
        let body = RigidBody::new(Vec2::new({:.2}f32, {:.2}f32), BodyType::{})
            .with_mass({:.2}f32);
        world.add_component(entity, body.clone());
        physics.add_rigid_body(entity, body);
"#,
                                object.position.x, object.position.y,
                                body_type,
                                mass
                            ));
                        }
                        
                        "Collider" => {
                            let collider_type = component.data.get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("circle");
                            
                            match collider_type {
                                "circle" => {
                                    let radius = component.data.get("radius")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(32.0) as f32;
                                    
                                    entities_code.push_str(&format!(
                                        r#"        
        // Add collider
        let collider = Collider::circle({:.2}f32);
        world.add_component(entity, collider.clone());
        physics.add_collider(entity, collider);
"#,
                                        radius
                                    ));
                                }
                                "box" => {
                                    let width = component.data.get("width")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(64.0) as f32;
                                    let height = component.data.get("height")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(64.0) as f32;
                                    
                                    entities_code.push_str(&format!(
                                        r#"        
        // Add collider
        let collider = Collider::box_collider({:.2}f32, {:.2}f32);
        world.add_component(entity, collider.clone());
        physics.add_collider(entity, collider);
"#,
                                        width, height
                                    ));
                                }
                                _ => {}
                            }
                        }
                        
                        _ => {
                            // Custom components would be handled here
                        }
                    }
                }
                
                entities_code.push_str("    }\n\n");
            }
        }
        
        entities_code.push_str("}\n");
        
        fs::write(build_dir.join("src/entities.rs"), entities_code)?;
        Ok(())
    }
    
    async fn process_assets(&self, build_dir: &Path) -> Result<u64, CompilerError> {
        let assets_dir = build_dir.join("assets");
        let mut total_size = 0u64;
        
        // Create asset manifest
        let mut manifest = AssetManifest {
            textures: HashMap::new(),
            audio: HashMap::new(),
            data: HashMap::new(),
        };
        
        // Process each asset
        for asset in &self.project.assets {
            let source_path = Path::new(&asset.path);
            if !source_path.exists() {
                eprintln!("Warning: Asset not found: {}", asset.path);
                continue;
            }
            
            let file_size = fs::metadata(&source_path)?.len();
            total_size += file_size;
            
            match asset.asset_type.as_str() {
                "texture" | "sprite" => {
                    // Optimize textures (simplified - just copy for now)
                    let dest_name = format!("{}.png", asset.id);
                    let dest_path = assets_dir.join(&dest_name);
                    fs::copy(&source_path, &dest_path)?;
                    
                    manifest.textures.insert(asset.id.clone(), dest_name);
                }
                
                "audio" => {
                    // Compress audio (simplified - just copy for now)
                    let dest_name = format!("{}.ogg", asset.id);
                    let dest_path = assets_dir.join(&dest_name);
                    fs::copy(&source_path, &dest_path)?;
                    
                    manifest.audio.insert(asset.id.clone(), dest_name);
                }
                
                _ => {
                    // Copy other assets as-is
                    let dest_name = format!("{}.dat", asset.id);
                    let dest_path = assets_dir.join(&dest_name);
                    fs::copy(&source_path, &dest_path)?;
                    
                    manifest.data.insert(asset.id.clone(), dest_name);
                }
            }
        }
        
        // Create asset pack
        let manifest_bytes = bincode::serialize(&manifest)?;
        fs::write(assets_dir.join("manifest.bin"), &manifest_bytes)?;
        
        // In production, you'd create a single PAK file
        // For now, create a placeholder
        fs::write(assets_dir.join("assets.pak"), b"PLACEHOLDER")?;
        
        Ok(total_size)
    }
    
    async fn build_executable(&self, build_dir: &Path) -> Result<PathBuf, CompilerError> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(build_dir);
        
        // Set target based on build target
        match self.target {
            BuildTarget::Native => {
                cmd.arg("build");
            }
            BuildTarget::WebAssembly => {
                cmd.arg("build")
                   .arg("--target")
                   .arg("wasm32-unknown-unknown");
            }
            BuildTarget::Windows => {
                cmd.arg("build")
                   .arg("--target")
                   .arg("x86_64-pc-windows-gnu");
            }
            BuildTarget::Linux => {
                cmd.arg("build")
                   .arg("--target")
                   .arg("x86_64-unknown-linux-gnu");
            }
            BuildTarget::MacOS => {
                cmd.arg("build")
                   .arg("--target")
                   .arg("x86_64-apple-darwin");
            }
        }
        
        // Set optimization level
        match self.optimize_level {
            OptimizeLevel::Debug => {}
            OptimizeLevel::Release => {
                cmd.arg("--release");
            }
            OptimizeLevel::ReleaseSmall => {
                cmd.arg("--profile")
                   .arg("release-small");
            }
        }
        
        // Run the build
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CompilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Build failed: {}", stderr)
            )));
        }
        
        // Find the output executable
        let exe_name = self.project.name.to_lowercase().replace(' ', "_");
        let exe_path = match self.target {
            BuildTarget::Windows => build_dir.join(format!("target/release/{}.exe", exe_name)),
            _ => build_dir.join(format!("target/release/{}", exe_name)),
        };
        
        Ok(exe_path)
    }
    
    async fn package_game(
        &self,
        build_dir: &Path,
        executable: PathBuf,
        asset_size: u64
    ) -> Result<BuildResult, CompilerError> {
        let output_dir = Path::new("target/games").join(&self.project.name);
        fs::create_dir_all(&output_dir)?;
        
        // Copy executable
        let final_exe = output_dir.join(executable.file_name().unwrap());
        fs::copy(&executable, &final_exe)?;
        
        // Copy assets
        let assets_output = output_dir.join("assets");
        if build_dir.join("assets").exists() {
            copy_dir_all(build_dir.join("assets"), &assets_output)?;
        }
        
        // Get executable size
        let exe_size = fs::metadata(&final_exe)?.len();
        
        // Create platform-specific launcher if needed
        #[cfg(unix)]
        if matches!(self.target, BuildTarget::Linux | BuildTarget::MacOS) {
            let launcher = output_dir.join("launch.sh");
            fs::write(&launcher, format!(
                "#!/bin/bash\ncd \"$(dirname \"$0\")\"\n./{}\n",
                executable.file_name().unwrap().to_str().unwrap()
            ))?;
            
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755))?;
            fs::set_permissions(&final_exe, fs::Permissions::from_mode(0o755))?;
        }
        
        Ok(BuildResult {
            executable_path: final_exe.to_string_lossy().to_string(),
            assets_path: assets_output.to_string_lossy().to_string(),
            size_bytes: exe_size + asset_size,
            warnings: vec![],
        })
    }
}

#[derive(Serialize, Deserialize)]
struct AssetManifest {
    textures: HashMap<String, String>,
    audio: HashMap<String, String>,
    data: HashMap<String, String>,
}

fn to_rust_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_')
        .collect::<String>()
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}