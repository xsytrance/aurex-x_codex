use serde::{Deserialize, Serialize};

use crate::{Scene, automation::AutomationBinding};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Demo {
    pub timeline: DemoTimeline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DemoTimeline {
    #[serde(default)]
    pub entries: Vec<DemoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DemoEntry {
    SceneBlock(DemoBlock),
    Transition(Transition),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemoBlock {
    pub scene_reference: String,
    pub duration: f32,
    #[serde(default)]
    pub camera_style: Option<String>,
    #[serde(default)]
    pub lighting_preset: Option<String>,
    #[serde(default)]
    pub automation_tracks: Vec<AutomationBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transition {
    pub transition_type: TransitionType,
    pub duration: f32,
    #[serde(default = "default_intensity")]
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransitionType {
    Fade,
    PulseFlash,
    PatternMorph,
    FractalZoom,
    GeometryDissolve,
}

impl DemoTimeline {
    pub fn total_duration(&self) -> f32 {
        self.entries
            .iter()
            .map(|e| match e {
                DemoEntry::SceneBlock(b) => b.duration,
                DemoEntry::Transition(t) => t.duration,
            })
            .sum()
    }

    pub fn entry_at_time(&self, time: f32) -> Option<&DemoEntry> {
        let mut cursor = 0.0;
        for entry in &self.entries {
            let len = match entry {
                DemoEntry::SceneBlock(b) => b.duration,
                DemoEntry::Transition(t) => t.duration,
            }
            .max(1e-6);
            if time >= cursor && time < cursor + len {
                return Some(entry);
            }
            cursor += len;
        }
        self.entries.last()
    }
}

impl Demo {
    pub fn apply_at_time(&self, scene: &mut Scene, time: f32) {
        let total = self.timeline.total_duration().max(1e-6);
        let t = time.rem_euclid(total);
        if let Some(entry) = self.timeline.entry_at_time(t) {
            match entry {
                DemoEntry::SceneBlock(block) => {
                    if let Some(style) = &block.camera_style {
                        if style == "wide" {
                            scene.sdf.camera.fov_degrees = scene.sdf.camera.fov_degrees.max(68.0);
                        } else if style == "close" {
                            scene.sdf.camera.fov_degrees = scene.sdf.camera.fov_degrees.min(45.0);
                        }
                    }
                    if let Some(preset) = &block.lighting_preset {
                        match preset.as_str() {
                            "bright" => scene.sdf.lighting.ambient_light *= 1.2,
                            "dark" => scene.sdf.lighting.ambient_light *= 0.75,
                            "neon" => scene.sdf.lighting.volumetric.shaft_intensity *= 1.15,
                            _ => {}
                        }
                    }
                }
                DemoEntry::Transition(tx) => {
                    let f = tx.intensity;
                    match tx.transition_type {
                        TransitionType::Fade => {
                            scene.sdf.lighting.ambient_light *= (1.0 - 0.35 * f).clamp(0.0, 1.0)
                        }
                        TransitionType::PulseFlash => {
                            for l in &mut scene.sdf.lighting.key_lights {
                                l.intensity *= 1.0 + 0.3 * f;
                            }
                        }
                        TransitionType::PatternMorph => {
                            for o in &mut scene.sdf.objects {
                                o.material.parameters.insert(
                                    "pattern_morph".into(),
                                    (0.5 + 0.5 * f).clamp(0.0, 1.0),
                                );
                            }
                        }
                        TransitionType::FractalZoom => {
                            scene.sdf.camera.fov_degrees = (scene.sdf.camera.fov_degrees
                                * (1.0 - 0.12 * f))
                                .clamp(25.0, 120.0);
                        }
                        TransitionType::GeometryDissolve => {
                            for o in &mut scene.sdf.objects {
                                o.material.emissive_strength *= 1.0 + 0.25 * f;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn default_intensity() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_sequence_transitions_are_stable() {
        let timeline = DemoTimeline {
            entries: vec![
                DemoEntry::SceneBlock(DemoBlock {
                    scene_reference: "a".into(),
                    duration: 4.0,
                    camera_style: Some("wide".into()),
                    lighting_preset: None,
                    automation_tracks: vec![],
                }),
                DemoEntry::Transition(Transition {
                    transition_type: TransitionType::Fade,
                    duration: 2.0,
                    intensity: 0.8,
                }),
            ],
        };
        let a = timeline.entry_at_time(4.5).cloned();
        let b = timeline.entry_at_time(4.5).cloned();
        assert_eq!(a, b);
    }
}
