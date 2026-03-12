use serde::{Deserialize, Serialize};

use crate::{Scene, SdfPrimitive, demo::TransitionType, transition::TransitionStyle};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectorRule {
    pub name: String,
    pub min_audio_intensity: f32,
    pub preferred_transition: TransitionStyle,
    pub duration: f32,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DirectorRuleSet {
    #[serde(default)]
    pub rules: Vec<DirectorRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionRecommendation {
    pub transition_style: TransitionStyle,
    pub duration: f32,
    pub intensity: f32,
}

impl DirectorRuleSet {
    pub fn recommend(
        &self,
        source: &Scene,
        target: &Scene,
        audio_intensity: f32,
    ) -> TransitionRecommendation {
        for rule in &self.rules {
            if audio_intensity >= rule.min_audio_intensity {
                return TransitionRecommendation {
                    transition_style: rule.preferred_transition,
                    duration: rule.duration,
                    intensity: rule.intensity,
                };
            }
        }

        default_recommendation(source, target, audio_intensity)
    }
}

pub fn default_recommendation(
    source: &Scene,
    target: &Scene,
    audio_intensity: f32,
) -> TransitionRecommendation {
    let tunnelish = has_primitive(source, |p| matches!(p, SdfPrimitive::Torus { .. }))
        || has_primitive(target, |p| matches!(p, SdfPrimitive::Torus { .. }));
    let templeish = has_primitive(source, |p| matches!(p, SdfPrimitive::Cylinder { .. }))
        || has_primitive(target, |p| matches!(p, SdfPrimitive::Cylinder { .. }));
    let pattern_complexity = source.sdf.patterns.len() + target.sdf.patterns.len();

    if audio_intensity > 0.75 {
        return TransitionRecommendation {
            transition_style: TransitionStyle::RhythmStutter,
            duration: 1.2,
            intensity: 0.95,
        };
    }

    if tunnelish {
        return TransitionRecommendation {
            transition_style: TransitionStyle::TunnelSnap,
            duration: 1.8,
            intensity: 0.8,
        };
    }

    if templeish {
        return TransitionRecommendation {
            transition_style: TransitionStyle::CathedralBloom,
            duration: 2.4,
            intensity: 0.75,
        };
    }

    if pattern_complexity > 2 {
        return TransitionRecommendation {
            transition_style: TransitionStyle::PatternDissolve,
            duration: 2.1,
            intensity: 0.7,
        };
    }

    TransitionRecommendation {
        transition_style: TransitionStyle::HarmonicSmear,
        duration: 2.0,
        intensity: 0.6,
    }
}

fn has_primitive<F: Fn(&SdfPrimitive) -> bool>(scene: &Scene, pred: F) -> bool {
    fn walk<F: Fn(&SdfPrimitive) -> bool>(node: &crate::SdfNode, pred: &F) -> bool {
        match node {
            crate::SdfNode::Primitive { object } => pred(&object.primitive),
            crate::SdfNode::Group { children }
            | crate::SdfNode::Union { children }
            | crate::SdfNode::SmoothUnion { children, .. }
            | crate::SdfNode::Intersect { children }
            | crate::SdfNode::Blend { children, .. } => children.iter().any(|c| walk(c, pred)),
            crate::SdfNode::Transform { child, .. } => walk(child, pred),
            crate::SdfNode::Subtract { base, subtract } => {
                walk(base, pred) || subtract.iter().any(|c| walk(c, pred))
            }
            crate::SdfNode::Empty => false,
        }
    }

    scene.sdf.objects.iter().any(|o| pred(&o.primitive)) || walk(&scene.sdf.root, &pred)
}

impl From<TransitionStyle> for TransitionType {
    fn from(value: TransitionStyle) -> Self {
        match value {
            TransitionStyle::PulseFlash => TransitionType::PulseFlash,
            TransitionStyle::PatternDissolve => TransitionType::PatternMorph,
            TransitionStyle::FractalZoom => TransitionType::FractalZoom,
            TransitionStyle::HarmonicSmear => TransitionType::Fade,
            TransitionStyle::GeometryMelt => TransitionType::GeometryDissolve,
            TransitionStyle::TunnelSnap => TransitionType::PulseFlash,
            TransitionStyle::CathedralBloom => TransitionType::PatternMorph,
            TransitionStyle::RhythmStutter => TransitionType::PulseFlash,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DirectorRecommendationCache {
    entries: std::collections::BTreeMap<(u32, u32, i32), TransitionRecommendation>,
}

impl DirectorRecommendationCache {
    pub fn get_or_compute<F: FnOnce() -> TransitionRecommendation>(
        &mut self,
        source_seed: u32,
        target_seed: u32,
        audio_intensity: f32,
        compute: F,
    ) -> TransitionRecommendation {
        let key = (source_seed, target_seed, (audio_intensity * 1000.0) as i32);
        if let Some(v) = self.entries.get(&key).cloned() {
            v
        } else {
            let v = compute();
            self.entries.insert(key, v.clone());
            v
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SdfCamera, SdfLighting, SdfObject, SdfScene, Vec3};

    #[test]
    fn director_rule_selection_is_stable() {
        let scene = Scene {
            sdf: SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 0.0, -5.0),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 1.777,
                },
                lighting: SdfLighting {
                    ambient_light: 0.2,
                    key_lights: vec![],
                    fog_color: Vec3::new(0.1, 0.1, 0.1),
                    fog_density: 0.0,
                    fog_height_falloff: 0.0,
                    volumetric: Default::default(),
                },
                seed: 1,
                objects: vec![SdfObject {
                    primitive: SdfPrimitive::Torus {
                        major_radius: 2.0,
                        minor_radius: 0.4,
                    },
                    modifiers: vec![],
                    material: Default::default(),
                    bounds_radius: None,
                }],
                root: crate::SdfNode::Empty,
                timeline: None,
                generator: None,
                fields: vec![],
                patterns: vec![],
                harmonics: None,
                rhythm: None,
                audio: None,
                effect_graph: None,
                automation_tracks: vec![],
                demo_sequence: None,
                temporal_effects: vec![],
            },
        };
        let a = default_recommendation(&scene, &scene, 0.5);
        let b = default_recommendation(&scene, &scene, 0.5);
        assert_eq!(a, b);
        assert_eq!(a.transition_style, TransitionStyle::TunnelSnap);
    }
}
