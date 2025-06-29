// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Add this to src-tauri/src/lib.rs (merge with existing content)

use dream_engine::tauri_integration::{
    create_preview_engine,
    update_preview_scene,
    render_preview_frame,
    destroy_preview_engine,
    compile_visual_script,
};

// Update the main function to include engine commands
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Engine preview commands
            create_preview_engine,
            update_preview_scene,
            render_preview_frame,
            destroy_preview_engine,
            compile_visual_script,
            // Project management
            create_project,
            load_project,
            save_project,
            // Asset management
            import_asset,
            get_project_assets,
            // Build commands
            build_game,
            export_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Additional commands for project management
#[tauri::command]
async fn create_project(name: String, engine_type: String) -> Result<String, String> {
    use std::fs;
    use std::path::PathBuf;
    
    let project_id = uuid::Uuid::new_v4().to_string();
    let projects_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or("Failed to get app data directory")?
        .join("projects");
    
    fs::create_dir_all(&projects_dir)
        .map_err(|e| format!("Failed to create projects directory: {}", e))?;
    
    let project_dir = projects_dir.join(&project_id);
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;
    
    // Create project structure
    fs::create_dir_all(project_dir.join("assets"))
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
    fs::create_dir_all(project_dir.join("scripts"))
        .map_err(|e| format!("Failed to create scripts directory: {}", e))?;
    fs::create_dir_all(project_dir.join("scenes"))
        .map_err(|e| format!("Failed to create scenes directory: {}", e))?;
    
    // Create project metadata
    let project_meta = serde_json::json!({
        "id": project_id,
        "name": name,
        "engineType": engine_type,
        "version": "0.1.0",
        "createdAt": chrono::Utc::now().to_rfc3339(),
    });
    
    fs::write(
        project_dir.join("project.json"),
        serde_json::to_string_pretty(&project_meta).unwrap()
    ).map_err(|e| format!("Failed to write project metadata: {}", e))?;
    
    Ok(project_id)
}

#[tauri::command]
async fn load_project(project_id: String) -> Result<serde_json::Value, String> {
    use std::fs;
    
    let projects_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or("Failed to get app data directory")?
        .join("projects");
    
    let project_file = projects_dir.join(&project_id).join("project.json");
    
    let content = fs::read_to_string(project_file)
        .map_err(|e| format!("Failed to read project file: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project file: {}", e))
}

#[tauri::command]
async fn save_project(project_id: String, data: serde_json::Value) -> Result<(), String> {
    use std::fs;
    
    let projects_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or("Failed to get app data directory")?
        .join("projects");
    
    let project_file = projects_dir.join(&project_id).join("project.json");
    
    fs::write(
        project_file,
        serde_json::to_string_pretty(&data).unwrap()
    ).map_err(|e| format!("Failed to save project: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn import_asset(project_id: String, asset_path: String, asset_type: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    let source_path = Path::new(&asset_path);
    if !source_path.exists() {
        return Err("Asset file not found".to_string());
    }
    
    let asset_id = uuid::Uuid::new_v4().to_string();
    let file_name = source_path.file_name()
        .ok_or("Invalid file path")?
        .to_string_lossy();
    
    let projects_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or("Failed to get app data directory")?
        .join("projects");
    
    let asset_dir = projects_dir
        .join(&project_id)
        .join("assets")
        .join(&asset_type);
    
    fs::create_dir_all(&asset_dir)
        .map_err(|e| format!("Failed to create asset directory: {}", e))?;
    
    let dest_path = asset_dir.join(&file_name);
    
    fs::copy(source_path, &dest_path)
        .map_err(|e| format!("Failed to copy asset: {}", e))?;
    
    // Create asset metadata
    let asset_meta = serde_json::json!({
        "id": asset_id,
        "name": file_name,
        "type": asset_type,
        "path": dest_path.to_string_lossy(),
        "importedAt": chrono::Utc::now().to_rfc3339(),
    });
    
    let meta_path = asset_dir.join(format!("{}.meta", file_name));
    fs::write(meta_path, serde_json::to_string_pretty(&asset_meta).unwrap())
        .map_err(|e| format!("Failed to write asset metadata: {}", e))?;
    
    Ok(asset_id)
}

#[tauri::command]
async fn get_project_assets(project_id: String) -> Result<Vec<serde_json::Value>, String> {
    use std::fs;
    use walkdir::WalkDir;
    
    let projects_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or("Failed to get app data directory")?
        .join("projects");
    
    let assets_dir = projects_dir.join(&project_id).join("assets");
    let mut assets = Vec::new();
    
    for entry in WalkDir::new(&assets_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "meta"))
    {
        let content = fs::read_to_string(entry.path())
            .map_err(|e| format!("Failed to read asset metadata: {}", e))?;
        
        if let Ok(asset_meta) = serde_json::from_str(&content) {
            assets.push(asset_meta);
        }
    }
    
    Ok(assets)
}

#[tauri::command]
async fn build_game(project_id: String, target: String) -> Result<String, String> {
    use dream_engine::compiler::{GameCompiler, BuildTarget};
    
    // Load project
    let project_data = load_project(project_id.clone()).await?;
    
    // Convert to engine Project type
    let project: dream_engine::Project = serde_json::from_value(project_data)
        .map_err(|e| format!("Failed to parse project: {}", e))?;
    
    // Determine build target
    let build_target = match target.as_str() {
        "native" => BuildTarget::Native,
        "web" => BuildTarget::WebAssembly,
        "windows" => BuildTarget::Windows,
        "linux" => BuildTarget::Linux,
        "macos" => BuildTarget::MacOS,
        _ => return Err("Invalid build target".to_string()),
    };
    
    // Compile the game
    let compiler = GameCompiler::new(project, build_target);
    let result = compiler.compile().await
        .map_err(|e| format!("Build failed: {}", e))?;
    
    Ok(serde_json::json!({
        "executable": result.executable_path,
        "assets": result.assets_path,
        "size": result.size_bytes,
        "warnings": result.warnings,
    }).to_string())
}

#[tauri::command]
async fn export_game(project_id: String, output_path: String) -> Result<(), String> {
    // Build the game first
    let build_result = build_game(project_id, "native".to_string()).await?;
    
    // Copy to output location
    use std::fs;
    let result: serde_json::Value = serde_json::from_str(&build_result)
        .map_err(|e| format!("Failed to parse build result: {}", e))?;
    
    let exe_path = result["executable"].as_str()
        .ok_or("Invalid executable path")?;
    
    fs::copy(exe_path, &output_path)
        .map_err(|e| format!("Failed to export game: {}", e))?;
    
    Ok(())
}

// Also add to Cargo.toml dependencies:
