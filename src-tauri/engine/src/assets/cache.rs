// src-tauri/engine/src/assets/cache.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::any::{Any, TypeId};

pub struct AssetCache {
    assets: HashMap<String, Arc<dyn Any + Send + Sync>>,
    type_map: HashMap<String, TypeId>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            type_map: HashMap::new(),
        }
    }
    
    pub fn insert<T: Asset>(&mut self, path: String, asset: T) -> AssetHandle<T> {
        let arc = Arc::new(asset);
        self.assets.insert(path.clone(), arc.clone() as Arc<dyn Any + Send + Sync>);
        self.type_map.insert(path.clone(), TypeId::of::<T>());
        
        AssetHandle {
            path,
            asset: arc,
        }
    }
    
    pub fn get<T: Asset>(&self, path: &str) -> Option<AssetHandle<T>> {
        // Check type matches
        let expected_type = TypeId::of::<T>();
        let actual_type = self.type_map.get(path)?;
        
        if expected_type != *actual_type {
            return None;
        }
        
        let asset = self.assets.get(path)?;
        let typed_asset = asset.clone()
            .downcast::<T>()
            .ok()?;
        
        Some(AssetHandle {
            path: path.to_string(),
            asset: typed_asset,
        })
    }
    
    pub fn remove(&mut self, path: &str) -> bool {
        self.type_map.remove(path);
        self.assets.remove(path).is_some()
    }
    
    pub fn clear(&mut self) {
        self.assets.clear();
        self.type_map.clear();
    }
    
    pub fn size(&self) -> usize {
        self.assets.len()
    }
}

#[derive(Clone)]
pub struct AssetHandle<T: Asset> {
    pub path: String,
    asset: Arc<T>,
}

impl<T: Asset> AssetHandle<T> {
    pub fn get(&self) -> &T {
        &self.asset
    }
}

impl<T: Asset> std::ops::Deref for AssetHandle<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.asset
    }
}