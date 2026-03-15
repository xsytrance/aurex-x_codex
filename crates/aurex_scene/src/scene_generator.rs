use crate::{
    Scene, SdfCamera, SdfLighting, SdfMaterial, SdfMaterialType, SdfNode, SdfObject, SdfPrimitive,
    SdfScene, Vec3,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PulseBlueprint {
    pub bpm: f32,
    pub beat_ticks: Vec<u64>,
    pub energy_level: f32,
    pub pitch_span: u8,
    pub density_level: f32,
}

pub fn generate_scene_from_blueprint(blueprint: &PulseBlueprint) -> Scene {
    let pillar_count = pillar_count_from_density(blueprint.density_level);
    let pillar_height = pillar_height_from_pitch_span(blueprint.pitch_span);
    let spacing = pillar_spacing_from_energy(blueprint.energy_level);

    let mut children = Vec::with_capacity(pillar_count + 1);
    children.push(generate_ground_plane());
    children.extend(generate_pillars(
        pillar_count,
        pillar_height,
        spacing,
        blueprint,
    ));

    let speed = animation_speed_factor(blueprint.bpm);
    let seed = seed_from_blueprint(blueprint);

    Scene {
        sdf: SdfScene {
            camera: SdfCamera {
                position: Vec3::new(0.0, 2.0 + speed * 0.6, -12.0),
                target: Vec3::new(0.0, 0.8, 0.0),
                fov_degrees: 60.0,
                aspect_ratio: 16.0 / 9.0,
            },
            lighting: SdfLighting {
                ambient_light: (0.14 + blueprint.energy_level.clamp(0.0, 1.0) * 0.28)
                    .clamp(0.05, 1.0),
                key_lights: vec![],
                fog_color: Vec3::new(0.08, 0.12, 0.18),
                fog_density: 0.01 + speed * 0.03,
                fog_height_falloff: 0.08,
                volumetric: Default::default(),
            },
            seed,
            objects: vec![],
            root: SdfNode::Union { children },
            timeline: None,
            generator: None,
            generator_stack: None,
            fields: vec![],
            patterns: vec![],
            harmonics: None,
            rhythm: None,
            audio: None,
            effect_graph: None,
            automation_tracks: vec![],
            demo_sequence: None,
            temporal_effects: vec![],
            runtime_modulation: None,
        },
    }
}

fn seed_from_blueprint(blueprint: &PulseBlueprint) -> u32 {
    let energy = (blueprint.energy_level.max(0.0) * 1000.0).round() as u64;
    let density = (blueprint.density_level.max(0.0) * 1000.0).round() as u64;
    let bpm = blueprint.bpm.max(0.0).round() as u64;
    let seed = (u64::from(blueprint.pitch_span) << 24)
        ^ (blueprint.beat_ticks.len() as u64)
        ^ (energy << 8)
        ^ (density << 4)
        ^ bpm;
    (seed & u64::from(u32::MAX)) as u32
}

fn animation_speed_factor(bpm: f32) -> f32 {
    if !bpm.is_finite() {
        return 1.0;
    }
    (bpm / 120.0).clamp(0.25, 4.0)
}

fn generate_ground_plane() -> SdfNode {
    SdfNode::Primitive {
        object: SdfObject {
            primitive: SdfPrimitive::Plane {
                normal: Vec3::new(0.0, 1.0, 0.0),
                offset: 0.0,
            },
            modifiers: vec![],
            material: SdfMaterial {
                material_type: SdfMaterialType::SolidColor,
                base_color: Vec3::new(0.18, 0.2, 0.24),
                emissive_strength: 0.0,
                ..SdfMaterial::default()
            },
            bounds_radius: Some(256.0),
        },
    }
}

fn generate_pillars(
    count: usize,
    pillar_height: f32,
    spacing: f32,
    blueprint: &PulseBlueprint,
) -> Vec<SdfNode> {
    if count == 0 {
        return Vec::new();
    }

    let side = ((count as f64).sqrt().ceil() as usize).max(1);
    let mut pillars = Vec::with_capacity(count);
    let half = (side as f32 - 1.0) * 0.5;

    for idx in 0..count {
        let gx = idx % side;
        let gz = idx / side;
        let x = (gx as f32 - half) * spacing;
        let z = (gz as f32 - half) * spacing;
        let beat_bias = (blueprint.beat_ticks.len() % 5) as f32 * 0.04;
        let emissive = (blueprint.energy_level.clamp(0.0, 2.0) * 0.3 + beat_bias).clamp(0.0, 1.0);

        pillars.push(SdfNode::Transform {
            modifiers: vec![crate::SdfModifier::Translate {
                offset: Vec3::new(x, pillar_height * 0.5, z),
            }],
            child: Box::new(SdfNode::Primitive {
                object: SdfObject {
                    primitive: SdfPrimitive::Box {
                        size: Vec3::new(0.3, pillar_height * 0.5, 0.3),
                    },
                    modifiers: vec![],
                    material: SdfMaterial {
                        material_type: SdfMaterialType::SolidColor,
                        base_color: Vec3::new(
                            0.2 + blueprint.energy_level.clamp(0.0, 1.0) * 0.5,
                            0.35,
                            0.8 - blueprint.density_level.clamp(0.0, 1.0) * 0.35,
                        ),
                        emissive_strength: emissive,
                        ..SdfMaterial::default()
                    },
                    bounds_radius: Some((pillar_height + spacing).max(1.0)),
                },
            }),
            bounds_radius: Some((pillar_height + spacing).max(1.0)),
        });
    }

    pillars
}

fn pillar_height_from_pitch_span(pitch_span: u8) -> f32 {
    1.0 + f32::from(pitch_span) * 0.05
}

fn pillar_count_from_density(density_level: f32) -> usize {
    if !density_level.is_finite() {
        return 1;
    }
    let count = (density_level.max(0.0) * 10.0).ceil() as usize;
    count.clamp(1, 64)
}

fn pillar_spacing_from_energy(energy_level: f32) -> f32 {
    if !energy_level.is_finite() {
        return 1.0;
    }
    1.0 + energy_level.clamp(0.0, 6.0)
}
