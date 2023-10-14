use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub mass: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub z_vel: f32,
    pub bright: f32
}
