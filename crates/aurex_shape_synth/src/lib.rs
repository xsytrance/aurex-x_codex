#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
    Circle,
    Polygon,
    Ring,
    Tube,
}

#[derive(Debug, Clone)]
pub struct ShapeDescriptor {
    pub primitive_type: PrimitiveType,
    pub seed: u64,
}
