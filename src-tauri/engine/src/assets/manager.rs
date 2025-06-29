// src-tauri/engine/src/assets/manager.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{AssetLoader, AssetCache, Asset, AssetHandle};

pub struct AssetManager {
    loaders: HashMap<String, Box<dyn AssetLoader>>,
    cache: Arc<RwLock<AssetCache>>,
    base_path: PathBuf,
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let mut manager = Self {
            loaders: HashMap::new(),
            cache: Arc::new(RwLock::new(AssetCache::new())),
            base_path: base_path.as_ref().to_path_buf(),
        };
        
        // Register default loaders
        manager.register_loader("png", Box::new(TextureLoader));
        manager.register_loader("jpg", Box::new(TextureLoader));
        manager.register_loader("jpeg", Box::new(TextureLoader));
        manager.register_loader("ogg", Box::new(AudioLoader));
        manager.register_loader("wav", Box::new(AudioLoader));
        manager.register_loader("json", Box::new(JsonLoader));
        
        manager
    }
    
    pub fn register_loader(&mut self, extension: &str, loader: Box<dyn AssetLoader>) {
        self.loaders.insert(extension.to_lowercase(), loader);
    }
    
    pub async fn load<T: Asset>(&self, path: &str) -> Result<AssetHandle<T>, AssetError> {
        // Check cache first
        let cache = self.cache.read().await;
        if let Some(handle) = cache.get::<T>(path) {
            return Ok(handle);
        }
        drop(cache);
        
        // Load asset
        let full_path = self.base_path.join(path);
        let extension = full_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or(AssetError::InvalidPath)?;
        
        let loader = self.loaders.get(extension)
            .ok_or(AssetError::UnsupportedFormat(extension.to_string()))?;
        
        let data = tokio::fs::read(&full_path).await
            .map_err(|e| AssetError::Io(e))?;
        
        let asset = loader.load::<T>(&data).await?;
        
        // Cache the asset
        let mut cache = self.cache.write().await;
        let handle = cache.insert(path.to_string(), asset);
        
        Ok(handle)
    }
    
    pub async fn load_batch<T: Asset>(&self, paths: &[&str]) -> Result<Vec<AssetHandle<T>>, AssetError> {
        let mut handles = Vec::with_capacity(paths.len());
        
        for path in paths {
            handles.push(self.load::<T>(path).await?);
        }
        
        Ok(handles)
    }
    
    pub async fn preload_directory(&self, dir: &str) -> Result<usize, AssetError> {
        use tokio::fs;
        use tokio_stream::{StreamExt, wrappers::ReadDirStream};
        
        let full_dir = self.base_path.join(dir);
        let mut count = 0;
        
        let mut entries = ReadDirStream::new(fs::read_dir(full_dir).await?);
        
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(rel_path) = path.strip_prefix(&self.base_path).ok() {
                    let path_str = rel_path.to_string_lossy();
                    
                    // Determine asset type based on extension
                    match path.extension().and_then(|e| e.to_str()) {
                        Some("png") | Some("jpg") | Some("jpeg") => {
                            self.load::<Texture>(&path_str).await.ok();
                        }
                        Some("ogg") | Some("wav") => {
                            self.load::<AudioClip>(&path_str).await.ok();
                        }
                        _ => continue,
                    }
                    
                    count += 1;
                }
            }
        }
        
        Ok(count)
    }
    
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    pub async fn get_cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.size()
    }
}