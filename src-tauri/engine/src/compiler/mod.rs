// src-tauri/engine/src/compiler/mod.rs
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::{VisualScript, Project};

#[derive(Debug, Clone)]
pub enum BuildTarget {
    Native,
    WebAssembly,
    Windows,
    Linux,
    MacOS,
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

#[derive(Debug, Clone)]
pub enum OptimizeLevel {
    Debug,
    Release,
    ReleaseSmall,  // Optimize for size
}

impl GameCompiler {
    pub fn new(project: Project, target: BuildTarget) -> Self {
        Self {
            project,
            target,
            optimize_level: OptimizeLevel::Release,
        }
    }
    
    pub async fn compile(&self) -> Result<BuildResult, CompilerError> {
        let build_dir = self.prepare_build_directory()?;
        
        // Step 1: Compile visual scripts to Rust
        self.generate_systems_code(&build_dir).await?;
        
        // Step 2: Generate entity data
        self.generate_entity_data(&build_dir)?;
        
        // Step 3: Process and optimize assets
        self.process_assets(&build_dir).await?;
        
        // Step 4: Generate main.rs with embedded data
        self.generate_main_file(&build_dir)?;
        
        // Step 5: Create Cargo.toml
        self.generate_cargo_toml(&build_dir)?;
        
        // Step 6: Build the game
        let executable = self.build_executable(&build_dir).await?;
        
        // Step 7: Package final game
        let result = self.package_game(&build_dir, executable).await?;
        
        Ok(result)
    }
    
    fn prepare_build_directory(&self) -> Result<PathBuf, CompilerError> {
        let build_dir = Path::new("target/game_builds").join(&self.project.id);
        
        // Clean previous build
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
        }
        
        // Create directory structure
        fs::create_dir_all(&build_dir)?;
        fs::create_dir_all(build_dir.join("src"))?;
        fs::create_dir_all(build_dir.join("assets"))?;
        
        Ok(build_dir)
    }
    
    async fn generate_systems_code(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let mut all_systems = Vec::new();
        
        // Compile each visual script
        for script in &self.project.scripts {
            let compiled = compile_visual_script(script)?;
            all_systems.push(compiled.code);
        }
        
        // Write systems.rs
        let systems_code = format!(
            r#"
use dream_engine::{{World, PhysicsWorld, System}};

{}

pub fn register_systems(schedule: &mut SystemSchedule) {{
    {}
}}
"#,
            all_systems.join("\n\n"),
            self.project.scripts.iter()
                .map(|s| format!("schedule.add_system(Box::new({}System {{}});", to_rust_name(&s.name)))
                .collect::<Vec<_>>()
                .join("\n    ")
        );
        
        fs::write(build_dir.join("src/systems.rs"), systems_code)?;
        Ok(())
    }
    
    fn generate_entity_data(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let mut entities_code = String::new();
        
        entities_code.push_str("use dream_engine::*;\n\n");
        entities_code.push_str("pub fn create_entities(world: &mut World) {\n");
        
        // Generate entity creation code for each scene
        for scene in &self.project.scenes {
            for entity in &scene.objects {
                entities_code.push_str(&format!(
                    r#"    {{
        let entity = world.create_entity();
        world.add_component(entity, Transform {{
            position: Vec3 {{ x: {}f32, y: {}f32, z: 0.0 }},
            rotation: Quat::from_rotation_z({}f32),
            scale: Vec3 {{ x: {}f32, y: {}f32, z: 1.0 }},
        }});
"#,
                    entity.position.x, entity.position.y,
                    entity.rotation,
                    entity.scale.x, entity.scale.y
                ));
                
                // Add other components
                for component in &entity.components {
                    entities_code.push_str(&self.generate_component_code(component)?);
                }
                
                entities_code.push_str("    }\n");
            }
        }
        
        entities_code.push_str("}\n");
        
        fs::write(build_dir.join("src/entities.rs"), entities_code)?;
        Ok(())
    }
    
    async fn process_assets(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let assets_dir = build_dir.join("assets");
        
        for asset in &self.project.assets {
            match asset.asset_type.as_str() {
                "texture" => {
                    // Optimize textures
                    self.optimize_texture(&asset.path, &assets_dir).await?;
                }
                "audio" => {
                    // Compress audio
                    self.compress_audio(&asset.path, &assets_dir).await?;
                }
                _ => {
                    // Copy as-is
                    let dest = assets_dir.join(&asset.name);
                    fs::copy(&asset.path, dest)?;
                }
            }
        }
        
        // Generate asset manifest
        let manifest = AssetManifest {
            textures: self.project.assets.iter()
                .filter(|a| a.asset_type == "texture")
                .map(|a| (a.id.clone(), format!("assets/{}", a.name)))
                .collect(),
            audio: self.project.assets.iter()
                .filter(|a| a.asset_type == "audio")
                .map(|a| (a.id.clone(), format!("assets/{}", a.name)))
                .collect(),
        };
        
        let manifest_data = bincode::serialize(&manifest)?;
        fs::write(assets_dir.join("manifest.bin"), manifest_data)?;
        
        Ok(())
    }
    
    fn generate_main_file(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let main_code = format!(
            r#"
use dream_engine::{{DreamEngine, EngineConfig}};

mod systems;
mod entities;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize engine
    let config = EngineConfig {{
        target_fps: 60,
        fixed_timestep: 1.0 / 60.0,
        max_entities: 10000,
    }};
    
    let mut engine = DreamEngine::new(config)?;
    
    // Register all systems
    systems::register_systems(engine.systems_mut());
    
    // Create initial entities
    entities::create_entities(engine.world_mut());
    
    // Load embedded assets
    let asset_manifest = include_bytes!("../assets/manifest.bin");
    engine.load_asset_manifest(asset_manifest)?;
    
    // Run the game
    engine.run()?;
    
    Ok(())
}}

// Embedded asset data for standalone executable
const ASSET_DATA: &[u8] = include_bytes!("../assets/assets.pak");
"#
        );
        
        fs::write(build_dir.join("src/main.rs"), main_code)?;
        Ok(())
    }
    
    fn generate_cargo_toml(&self, build_dir: &Path) -> Result<(), CompilerError> {
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[dependencies]
dream-engine = {{ path = "../../engine" }}

[profile.release]
opt-level = {}
lto = true
codegen-units = 1
strip = true

[profile.release-small]
inherits = "release"
opt-level = "z"
panic = "abort"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
            self.project.name.to_lowercase().replace(' ', "_"),
            match self.optimize_level {
                OptimizeLevel::Debug => "0",
                OptimizeLevel::Release => "3",
                OptimizeLevel::ReleaseSmall => "\"z\"",
            },
            self.project.name.to_lowercase().replace(' ', "_")
        );
        
        fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;
        Ok(())
    }
    
    async fn build_executable(&self, build_dir: &Path) -> Result<PathBuf, CompilerError> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(build_dir);
        
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
            _ => {
                cmd.arg("build");
            }
        }
        
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
        
        // Use Zig for linking if available (better cross-compilation)
        if self.has_zig_installed() {
            cmd.env("CC", "zig cc");
            cmd.env("CXX", "zig c++");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(CompilerError::BuildFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Find the built executable
        let exe_name = self.project.name.to_lowercase().replace(' ', "_");
        let exe_path = match self.target {
            BuildTarget::Windows => build_dir.join(format!("target/release/{}.exe", exe_name)),
            _ => build_dir.join(format!("target/release/{}", exe_name)),
        };
        
        Ok(exe_path)
    }
    
    async fn package_game(&self, build_dir: &Path, executable: PathBuf) -> Result<BuildResult, CompilerError> {
        let output_dir = Path::new("target/games").join(&self.project.name);
        fs::create_dir_all(&output_dir)?;
        
        // Copy executable
        let final_exe = output_dir.join(executable.file_name().unwrap());
        fs::copy(&executable, &final_exe)?;
        
        // Copy assets
        let assets_output = output_dir.join("assets");
        fs::create_dir_all(&assets_output)?;
        copy_dir_all(build_dir.join("assets"), &assets_output)?;
        
        // Calculate final size
        let exe_size = fs::metadata(&final_exe)?.len();
        let assets_size = dir_size(&assets_output)?;
        
        // Create launcher script for Linux/Mac
        if matches!(self.target, BuildTarget::Linux | BuildTarget::MacOS) {
            let launcher = output_dir.join("launch.sh");
            fs::write(&launcher, format!(
                "#!/bin/bash\ncd \"$(dirname \"$0\")\"\n./{}\n",
                executable.file_name().unwrap().to_str().unwrap()
            ))?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755))?;
                fs::set_permissions(&final_exe, fs::Permissions::from_mode(0o755))?;
            }
        }
        
        Ok(BuildResult {
            executable_path: final_exe.to_string_lossy().to_string(),
            assets_path: assets_output.to_string_lossy().to_string(),
            size_bytes: exe_size + assets_size,
            warnings: vec![],
        })
    }
    
    fn has_zig_installed(&self) -> bool {
        Command::new("zig")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    fn generate_component_code(&self, component: &Component) -> Result<String, CompilerError> {
        match component.component_type.as_str() {
            "Sprite" => {
                Ok(format!(
                    r#"        world.add_component(entity, Sprite {{
            texture_id: "{}".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
        }});
"#,
                    component.data.get("texture_id").unwrap_or(&"default".to_string())
                ))
            }
            _ => Ok(String::new()),
        }
    }
    
    async fn optimize_texture(&self, input: &Path, output_dir: &Path) -> Result<(), CompilerError> {
        // Use image crate or call out to external optimizer
        // For now, just copy
        let filename = input.file_name().unwrap();
        fs::copy(input, output_dir.join(filename))?;
        Ok(())
    }
    
    async fn compress_audio(&self, input: &Path, output_dir: &Path) -> Result<(), CompilerError> {
        // Use audio compression library or external tool
        // For now, just copy
        let filename = input.file_name().unwrap();
        fs::copy(input, output_dir.join(filename))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("Build failed: {0}")]
    BuildFailed(String),
    
    #[error("Visual script compilation failed: {0}")]
    ScriptCompilation(String),
}

#[derive(Serialize, Deserialize)]
struct AssetManifest {
    textures: Vec<(String, String)>,
    audio: Vec<(String, String)>,
}

fn to_rust_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
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

fn dir_size(path: impl AsRef<Path>) -> std::io::Result<u64> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size += dir_size(entry.path())?;
        } else {
            size += metadata.len();
        }
    }
    Ok(size)
}

// Public API for Tauri commands
pub async fn compile_project(
    project_path: &str,
    output_path: &str,
    target: BuildTarget,
) -> Result<BuildResult, CompilerError> {
    // Load project
    let project_data = fs::read(project_path)?;
    let project: Project = serde_json::from_slice(&project_data)
        .map_err(|e| CompilerError::ScriptCompilation(e.to_string()))?;
    
    // Compile
    let compiler = GameCompiler::new(project, target);
    compiler.compile().await
}