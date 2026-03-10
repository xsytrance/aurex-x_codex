#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone)]
pub struct Transform2p5D {
    pub position: [f32; 3],
    pub rotation_yaw_pitch_roll: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for Transform2p5D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation_yaw_pitch_roll: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}
