// src-tauri/engine/src/math/quaternion.rs
use super::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let sin = half_angle.sin();
        let axis = axis.normalize();
        
        Self {
            x: axis.x * sin,
            y: axis.y * sin,
            z: axis.z * sin,
            w: half_angle.cos(),
        }
    }
    
    pub fn from_rotation_z(angle: f32) -> Self {
        let half_angle = angle * 0.5;
        Self {
            x: 0.0,
            y: 0.0,
            z: half_angle.sin(),
            w: half_angle.cos(),
        }
    }
    
    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        let half_pitch = pitch * 0.5;
        let half_yaw = yaw * 0.5;
        let half_roll = roll * 0.5;
        
        let cp = half_pitch.cos();
        let sp = half_pitch.sin();
        let cy = half_yaw.cos();
        let sy = half_yaw.sin();
        let cr = half_roll.cos();
        let sr = half_roll.sin();
        
        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }
    
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
    
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }
    
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }
    
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            Self::IDENTITY
        }
    }
    
    pub fn conjugate(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }
    
    pub fn inverse(self) -> Self {
        self.conjugate() / self.length_squared()
    }
    
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.dot(other);
        
        // If the dot product is negative, negate one quaternion
        let mut other = other;
        if dot < 0.0 {
            other = -other;
            dot = -dot;
        }
        
        // If quaternions are nearly identical, use linear interpolation
        if dot > 0.9995 {
            return Self {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            }.normalize();
        }
        
        // Spherical interpolation
        let theta = dot.acos();
        let sin_theta = theta.sin();
        let scale_self = ((1.0 - t) * theta).sin() / sin_theta;
        let scale_other = (t * theta).sin() / sin_theta;
        
        Self {
            x: scale_self * self.x + scale_other * other.x,
            y: scale_self * self.y + scale_other * other.y,
            z: scale_self * self.z + scale_other * other.z,
            w: scale_self * self.w + scale_other * other.w,
        }
    }
    
    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let uv = qv.cross(v);
        let uuv = qv.cross(uv);
        v + ((uv * self.w) + uuv) * 2.0
    }
}

impl std::ops::Mul for Quat {
    type Output = Self;
    
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }
}

impl std::ops::Neg for Quat {
    type Output = Self;
    
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl std::ops::Div<f32> for Quat {
    type Output = Self;
    
    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        }
    }
}
