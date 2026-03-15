use aurex_pulse::{camera_rig::CameraRig, demo_sequencer::DemoSequencer};
use aurex_scene::{
    Scene, SdfCamera, SdfLighting, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject,
    SdfPrimitive, SdfScene, Vec3, particle_swarm::ParticleSwarm,
    typography_generator::TypographyGenerator,
};

const STAR_COUNT: usize = 2400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootScreenMode {
    Cinematic,
    Library,
}

#[derive(Debug, Clone, Copy)]
struct Star {
    position: [f32; 3],
    radius: f32,
    emissive: f32,
}

#[derive(Debug, Clone, Copy)]
struct Planet {
    radius: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    phase: f32,
    height: f32,
    color: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootStage {
    StarfieldFadeIn,
    PlanetsAppear,
    SpaceDrift,
}

pub struct BootRuntime {
    pub scene: Scene,
    pub particle_swarm: ParticleSwarm,
    pub camera_rig: CameraRig,
    pub demo_sequencer: DemoSequencer,
    pub typography_generator: TypographyGenerator,
    pub boot_timer: f32,
    screen_mode: BootScreenMode,
    stars: Vec<Star>,
    planets: Vec<Planet>,
}

impl BootRuntime {
    pub fn new(seed: u64) -> Self {
        let stars = generate_stars(seed, STAR_COUNT);
        let planets = generate_planets(seed);
        let scene = Scene {
            sdf: SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 3.5, -18.0),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 16.0 / 9.0,
                },
                lighting: SdfLighting {
                    ambient_light: 0.08,
                    key_lights: vec![],
                    fog_color: Vec3::new(0.01, 0.02, 0.04),
                    fog_density: 0.002,
                    fog_height_falloff: 0.06,
                    volumetric: Default::default(),
                },
                seed: seed as u32,
                objects: vec![],
                root: SdfNode::Union {
                    children: Vec::new(),
                },
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
        };

        let mut runtime = Self {
            scene,
            particle_swarm: ParticleSwarm::new(seed.wrapping_add(11), 256),
            camera_rig: CameraRig::new(),
            demo_sequencer: DemoSequencer::new(),
            typography_generator: TypographyGenerator::new(seed),
            boot_timer: 0.0,
            screen_mode: BootScreenMode::Cinematic,
            stars,
            planets,
        };
        runtime.particle_swarm.clear_targets();
        runtime.rebuild_scene();
        runtime
    }

    pub fn screen_mode(&self) -> BootScreenMode {
        self.screen_mode
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let dt = delta_seconds.max(0.0);
        self.boot_timer += dt;

        if self.screen_mode == BootScreenMode::Library {
            return;
        }

        let _ = self.demo_sequencer.update(dt);
        self.update_camera();
        self.update_lighting();

        self.particle_swarm.update(dt * 0.4);
        self.rebuild_scene();

        if self.boot_timer >= 24.0 {
            self.enter_library_mode();
        }
    }

    pub fn enter_library_mode(&mut self) {
        self.screen_mode = BootScreenMode::Library;

        let mut lib_scene = Scene {
            sdf: SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 6.0, -14.0),
                    target: Vec3::new(0.0, 0.5, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 16.0 / 9.0,
                },
                lighting: SdfLighting {
                    ambient_light: 0.3,
                    key_lights: vec![],
                    fog_color: Vec3::new(0.03, 0.03, 0.04),
                    fog_density: 0.01,
                    fog_height_falloff: 0.08,
                    volumetric: Default::default(),
                },
                seed: self.scene.sdf.seed,
                objects: vec![],
                root: SdfNode::Union { children: vec![] },
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
        };
        self.typography_generator
            .apply_word_to_scene(&mut lib_scene, "AUREX-X");

        let mut children = match lib_scene.sdf.root {
            SdfNode::Union { children } => children,
            _ => vec![],
        };
        children.push(menu_bar(
            Vec3::new(0.0, 0.6, -1.8),
            Vec3::new(6.0, 0.3, 0.4),
            Vec3::new(0.28, 0.32, 0.42),
        ));
        self.scene.sdf.root = SdfNode::Union { children };
        self.scene.sdf.camera = lib_scene.sdf.camera;
        self.scene.sdf.lighting = lib_scene.sdf.lighting;
    }

    fn stage(&self) -> BootStage {
        if self.boot_timer < 3.0 {
            BootStage::StarfieldFadeIn
        } else if self.boot_timer < 8.0 {
            BootStage::PlanetsAppear
        } else {
            BootStage::SpaceDrift
        }
    }

    fn update_camera(&mut self) {
        let t = self.boot_timer;
        self.camera_rig.orbit_radius = 16.0 + (t * 0.07).sin() * 1.6;
        self.camera_rig.orbit_speed = 0.12;
        self.camera_rig.position[1] = 3.2 + (t * 0.05).sin() * 1.1;
        self.camera_rig.target = [0.0, 0.6, 0.0];
        self.camera_rig.update(1.0 / 60.0);
        self.camera_rig.apply_to_scene(&mut self.scene);
        self.scene.sdf.camera.position.z = -self.scene.sdf.camera.position.z.abs();
    }

    fn update_lighting(&mut self) {
        let fade = (self.boot_timer / 3.0).clamp(0.0, 1.0);
        self.scene.sdf.lighting.ambient_light = 0.02 + 0.12 * fade;
        self.scene.sdf.lighting.fog_density = 0.001 + 0.003 * fade;
    }

    fn rebuild_scene(&mut self) {
        let stage = self.stage();
        let fade = (self.boot_timer / 3.0).clamp(0.0, 1.0);
        let visible_stars = ((self.stars.len() as f32) * fade).round() as usize;

        let mut children = Vec::with_capacity(visible_stars + self.planets.len() * 2 + 280);

        for star in self.stars.iter().take(visible_stars) {
            children.push(SdfNode::Transform {
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(star.position[0], star.position[1], star.position[2]),
                }],
                child: Box::new(SdfNode::Primitive {
                    object: SdfObject {
                        primitive: SdfPrimitive::Sphere {
                            radius: star.radius,
                        },
                        modifiers: vec![],
                        material: SdfMaterial {
                            material_type: SdfMaterialType::SolidColor,
                            base_color: Vec3::new(0.8, 0.9, 1.0),
                            emissive_strength: star.emissive * fade,
                            ..SdfMaterial::default()
                        },
                        bounds_radius: Some(star.radius * 4.0),
                    },
                }),
                bounds_radius: Some(star.radius * 4.0),
            });
        }

        if stage != BootStage::StarfieldFadeIn {
            let appear = ((self.boot_timer - 3.0) / 5.0).clamp(0.0, 1.0);
            for planet in &self.planets {
                let angle = planet.phase + self.boot_timer * planet.orbit_speed;
                let x = angle.cos() * planet.orbit_radius;
                let z = angle.sin() * planet.orbit_radius - 38.0;
                let y = planet.height;
                let radius = planet.radius * (0.4 + 0.6 * appear);

                children.push(SdfNode::Transform {
                    modifiers: vec![SdfModifier::Translate {
                        offset: Vec3::new(x, y, z),
                    }],
                    child: Box::new(SdfNode::Primitive {
                        object: SdfObject {
                            primitive: SdfPrimitive::Sphere { radius },
                            modifiers: vec![],
                            material: SdfMaterial {
                                material_type: SdfMaterialType::SolidColor,
                                base_color: Vec3::new(
                                    planet.color[0],
                                    planet.color[1],
                                    planet.color[2],
                                ),
                                emissive_strength: 0.05 + appear * 0.06,
                                ..SdfMaterial::default()
                            },
                            bounds_radius: Some(radius + 0.2),
                        },
                    }),
                    bounds_radius: Some(radius + 0.2),
                });

                children.push(SdfNode::Transform {
                    modifiers: vec![SdfModifier::Translate {
                        offset: Vec3::new(x, y, z),
                    }],
                    child: Box::new(SdfNode::Primitive {
                        object: SdfObject {
                            primitive: SdfPrimitive::Sphere {
                                radius: radius * 1.08,
                            },
                            modifiers: vec![],
                            material: SdfMaterial {
                                material_type: SdfMaterialType::SolidColor,
                                base_color: Vec3::new(0.28, 0.36, 0.55),
                                emissive_strength: 0.02 + appear * 0.03,
                                ..SdfMaterial::default()
                            },
                            bounds_radius: Some(radius * 1.12),
                        },
                    }),
                    bounds_radius: Some(radius * 1.12),
                });
            }
        }

        self.scene.sdf.root = SdfNode::Union { children };
        self.particle_swarm.apply_to_scene(&mut self.scene);
    }
}

fn generate_stars(seed: u64, count: usize) -> Vec<Star> {
    let mut stars = Vec::with_capacity(count);
    for i in 0..count {
        let s = seed ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
        let x = sample_symmetric(s.wrapping_mul(3), 220.0);
        let y = sample_symmetric(s.wrapping_mul(5), 120.0);
        let z = -90.0 - sample_unit(s.wrapping_mul(7)) * 260.0;
        let radius = 0.05 + sample_unit(s.wrapping_mul(11)) * 0.09;
        let emissive = 0.25 + sample_unit(s.wrapping_mul(13)) * 0.7;
        stars.push(Star {
            position: [x, y, z],
            radius,
            emissive,
        });
    }
    stars
}

fn generate_planets(seed: u64) -> Vec<Planet> {
    (0..3)
        .map(|i| {
            let s = seed ^ (i as u64).wrapping_mul(0xD1342543DE82EF95);
            Planet {
                radius: 4.5 + sample_unit(s.wrapping_mul(3)) * 5.5,
                orbit_radius: 28.0 + sample_unit(s.wrapping_mul(5)) * 28.0,
                orbit_speed: 0.015 + sample_unit(s.wrapping_mul(7)) * 0.03,
                phase: sample_unit(s.wrapping_mul(11)) * std::f32::consts::TAU,
                height: -6.0 + sample_symmetric(s.wrapping_mul(13), 14.0),
                color: [
                    0.18 + sample_unit(s.wrapping_mul(17)) * 0.5,
                    0.16 + sample_unit(s.wrapping_mul(19)) * 0.45,
                    0.2 + sample_unit(s.wrapping_mul(23)) * 0.55,
                ],
            }
        })
        .collect()
}

fn menu_bar(offset: Vec3, size: Vec3, color: Vec3) -> SdfNode {
    SdfNode::Transform {
        modifiers: vec![SdfModifier::Translate { offset }],
        child: Box::new(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Box { size },
                modifiers: vec![],
                material: SdfMaterial {
                    material_type: SdfMaterialType::SolidColor,
                    base_color: color,
                    emissive_strength: 0.12,
                    ..SdfMaterial::default()
                },
                bounds_radius: Some(7.0),
            },
        }),
        bounds_radius: Some(7.0),
    }
}

fn sample_unit(seed: u64) -> f32 {
    let mixed = splitmix64(seed);
    let mantissa = (mixed >> 40) as u32;
    mantissa as f32 / (u32::MAX >> 8) as f32
}

fn sample_symmetric(seed: u64, amplitude: f32) -> f32 {
    (sample_unit(seed) * 2.0 - 1.0) * amplitude
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::{BootRuntime, SdfNode};

    #[test]
    fn boot_runtime_generates_visible_geometry() {
        let runtime = BootRuntime::new(2026);
        match runtime.scene.sdf.root {
            SdfNode::Union { ref children } => assert!(!children.is_empty()),
            _ => panic!("boot runtime root should be union"),
        }
    }

    #[test]
    fn boot_runtime_is_deterministic_for_seed() {
        let a = BootRuntime::new(42);
        let b = BootRuntime::new(42);
        assert_eq!(a.scene, b.scene);
    }
}
