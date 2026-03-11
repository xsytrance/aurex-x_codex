use serde::{Deserialize, Serialize};
use std::path::Path;

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
    pub objects: Vec<SdfObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfObject {
    pub primitive: SdfPrimitive,
    #[serde(default)]
    pub modifiers: Vec<SdfModifier>,
    #[serde(default)]
    pub material: SdfMaterial,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfMaterial {
    #[serde(default = "default_color")]
    pub color: Vec3,
}

fn default_color() -> Vec3 {
    Vec3::new(0.75, 0.85, 1.0)
}

impl Default for SdfMaterial {
    fn default() -> Self {
        Self {
            color: default_color(),
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
    use super::{Scene, load_scene_from_json_str};

    #[test]
    fn parses_scene_json() {
        let json = r#"{
            "sdf": {
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
                "objects": [{
                    "primitive": {"Sphere": {"radius": 1.0}},
                    "modifiers": []
                }]
            }
        }"#;

        let scene: Scene = load_scene_from_json_str(json).expect("scene json should parse");
        assert_eq!(scene.sdf.objects.len(), 1);
    }
}
