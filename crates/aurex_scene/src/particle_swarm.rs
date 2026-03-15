use crate::{
    Scene, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject, SdfPrimitive, Vec3,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub target: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSwarm {
    particles: Vec<Particle>,
    seed: u64,
}

impl ParticleSwarm {
    pub fn new(seed: u64, count: usize) -> Self {
        let mut particles = Vec::with_capacity(count);
        for idx in 0..count {
            let salt = idx as u64 ^ seed;
            let position = [
                sample_symmetric(seed, salt.wrapping_mul(3), 28.0),
                sample_symmetric(seed, salt.wrapping_mul(5), 14.0) + 8.0,
                sample_symmetric(seed, salt.wrapping_mul(7), 28.0),
            ];

            particles.push(Particle {
                position,
                velocity: [0.0, 0.0, 0.0],
                target: position,
            });
        }

        Self { particles, seed }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn set_targets(&mut self, targets: &[[f32; 3]]) {
        if targets.is_empty() {
            for particle in &mut self.particles {
                particle.target = particle.position;
            }
            return;
        }

        let offset = (self.seed as usize) % targets.len();
        for (idx, particle) in self.particles.iter_mut().enumerate() {
            let target_idx = (idx + offset) % targets.len();
            particle.target = targets[target_idx];
        }
    }

    pub fn clear_targets(&mut self) {
        for (idx, particle) in self.particles.iter_mut().enumerate() {
            let salt = idx as u64 ^ self.seed;
            particle.target = [
                sample_symmetric(self.seed, salt.wrapping_mul(13), 35.0),
                sample_symmetric(self.seed, salt.wrapping_mul(17), 18.0) + 6.0,
                sample_symmetric(self.seed, salt.wrapping_mul(19), 35.0),
            ];
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let dt = delta_seconds.max(0.0);
        if dt == 0.0 {
            return;
        }

        let attraction_gain = 1.8_f32;
        let swirl_gain = 0.42_f32;
        let damping = 0.92_f32;
        let max_speed = 18.0_f32;

        for (idx, particle) in self.particles.iter_mut().enumerate() {
            let to_target = [
                particle.target[0] - particle.position[0],
                particle.target[1] - particle.position[1],
                particle.target[2] - particle.position[2],
            ];

            let swirl_dir = [
                -particle.position[2],
                sample_symmetric(self.seed, idx as u64, 0.15),
                particle.position[0],
            ];
            let swirl_norm = normalize(swirl_dir);

            particle.velocity[0] +=
                (to_target[0] * attraction_gain + swirl_norm[0] * swirl_gain) * dt;
            particle.velocity[1] +=
                (to_target[1] * attraction_gain + swirl_norm[1] * swirl_gain) * dt;
            particle.velocity[2] +=
                (to_target[2] * attraction_gain + swirl_norm[2] * swirl_gain) * dt;

            particle.velocity[0] *= damping;
            particle.velocity[1] *= damping;
            particle.velocity[2] *= damping;

            let speed = length(particle.velocity);
            if speed > max_speed {
                let scale = max_speed / speed;
                particle.velocity[0] *= scale;
                particle.velocity[1] *= scale;
                particle.velocity[2] *= scale;
            }

            particle.position[0] += particle.velocity[0] * dt;
            particle.position[1] += particle.velocity[1] * dt;
            particle.position[2] += particle.velocity[2] * dt;
        }
    }

    pub fn apply_to_scene(&self, scene: &mut Scene) {
        let mut nodes = Vec::with_capacity(self.particles.len());

        for (idx, particle) in self.particles.iter().enumerate() {
            let primitive = match idx % 3 {
                0 => SdfPrimitive::Sphere { radius: 0.12 },
                1 => SdfPrimitive::Box {
                    size: Vec3::new(0.1, 0.1, 0.1),
                },
                _ => SdfPrimitive::Cylinder {
                    radius: 0.08,
                    half_height: 0.14,
                },
            };

            nodes.push(SdfNode::Transform {
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(
                        particle.position[0],
                        particle.position[1],
                        particle.position[2],
                    ),
                }],
                child: Box::new(SdfNode::Primitive {
                    object: SdfObject {
                        primitive,
                        modifiers: vec![],
                        material: SdfMaterial {
                            material_type: SdfMaterialType::SolidColor,
                            base_color: Vec3::new(0.9, 0.95, 1.0),
                            emissive_strength: 0.28,
                            ..SdfMaterial::default()
                        },
                        bounds_radius: Some(0.5),
                    },
                }),
                bounds_radius: Some(0.5),
            });
        }

        match &mut scene.sdf.root {
            SdfNode::Union { children } => children.extend(nodes),
            root => {
                let existing = std::mem::replace(root, SdfNode::Empty);
                *root = SdfNode::Union {
                    children: std::iter::once(existing).chain(nodes).collect(),
                };
            }
        }
    }
}

fn sample_unit(seed: u64, salt: u64) -> f32 {
    let mixed = splitmix64(seed ^ salt);
    let mantissa = (mixed >> 40) as u32;
    mantissa as f32 / (u32::MAX >> 8) as f32
}

fn sample_symmetric(seed: u64, salt: u64, amplitude: f32) -> f32 {
    (sample_unit(seed, salt) * 2.0 - 1.0) * amplitude
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = length(v).max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}
