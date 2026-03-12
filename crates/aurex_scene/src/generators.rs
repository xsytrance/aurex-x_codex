use crate::{
    SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject, SdfPrimitive, Vec3, fields,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SceneGenerator {
    Tunnel(TunnelGenerator),
    FractalTemple(FractalTempleGenerator),
    CircuitBoard(CircuitBoardGenerator),
    ParticleGalaxy(ParticleGalaxyGenerator),
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

#[cfg(test)]
mod tests {
    use super::{
        CircuitBoardGenerator, FractalTempleGenerator, ParticleGalaxyGenerator, SceneGenerator,
        TunnelGenerator, expand_generator,
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
        ];
        for g in gs {
            let node = expand_generator(&g, 7, 0.5, &[]);
            assert!(!matches!(node, crate::SdfNode::Empty));
        }
    }
}
