use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scene {
    pub sdf: SdfScene,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfScene {
    pub camera: SdfCamera,
    pub lighting: SdfLighting,
    #[serde(default)]
    pub seed: u32,
    #[serde(default)]
    pub objects: Vec<SdfObject>,
    #[serde(default)]
    pub root: SdfNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfObject {
    pub primitive: SdfPrimitive,
    #[serde(default)]
    pub modifiers: Vec<SdfModifier>,
    #[serde(default)]
    pub material: SdfMaterial,
    #[serde(default)]
    pub bounds_radius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SdfNode {
    Empty,
    Primitive {
        object: SdfObject,
    },
    Group {
        children: Vec<SdfNode>,
    },
    Transform {
        modifiers: Vec<SdfModifier>,
        child: Box<SdfNode>,
        #[serde(default)]
        bounds_radius: Option<f32>,
    },
    Union {
        children: Vec<SdfNode>,
    },
    SmoothUnion {
        children: Vec<SdfNode>,
        k: f32,
    },
    Subtract {
        base: Box<SdfNode>,
        subtract: Vec<SdfNode>,
    },
    Intersect {
        children: Vec<SdfNode>,
    },
    Blend {
        children: Vec<SdfNode>,
        #[serde(default)]
        weights: Vec<f32>,
    },
}

impl Default for SdfNode {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SdfPrimitive {
    Sphere {
        radius: f32,
    },
    Box {
        size: Vec3,
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
    Plane {
        normal: Vec3,
        offset: f32,
    },
    Cylinder {
        radius: f32,
        half_height: f32,
    },
    Capsule {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    Mandelbulb {
        power: f32,
        iterations: u32,
        bailout: f32,
    },
    NoiseField {
        radius: f32,
        amplitude: f32,
        frequency: f32,
        seed: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SdfModifier {
    Repeat {
        cell: Vec3,
    },
    Twist {
        strength: f32,
    },
    Bend {
        strength: f32,
    },
    Scale {
        factor: f32,
    },
    Rotate {
        axis: Vec3,
        radians: f32,
    },
    Translate {
        offset: Vec3,
    },
    NoiseDisplacement {
        amplitude: f32,
        frequency: f32,
        seed: u32,
    },
    Mirror {
        normal: Vec3,
        offset: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub fov_degrees: f32,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfLighting {
    pub ambient_light: f32,
    #[serde(default)]
    pub key_lights: Vec<KeyLight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyLight {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SdfMaterialType {
    SolidColor,
    NeonGrid,
    Plasma,
    FractalMetal,
    NoiseSurface,
    Holographic,
    Lava,
    Wireframe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SdfPattern {
    None,
    Bands,
    Rings,
    Checker,
    Noise,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfMaterial {
    #[serde(default)]
    pub material_type: SdfMaterialType,
    #[serde(default = "default_base_color", alias = "color")]
    pub base_color: Vec3,
    #[serde(default)]
    pub emissive_strength: f32,
    #[serde(default = "default_roughness")]
    pub roughness: f32,
    #[serde(default)]
    pub pattern: SdfPattern,
    #[serde(default)]
    pub parameters: BTreeMap<String, f32>,
}

fn default_base_color() -> Vec3 {
    Vec3::new(0.75, 0.85, 1.0)
}

fn default_roughness() -> f32 {
    0.45
}

impl Default for SdfMaterialType {
    fn default() -> Self {
        Self::SolidColor
    }
}

impl Default for SdfPattern {
    fn default() -> Self {
        Self::None
    }
}

impl Default for SdfMaterial {
    fn default() -> Self {
        Self {
            material_type: SdfMaterialType::SolidColor,
            base_color: default_base_color(),
            emissive_strength: 0.0,
            roughness: default_roughness(),
            pattern: SdfPattern::None,
            parameters: BTreeMap::new(),
        }
    }
}

pub fn load_scene_from_json_str(contents: &str) -> Result<Scene, serde_json::Error> {
    serde_json::from_str(contents)
}

pub fn load_scene_from_json_path(
    path: impl AsRef<Path>,
) -> Result<Scene, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    Ok(load_scene_from_json_str(&data)?)
}

#[cfg(test)]
mod tests {
    use super::{Scene, SdfMaterialType, SdfNode, load_scene_from_json_str};

    #[test]
    fn parses_scene_json() {
        let json = r#"{
            "sdf": {
                "seed": 101,
                "camera": {
                    "position": {"x": 0.0, "y": 0.0, "z": -5.0},
                    "target": {"x": 0.0, "y": 0.0, "z": 0.0},
                    "fov_degrees": 60.0,
                    "aspect_ratio": 1.7777
                },
                "lighting": {
                    "ambient_light": 0.2,
                    "key_lights": []
                },
                "root": {
                    "Union": {
                        "children": [
                            {"Primitive": {"object": {"primitive": {"Sphere": {"radius": 1.0}}}}},
                            {"Transform": {
                                "modifiers": [{"Translate": {"offset": {"x": 1.3, "y": 0.0, "z": 0.0}}}],
                                "child": {"Primitive": {"object": {
                                    "primitive": {"Box": {"size": {"x": 0.5, "y": 0.5, "z": 0.5}}},
                                    "material": {"material_type": "NeonGrid"}
                                }}}
                            }}
                        ]
                    }
                }
            }
        }"#;

        let scene: Scene = load_scene_from_json_str(json).expect("scene json should parse");
        assert_eq!(scene.sdf.seed, 101);
        match scene.sdf.root {
            SdfNode::Union { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("expected union root"),
        }
    }

    #[test]
    fn old_material_field_alias_still_parses() {
        let json = r#"{
            "sdf": {
                "camera": {
                    "position": {"x": 0.0, "y": 0.0, "z": -5.0},
                    "target": {"x": 0.0, "y": 0.0, "z": 0.0},
                    "fov_degrees": 60.0,
                    "aspect_ratio": 1.7777
                },
                "lighting": {"ambient_light": 0.2},
                "objects": [{
                    "primitive": {"Sphere": {"radius": 1.0}},
                    "material": {
                        "material_type": "Plasma",
                        "color": {"x": 1.0, "y": 0.2, "z": 0.8}
                    }
                }]
            }
        }"#;
        let scene: Scene = load_scene_from_json_str(json).expect("scene json should parse");
        assert_eq!(
            scene.sdf.objects[0].material.material_type,
            SdfMaterialType::Plasma
        );
        assert_eq!(scene.sdf.objects[0].material.base_color.x, 1.0);
    }
}
