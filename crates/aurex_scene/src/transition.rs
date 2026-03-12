use serde::{Deserialize, Serialize};

use crate::{Scene, SdfNode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TransitionContext {
    pub seed: u32,
    pub time_seconds: f32,
    pub beat: f32,
    pub measure: f32,
    pub phrase: f32,
    pub tempo: f32,
    pub low_freq_energy: f32,
    pub mid_freq_energy: f32,
    pub high_freq_energy: f32,
    pub dominant_frequency: f32,
}

impl Default for TransitionContext {
    fn default() -> Self {
        Self {
            seed: 0,
            time_seconds: 0.0,
            beat: 0.0,
            measure: 0.0,
            phrase: 0.0,
            tempo: 120.0,
            low_freq_energy: 0.0,
            mid_freq_energy: 0.0,
            high_freq_energy: 0.0,
            dominant_frequency: 220.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TransitionSignal {
    Beat,
    Measure,
    Phrase,
    Tempo,
    LowFreqEnergy,
    MidFreqEnergy,
    HighFreqEnergy,
    DominantFrequency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransitionStyle {
    PulseFlash,
    PatternDissolve,
    FractalZoom,
    HarmonicSmear,
    GeometryMelt,
    TunnelSnap,
    CathedralBloom,
    RhythmStutter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionSpec {
    pub style: TransitionStyle,
    pub duration: f32,
    #[serde(default = "default_intensity")]
    pub intensity: f32,
    #[serde(default)]
    pub distortion: f32,
    #[serde(default)]
    pub pattern_strength: f32,
    #[serde(default)]
    pub harmonic_strength: f32,
    #[serde(default)]
    pub progress_signal: Option<TransitionSignal>,
}

fn default_intensity() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TransitionState {
    pub progress: f32,
}

impl TransitionState {
    pub fn from_time(time_in_transition: f32, duration: f32) -> Self {
        Self {
            progress: (time_in_transition / duration.max(1e-6)).clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransitionEngine;

impl TransitionEngine {
    pub fn blend_scenes(
        &self,
        source: &Scene,
        target: &Scene,
        spec: &TransitionSpec,
        state: TransitionState,
        ctx: TransitionContext,
    ) -> Scene {
        let p = modulated_progress(spec, state.progress, ctx);
        let mut out = source.clone();

        out.sdf.seed = source.sdf.seed ^ target.sdf.seed ^ ctx.seed;
        out.sdf.camera.position = source
            .sdf
            .camera
            .position
            .lerp(target.sdf.camera.position, p);
        out.sdf.camera.target = source.sdf.camera.target.lerp(target.sdf.camera.target, p);
        out.sdf.camera.fov_degrees = source.sdf.camera.fov_degrees
            + (target.sdf.camera.fov_degrees - source.sdf.camera.fov_degrees) * p;

        out.sdf.lighting.ambient_light = lerp(
            source.sdf.lighting.ambient_light,
            target.sdf.lighting.ambient_light,
            p,
        );
        out.sdf.lighting.fog_density = lerp(
            source.sdf.lighting.fog_density,
            target.sdf.lighting.fog_density,
            p,
        );
        out.sdf.lighting.fog_height_falloff = lerp(
            source.sdf.lighting.fog_height_falloff,
            target.sdf.lighting.fog_height_falloff,
            p,
        );
        out.sdf.lighting.fog_color = source
            .sdf
            .lighting
            .fog_color
            .lerp(target.sdf.lighting.fog_color, p);

        let min_lights = source
            .sdf
            .lighting
            .key_lights
            .len()
            .min(target.sdf.lighting.key_lights.len());
        for i in 0..min_lights {
            out.sdf.lighting.key_lights[i].intensity = lerp(
                source.sdf.lighting.key_lights[i].intensity,
                target.sdf.lighting.key_lights[i].intensity,
                p,
            );
            out.sdf.lighting.key_lights[i].color = source.sdf.lighting.key_lights[i]
                .color
                .lerp(target.sdf.lighting.key_lights[i].color, p);
        }

        let min_objects = source.sdf.objects.len().min(target.sdf.objects.len());
        for i in 0..min_objects {
            out.sdf.objects[i].material.base_color = source.sdf.objects[i]
                .material
                .base_color
                .lerp(target.sdf.objects[i].material.base_color, p);
            out.sdf.objects[i].material.emissive_strength = lerp(
                source.sdf.objects[i].material.emissive_strength,
                target.sdf.objects[i].material.emissive_strength,
                p,
            );
            out.sdf.objects[i].material.roughness = lerp(
                source.sdf.objects[i].material.roughness,
                target.sdf.objects[i].material.roughness,
                p,
            );
        }

        out.sdf.root = SdfNode::Blend {
            children: vec![source.sdf.root.clone(), target.sdf.root.clone()],
            weights: vec![1.0 - p, p],
        };

        self.apply_style(&mut out, spec, p, ctx);
        out
    }

    fn apply_style(
        &self,
        scene: &mut Scene,
        spec: &TransitionSpec,
        p: f32,
        ctx: TransitionContext,
    ) {
        let e = spec.intensity.max(0.0);
        match spec.style {
            TransitionStyle::PulseFlash => {
                for l in &mut scene.sdf.lighting.key_lights {
                    l.intensity *= 1.0 + (1.0 - (2.0 * p - 1.0).abs()) * e * 0.8;
                }
            }
            TransitionStyle::PatternDissolve => {
                for o in &mut scene.sdf.objects {
                    o.material.parameters.insert(
                        "transition_pattern_dissolve".into(),
                        p * spec.pattern_strength.max(0.0),
                    );
                }
            }
            TransitionStyle::FractalZoom => {
                scene.sdf.camera.fov_degrees =
                    (scene.sdf.camera.fov_degrees * (1.0 - p * 0.18 * e)).clamp(22.0, 120.0);
            }
            TransitionStyle::HarmonicSmear => {
                let smear =
                    (ctx.mid_freq_energy + ctx.high_freq_energy) * spec.harmonic_strength.max(0.0);
                scene.sdf.camera.position.y += smear * (p * std::f32::consts::PI).sin() * 0.4;
            }
            TransitionStyle::GeometryMelt => {
                for o in &mut scene.sdf.objects {
                    o.material.emissive_strength *= 1.0 + p * spec.distortion.max(0.0) * 0.4;
                }
            }
            TransitionStyle::TunnelSnap => {
                let snap = if p < 0.5 { 0.0 } else { 1.0 };
                scene.sdf.camera.position.z += (snap - 0.5) * e * 0.7;
            }
            TransitionStyle::CathedralBloom => {
                let bloom = (p * std::f32::consts::PI).sin().abs() * e;
                scene.sdf.lighting.volumetric.shaft_intensity *= 1.0 + bloom * 0.6;
                scene.sdf.lighting.ambient_light *= 1.0 + bloom * 0.2;
            }
            TransitionStyle::RhythmStutter => {
                let t = ((ctx.beat + p * 8.0).floor() as i32).rem_euclid(2);
                if t == 0 {
                    scene.sdf.camera.position = scene
                        .sdf
                        .camera
                        .position
                        .lerp(scene.sdf.camera.target, 0.05 * e);
                }
            }
        }
    }
}

fn modulated_progress(spec: &TransitionSpec, progress: f32, ctx: TransitionContext) -> f32 {
    let base = progress.clamp(0.0, 1.0);
    let signal = match spec.progress_signal {
        None => 0.0,
        Some(TransitionSignal::Beat) => ctx.beat.fract(),
        Some(TransitionSignal::Measure) => ctx.measure.fract(),
        Some(TransitionSignal::Phrase) => ctx.phrase.fract(),
        Some(TransitionSignal::Tempo) => (ctx.tempo / 180.0).clamp(0.0, 1.0),
        Some(TransitionSignal::LowFreqEnergy) => ctx.low_freq_energy.clamp(0.0, 1.0),
        Some(TransitionSignal::MidFreqEnergy) => ctx.mid_freq_energy.clamp(0.0, 1.0),
        Some(TransitionSignal::HighFreqEnergy) => ctx.high_freq_energy.clamp(0.0, 1.0),
        Some(TransitionSignal::DominantFrequency) => {
            (ctx.dominant_frequency / 1000.0).clamp(0.0, 1.0)
        }
    };
    (base * 0.75 + signal * 0.25).clamp(0.0, 1.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeyLight, SdfCamera, SdfLighting, SdfObject, SdfPrimitive, SdfScene, Vec3};

    #[test]
    fn transition_is_deterministic() {
        let source = Scene {
            sdf: SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 0.0, -5.0),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 1.777,
                },
                lighting: SdfLighting {
                    ambient_light: 0.2,
                    key_lights: vec![KeyLight {
                        direction: Vec3::new(0.0, -1.0, 0.0),
                        intensity: 1.0,
                        color: Vec3::new(1.0, 1.0, 1.0),
                    }],
                    fog_color: Vec3::new(0.1, 0.1, 0.2),
                    fog_density: 0.02,
                    fog_height_falloff: 0.05,
                    volumetric: Default::default(),
                },
                seed: 1,
                objects: vec![SdfObject {
                    primitive: SdfPrimitive::Sphere { radius: 1.0 },
                    modifiers: vec![],
                    material: Default::default(),
                    bounds_radius: None,
                }],
                root: SdfNode::Empty,
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
        let mut target = source.clone();
        target.sdf.camera.position = Vec3::new(1.0, 0.5, -3.0);
        target.sdf.lighting.ambient_light = 0.35;
        let spec = TransitionSpec {
            style: TransitionStyle::CathedralBloom,
            duration: 2.0,
            intensity: 0.9,
            distortion: 0.2,
            pattern_strength: 0.4,
            harmonic_strength: 0.5,
            progress_signal: Some(TransitionSignal::Beat),
        };
        let state = TransitionState { progress: 0.4 };
        let ctx = TransitionContext {
            beat: 0.7,
            ..TransitionContext::default()
        };
        let engine = TransitionEngine;
        let a = engine.blend_scenes(&source, &target, &spec, state, ctx);
        let b = engine.blend_scenes(&source, &target, &spec, state, ctx);
        assert_eq!(a, b);
    }
}
