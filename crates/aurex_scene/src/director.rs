use serde::{Deserialize, Serialize};

use crate::{
    Scene, SdfNode, SdfPrimitive, Vec3,
    camera::{
        BezierPathCamera, CameraRig, FlythroughCamera, OrbitCamera, RhythmCamera, RhythmSync,
        estimate_framing_distance,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shot {
    pub start: f32,
    pub end: f32,
    pub camera: CameraRig,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ShotSequence {
    pub shots: Vec<Shot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraDirector {
    #[serde(default = "default_tempo")]
    pub tempo_bpm: f32,
    #[serde(default = "default_phrase")]
    pub phrase_length_beats: f32,
}

impl Default for CameraDirector {
    fn default() -> Self {
        Self {
            tempo_bpm: default_tempo(),
            phrase_length_beats: default_phrase(),
        }
    }
}

impl CameraDirector {
    pub fn generate(&self, scene: &Scene, duration: f32) -> ShotSequence {
        let scale = estimate_scene_scale(scene).max(2.0);
        let has_tunnel =
            scene_primitive_count(&scene.sdf.root, |p| matches!(p, SdfPrimitive::Torus { .. })) > 0;
        let has_particles = scene_primitive_count(&scene.sdf.root, |p| {
            matches!(p, SdfPrimitive::NoiseField { .. })
        }) > 0;
        let pattern_density = scene.sdf.patterns.len() as f32
            + scene
                .sdf
                .objects
                .iter()
                .filter(|o| o.material.pattern_network.is_some())
                .count() as f32;

        let mut seq = ShotSequence::default();
        let phrase_time = (60.0 / self.tempo_bpm.max(1.0)) * self.phrase_length_beats;
        let shot_len = (phrase_time * 0.5).clamp(2.0, 6.0);
        let mut t = 0.0;
        let mut idx = 0u32;

        while t < duration {
            let end = (t + shot_len).min(duration);
            let camera = if has_tunnel {
                CameraRig::Flythrough(FlythroughCamera {
                    start: Vec3::new(0.0, scale * 0.12, -estimate_framing_distance(scale, 58.0)),
                    end: Vec3::new(0.0, scale * 0.12, estimate_framing_distance(scale, 58.0)),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 58.0,
                    roll: 0.0,
                    rhythm: RhythmSync {
                        tempo_sync: 1.0,
                        beat_shake: 0.7,
                    },
                })
            } else if has_particles {
                CameraRig::BezierPath(BezierPathCamera {
                    control_points: vec![
                        Vec3::new(-scale * 0.8, scale * 0.3, -scale * 0.9),
                        Vec3::new(scale * 0.4, scale * 0.5, -scale * 0.3),
                        Vec3::new(0.0, scale * 0.2, scale * 0.6),
                    ],
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 62.0,
                    roll: 0.0,
                    rhythm: RhythmSync {
                        tempo_sync: 0.6,
                        beat_shake: 0.25,
                    },
                })
            } else if pattern_density > 1.0 {
                CameraRig::Orbit(OrbitCamera {
                    center: Vec3::new(0.0, 0.0, 0.0),
                    radius: estimate_framing_distance(scale, 45.0),
                    speed: 0.35,
                    height: scale * 0.2,
                    fov_degrees: 45.0,
                    roll: 0.0,
                    rhythm: RhythmSync {
                        tempo_sync: 0.5,
                        beat_shake: 0.2,
                    },
                })
            } else {
                CameraRig::Rhythm(RhythmCamera {
                    position: Vec3::new(0.0, scale * 0.25, -estimate_framing_distance(scale, 52.0)),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 52.0,
                    roll: 0.0,
                    tempo_sync: 0.8,
                })
            };

            seq.shots.push(Shot {
                start: t,
                end,
                camera,
                label: format!("shot_{}", idx),
            });
            idx += 1;
            t = end;
        }

        seq
    }

    pub fn shot_for_time<'a>(&self, sequence: &'a ShotSequence, time: f32) -> Option<&'a Shot> {
        sequence
            .shots
            .iter()
            .find(|s| time >= s.start && time <= s.end)
    }
}

fn default_tempo() -> f32 {
    132.0
}
fn default_phrase() -> f32 {
    8.0
}

fn scene_primitive_count<F: Fn(&SdfPrimitive) -> bool>(node: &SdfNode, pred: F) -> usize {
    fn walk<F: Fn(&SdfPrimitive) -> bool>(node: &SdfNode, pred: &F, count: &mut usize) {
        match node {
            SdfNode::Primitive { object } => {
                if pred(&object.primitive) {
                    *count += 1;
                }
            }
            SdfNode::Group { children }
            | SdfNode::Union { children }
            | SdfNode::SmoothUnion { children, .. }
            | SdfNode::Intersect { children }
            | SdfNode::Blend { children, .. } => {
                for child in children {
                    walk(child, pred, count);
                }
            }
            SdfNode::Transform { child, .. } => walk(child, pred, count),
            SdfNode::Subtract { base, subtract } => {
                walk(base, pred, count);
                for child in subtract {
                    walk(child, pred, count);
                }
            }
            SdfNode::Empty => {}
        }
    }

    let mut count = 0;
    walk(node, &pred, &mut count);
    count
}

pub fn estimate_scene_scale(scene: &Scene) -> f32 {
    let mut max_radius = 1.0_f32;
    for o in &scene.sdf.objects {
        max_radius = max_radius.max(o.bounds_radius.unwrap_or(primitive_radius(&o.primitive)));
    }
    max_radius.max(node_radius(&scene.sdf.root))
}

fn node_radius(node: &SdfNode) -> f32 {
    match node {
        SdfNode::Primitive { object } => object
            .bounds_radius
            .unwrap_or(primitive_radius(&object.primitive)),
        SdfNode::Group { children }
        | SdfNode::Union { children }
        | SdfNode::SmoothUnion { children, .. }
        | SdfNode::Intersect { children }
        | SdfNode::Blend { children, .. } => {
            children.iter().map(node_radius).fold(1.0_f32, f32::max)
        }
        SdfNode::Transform {
            bounds_radius,
            child,
            ..
        } => bounds_radius.unwrap_or_else(|| node_radius(child)),
        SdfNode::Subtract { base, subtract } => {
            let mut r = node_radius(base);
            for c in subtract {
                r = r.max(node_radius(c));
            }
            r
        }
        SdfNode::Empty => 1.0,
    }
}

fn primitive_radius(primitive: &SdfPrimitive) -> f32 {
    match primitive {
        SdfPrimitive::Sphere { radius } => *radius,
        SdfPrimitive::Box { size } => (size.x * size.x + size.y * size.y + size.z * size.z).sqrt(),
        SdfPrimitive::Torus {
            major_radius,
            minor_radius,
        } => major_radius + minor_radius,
        SdfPrimitive::Plane { .. } => 10.0,
        SdfPrimitive::Cylinder {
            radius,
            half_height,
        } => (radius * radius + half_height * half_height).sqrt(),
        SdfPrimitive::Capsule { a, b, radius } => {
            let d = Vec3::new(b.x - a.x, b.y - a.y, b.z - a.z);
            (d.x * d.x + d.y * d.y + d.z * d.z).sqrt() + radius
        }
        SdfPrimitive::Mandelbulb { bailout, .. } => *bailout,
        SdfPrimitive::NoiseField {
            radius, amplitude, ..
        } => radius + amplitude.abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SdfCamera, SdfLighting, SdfScene};

    #[test]
    fn director_generation_is_deterministic() {
        let scene = Scene {
            sdf: SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 0.0, -6.0),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 1.77,
                },
                lighting: SdfLighting {
                    ambient_light: 0.2,
                    key_lights: vec![],
                    fog_color: Vec3::new(0.0, 0.0, 0.0),
                    fog_density: 0.0,
                    fog_height_falloff: 0.0,
                    volumetric: Default::default(),
                },
                seed: 1,
                objects: vec![],
                root: SdfNode::Primitive {
                    object: crate::SdfObject {
                        primitive: SdfPrimitive::Torus {
                            major_radius: 2.0,
                            minor_radius: 0.4,
                        },
                        modifiers: vec![],
                        material: Default::default(),
                        bounds_radius: None,
                    },
                },
                timeline: None,
                generator: None,
                fields: vec![],
                patterns: vec![],
                harmonics: None,
                rhythm: None,
                audio: None,
            },
        };
        let director = CameraDirector::default();
        let a = director.generate(&scene, 12.0);
        let b = director.generate(&scene, 12.0);
        assert_eq!(a, b);
        assert!(!a.shots.is_empty());
    }
}
