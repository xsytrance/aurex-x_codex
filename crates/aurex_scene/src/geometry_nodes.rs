use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScalarExpr {
    Value { value: f32 },
    Expression { expression: String },
}

impl ScalarExpr {
    pub fn value(value: f32) -> Self {
        Self::Value { value }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vec3Expr {
    pub x: ScalarExpr,
    pub y: ScalarExpr,
    pub z: ScalarExpr,
}

impl Vec3Expr {
    pub fn splat(value: f32) -> Self {
        Self {
            x: ScalarExpr::value(value),
            y: ScalarExpr::value(value),
            z: ScalarExpr::value(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SceneGeometryNode {
    Sphere {
        radius: ScalarExpr,
        seed: Option<u64>,
    },
    Torus {
        major_radius: ScalarExpr,
        minor_radius: ScalarExpr,
        seed: Option<u64>,
    },
    Box {
        size: Vec3Expr,
        roundness: ScalarExpr,
        seed: Option<u64>,
    },
    Plane {
        normal: Vec3Expr,
        offset: ScalarExpr,
        seed: Option<u64>,
    },
    Cylinder {
        radius: ScalarExpr,
        height: ScalarExpr,
        seed: Option<u64>,
    },
    Capsule {
        radius: ScalarExpr,
        height: ScalarExpr,
        seed: Option<u64>,
    },
    FractalMandelbulb {
        power: ScalarExpr,
        iterations: u32,
        bailout: ScalarExpr,
        scale: ScalarExpr,
        seed: Option<u64>,
    },
    NoiseField {
        scale: ScalarExpr,
        amplitude: ScalarExpr,
        octaves: u32,
        seed: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeometryModifierNode {
    Repeat {
        count: u32,
        spacing: ScalarExpr,
    },
    Twist {
        angle: ScalarExpr,
    },
    Bend {
        amount: ScalarExpr,
    },
    Scale {
        scale: Vec3Expr,
    },
    Rotate {
        axis: Vec3Expr,
        angle: ScalarExpr,
    },
    Translate {
        offset: Vec3Expr,
    },
    NoiseDisplacement {
        strength: ScalarExpr,
        frequency: ScalarExpr,
        seed: Option<u64>,
    },
    Mirror {
        normal: Vec3Expr,
        offset: ScalarExpr,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryPipeline {
    pub base: SceneGeometryNode,
    #[serde(default)]
    pub modifiers: Vec<GeometryModifierNode>,
}

impl GeometryPipeline {
    pub fn evaluation_order(&self) -> Vec<String> {
        let mut order = Vec::with_capacity(self.modifiers.len() + 1);
        order.push(format!("base:{}", node_type_name(&self.base)));
        for modifier in &self.modifiers {
            order.push(format!("modifier:{}", modifier_type_name(modifier)));
        }
        order
    }
}

fn node_type_name(node: &SceneGeometryNode) -> &'static str {
    match node {
        SceneGeometryNode::Sphere { .. } => "sphere",
        SceneGeometryNode::Torus { .. } => "torus",
        SceneGeometryNode::Box { .. } => "box",
        SceneGeometryNode::Plane { .. } => "plane",
        SceneGeometryNode::Cylinder { .. } => "cylinder",
        SceneGeometryNode::Capsule { .. } => "capsule",
        SceneGeometryNode::FractalMandelbulb { .. } => "fractal_mandelbulb",
        SceneGeometryNode::NoiseField { .. } => "noise_field",
    }
}

fn modifier_type_name(node: &GeometryModifierNode) -> &'static str {
    match node {
        GeometryModifierNode::Repeat { .. } => "repeat",
        GeometryModifierNode::Twist { .. } => "twist",
        GeometryModifierNode::Bend { .. } => "bend",
        GeometryModifierNode::Scale { .. } => "scale",
        GeometryModifierNode::Rotate { .. } => "rotate",
        GeometryModifierNode::Translate { .. } => "translate",
        GeometryModifierNode::NoiseDisplacement { .. } => "noise_displacement",
        GeometryModifierNode::Mirror { .. } => "mirror",
    }
}
