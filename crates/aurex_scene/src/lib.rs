pub mod geometry_nodes;

use geometry_nodes::GeometryPipeline;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub seed: Option<u64>,
    pub sdf_scene: SdfScene,
    #[serde(default)]
    pub audio_sync_bindings: AudioSyncBindings,
}

impl Scene {
    pub fn from_json_str(input: &str) -> serde_json::Result<Self> {
        serde_json::from_str(input)
    }

    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn derive_seed(&self, scope: &str) -> u64 {
        let mut state = self.seed.unwrap_or(0xA6E0_0001_u64);
        for &b in scope.as_bytes() {
            state = splitmix64(state ^ b as u64);
        }
        state
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SdfScene {
    pub root_node: GeometryPipeline,
    #[serde(default)]
    pub materials: Vec<SdfMaterial>,
    pub lighting: SdfLighting,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SdfMaterial {
    pub id: String,
    pub albedo: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    #[serde(default)]
    pub emissive: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SdfLighting {
    #[serde(default)]
    pub ambient: [f32; 3],
    #[serde(default)]
    pub key_lights: Vec<KeyLight>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyLight {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub jitter_seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AudioSyncBindings {
    #[serde(default)]
    pub reactive_parameters: Vec<AudioReactiveParameter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioReactiveParameter {
    pub node: String,
    pub parameter: String,
    pub audio_source: AudioSource,
    pub multiplier: f32,
    #[serde(default)]
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioSource {
    Kick,
    Snare,
    Bass,
    HiHat,
    Custom(String),
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_nodes::{GeometryModifierNode, ScalarExpr, SceneGeometryNode, Vec3Expr};

    fn sample_scene(seed: Option<u64>) -> Scene {
        Scene {
            name: "neon_tunnel".to_string(),
            seed,
            sdf_scene: SdfScene {
                root_node: GeometryPipeline {
                    base: SceneGeometryNode::Sphere {
                        radius: ScalarExpr::value(1.0),
                        seed: None,
                    },
                    modifiers: vec![
                        GeometryModifierNode::Repeat {
                            count: 16,
                            spacing: ScalarExpr::value(2.0),
                        },
                        GeometryModifierNode::Twist {
                            angle: ScalarExpr::Expression {
                                expression: "time*0.8".to_string(),
                            },
                        },
                        GeometryModifierNode::NoiseDisplacement {
                            strength: ScalarExpr::value(0.2),
                            frequency: ScalarExpr::value(1.4),
                            seed: Some(88),
                        },
                        GeometryModifierNode::Translate {
                            offset: Vec3Expr::splat(0.0),
                        },
                    ],
                },
                materials: vec![SdfMaterial {
                    id: "neon".to_string(),
                    albedo: [0.1, 0.9, 1.0],
                    roughness: 0.2,
                    metallic: 0.7,
                    emissive: [0.2, 1.0, 1.0],
                }],
                lighting: SdfLighting {
                    ambient: [0.05, 0.05, 0.07],
                    key_lights: vec![KeyLight {
                        direction: [-0.3, -1.0, 0.2],
                        color: [0.9, 0.7, 1.0],
                        intensity: 2.1,
                        jitter_seed: Some(13),
                    }],
                },
            },
            audio_sync_bindings: AudioSyncBindings {
                reactive_parameters: vec![AudioReactiveParameter {
                    node: "TunnelGenerator".to_string(),
                    parameter: "radius".to_string(),
                    audio_source: AudioSource::Bass,
                    multiplier: 1.5,
                    offset: 0.0,
                }],
            },
        }
    }

    #[test]
    fn scene_json_roundtrip() {
        let scene = sample_scene(Some(1337));
        let json = scene.to_json_pretty().expect("serialize scene");
        let parsed = Scene::from_json_str(&json).expect("parse scene");
        assert_eq!(scene, parsed);
    }

    #[test]
    fn seed_reproducibility() {
        let scene_a = sample_scene(Some(42));
        let scene_b = sample_scene(Some(42));
        let scene_c = sample_scene(Some(43));

        let scope = "noise:root";
        assert_eq!(scene_a.derive_seed(scope), scene_b.derive_seed(scope));
        assert_ne!(scene_a.derive_seed(scope), scene_c.derive_seed(scope));
    }

    #[test]
    fn pipeline_evaluation_order_is_stable() {
        let scene = sample_scene(Some(7));
        let order = scene.sdf_scene.root_node.evaluation_order();
        assert_eq!(
            order,
            vec![
                "base:sphere".to_string(),
                "modifier:repeat".to_string(),
                "modifier:twist".to_string(),
                "modifier:noise_displacement".to_string(),
                "modifier:translate".to_string(),
            ]
        );
    }
}
