pub mod automation;
pub mod camera;
pub mod demo;
pub mod director;
pub mod effect_graph;
pub mod fields;
pub mod generators;
pub mod harmonics;
pub mod patterns;

use aurex_audio::ProceduralAudioConfig;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        Self::new(
            self.x + (rhs.x - self.x) * t,
            self.y + (rhs.y - self.y) * t,
            self.z + (rhs.z - self.z) * t,
        )
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
    #[serde(default)]
    pub timeline: Option<SceneTimeline>,
    #[serde(default)]
    pub generator: Option<generators::SceneGenerator>,
    #[serde(default)]
    pub fields: Vec<fields::SceneField>,
    #[serde(default)]
    pub patterns: Vec<patterns::PatternNetwork>,
    #[serde(default)]
    pub harmonics: Option<harmonics::SceneHarmonicsConfig>,
    #[serde(default)]
    pub rhythm: Option<RhythmSpaceConfig>,
    #[serde(default)]
    pub audio: Option<ProceduralAudioConfig>,
    #[serde(default)]
    pub effect_graph: Option<effect_graph::EffectGraph>,
    #[serde(default)]
    pub automation_tracks: Vec<automation::AutomationBinding>,
    #[serde(default)]
    pub demo_sequence: Option<demo::Demo>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RhythmParticleMode {
    Bass,
    Snare,
    Melody,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TimeWarpConfig {
    #[serde(default = "default_time_scale")]
    pub time_scale: f32,
    #[serde(default)]
    pub time_delay: f32,
    #[serde(default)]
    pub time_echo: f32,
    #[serde(default)]
    pub time_reverse: bool,
}

fn default_time_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RhythmSpaceConfig {
    #[serde(default)]
    pub beat_geometry: bool,
    #[serde(default)]
    pub echo_effect: bool,
    #[serde(default)]
    pub particle_mode: Option<RhythmParticleMode>,
    #[serde(default)]
    pub time_warp: Option<TimeWarpConfig>,
}

impl Default for RhythmSpaceConfig {
    fn default() -> Self {
        Self {
            beat_geometry: false,
            echo_effect: false,
            particle_mode: None,
            time_warp: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneTimeline {
    pub duration: f32,
    #[serde(default)]
    pub loops: bool,
    #[serde(default)]
    pub keyframes: Vec<TimelineKeyframe>,
    #[serde(default)]
    pub events: Vec<TimelineEvent>,
    #[serde(default)]
    pub camera_path: Option<CameraPath>,
    #[serde(default)]
    pub cinematic_camera: Option<camera::CameraRig>,
    #[serde(default)]
    pub shot_sequence: Option<director::ShotSequence>,
}

impl SceneTimeline {
    pub fn normalized_time(&self, time_seconds: f32) -> f32 {
        let duration = self.duration.max(1e-6);
        if self.loops {
            time_seconds.rem_euclid(duration)
        } else {
            time_seconds.clamp(0.0, duration)
        }
    }

    pub fn sample_keyframe_value(&self, target: &str, time_seconds: f32) -> Option<TimelineValue> {
        let t = self.normalized_time(time_seconds);
        let mut keys: Vec<&TimelineKeyframe> = self
            .keyframes
            .iter()
            .filter(|k| k.target == target)
            .collect();

        if keys.is_empty() {
            return None;
        }

        keys.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let prev = keys
            .iter()
            .rev()
            .find(|k| k.time <= t)
            .copied()
            .unwrap_or(keys[0]);
        let next = keys
            .iter()
            .find(|k| k.time >= t)
            .copied()
            .unwrap_or(keys[keys.len() - 1]);

        if std::ptr::eq(prev, next) {
            return Some(prev.value.clone());
        }

        let span = (next.time - prev.time).max(1e-6);
        let alpha = ((t - prev.time) / span).clamp(0.0, 1.0);
        let shaped = prev.interpolation.apply(alpha);
        TimelineValue::interpolate(&prev.value, &next.value, shaped).or(Some(prev.value.clone()))
    }

    pub fn event_strength(&self, hook: AudioSyncHook, time_seconds: f32) -> f32 {
        let t = self.normalized_time(time_seconds);
        let mut strength: f32 = 0.0;
        for event in &self.events {
            if event.audio_hook == Some(hook) {
                let dt = (t - event.time).abs();
                let window = 0.12;
                let pulse = (1.0 - dt / window).clamp(0.0, 1.0);
                strength = strength.max(pulse * pulse);
            }
        }
        strength
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineKeyframe {
    pub time: f32,
    pub target: String,
    pub value: TimelineValue,
    #[serde(default)]
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimelineValue {
    Float { value: f32 },
    Vec3 { value: Vec3 },
}

impl TimelineValue {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Option<Self> {
        match (a, b) {
            (TimelineValue::Float { value: a }, TimelineValue::Float { value: b }) => {
                Some(TimelineValue::Float {
                    value: *a + (*b - *a) * t,
                })
            }
            (TimelineValue::Vec3 { value: a }, TimelineValue::Vec3 { value: b }) => {
                Some(TimelineValue::Vec3 {
                    value: a.lerp(*b, t),
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterpolationType {
    Linear,
    EaseIn,
    EaseOut,
    Smoothstep,
}

impl Default for InterpolationType {
    fn default() -> Self {
        Self::Linear
    }
}

impl InterpolationType {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            InterpolationType::Linear => t,
            InterpolationType::EaseIn => t * t,
            InterpolationType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            InterpolationType::Smoothstep => t * t * (3.0 - 2.0 * t),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEvent {
    pub time: f32,
    pub name: String,
    #[serde(default)]
    pub audio_hook: Option<AudioSyncHook>,
    #[serde(default)]
    pub parameters: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioSyncHook {
    Kick,
    Snare,
    Bass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraPath {
    pub path_type: CameraPathType,
    pub origin: Vec3,
    pub target: Vec3,
    #[serde(default = "default_camera_radius")]
    pub radius: f32,
    #[serde(default = "default_camera_speed")]
    pub speed: f32,
    #[serde(default)]
    pub height: f32,
}

fn default_camera_radius() -> f32 {
    6.0
}

fn default_camera_speed() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CameraPathType {
    Orbit,
    Flythrough,
    Spiral,
    Dolly,
}

impl CameraPath {
    pub fn sample(&self, time_seconds: f32, base: &SdfCamera, duration: f32) -> SdfCamera {
        let phase = (time_seconds / duration.max(1e-6)) * self.speed;
        let mut camera = base.clone();
        match self.path_type {
            CameraPathType::Orbit => {
                let ang = phase * std::f32::consts::TAU;
                camera.position = Vec3::new(
                    self.origin.x + ang.cos() * self.radius,
                    self.origin.y + self.height,
                    self.origin.z + ang.sin() * self.radius,
                );
                camera.target = self.target;
            }
            CameraPathType::Flythrough => {
                camera.position = Vec3::new(
                    self.origin.x,
                    self.origin.y + self.height,
                    self.origin.z + phase * self.radius,
                );
                camera.target = Vec3::new(
                    self.target.x,
                    self.target.y,
                    self.target.z + phase * self.radius,
                );
            }
            CameraPathType::Spiral => {
                let ang = phase * std::f32::consts::TAU;
                let r = self.radius * (1.0 + 0.2 * phase);
                camera.position = Vec3::new(
                    self.origin.x + ang.cos() * r,
                    self.origin.y + self.height + phase,
                    self.origin.z + ang.sin() * r,
                );
                camera.target = self.target;
            }
            CameraPathType::Dolly => {
                camera.position = Vec3::new(
                    self.origin.x + phase * self.radius,
                    self.origin.y + self.height,
                    self.origin.z,
                );
                camera.target = self.target;
            }
        }
        camera
    }
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
pub struct VolumetricLighting {
    #[serde(default = "default_volumetric_steps")]
    pub scattering_steps: u32,
    #[serde(default)]
    pub beam_falloff: f32,
    #[serde(default)]
    pub beam_density: f32,
    #[serde(default)]
    pub shaft_intensity: f32,
}

fn default_volumetric_steps() -> u32 {
    8
}

impl Default for VolumetricLighting {
    fn default() -> Self {
        Self {
            scattering_steps: default_volumetric_steps(),
            beam_falloff: 0.7,
            beam_density: 0.12,
            shaft_intensity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfLighting {
    pub ambient_light: f32,
    #[serde(default)]
    pub key_lights: Vec<KeyLight>,
    #[serde(default = "default_fog_color")]
    pub fog_color: Vec3,
    #[serde(default)]
    pub fog_density: f32,
    #[serde(default)]
    pub fog_height_falloff: f32,
    #[serde(default)]
    pub volumetric: VolumetricLighting,
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
    SpectralReactive,
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
    pub pattern_network: Option<patterns::PatternNetwork>,
    #[serde(default)]
    pub parameters: BTreeMap<String, f32>,
}

fn default_base_color() -> Vec3 {
    Vec3::new(0.75, 0.85, 1.0)
}

fn default_roughness() -> f32 {
    0.45
}

fn default_fog_color() -> Vec3 {
    Vec3::new(0.08, 0.12, 0.18)
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
            pattern_network: None,
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
    use super::{
        AudioSyncHook, InterpolationType, Scene, SceneTimeline, SdfMaterialType, SdfNode,
        TimelineValue, Vec3, load_scene_from_json_str,
    };
    use std::collections::BTreeMap;

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
                "timeline": {
                    "duration": 8.0,
                    "loops": true,
                    "keyframes": [
                      {"time": 0.0, "target": "camera.position", "value": {"Vec3": {"value": {"x": 0.0, "y": 0.0, "z": -6.0}}}, "interpolation": "Smoothstep"},
                      {"time": 4.0, "target": "camera.position", "value": {"Vec3": {"value": {"x": 0.0, "y": 1.0, "z": -4.0}}}, "interpolation": "Linear"}
                    ],
                    "events": [
                      {"time": 1.0, "name": "kick", "audio_hook": "Kick"}
                    ]
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
        assert!(scene.sdf.timeline.is_some());
        match scene.sdf.root {
            SdfNode::Union { children } => assert_eq!(children.len(), 2),
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

    #[test]
    fn pattern_network_json_roundtrip_parses() {
        let json = r#"{
            "sdf": {
                "camera": {"position": {"x": 0.0, "y": 0.0, "z": -5.0}, "target": {"x": 0.0, "y": 0.0, "z": 0.0}, "fov_degrees": 60.0, "aspect_ratio": 1.7777},
                "lighting": {"ambient_light": 0.2},
                "patterns": [{"name": "id", "preset": "PsySpiral", "layers": []}],
                "objects": [{
                    "primitive": {"Sphere": {"radius": 1.0}},
                    "material": {
                        "material_type": "SolidColor",
                        "pattern_network": {"name": "mat", "preset": "ElectronicCircuit", "layers": []}
                    }
                }]
            }
        }"#;

        let scene: Scene = load_scene_from_json_str(json).expect("pattern scene should parse");
        assert_eq!(scene.sdf.patterns.len(), 1);
        assert!(scene.sdf.objects[0].material.pattern_network.is_some());
    }

    #[test]
    fn timeline_sampling_and_interpolation_work() {
        let timeline = SceneTimeline {
            duration: 10.0,
            loops: true,
            keyframes: vec![
                super::TimelineKeyframe {
                    time: 0.0,
                    target: "camera.position".to_string(),
                    value: TimelineValue::Vec3 {
                        value: Vec3::new(0.0, 0.0, -6.0),
                    },
                    interpolation: InterpolationType::Smoothstep,
                },
                super::TimelineKeyframe {
                    time: 10.0,
                    target: "camera.position".to_string(),
                    value: TimelineValue::Vec3 {
                        value: Vec3::new(0.0, 2.0, -4.0),
                    },
                    interpolation: InterpolationType::Linear,
                },
            ],
            events: vec![super::TimelineEvent {
                time: 2.0,
                name: "kick".into(),
                audio_hook: Some(AudioSyncHook::Kick),
                parameters: BTreeMap::new(),
            }],
            camera_path: None,
            cinematic_camera: None,
            shot_sequence: None,
        };

        let v = timeline
            .sample_keyframe_value("camera.position", 5.0)
            .expect("keyframe sample");
        match v {
            TimelineValue::Vec3 { value } => assert!(value.y > 0.2),
            _ => panic!("expected vec3"),
        }

        assert!(timeline.event_strength(AudioSyncHook::Kick, 2.0) > 0.9);
    }
}
