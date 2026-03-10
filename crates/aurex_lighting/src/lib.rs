#[derive(Debug, Clone, Copy)]
pub enum LightKind {
    Ambient,
    Point,
    Spot,
    Pulse,
}

#[derive(Debug, Clone)]
pub struct LightDescriptor {
    pub kind: LightKind,
    pub intensity: f32,
    pub color_rgb: [f32; 3],
}
