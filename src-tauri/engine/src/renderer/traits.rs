// src-tauri/engine/src/renderer/traits.rs
use crate::math::{Transform, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use crate::ecs::Component;

pub trait Renderer: Send + Sync {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn clear(&mut self, color: [f32; 4]);
    
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform, interpolation: f32);
    fn draw_rect(&mut self, position: Vec2, size: Vec2, color: [f32; 4]);
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: [f32; 4], width: f32);
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: [f32; 4]);
    
    fn set_camera(&mut self, position: Vec2, zoom: f32);
    fn screen_to_world(&self, screen_pos: Vec2) -> Vec2;
    fn world_to_screen(&self, world_pos: Vec2) -> Vec2;
    
    fn get_frame_data(&self) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub texture_id: String,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
    pub source_rect: Option<Rect>,
    pub pivot: Vec2,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_id: String::new(),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            source_rect: None,
            pivot: Vec2::new(0.5, 0.5),
        }
    }
}

impl Component for Sprite {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

pub fn create_renderer(backend: RendererBackend) -> Result<Box<dyn Renderer>, RendererError> {
    match backend {
        RendererBackend::Canvas => Ok(Box::new(CanvasRenderer::new())),
        RendererBackend::Wgpu => {
            // For now, fall back to canvas renderer
            // WGPU implementation would be added later for native performance
            Ok(Box::new(CanvasRenderer::new()))
        }
    }
}

pub enum RendererBackend {
    Canvas,
    Wgpu,
}

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Failed to initialize renderer: {0}")]
    InitializationError(String),
    
    #[error("Texture not found: {0}")]
    TextureNotFound(String),
}