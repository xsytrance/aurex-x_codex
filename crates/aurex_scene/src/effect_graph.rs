use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    Scene, SdfMaterialType, SdfModifier,
    fields::SceneField,
    generators::{
        CircuitBoardGenerator, FractalTempleGenerator, ParticleGalaxyGenerator, SceneGenerator,
        TunnelGenerator,
    },
    patterns::{PatternNetwork, PatternPreset},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectNodeId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectConnection {
    pub from: EffectNodeId,
    pub to: EffectNodeId,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EffectContext {
    pub time_seconds: f32,
    pub seed: u32,
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub tempo: f32,
    pub beat_phase: f32,
}

impl Default for EffectContext {
    fn default() -> Self {
        Self {
            time_seconds: 0.0,
            seed: 0,
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            tempo: 120.0,
            beat_phase: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectGraph {
    #[serde(default)]
    pub nodes: Vec<EffectNode>,
    #[serde(default)]
    pub connections: Vec<EffectConnection>,
}

impl EffectGraph {
    pub fn evaluate_scene(&self, scene: &mut Scene, ctx: EffectContext) {
        let mut nodes = self.nodes.clone();
        nodes.sort_by_key(|n| n.id);
        for node in &nodes {
            node.evaluate(scene, ctx);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectNode {
    pub id: EffectNodeId,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub parameters: BTreeMap<String, f32>,
    pub node: EffectNodeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EffectNodeKind {
    FractalTempleGenerator,
    TunnelGenerator,
    CircuitBoardGenerator,
    ParticleGalaxyGenerator,
    TwistModifier,
    RepeatModifier,
    WarpModifier,
    ScaleModifier,
    PatternNetworkNode,
    PatternPresetNode,
    SpatialFieldNode,
    HarmonicFieldNode,
    RhythmFieldNode,
    MaterialNode,
    SpectralMaterialNode,
    LightingNode,
    VolumetricLightingNode,
    BloomNode,
    ToneMapNode,
    ColorShiftNode,
}

impl EffectNode {
    pub fn evaluate(&self, scene: &mut Scene, ctx: EffectContext) {
        let param = |k: &str, d: f32| self.parameters.get(k).copied().unwrap_or(d);
        match self.node {
            EffectNodeKind::FractalTempleGenerator => {
                scene.sdf.generator = Some(SceneGenerator::FractalTemple(FractalTempleGenerator {
                    grid_size: 6,
                    pillar_height: 2.6,
                    pillar_spacing: 1.0,
                    fractal_scale: 1.2,
                }));
            }
            EffectNodeKind::TunnelGenerator => {
                scene.sdf.generator = Some(SceneGenerator::Tunnel(TunnelGenerator {
                    radius: 1.8,
                    segment_count: 28,
                    twist: 0.35,
                    repeat_distance: 1.8,
                }));
            }
            EffectNodeKind::CircuitBoardGenerator => {
                scene.sdf.generator = Some(SceneGenerator::CircuitBoard(CircuitBoardGenerator {
                    grid_resolution: 18,
                    component_density: 0.55,
                    trace_width: 0.08,
                    height_variation: 0.4,
                }));
            }
            EffectNodeKind::ParticleGalaxyGenerator => {
                scene.sdf.generator =
                    Some(SceneGenerator::ParticleGalaxy(ParticleGalaxyGenerator {
                        particle_count: 300,
                        radius: 4.5,
                        noise_spread: 0.6,
                        rotation_speed: 0.45,
                    }));
            }
            EffectNodeKind::TwistModifier => {
                let strength = param("strength", 0.3) * (1.0 + ctx.beat_phase * 0.25);
                for obj in &mut scene.sdf.objects {
                    obj.modifiers.push(SdfModifier::Twist { strength });
                }
            }
            EffectNodeKind::RepeatModifier => {
                let cell = param("cell", 6.0);
                for obj in &mut scene.sdf.objects {
                    obj.modifiers.push(SdfModifier::Repeat {
                        cell: crate::Vec3::new(cell, cell, cell),
                    });
                }
            }
            EffectNodeKind::WarpModifier => {
                let amp = param("amplitude", 0.25) * (1.0 + ctx.mid_energy * 0.5);
                for obj in &mut scene.sdf.objects {
                    obj.modifiers.push(SdfModifier::NoiseDisplacement {
                        amplitude: amp,
                        frequency: param("frequency", 1.4),
                        seed: ctx.seed,
                    });
                }
            }
            EffectNodeKind::ScaleModifier => {
                let factor = param("factor", 1.0).max(0.1);
                for obj in &mut scene.sdf.objects {
                    obj.modifiers.push(SdfModifier::Scale { factor });
                    obj.bounds_radius = obj.bounds_radius.map(|r| r * factor).or(Some(factor));
                }
            }
            EffectNodeKind::PatternNetworkNode => {
                if let Some(network) = scene.sdf.patterns.first().cloned() {
                    for obj in &mut scene.sdf.objects {
                        obj.material.pattern_network = Some(network.clone());
                    }
                }
            }
            EffectNodeKind::PatternPresetNode => {
                let preset = PatternPreset::ElectronicCircuit;
                let network = PatternNetwork {
                    name: Some("effect-graph-preset".to_string()),
                    preset: Some(preset),
                    layers: vec![],
                };
                scene.sdf.patterns.push(network);
            }
            EffectNodeKind::SpatialFieldNode => {
                scene
                    .sdf
                    .fields
                    .push(SceneField::Noise(crate::fields::NoiseField {
                        scale: param("scale", 1.2),
                        octaves: param("octaves", 4.0).round().clamp(1.0, 8.0) as u32,
                        strength: param("strength", 0.3),
                        speed: param("speed", 0.2),
                    }));
            }
            EffectNodeKind::HarmonicFieldNode => {
                scene
                    .sdf
                    .fields
                    .push(SceneField::Pulse(crate::fields::PulseField {
                        origin: crate::Vec3::new(0.0, 0.0, 0.0),
                        frequency: param("frequency", 2.0),
                        amplitude: param("amplitude", 0.15),
                        falloff: param("falloff", 0.4),
                    }));
            }
            EffectNodeKind::RhythmFieldNode => {
                scene
                    .sdf
                    .fields
                    .push(SceneField::Flow(crate::fields::FlowField {
                        direction: crate::Vec3::new(0.0, 1.0, 0.0),
                        turbulence: param("turbulence", 0.3),
                        strength: param("strength", 0.2),
                    }));
            }
            EffectNodeKind::MaterialNode => {
                for obj in &mut scene.sdf.objects {
                    obj.material.roughness = param("roughness", obj.material.roughness);
                    obj.material.emissive_strength =
                        param("emission", obj.material.emissive_strength);
                }
            }
            EffectNodeKind::SpectralMaterialNode => {
                for obj in &mut scene.sdf.objects {
                    obj.material.material_type = SdfMaterialType::SpectralReactive;
                    obj.material
                        .parameters
                        .insert("effect_high_boost".into(), ctx.high_energy);
                }
            }
            EffectNodeKind::LightingNode => {
                let boost = param("intensity", 1.0) * (1.0 + ctx.bass_energy * 0.2);
                for light in &mut scene.sdf.lighting.key_lights {
                    light.intensity *= boost;
                }
            }
            EffectNodeKind::VolumetricLightingNode => {
                scene.sdf.lighting.volumetric.beam_density =
                    param("beam_density", scene.sdf.lighting.volumetric.beam_density);
                scene.sdf.lighting.volumetric.shaft_intensity = param(
                    "shaft_intensity",
                    scene.sdf.lighting.volumetric.shaft_intensity,
                );
            }
            EffectNodeKind::BloomNode => {
                for obj in &mut scene.sdf.objects {
                    obj.material.emissive_strength *= 1.0 + param("bloom_boost", 0.15);
                }
            }
            EffectNodeKind::ToneMapNode => {
                scene.sdf.lighting.fog_density *= (1.0 - param("fog_cut", 0.05)).clamp(0.0, 1.0);
            }
            EffectNodeKind::ColorShiftNode => {
                let s = param("shift", 0.05) * (ctx.time_seconds * 0.3).sin().abs();
                for obj in &mut scene.sdf.objects {
                    obj.material.base_color.x = (obj.material.base_color.x + s).clamp(0.0, 1.0);
                    obj.material.base_color.z =
                        (obj.material.base_color.z + s * 0.7).clamp(0.0, 1.0);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GraphMorphStrategy {
    NodeParameterBlend,
    DistanceFieldBlend,
    PatternCrossfade,
    HarmonicPhaseBlend,
    GeneratorMorph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphMorphSpec {
    pub strategy: GraphMorphStrategy,
    pub duration: f32,
    #[serde(default = "default_morph_intensity")]
    pub intensity: f32,
}

fn default_morph_intensity() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GraphMorphState {
    pub progress: f32,
}

#[derive(Debug, Clone, Default)]
pub struct GraphMorph;

impl GraphMorph {
    pub fn morph(
        graph_a: &EffectGraph,
        graph_b: &EffectGraph,
        spec: &GraphMorphSpec,
        state: GraphMorphState,
    ) -> EffectGraph {
        let p = state.progress.clamp(0.0, 1.0) * spec.intensity.max(0.0);
        let mut out_nodes: Vec<EffectNode> = Vec::new();

        let mut a_nodes = graph_a.nodes.clone();
        let mut b_nodes = graph_b.nodes.clone();
        a_nodes.sort_by_key(|n| n.id);
        b_nodes.sort_by_key(|n| n.id);

        for node_a in &a_nodes {
            let node_b = b_nodes.iter().find(|n| n.id == node_a.id);
            let mut mixed = node_a.clone();
            if let Some(node_b) = node_b {
                for (k, v_a) in &node_a.parameters {
                    let v_b = node_b.parameters.get(k).copied().unwrap_or(*v_a);
                    mixed
                        .parameters
                        .insert(k.clone(), blend_param(*v_a, v_b, p, spec.strategy));
                }
                for (k, v_b) in &node_b.parameters {
                    mixed.parameters.entry(k.clone()).or_insert(*v_b);
                }
            }
            out_nodes.push(mixed);
        }

        for node_b in &b_nodes {
            if !out_nodes.iter().any(|n| n.id == node_b.id) {
                out_nodes.push(node_b.clone());
            }
        }

        out_nodes.sort_by_key(|n| n.id);

        let mut connections = graph_a.connections.clone();
        for c in &graph_b.connections {
            if !connections.iter().any(|e| e.from == c.from && e.to == c.to) {
                connections.push(c.clone());
            }
        }

        EffectGraph {
            nodes: out_nodes,
            connections,
        }
    }
}

fn blend_param(a: f32, b: f32, p: f32, strategy: GraphMorphStrategy) -> f32 {
    match strategy {
        GraphMorphStrategy::NodeParameterBlend => a + (b - a) * p,
        GraphMorphStrategy::DistanceFieldBlend => (a * (1.0 - p) + b * p).tanh(),
        GraphMorphStrategy::PatternCrossfade => a * (1.0 - p).powf(1.5) + b * p,
        GraphMorphStrategy::HarmonicPhaseBlend => {
            let phase = p * std::f32::consts::PI;
            a * phase.cos().abs() + b * phase.sin().abs()
        }
        GraphMorphStrategy::GeneratorMorph => {
            let curved = p * p * (3.0 - 2.0 * p);
            a + (b - a) * curved
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeyLight, Scene, SdfCamera, SdfLighting, SdfObject, SdfPrimitive, SdfScene, Vec3};

    #[test]
    fn effect_graph_evaluation_is_deterministic() {
        let graph = EffectGraph {
            nodes: vec![
                EffectNode {
                    id: EffectNodeId(2),
                    name: "spectral".into(),
                    inputs: vec![],
                    outputs: vec![],
                    parameters: BTreeMap::new(),
                    node: EffectNodeKind::SpectralMaterialNode,
                },
                EffectNode {
                    id: EffectNodeId(1),
                    name: "warp".into(),
                    inputs: vec![],
                    outputs: vec![],
                    parameters: BTreeMap::from([("amplitude".into(), 0.22)]),
                    node: EffectNodeKind::WarpModifier,
                },
            ],
            connections: vec![],
        };
        let base = Scene {
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
                    fog_color: Vec3::new(0.1, 0.1, 0.1),
                    fog_density: 0.03,
                    fog_height_falloff: 0.05,
                    volumetric: Default::default(),
                },
                seed: 9,
                objects: vec![SdfObject {
                    primitive: SdfPrimitive::Sphere { radius: 1.0 },
                    modifiers: vec![],
                    material: Default::default(),
                    bounds_radius: None,
                }],
                root: Default::default(),
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
            },
        };
        let mut a = base.clone();
        let mut b = base;
        let ctx = EffectContext {
            time_seconds: 1.0,
            seed: 42,
            bass_energy: 0.3,
            mid_energy: 0.2,
            high_energy: 0.1,
            tempo: 132.0,
            beat_phase: 0.5,
        };
        graph.evaluate_scene(&mut a, ctx);
        graph.evaluate_scene(&mut b, ctx);
        assert_eq!(a, b);
    }

    #[test]
    fn graph_morph_is_stable() {
        let g1 = EffectGraph {
            nodes: vec![EffectNode {
                id: EffectNodeId(1),
                name: "a".into(),
                inputs: vec![],
                outputs: vec![],
                parameters: BTreeMap::from([("x".into(), 0.2)]),
                node: EffectNodeKind::BloomNode,
            }],
            connections: vec![],
        };
        let g2 = EffectGraph {
            nodes: vec![EffectNode {
                id: EffectNodeId(1),
                name: "a".into(),
                inputs: vec![],
                outputs: vec![],
                parameters: BTreeMap::from([("x".into(), 0.8)]),
                node: EffectNodeKind::BloomNode,
            }],
            connections: vec![],
        };
        let spec = GraphMorphSpec {
            strategy: GraphMorphStrategy::GeneratorMorph,
            duration: 2.0,
            intensity: 1.0,
        };
        let a = GraphMorph::morph(&g1, &g2, &spec, GraphMorphState { progress: 0.5 });
        let b = GraphMorph::morph(&g1, &g2, &spec, GraphMorphState { progress: 0.5 });
        assert_eq!(a, b);
    }
}
