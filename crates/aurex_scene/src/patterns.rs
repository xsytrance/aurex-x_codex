use crate::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PatternSpace {
    #[default]
    World,
    Local,
    Surface,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PatternReactiveSource {
    #[default]
    None,
    Low,
    Mid,
    High,
    DominantFrequency,
    Beat,
    Measure,
    Phrase,
    Tempo,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PatternBinding {
    #[serde(default)]
    pub space: PatternSpace,
    #[serde(default)]
    pub react_to: PatternReactiveSource,
    #[serde(default = "default_binding_strength")]
    pub strength: f32,
}

fn default_binding_strength() -> f32 {
    1.0
}

impl Default for PatternBinding {
    fn default() -> Self {
        Self {
            space: PatternSpace::World,
            react_to: PatternReactiveSource::None,
            strength: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PatternParams {
    pub scale: f32,
    pub thickness: f32,
    pub contrast: f32,
    pub density: f32,
    pub rotation: f32,
    pub distortion: f32,
    pub seed: u32,
}

impl Default for PatternParams {
    fn default() -> Self {
        Self {
            scale: 1.0,
            thickness: 0.1,
            contrast: 1.0,
            density: 1.0,
            rotation: 0.0,
            distortion: 0.0,
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternNode {
    GridPattern(PatternParams),
    HexPattern(PatternParams),
    CircuitTracePattern(PatternParams),
    WaveformPattern(PatternParams),
    ConcentricPulsePattern(PatternParams),
    GlyphStripePattern(PatternParams),
    SpiralPattern(PatternParams),
    FractalVeinPattern(PatternParams),
    LatticePattern(PatternParams),
    MosaicPattern(PatternParams),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PatternComposeOp {
    Add,
    Multiply,
    Mask,
    #[default]
    Blend,
    Max,
    Min,
    Invert,
    Warp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternLayer {
    pub node: PatternNode,
    #[serde(default)]
    pub op: PatternComposeOp,
    #[serde(default = "default_layer_weight")]
    pub weight: f32,
    #[serde(default)]
    pub binding: PatternBinding,
}

fn default_layer_weight() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternPreset {
    ElectronicCircuit,
    PsySpiral,
    PrimePulseTemple,
    JazzLoungeGlow,
    OperaCathedral,
    ReggaeSunwave,
    ClassicalOrnament,
    HipHopSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PatternNetwork {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub preset: Option<PatternPreset>,
    #[serde(default)]
    pub layers: Vec<PatternLayer>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatternContext {
    pub low_freq_energy: f32,
    pub mid_freq_energy: f32,
    pub high_freq_energy: f32,
    pub dominant_frequency: f32,
    pub current_beat: u32,
    pub current_measure: u32,
    pub current_phrase: u32,
    pub beat_phase: f32,
    pub tempo: f32,
}

impl Default for PatternContext {
    fn default() -> Self {
        Self {
            low_freq_energy: 0.0,
            mid_freq_energy: 0.0,
            high_freq_energy: 0.0,
            dominant_frequency: 0.0,
            current_beat: 0,
            current_measure: 0,
            current_phrase: 0,
            beat_phase: 0.0,
            tempo: 120.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatternSample {
    pub value: f32,
    pub distortion: f32,
}

pub fn preset_network(p: PatternPreset) -> PatternNetwork {
    let mk = |node, op| PatternLayer {
        node,
        op,
        weight: 1.0,
        binding: PatternBinding::default(),
    };

    let layers = match p {
        PatternPreset::ElectronicCircuit => vec![
            mk(
                PatternNode::CircuitTracePattern(PatternParams {
                    scale: 3.2,
                    density: 1.3,
                    thickness: 0.08,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::GridPattern(PatternParams {
                    scale: 5.0,
                    thickness: 0.04,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Add,
            ),
        ],
        PatternPreset::PsySpiral => vec![
            mk(
                PatternNode::SpiralPattern(PatternParams {
                    scale: 2.2,
                    distortion: 0.4,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::WaveformPattern(PatternParams {
                    scale: 3.4,
                    contrast: 1.4,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Warp,
            ),
        ],
        PatternPreset::PrimePulseTemple => vec![
            mk(
                PatternNode::ConcentricPulsePattern(PatternParams {
                    scale: 2.0,
                    density: 1.2,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::GlyphStripePattern(PatternParams {
                    scale: 4.4,
                    thickness: 0.07,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Add,
            ),
        ],
        PatternPreset::JazzLoungeGlow => vec![
            mk(
                PatternNode::WaveformPattern(PatternParams {
                    scale: 1.8,
                    distortion: 0.2,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::MosaicPattern(PatternParams {
                    scale: 3.0,
                    thickness: 0.12,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Mask,
            ),
        ],
        PatternPreset::OperaCathedral => vec![
            mk(
                PatternNode::classical_ornament_params(),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::ConcentricPulsePattern(PatternParams {
                    scale: 1.3,
                    contrast: 1.2,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Add,
            ),
        ],
        PatternPreset::ReggaeSunwave => vec![
            mk(
                PatternNode::WaveformPattern(PatternParams {
                    scale: 2.4,
                    density: 0.9,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::GlyphStripePattern(PatternParams {
                    scale: 3.0,
                    rotation: 0.3,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Multiply,
            ),
        ],
        PatternPreset::ClassicalOrnament => vec![
            mk(
                PatternNode::classical_ornament_params(),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::LatticePattern(PatternParams {
                    scale: 3.8,
                    thickness: 0.05,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Add,
            ),
        ],
        PatternPreset::HipHopSignal => vec![
            mk(
                PatternNode::CircuitTracePattern(PatternParams {
                    scale: 2.6,
                    thickness: 0.12,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Blend,
            ),
            mk(
                PatternNode::WaveformPattern(PatternParams {
                    scale: 4.0,
                    contrast: 1.5,
                    density: 1.2,
                    ..PatternParams::default()
                }),
                PatternComposeOp::Add,
            ),
        ],
    };

    PatternNetwork {
        name: Some(format!("{:?}", p)),
        preset: Some(p),
        layers,
    }
}

impl PatternNode {
    fn classical_ornament_params() -> Self {
        PatternNode::FractalVeinPattern(PatternParams {
            scale: 3.1,
            thickness: 0.06,
            contrast: 1.25,
            density: 1.1,
            rotation: 0.4,
            distortion: 0.35,
            seed: 19,
        })
    }
}

pub fn sample_network(
    net: &PatternNetwork,
    world_pos: Vec3,
    local_pos: Vec3,
    surface_uv: Vec3,
    time: f32,
    seed: u32,
    ctx: PatternContext,
) -> PatternSample {
    let mut composed = 0.0;
    let mut distort = 0.0;

    let layers = if net.layers.is_empty() {
        net.preset
            .map(preset_network)
            .map(|n| n.layers)
            .unwrap_or_default()
    } else {
        net.layers.clone()
    };

    for layer in layers {
        let src_pos = match layer.binding.space {
            PatternSpace::World => world_pos,
            PatternSpace::Local => local_pos,
            PatternSpace::Surface => surface_uv,
        };
        let react = reactive_gain(layer.binding.react_to, ctx) * layer.binding.strength;
        let v = sample_node(&layer.node, src_pos, time, seed, react);
        distort += v * 0.08;
        composed = compose(composed, v * layer.weight, layer.op);
    }

    PatternSample {
        value: composed.clamp(0.0, 1.0),
        distortion: distort.clamp(-1.0, 1.0),
    }
}

fn reactive_gain(src: PatternReactiveSource, ctx: PatternContext) -> f32 {
    match src {
        PatternReactiveSource::None => 1.0,
        PatternReactiveSource::Low => 1.0 + ctx.low_freq_energy,
        PatternReactiveSource::Mid => 1.0 + ctx.mid_freq_energy,
        PatternReactiveSource::High => 1.0 + ctx.high_freq_energy,
        PatternReactiveSource::DominantFrequency => 1.0 + (ctx.dominant_frequency / 4000.0),
        PatternReactiveSource::Beat => 1.0 + (1.0 - ctx.beat_phase),
        PatternReactiveSource::Measure => 1.0 + ((ctx.current_measure % 4) as f32) * 0.25,
        PatternReactiveSource::Phrase => 1.0 + ((ctx.current_phrase % 4) as f32) * 0.35,
        PatternReactiveSource::Tempo => 1.0 + (ctx.tempo / 200.0),
    }
}

fn compose(a: f32, b: f32, op: PatternComposeOp) -> f32 {
    match op {
        PatternComposeOp::Add => (a + b).clamp(0.0, 1.0),
        PatternComposeOp::Multiply => (a * b).clamp(0.0, 1.0),
        PatternComposeOp::Mask => (a * b).clamp(0.0, 1.0),
        PatternComposeOp::Blend => (a * 0.5 + b * 0.5).clamp(0.0, 1.0),
        PatternComposeOp::Max => a.max(b),
        PatternComposeOp::Min => a.min(b),
        PatternComposeOp::Invert => (1.0 - b).clamp(0.0, 1.0),
        PatternComposeOp::Warp => (a + (b - 0.5) * 0.35).clamp(0.0, 1.0),
    }
}

fn sample_node(node: &PatternNode, p: Vec3, t: f32, seed: u32, react: f32) -> f32 {
    let params = match node {
        PatternNode::GridPattern(v)
        | PatternNode::HexPattern(v)
        | PatternNode::CircuitTracePattern(v)
        | PatternNode::WaveformPattern(v)
        | PatternNode::ConcentricPulsePattern(v)
        | PatternNode::GlyphStripePattern(v)
        | PatternNode::SpiralPattern(v)
        | PatternNode::FractalVeinPattern(v)
        | PatternNode::LatticePattern(v)
        | PatternNode::MosaicPattern(v) => *v,
    };

    let rp = rotate2(p.x, p.z, params.rotation);
    let x = rp.0 * params.scale.max(0.001);
    let z = rp.1 * params.scale.max(0.001);
    let y = p.y * params.scale.max(0.001);
    let d = hash3(x + t * params.distortion, y, z, seed ^ params.seed);

    let base = match node {
        PatternNode::GridPattern(_) => {
            let gx = ((x.fract() - 0.5).abs() * 2.0).clamp(0.0, 1.0);
            let gz = ((z.fract() - 0.5).abs() * 2.0).clamp(0.0, 1.0);
            1.0 - ((gx.min(gz) - params.thickness).max(0.0) / (1.0 - params.thickness).max(1e-4))
        }
        PatternNode::HexPattern(_) => {
            let qx = (x * 0.57735 - z * 0.33333).fract() - 0.5;
            let qz = (z * 0.66666).fract() - 0.5;
            let edge = (qx.abs() + qz.abs() * 0.8).clamp(0.0, 1.0);
            1.0 - edge
        }
        PatternNode::CircuitTracePattern(_) => {
            let line = (x * params.density)
                .sin()
                .abs()
                .min((z * params.density * 1.3).sin().abs());
            (1.0 - (line - params.thickness).max(0.0)).clamp(0.0, 1.0)
        }
        PatternNode::WaveformPattern(_) => {
            ((x + t * 0.9).sin() * 0.5 + (z * 1.7 - t * 0.4).sin() * 0.5 + 1.0) * 0.5
        }
        PatternNode::ConcentricPulsePattern(_) => {
            let r = (x * x + z * z).sqrt();
            ((r * params.density - t * react).sin() * 0.5 + 0.5).powf(1.0 + params.contrast)
        }
        PatternNode::GlyphStripePattern(_) => {
            ((x * params.density).sin().abs().powf(1.0 + params.contrast))
                * ((z * params.density * 0.5 + t * 0.2).cos().abs())
        }
        PatternNode::SpiralPattern(_) => {
            let ang = z.atan2(x);
            let r = (x * x + z * z).sqrt();
            (ang * params.density + r * 2.0 - t * react).sin() * 0.5 + 0.5
        }
        PatternNode::FractalVeinPattern(_) => {
            let mut v = 0.0;
            let mut amp = 0.5;
            let mut fx = x;
            let mut fz = z;
            for i in 0..4u32 {
                let n = hash3(fx, y, fz, seed ^ params.seed ^ (i * 53));
                v += n.abs() * amp;
                fx *= 1.9;
                fz *= 2.1;
                amp *= 0.5;
            }
            (1.0 - (v - params.thickness).abs()).clamp(0.0, 1.0)
        }
        PatternNode::LatticePattern(_) => {
            let a = (x * params.density).sin().abs();
            let b = (y * params.density * 0.7).sin().abs();
            let c = (z * params.density).sin().abs();
            1.0 - (a.min(b).min(c) - params.thickness).max(0.0)
        }
        PatternNode::MosaicPattern(_) => {
            let cx = (x * params.density).floor();
            let cz = (z * params.density).floor();
            hash3(cx, y.floor(), cz, seed ^ params.seed).abs()
        }
    };

    ((base + d * params.distortion) * params.contrast * react).clamp(0.0, 1.0)
}

fn rotate2(x: f32, y: f32, r: f32) -> (f32, f32) {
    let c = r.cos();
    let s = r.sin();
    (x * c - y * s, x * s + y * c)
}

fn hash3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let v = (x * 127.1 + y * 311.7 + z * 74.7 + seed as f32 * 19.19).sin() * 43_758.547;
    v.fract() * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::{
        PatternBinding, PatternComposeOp, PatternContext, PatternLayer, PatternNetwork,
        PatternNode, PatternParams, PatternReactiveSource, PatternSample, PatternSpace,
        sample_network,
    };
    use crate::Vec3;

    #[test]
    fn deterministic_pattern_sampling() {
        let net = PatternNetwork {
            name: Some("det".into()),
            preset: None,
            layers: vec![PatternLayer {
                node: PatternNode::GridPattern(PatternParams::default()),
                op: PatternComposeOp::Blend,
                weight: 1.0,
                binding: PatternBinding::default(),
            }],
        };
        let c = PatternContext::default();
        let a = sample_network(
            &net,
            Vec3::new(1.0, 0.5, -2.0),
            Vec3::new(1.0, 0.5, -2.0),
            Vec3::new(0.3, 0.2, 0.0),
            1.2,
            42,
            c,
        );
        let b = sample_network(
            &net,
            Vec3::new(1.0, 0.5, -2.0),
            Vec3::new(1.0, 0.5, -2.0),
            Vec3::new(0.3, 0.2, 0.0),
            1.2,
            42,
            c,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn composition_ops_are_stable() {
        let net = PatternNetwork {
            name: None,
            preset: None,
            layers: vec![
                PatternLayer {
                    node: PatternNode::WaveformPattern(PatternParams::default()),
                    op: PatternComposeOp::Add,
                    weight: 1.0,
                    binding: PatternBinding::default(),
                },
                PatternLayer {
                    node: PatternNode::MosaicPattern(PatternParams {
                        scale: 2.0,
                        ..PatternParams::default()
                    }),
                    op: PatternComposeOp::Mask,
                    weight: 0.8,
                    binding: PatternBinding {
                        space: PatternSpace::Surface,
                        react_to: PatternReactiveSource::Beat,
                        strength: 1.0,
                    },
                },
            ],
        };

        let out: PatternSample = sample_network(
            &net,
            Vec3::new(0.8, 0.2, 1.4),
            Vec3::new(0.8, 0.2, 1.4),
            Vec3::new(0.21, 0.73, 0.0),
            0.9,
            77,
            PatternContext {
                beat_phase: 0.25,
                ..PatternContext::default()
            },
        );

        assert!(out.value >= 0.0 && out.value <= 1.0);
        assert!(out.distortion >= -1.0 && out.distortion <= 1.0);
    }
}
