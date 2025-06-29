// src-tauri/engine/src/assets/loader.rs
use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait AssetLoader: Send + Sync {
    async fn load<T: Asset>(&self, data: &[u8]) -> Result<T, AssetError>;
}

pub trait Asset: Send + Sync + 'static {
    fn type_name() -> &'static str where Self: Sized;
}

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid asset path")]
    InvalidPath,
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Failed to decode asset: {0}")]
    DecodingError(String),
    
    #[error("Asset not found: {0}")]
    NotFound(String),
}

// Texture asset and loader
#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    R8,
}

impl Asset for Texture {
    fn type_name() -> &'static str {
        "Texture"
    }
}

pub struct TextureLoader;

#[async_trait]
impl AssetLoader for TextureLoader {
    async fn load<T: Asset>(&self, data: &[u8]) -> Result<T, AssetError> {
        // This is a hack to work around Rust's type system
        let texture = self.load_texture(data).await?;
        
        // SAFETY: We know T is Texture because of how the AssetManager calls this
        let any_texture = Box::new(texture) as Box<dyn Any>;
        match any_texture.downcast::<T>() {
            Ok(texture) => Ok(*texture),
            Err(_) => Err(AssetError::DecodingError("Type mismatch".to_string())),
        }
    }
}

impl TextureLoader {
    async fn load_texture(&self, data: &[u8]) -> Result<Texture, AssetError> {
        use image::GenericImageView;
        
        let img = image::load_from_memory(data)
            .map_err(|e| AssetError::DecodingError(e.to_string()))?;
        
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        
        Ok(Texture {
            width,
            height,
            format: TextureFormat::Rgba8,
            data: rgba.into_raw(),
        })
    }
}

// Audio asset and loader
pub struct AudioClip {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

impl Asset for AudioClip {
    fn type_name() -> &'static str {
        "AudioClip"
    }
}

pub struct AudioLoader;

#[async_trait]
impl AssetLoader for AudioLoader {
    async fn load<T: Asset>(&self, data: &[u8]) -> Result<T, AssetError> {
        // Simplified - in production you'd use rodio or similar
        let audio = AudioClip {
            sample_rate: 44100,
            channels: 2,
            samples: vec![],
        };
        
        let any_audio = Box::new(audio) as Box<dyn Any>;
        match any_audio.downcast::<T>() {
            Ok(audio) => Ok(*audio),
            Err(_) => Err(AssetError::DecodingError("Type mismatch".to_string())),
        }
    }
}

// JSON loader for data files
pub struct JsonAsset {
    pub data: serde_json::Value,
}

impl Asset for JsonAsset {
    fn type_name() -> &'static str {
        "JsonAsset"
    }
}

pub struct JsonLoader;

#[async_trait]
impl AssetLoader for JsonLoader {
    async fn load<T: Asset>(&self, data: &[u8]) -> Result<T, AssetError> {
        let json_str = std::str::from_utf8(data)
            .map_err(|e| AssetError::DecodingError(e.to_string()))?;
        
        let json_value = serde_json::from_str(json_str)
            .map_err(|e| AssetError::DecodingError(e.to_string()))?;
        
        let asset = JsonAsset { data: json_value };
        
        let any_asset = Box::new(asset) as Box<dyn Any>;
        match any_asset.downcast::<T>() {
            Ok(asset) => Ok(*asset),
            Err(_) => Err(AssetError::DecodingError("Type mismatch".to_string())),
        }
    }
}