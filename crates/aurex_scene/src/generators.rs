use crate::{
    SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject, SdfPrimitive, Vec3, fields,
    patterns::{PatternPreset, preset_network},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct RhythmFieldContext {
    #[serde(default)]
    pub beat_phase: f32,
    #[serde(default)]
    pub beat_strength: f32,
    #[serde(default)]
    pub bass_energy: f32,
    #[serde(default)]
    pub harmonic_energy: f32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimeModulationContext {
    #[serde(default)]
    pub rhythm_field: Option<RhythmFieldContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratorStack {
    #[serde(default)]
    pub layers: Vec<SceneGeneratorLayerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SceneGeneratorLayerSpec {
    BaseGenerator(BaseGeneratorLayer),
    StructureLayer(StructureLayer),
    DetailLayer(DetailLayer),
    ParticleLayer(ParticleLayer),
    RhythmModulationLayer(RhythmModulationLayer),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseGeneratorLayer {
    pub generator: SceneGenerator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructureLayer {
    pub generator: SceneGenerator,
    #[serde(default = "default_layer_strength")]
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailLayer {
    pub generator: SceneGenerator,
    #[serde(default = "default_layer_strength")]
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleLayer {
    pub generator: SceneGenerator,
    #[serde(default = "default_layer_strength")]
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RhythmModulationLayer {
    pub generator: SceneGenerator,
    #[serde(default = "default_layer_strength")]
    pub beat_influence: f32,
    #[serde(default = "default_layer_strength")]
    pub bass_influence: f32,
    #[serde(default = "default_layer_strength")]
    pub harmonic_influence: f32,
}

fn default_layer_strength() -> f32 {
    1.0
}

pub trait SceneGeneratorLayer {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        runtime_context: RuntimeModulationContext,
    ) -> SdfNode;
}

impl SceneGeneratorLayer for BaseGeneratorLayer {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        _runtime_context: RuntimeModulationContext,
    ) -> SdfNode {
        expand_generator(&self.generator, scene_seed, time, scene_fields)
    }
}

impl SceneGeneratorLayer for StructureLayer {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        _runtime_context: RuntimeModulationContext,
    ) -> SdfNode {
        layer_node_with_strength(
            expand_generator(&self.generator, scene_seed, time, scene_fields),
            self.strength,
        )
    }
}

impl SceneGeneratorLayer for DetailLayer {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        _runtime_context: RuntimeModulationContext,
    ) -> SdfNode {
        layer_node_with_strength(
            expand_generator(&self.generator, scene_seed, time, scene_fields),
            self.strength * 0.6,
        )
    }
}

impl SceneGeneratorLayer for ParticleLayer {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        _runtime_context: RuntimeModulationContext,
    ) -> SdfNode {
        layer_node_with_strength(
            expand_generator(&self.generator, scene_seed, time, scene_fields),
            self.strength * 0.35,
        )
    }
}

impl SceneGeneratorLayer for RhythmModulationLayer {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        runtime_context: RuntimeModulationContext,
    ) -> SdfNode {
        let rf = runtime_context.rhythm_field.unwrap_or_default();
        let rhythm_strength = (rf.beat_strength * self.beat_influence
            + rf.bass_energy * self.bass_influence
            + rf.harmonic_energy * self.harmonic_influence)
            .clamp(0.0, 3.0);
        layer_node_with_strength(
            expand_generator(
                &self.generator,
                scene_seed,
                time + rf.beat_phase * 0.1,
                scene_fields,
            ),
            1.0 + rhythm_strength * 0.2,
        )
    }
}

impl SceneGeneratorLayer for SceneGeneratorLayerSpec {
    fn expand_layer(
        &self,
        scene_seed: u32,
        time: f32,
        scene_fields: &[crate::fields::SceneField],
        runtime_context: RuntimeModulationContext,
    ) -> SdfNode {
        match self {
            SceneGeneratorLayerSpec::BaseGenerator(layer) => {
                layer.expand_layer(scene_seed, time, scene_fields, runtime_context)
            }
            SceneGeneratorLayerSpec::StructureLayer(layer) => {
                layer.expand_layer(scene_seed, time, scene_fields, runtime_context)
            }
            SceneGeneratorLayerSpec::DetailLayer(layer) => {
                layer.expand_layer(scene_seed, time, scene_fields, runtime_context)
            }
            SceneGeneratorLayerSpec::ParticleLayer(layer) => {
                layer.expand_layer(scene_seed, time, scene_fields, runtime_context)
            }
            SceneGeneratorLayerSpec::RhythmModulationLayer(layer) => {
                layer.expand_layer(scene_seed, time, scene_fields, runtime_context)
            }
        }
    }
}

pub fn expand_generator_stack(
    stack: &GeneratorStack,
    scene_seed: u32,
    time: f32,
    scene_fields: &[crate::fields::SceneField],
    runtime_context: RuntimeModulationContext,
) -> SdfNode {
    let mut children = Vec::new();
    for layer in &stack.layers {
        let expanded = layer.expand_layer(scene_seed, time, scene_fields, runtime_context);
        if !matches!(expanded, SdfNode::Empty) {
            children.push(expanded);
        }
    }
    if children.is_empty() {
        SdfNode::Empty
    } else {
        SdfNode::Group { children }
    }
}

fn layer_node_with_strength(node: SdfNode, strength: f32) -> SdfNode {
    let s = strength.clamp(0.1, 4.0);
    SdfNode::Transform {
        modifiers: vec![SdfModifier::Scale { factor: s }],
        bounds_radius: None,
        child: Box::new(node),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SceneGenerator {
    Tunnel(TunnelGenerator),
    FractalTemple(FractalTempleGenerator),
    CircuitBoard(CircuitBoardGenerator),
    ParticleGalaxy(ParticleGalaxyGenerator),
    HarmonicParticleField(HarmonicParticleFieldGenerator),
    ElectronicCity(ElectronicCityGenerator),
    JazzImprovisation(JazzImprovisationGenerator),
    RockAmpMountain(RockAmpMountainGenerator),
    PopStageWorld(PopStageWorldGenerator),
    ReggaeIsland(ReggaeIslandGenerator),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TunnelGenerator {
    pub radius: f32,
    pub segment_count: u32,
    pub twist: f32,
    pub repeat_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FractalTempleGenerator {
    pub grid_size: u32,
    pub pillar_height: f32,
    pub pillar_spacing: f32,
    pub fractal_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CircuitBoardGenerator {
    pub grid_resolution: u32,
    pub component_density: f32,
    pub trace_width: f32,
    pub height_variation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleGalaxyGenerator {
    pub particle_count: u32,
    pub radius: f32,
    pub noise_spread: f32,
    pub rotation_speed: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HarmonicParticleMode {
    BassBursts,
    MelodySpirals,
    ChordLattice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElectronicCityGenerator {
    pub block_count: u32,
    pub tower_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JazzImprovisationGenerator {
    pub ribbon_count: u32,
    pub swing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RockAmpMountainGenerator {
    pub peak_count: u32,
    pub amp_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PopStageWorldGenerator {
    pub stage_count: u32,
    pub spotlight_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReggaeIslandGenerator {
    pub island_count: u32,
    pub wave_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarmonicParticleFieldGenerator {
    pub particle_count: u32,
    pub radius: f32,
    pub thickness: f32,
    pub mode: HarmonicParticleMode,
}

pub fn electronic_city_stack() -> GeneratorStack {
    GeneratorStack {
        layers: vec![
            SceneGeneratorLayerSpec::BaseGenerator(BaseGeneratorLayer {
                generator: SceneGenerator::ElectronicCity(ElectronicCityGenerator {
                    block_count: 48,
                    tower_height: 2.8,
                }),
            }),
            SceneGeneratorLayerSpec::StructureLayer(StructureLayer {
                generator: SceneGenerator::CircuitBoard(CircuitBoardGenerator {
                    grid_resolution: 24,
                    component_density: 0.45,
                    trace_width: 0.05,
                    height_variation: 0.8,
                }),
                strength: 0.85,
            }),
            SceneGeneratorLayerSpec::RhythmModulationLayer(RhythmModulationLayer {
                generator: SceneGenerator::ParticleGalaxy(ParticleGalaxyGenerator {
                    particle_count: 96,
                    radius: 6.5,
                    noise_spread: 0.35,
                    rotation_speed: 0.9,
                }),
                beat_influence: 1.0,
                bass_influence: 1.2,
                harmonic_influence: 0.7,
            }),
        ],
    }
}

pub fn jazz_improvisation_stack() -> GeneratorStack {
    GeneratorStack {
        layers: vec![
            SceneGeneratorLayerSpec::BaseGenerator(BaseGeneratorLayer {
                generator: SceneGenerator::JazzImprovisation(JazzImprovisationGenerator {
                    ribbon_count: 28,
                    swing: 0.82,
                }),
            }),
            SceneGeneratorLayerSpec::DetailLayer(DetailLayer {
                generator: SceneGenerator::FractalTemple(FractalTempleGenerator {
                    grid_size: 5,
                    pillar_height: 1.0,
                    pillar_spacing: 2.1,
                    fractal_scale: 0.8,
                }),
                strength: 0.55,
            }),
            SceneGeneratorLayerSpec::RhythmModulationLayer(RhythmModulationLayer {
                generator: SceneGenerator::HarmonicParticleField(HarmonicParticleFieldGenerator {
                    particle_count: 72,
                    radius: 3.5,
                    thickness: 0.2,
                    mode: HarmonicParticleMode::MelodySpirals,
                }),
                beat_influence: 0.7,
                bass_influence: 0.5,
                harmonic_influence: 1.3,
            }),
        ],
    }
}

pub fn rock_mountain_stack() -> GeneratorStack {
    GeneratorStack {
        layers: vec![
            SceneGeneratorLayerSpec::BaseGenerator(BaseGeneratorLayer {
                generator: SceneGenerator::RockAmpMountain(RockAmpMountainGenerator {
                    peak_count: 12,
                    amp_height: 4.0,
                }),
            }),
            SceneGeneratorLayerSpec::StructureLayer(StructureLayer {
                generator: SceneGenerator::Tunnel(TunnelGenerator {
                    radius: 2.0,
                    segment_count: 10,
                    twist: 0.08,
                    repeat_distance: 2.6,
                }),
                strength: 0.65,
            }),
            SceneGeneratorLayerSpec::ParticleLayer(ParticleLayer {
                generator: SceneGenerator::HarmonicParticleField(HarmonicParticleFieldGenerator {
                    particle_count: 44,
                    radius: 4.2,
                    thickness: 0.25,
                    mode: HarmonicParticleMode::BassBursts,
                }),
                strength: 0.7,
            }),
        ],
    }
}

pub fn expand_generator(
    generator: &SceneGenerator,
    scene_seed: u32,
    time: f32,
    scene_fields: &[crate::fields::SceneField],
) -> SdfNode {
    match generator {
        SceneGenerator::Tunnel(g) => expand_tunnel(g, scene_seed, time, scene_fields),
        SceneGenerator::FractalTemple(g) => expand_temple(g, scene_seed, scene_fields, time),
        SceneGenerator::CircuitBoard(g) => expand_circuit(g, scene_seed, scene_fields, time),
        SceneGenerator::ParticleGalaxy(g) => expand_galaxy(g, scene_seed, time, scene_fields),
        SceneGenerator::HarmonicParticleField(g) => {
            expand_harmonic_particle_field(g, scene_seed, time, scene_fields)
        }
        SceneGenerator::ElectronicCity(g) => expand_electronic_city(g, scene_seed, time),
        SceneGenerator::JazzImprovisation(g) => expand_jazz_improv(g, scene_seed, time),
        SceneGenerator::RockAmpMountain(g) => expand_rock_amp_mountain(g, scene_seed, time),
        SceneGenerator::PopStageWorld(g) => expand_pop_stage_world(g, scene_seed, time),
        SceneGenerator::ReggaeIsland(g) => expand_reggae_island(g, scene_seed, time),
    }
}

fn expand_tunnel(
    g: &TunnelGenerator,
    seed: u32,
    time: f32,
    scene_fields: &[crate::fields::SceneField],
) -> SdfNode {
    let mut children = Vec::new();
    for i in 0..g.segment_count.max(1) {
        let z = i as f32 * g.repeat_distance;
        let pos = Vec3::new(0.0, 0.0, z);
        let fs = fields::sample_fields(scene_fields, pos, time, seed);
        let r = g.radius * (1.0 + 0.07 * ((time * 0.8) + i as f32 * 0.37).sin()) + fs.scalar * 0.08;
        children.push(SdfNode::Transform {
            modifiers: vec![
                SdfModifier::Translate {
                    offset: Vec3::new(0.0, 0.0, z),
                },
                SdfModifier::Twist {
                    strength: g.twist * (1.0 + i as f32 * 0.02),
                },
            ],
            bounds_radius: Some(r + 1.5),
            child: Box::new(SdfNode::Primitive {
                object: SdfObject {
                    primitive: SdfPrimitive::Torus {
                        major_radius: r,
                        minor_radius: 0.16,
                    },
                    modifiers: vec![],
                    material: SdfMaterial {
                        material_type: SdfMaterialType::NeonGrid,
                        base_color: Vec3::new(0.2, 0.92, 1.0),
                        emissive_strength: 0.8,
                        roughness: 0.14,
                        pattern: crate::SdfPattern::Bands,
                        pattern_network: Some(preset_network(PatternPreset::PsySpiral)),
                        parameters: Default::default(),
                    },
                    bounds_radius: Some(r + 0.4),
                },
            }),
        });
    }

    SdfNode::Group { children }
}

fn expand_temple(
    g: &FractalTempleGenerator,
    seed: u32,
    scene_fields: &[crate::fields::SceneField],
    time: f32,
) -> SdfNode {
    let mut pillars = Vec::new();
    let half = g.grid_size as i32 / 2;
    for x in -half..=half {
        for z in -half..=half {
            if x.abs() <= 1 && z.abs() <= 1 {
                continue;
            }
            let fs =
                fields::sample_fields(scene_fields, Vec3::new(x as f32, 0.0, z as f32), time, seed);
            pillars.push(SdfNode::Transform {
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(
                        x as f32 * g.pillar_spacing,
                        -0.3,
                        z as f32 * g.pillar_spacing,
                    ),
                }],
                bounds_radius: Some(g.pillar_height + 1.0),
                child: Box::new(SdfNode::Primitive {
                    object: SdfObject {
                        primitive: SdfPrimitive::Cylinder {
                            radius: 0.2 + hash01(seed, x, z) * 0.08 + fs.energy * 0.03,
                            half_height: g.pillar_height * (1.0 + fs.scalar.abs() * 0.08),
                        },
                        modifiers: vec![],
                        material: SdfMaterial {
                            material_type: SdfMaterialType::Wireframe,
                            base_color: Vec3::new(0.95, 0.82, 0.58),
                            emissive_strength: 0.12,
                            roughness: 0.35,
                            pattern: crate::SdfPattern::Checker,
                            pattern_network: Some(preset_network(PatternPreset::PrimePulseTemple)),
                            parameters: Default::default(),
                        },
                        bounds_radius: Some(g.pillar_height + 0.4),
                    },
                }),
            });
        }
    }

    let center = SdfNode::Primitive {
        object: SdfObject {
            primitive: SdfPrimitive::Mandelbulb {
                power: 8.0,
                iterations: 12,
                bailout: 4.0 * g.fractal_scale.max(0.1),
            },
            modifiers: vec![SdfModifier::Scale {
                factor: g.fractal_scale.max(0.1),
            }],
            material: SdfMaterial {
                material_type: SdfMaterialType::FractalMetal,
                base_color: Vec3::new(0.9, 0.75, 0.45),
                emissive_strength: 0.08,
                roughness: 0.24,
                pattern: crate::SdfPattern::Rings,
                pattern_network: Some(preset_network(PatternPreset::OperaCathedral)),
                parameters: Default::default(),
            },
            bounds_radius: Some(4.0),
        },
    };

    SdfNode::Union {
        children: vec![center, SdfNode::Group { children: pillars }],
    }
}

fn expand_circuit(
    g: &CircuitBoardGenerator,
    seed: u32,
    scene_fields: &[crate::fields::SceneField],
    time: f32,
) -> SdfNode {
    let mut children = Vec::new();
    let n = g.grid_resolution.max(2) as i32;
    let half = n / 2;

    for x in -half..=half {
        for z in -half..=half {
            let fs =
                fields::sample_fields(scene_fields, Vec3::new(x as f32, 0.0, z as f32), time, seed);
            let h = (hash01(seed, x, z) + fs.energy * 0.25).clamp(0.0, 1.0);
            if h < g.component_density {
                let tower_h = 0.12 + h * g.height_variation;
                children.push(SdfNode::Transform {
                    modifiers: vec![SdfModifier::Translate {
                        offset: Vec3::new(x as f32 * 0.9, -0.2 + tower_h, z as f32 * 0.9),
                    }],
                    bounds_radius: Some(0.8),
                    child: Box::new(SdfNode::Primitive {
                        object: SdfObject {
                            primitive: if h > 0.7 {
                                SdfPrimitive::Cylinder {
                                    radius: 0.18,
                                    half_height: tower_h,
                                }
                            } else {
                                SdfPrimitive::Box {
                                    size: Vec3::new(0.26, tower_h, 0.2),
                                }
                            },
                            modifiers: vec![],
                            material: SdfMaterial {
                                material_type: SdfMaterialType::NoiseSurface,
                                base_color: Vec3::new(0.12, 0.7, 0.2),
                                emissive_strength: 0.08,
                                roughness: 0.5,
                                pattern: crate::SdfPattern::Checker,
                                pattern_network: Some(preset_network(
                                    PatternPreset::ElectronicCircuit,
                                )),
                                parameters: Default::default(),
                            },
                            bounds_radius: Some(0.6),
                        },
                    }),
                });
            }

            if h > 0.35 {
                children.push(SdfNode::Transform {
                    modifiers: vec![SdfModifier::Translate {
                        offset: Vec3::new(x as f32 * 0.9, -0.35, z as f32 * 0.9),
                    }],
                    bounds_radius: Some(0.7),
                    child: Box::new(SdfNode::Primitive {
                        object: SdfObject {
                            primitive: SdfPrimitive::Box {
                                size: Vec3::new(0.45, g.trace_width.max(0.02), 0.08),
                            },
                            modifiers: vec![],
                            material: SdfMaterial {
                                material_type: SdfMaterialType::NeonGrid,
                                base_color: Vec3::new(0.2, 0.9, 0.35),
                                emissive_strength: 0.22,
                                roughness: 0.2,
                                pattern: crate::SdfPattern::Bands,
                                pattern_network: Some(preset_network(
                                    PatternPreset::ElectronicCircuit,
                                )),
                                parameters: Default::default(),
                            },
                            bounds_radius: Some(0.5),
                        },
                    }),
                });
            }
        }
    }

    SdfNode::Group { children }
}

fn expand_galaxy(
    g: &ParticleGalaxyGenerator,
    seed: u32,
    time: f32,
    scene_fields: &[crate::fields::SceneField],
) -> SdfNode {
    let mut children = Vec::new();
    let count = g.particle_count.max(8);

    for i in 0..count {
        let a = i as f32 / count as f32 * std::f32::consts::TAU;
        let phase = a + time * g.rotation_speed;
        let base = Vec3::new((i as f32 * 0.1).cos(), 0.0, (i as f32 * 0.1).sin());
        let fs = fields::sample_fields(scene_fields, base, time, seed);
        let radial_jitter =
            1.0 + (hash01(seed, i as i32, 13) - 0.5) * g.noise_spread + fs.scalar * 0.05;
        let r = g.radius * radial_jitter.max(0.1);
        let y = (hash01(seed, i as i32, 27) - 0.5) * g.noise_spread * g.radius * 0.35;
        let pos = Vec3::new(phase.cos() * r, y, phase.sin() * r);

        children.push(SdfNode::Transform {
            modifiers: vec![SdfModifier::Translate { offset: pos }],
            bounds_radius: Some(0.4),
            child: Box::new(SdfNode::Primitive {
                object: SdfObject {
                    primitive: SdfPrimitive::Sphere {
                        radius: 0.05 + hash01(seed, i as i32, 41) * 0.1,
                    },
                    modifiers: vec![],
                    material: SdfMaterial {
                        material_type: SdfMaterialType::Plasma,
                        base_color: Vec3::new(0.7, 0.88, 1.0),
                        emissive_strength: 0.95,
                        roughness: 0.25,
                        pattern: crate::SdfPattern::Noise,
                        pattern_network: Some(preset_network(PatternPreset::JazzLoungeGlow)),
                        parameters: Default::default(),
                    },
                    bounds_radius: Some(0.25),
                },
            }),
        });
    }

    SdfNode::Group { children }
}

fn expand_harmonic_particle_field(
    g: &HarmonicParticleFieldGenerator,
    seed: u32,
    time: f32,
    scene_fields: &[crate::fields::SceneField],
) -> SdfNode {
    let count = g.particle_count.max(12);
    let mut children = Vec::new();
    for i in 0..count {
        let k = i as f32 / count as f32;
        let fs =
            fields::sample_fields(scene_fields, Vec3::new(k * 3.0 - 1.5, 0.0, 0.0), time, seed);
        let pos = match g.mode {
            HarmonicParticleMode::BassBursts => {
                let a = k * std::f32::consts::TAU;
                let burst = 1.0 + fs.energy * 0.6 + (time * 3.0 + k * 17.0).sin().abs() * 0.3;
                Vec3::new(
                    a.cos() * g.radius * burst,
                    (k * 31.0).sin() * 0.3,
                    a.sin() * g.radius * burst,
                )
            }
            HarmonicParticleMode::MelodySpirals => {
                let turns = 4.0;
                let a = k * turns * std::f32::consts::TAU + time * 1.5;
                let r = g.radius * (0.2 + 0.8 * k) * (1.0 + fs.scalar.abs() * 0.2);
                Vec3::new(a.cos() * r, (k - 0.5) * g.radius * 1.6, a.sin() * r)
            }
            HarmonicParticleMode::ChordLattice => {
                let gx = ((i % 6) as f32 - 2.5) * g.thickness.max(0.05) * 2.0;
                let gy = (((i / 6) % 6) as f32 - 2.5) * g.thickness.max(0.05) * 2.0;
                let gz = ((i / 36) as f32 - 0.5) * g.thickness.max(0.05) * 3.0;
                Vec3::new(gx, gy + fs.scalar * 0.3, gz)
            }
        };

        children.push(SdfNode::Transform {
            modifiers: vec![SdfModifier::Translate { offset: pos }],
            bounds_radius: Some(0.5),
            child: Box::new(SdfNode::Primitive {
                object: SdfObject {
                    primitive: SdfPrimitive::Sphere {
                        radius: 0.05 + 0.08 * hash01(seed, i as i32, 91),
                    },
                    modifiers: vec![],
                    material: SdfMaterial {
                        material_type: SdfMaterialType::SpectralReactive,
                        base_color: Vec3::new(0.75, 0.85, 1.0),
                        emissive_strength: 0.55,
                        roughness: 0.2,
                        pattern: crate::SdfPattern::Noise,
                        pattern_network: Some(preset_network(PatternPreset::HipHopSignal)),
                        parameters: Default::default(),
                    },
                    bounds_radius: Some(0.25),
                },
            }),
        });
    }

    SdfNode::Group { children }
}

fn hash01(seed: u32, x: i32, y: i32) -> f32 {
    let mut n = seed as i32;
    n ^= x.wrapping_mul(374_761_393);
    n ^= y.wrapping_mul(668_265_263);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n & 0x7fff_ffff) as f32) / 2_147_483_647.0
}

fn expand_electronic_city(g: &ElectronicCityGenerator, seed: u32, time: f32) -> SdfNode {
    let mut children = Vec::new();
    for i in 0..g.block_count.max(1) {
        let x = (i as f32 - g.block_count as f32 * 0.5) * 1.6;
        let h = g.tower_height * (0.5 + ((seed as f32 * 0.01 + i as f32 + time).sin().abs()));
        children.push(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Box {
                    size: Vec3::new(0.35, h, 0.35),
                },
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(x, h, 0.0),
                }],
                material: SdfMaterial::default(),
                bounds_radius: Some(h + 1.0),
            },
        });
    }
    SdfNode::Group { children }
}

fn expand_jazz_improv(g: &JazzImprovisationGenerator, seed: u32, time: f32) -> SdfNode {
    let mut children = Vec::new();
    for i in 0..g.ribbon_count.max(1) {
        let z = i as f32 * 1.2;
        children.push(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Torus {
                    major_radius: 1.0 + (i as f32 * 0.05),
                    minor_radius: 0.08 + g.swing.abs() * 0.04,
                },
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(
                        (time * g.swing + i as f32 + seed as f32 * 0.001).sin(),
                        0.0,
                        z,
                    ),
                }],
                material: SdfMaterial::default(),
                bounds_radius: Some(2.0),
            },
        });
    }
    SdfNode::Group { children }
}

fn expand_rock_amp_mountain(g: &RockAmpMountainGenerator, _seed: u32, _time: f32) -> SdfNode {
    let mut children = Vec::new();
    for i in 0..g.peak_count.max(1) {
        let x = (i as f32 - g.peak_count as f32 * 0.5) * 2.5;
        children.push(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Capsule {
                    a: Vec3::new(x, 0.0, 0.0),
                    b: Vec3::new(x, g.amp_height, 0.0),
                    radius: 0.9,
                },
                modifiers: vec![],
                material: SdfMaterial::default(),
                bounds_radius: Some(g.amp_height + 1.5),
            },
        });
    }
    SdfNode::Group { children }
}

fn expand_pop_stage_world(g: &PopStageWorldGenerator, _seed: u32, _time: f32) -> SdfNode {
    let mut children = Vec::new();
    for i in 0..g.stage_count.max(1) {
        let z = i as f32 * 2.2;
        children.push(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Cylinder {
                    radius: g.spotlight_radius.max(0.2),
                    half_height: 0.12,
                },
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(0.0, 0.0, z),
                }],
                material: SdfMaterial::default(),
                bounds_radius: Some(2.0),
            },
        });
    }
    SdfNode::Group { children }
}

fn expand_reggae_island(g: &ReggaeIslandGenerator, seed: u32, time: f32) -> SdfNode {
    let mut children = Vec::new();
    for i in 0..g.island_count.max(1) {
        let x = (i as f32 - g.island_count as f32 * 0.5) * 3.0;
        let y = (time * 0.5 + i as f32 + seed as f32 * 0.002).sin() * g.wave_scale;
        children.push(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Sphere {
                    radius: 1.0 + g.wave_scale * 0.4,
                },
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(x, y, 0.0),
                }],
                material: SdfMaterial::default(),
                bounds_radius: Some(2.0),
            },
        });
    }
    SdfNode::Group { children }
}

#[cfg(test)]
mod tests {
    use super::{
        CircuitBoardGenerator, FractalTempleGenerator, HarmonicParticleFieldGenerator,
        HarmonicParticleMode, ParticleGalaxyGenerator, RuntimeModulationContext, SceneGenerator,
        TunnelGenerator, electronic_city_stack, expand_generator, expand_generator_stack,
        jazz_improvisation_stack, rock_mountain_stack,
    };

    #[test]
    fn generator_expansion_is_deterministic() {
        let g = SceneGenerator::ParticleGalaxy(ParticleGalaxyGenerator {
            particle_count: 32,
            radius: 4.0,
            noise_spread: 0.5,
            rotation_speed: 1.2,
        });
        let a = expand_generator(&g, 42, 1.0, &[]);
        let b = expand_generator(&g, 42, 1.0, &[]);
        assert_eq!(a, b);
    }

    #[test]
    fn all_generators_expand() {
        let gs = vec![
            SceneGenerator::Tunnel(TunnelGenerator {
                radius: 1.8,
                segment_count: 8,
                twist: 0.1,
                repeat_distance: 2.2,
            }),
            SceneGenerator::FractalTemple(FractalTempleGenerator {
                grid_size: 5,
                pillar_height: 1.2,
                pillar_spacing: 2.0,
                fractal_scale: 1.0,
            }),
            SceneGenerator::CircuitBoard(CircuitBoardGenerator {
                grid_resolution: 10,
                component_density: 0.45,
                trace_width: 0.06,
                height_variation: 0.6,
            }),
            SceneGenerator::ParticleGalaxy(ParticleGalaxyGenerator {
                particle_count: 64,
                radius: 5.0,
                noise_spread: 0.4,
                rotation_speed: 0.8,
            }),
            SceneGenerator::HarmonicParticleField(HarmonicParticleFieldGenerator {
                particle_count: 48,
                radius: 3.2,
                thickness: 0.2,
                mode: HarmonicParticleMode::MelodySpirals,
            }),
        ];
        for g in gs {
            let node = expand_generator(&g, 7, 0.5, &[]);
            assert!(!matches!(node, crate::SdfNode::Empty));
        }
    }

    #[test]
    fn stack_expansion_is_deterministic() {
        let stack = electronic_city_stack();
        let ctx = RuntimeModulationContext::default();
        let a = expand_generator_stack(&stack, 101, 1.5, &[], ctx);
        let b = expand_generator_stack(&stack, 101, 1.5, &[], ctx);
        assert_eq!(a, b);
    }

    #[test]
    fn example_stacks_expand() {
        let stacks = vec![
            electronic_city_stack(),
            jazz_improvisation_stack(),
            rock_mountain_stack(),
        ];
        for stack in stacks {
            let node =
                expand_generator_stack(&stack, 13, 0.25, &[], RuntimeModulationContext::default());
            assert!(!matches!(node, crate::SdfNode::Empty));
        }
    }
}
