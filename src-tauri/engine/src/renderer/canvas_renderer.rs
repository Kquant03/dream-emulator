// src-tauri/engine/src/renderer/canvas_renderer.rs
use super::{Renderer, Sprite, RendererError};
use crate::math::{Transform, Vec2};
use std::collections::HashMap;

pub struct CanvasRenderer {
    frame_data: Vec<DrawCommand>,
    camera_position: Vec2,
    camera_zoom: f32,
    viewport_size: Vec2,
}

#[derive(Clone, Debug)]
enum DrawCommand {
    Clear { color: [f32; 4] },
    DrawSprite {
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        texture_id: String,
        color: [f32; 4],
        flip_x: bool,
        flip_y: bool,
    },
    DrawRect {
        position: Vec2,
        size: Vec2,
        color: [f32; 4],
    },
    DrawLine {
        start: Vec2,
        end: Vec2,
        color: [f32; 4],
        width: f32,
    },
    DrawCircle {
        center: Vec2,
        radius: f32,
        color: [f32; 4],
    },
}

impl CanvasRenderer {
    pub fn new() -> Self {
        Self {
            frame_data: Vec::with_capacity(1000),
            camera_position: Vec2::ZERO,
            camera_zoom: 1.0,
            viewport_size: Vec2::new(800.0, 600.0),
        }
    }
}

impl Renderer for CanvasRenderer {
    fn begin_frame(&mut self) {
        self.frame_data.clear();
    }
    
    fn end_frame(&mut self) {
        // Frame data is ready to be sent to the frontend
    }
    
    fn clear(&mut self, color: [f32; 4]) {
        self.frame_data.push(DrawCommand::Clear { color });
    }
    
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform, interpolation: f32) {
        // Convert 3D transform to 2D for top-down view
        let position = transform.position.xy();
        let scale = transform.scale.xy();
        
        // For 2D, we only care about Z rotation
        let rotation = transform.rotation.z.atan2(transform.rotation.w) * 2.0;
        
        self.frame_data.push(DrawCommand::DrawSprite {
            position,
            rotation,
            scale,
            texture_id: sprite.texture_id.clone(),
            color: sprite.color,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
        });
    }
    
    fn draw_rect(&mut self, position: Vec2, size: Vec2, color: [f32; 4]) {
        self.frame_data.push(DrawCommand::DrawRect {
            position,
            size,
            color,
        });
    }
    
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: [f32; 4], width: f32) {
        self.frame_data.push(DrawCommand::DrawLine {
            start,
            end,
            color,
            width,
        });
    }
    
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: [f32; 4]) {
        self.frame_data.push(DrawCommand::DrawCircle {
            center,
            radius,
            color,
        });
    }
    
    fn set_camera(&mut self, position: Vec2, zoom: f32) {
        self.camera_position = position;
        self.camera_zoom = zoom;
    }
    
    fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let centered = screen_pos - self.viewport_size * 0.5;
        let scaled = centered / self.camera_zoom;
        scaled + self.camera_position
    }
    
    fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let relative = world_pos - self.camera_position;
        let scaled = relative * self.camera_zoom;
        scaled + self.viewport_size * 0.5
    }
    
    fn get_frame_data(&self) -> Option<Vec<u8>> {
        // Serialize frame data to send to frontend
        // In a real implementation, you'd use a more efficient format
        serde_json::to_vec(&self.frame_data).ok()
    }
}
