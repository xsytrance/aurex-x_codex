use serde::{Deserialize, Serialize};

use crate::{
    Scene,
    automation::AutomationBinding,
    director_rules::{
        DirectorRecommendationCache, DirectorRuleSet, TransitionRecommendation,
        default_recommendation,
    },
    effect_graph::{GraphMorph, GraphMorphSpec, GraphMorphState, GraphMorphStrategy},
    transition::{
        TransitionContext, TransitionEngine, TransitionSpec, TransitionState, TransitionStyle,
    },
};

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
    #[serde(default)]
    pub auto: bool,
    #[serde(default)]
    pub spec: Option<TransitionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransitionType {
    Fade,
    PulseFlash,
    PatternMorph,
    FractalZoom,
    GeometryDissolve,
}

impl Transition {
    pub fn to_spec(&self) -> TransitionSpec {
        if let Some(spec) = &self.spec {
            return spec.clone();
        }
        TransitionSpec {
            style: match self.transition_type {
                TransitionType::Fade => TransitionStyle::HarmonicSmear,
                TransitionType::PulseFlash => TransitionStyle::PulseFlash,
                TransitionType::PatternMorph => TransitionStyle::PatternDissolve,
                TransitionType::FractalZoom => TransitionStyle::FractalZoom,
                TransitionType::GeometryDissolve => TransitionStyle::GeometryMelt,
            },
            duration: self.duration,
            intensity: self.intensity,
            distortion: 0.2,
            pattern_strength: 0.5,
            harmonic_strength: 0.5,
            progress_signal: None,
        }
    }
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
        self.entry_with_index_at_time(time).map(|(_, e, _)| e)
    }

    pub fn entry_with_index_at_time(&self, time: f32) -> Option<(usize, &DemoEntry, f32)> {
        let mut cursor = 0.0;
        for (idx, entry) in self.entries.iter().enumerate() {
            let len = match entry {
                DemoEntry::SceneBlock(b) => b.duration,
                DemoEntry::Transition(t) => t.duration,
            }
            .max(1e-6);
            if time >= cursor && time < cursor + len {
                return Some((idx, entry, (time - cursor) / len));
            }
            cursor += len;
        }
        self.entries
            .last()
            .map(|e| (self.entries.len().saturating_sub(1), e, 1.0))
    }

    pub fn neighboring_blocks(&self, idx: usize) -> (Option<&DemoBlock>, Option<&DemoBlock>) {
        let prev = self.entries[..idx].iter().rev().find_map(|e| match e {
            DemoEntry::SceneBlock(b) => Some(b),
            _ => None,
        });
        let next = self.entries.get(idx + 1..).and_then(|tail| {
            tail.iter().find_map(|e| match e {
                DemoEntry::SceneBlock(b) => Some(b),
                _ => None,
            })
        });
        (prev, next)
    }
}

impl Demo {
    pub fn apply_at_time(&self, scene: &mut Scene, time: f32) {
        let total = self.timeline.total_duration().max(1e-6);
        let t = time.rem_euclid(total);
        if let Some((_, entry, _)) = self.timeline.entry_with_index_at_time(t) {
            match entry {
                DemoEntry::SceneBlock(block) => apply_block(scene, block),
                DemoEntry::Transition(tx) => apply_legacy_transition_hint(scene, tx),
            }
        }
    }

    pub fn blend_scene_at_time(
        &self,
        fallback: &Scene,
        time: f32,
        ctx: TransitionContext,
        rule_set: &DirectorRuleSet,
    ) -> Option<Scene> {
        let total = self.timeline.total_duration().max(1e-6);
        let t = time.rem_euclid(total);
        let (idx, entry, progress) = self.timeline.entry_with_index_at_time(t)?;
        match entry {
            DemoEntry::SceneBlock(block) => {
                let mut scene = crate::load_scene_from_json_path(&block.scene_reference).ok()?;
                apply_block(&mut scene, block);
                Some(scene)
            }
            DemoEntry::Transition(tx) => {
                let (prev, next) = self.timeline.neighboring_blocks(idx);
                let source = prev
                    .and_then(|b| crate::load_scene_from_json_path(&b.scene_reference).ok())
                    .unwrap_or_else(|| fallback.clone());
                let target = next
                    .and_then(|b| crate::load_scene_from_json_path(&b.scene_reference).ok())
                    .unwrap_or_else(|| fallback.clone());

                let mut rec_cache = DirectorRecommendationCache::default();
                let recommendation = if tx.auto {
                    let audio_intensity =
                        (ctx.low_freq_energy + ctx.mid_freq_energy + ctx.high_freq_energy)
                            .clamp(0.0, 1.0);
                    rec_cache.get_or_compute(
                        source.sdf.seed,
                        target.sdf.seed,
                        audio_intensity,
                        || rule_set.recommend(&source, &target, audio_intensity),
                    )
                } else {
                    default_recommendation(&source, &target, 0.3)
                };

                let mut spec = tx.to_spec();
                if tx.auto {
                    apply_recommendation(&mut spec, recommendation);
                }
                spec.duration = tx.duration.max(1e-6);
                spec.intensity = tx.intensity.max(spec.intensity);

                let engine = TransitionEngine;
                let mut blended =
                    engine.blend_scenes(&source, &target, &spec, TransitionState { progress }, ctx);

                if let (Some(g_a), Some(g_b)) = (&source.sdf.effect_graph, &target.sdf.effect_graph)
                {
                    blended.sdf.effect_graph = Some(GraphMorph::morph(
                        g_a,
                        g_b,
                        &GraphMorphSpec {
                            strategy: GraphMorphStrategy::NodeParameterBlend,
                            duration: spec.duration,
                            intensity: spec.intensity,
                        },
                        GraphMorphState { progress },
                    ));
                }

                Some(blended)
            }
        }
    }
}

fn apply_recommendation(spec: &mut TransitionSpec, rec: TransitionRecommendation) {
    spec.style = rec.transition_style;
    spec.duration = rec.duration;
    spec.intensity = rec.intensity;
}

fn apply_block(scene: &mut Scene, block: &DemoBlock) {
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

fn apply_legacy_transition_hint(scene: &mut Scene, tx: &Transition) {
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
                o.material
                    .parameters
                    .insert("pattern_morph".into(), (0.5 + 0.5 * f).clamp(0.0, 1.0));
            }
        }
        TransitionType::FractalZoom => {
            scene.sdf.camera.fov_degrees =
                (scene.sdf.camera.fov_degrees * (1.0 - 0.12 * f)).clamp(25.0, 120.0);
        }
        TransitionType::GeometryDissolve => {
            for o in &mut scene.sdf.objects {
                o.material.emissive_strength *= 1.0 + 0.25 * f;
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
                    auto: false,
                    spec: None,
                }),
            ],
        };
        let a = timeline.entry_at_time(4.5).cloned();
        let b = timeline.entry_at_time(4.5).cloned();
        assert_eq!(a, b);
    }
}
