use aurex_pulse::{
    camera_rig::CameraRig,
    demo_sequencer::{DemoSequencer, DemoStageType, apply_stage_effect},
};
use aurex_scene::{
    Scene, SdfCamera, SdfLighting, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject,
    SdfPrimitive, SdfScene, Vec3, particle_swarm::ParticleSwarm,
    typography_generator::TypographyGenerator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootScreenMode {
    Cinematic,
    Library,
}

pub struct BootRuntime {
    pub scene: Scene,
    pub particle_swarm: ParticleSwarm,
    pub camera_rig: CameraRig,
    pub demo_sequencer: DemoSequencer,
    pub typography_generator: TypographyGenerator,
    pub boot_timer: f32,
    screen_mode: BootScreenMode,
    static_children: Vec<SdfNode>,
    targets_assigned: bool,
    released: bool,
}

impl BootRuntime {
    pub fn new(seed: u64) -> Self {
        let static_children = deep_space_environment(seed as u32);
        let scene = Scene {
            sdf: SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 10.0, 20.0),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 16.0 / 9.0,
                },
                lighting: SdfLighting {
                    ambient_light: 0.08,
                    key_lights: vec![],
                    fog_color: Vec3::new(0.02, 0.03, 0.06),
                    fog_density: 0.005,
                    fog_height_falloff: 0.06,
                    volumetric: Default::default(),
                },
                seed: seed as u32,
                objects: vec![],
                root: SdfNode::Union {
                    children: static_children.clone(),
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

        Self {
            scene,
            particle_swarm: ParticleSwarm::new(seed.wrapping_add(11), 1800),
            camera_rig: CameraRig::new(),
            demo_sequencer: DemoSequencer::new(),
            typography_generator: TypographyGenerator::new(seed),
            boot_timer: 0.0,
            screen_mode: BootScreenMode::Cinematic,
            static_children,
            targets_assigned: false,
            released: false,
        }
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
        let stage = self.demo_sequencer.current_stage_type();

        apply_stage_effect(stage, &mut self.scene);
        self.camera_rig.apply_stage_profile(stage);
        self.camera_rig.update(dt);
        self.camera_rig.apply_to_scene(&mut self.scene);

        match stage {
            DemoStageType::Bootstrap => {
                self.camera_rig.orbit_speed = 0.08;
            }
            DemoStageType::ParticleFormation => {
                self.camera_rig.orbit_speed = 0.12;
                self.particle_swarm.clear_targets();
            }
            DemoStageType::LogoAssembly => {
                if !self.targets_assigned {
                    let targets: Vec<[f32; 3]> = self
                        .typography_generator
                        .generate_word("AUREX-X")
                        .into_iter()
                        .map(|i| i.position)
                        .collect();
                    self.particle_swarm.set_targets(&targets);
                    self.targets_assigned = true;
                }
                self.camera_rig.orbit_radius = 26.0;
            }
            DemoStageType::LogoReveal => {
                self.camera_rig.orbit_speed = 0.22;
            }
            DemoStageType::EnergyPulse => {
                self.scene.sdf.lighting.ambient_light = 0.46;
            }
            DemoStageType::SceneCollapse => {
                if !self.released {
                    self.particle_swarm.clear_targets();
                    self.released = true;
                }
                self.camera_rig.position[1] += 0.02;
            }
            DemoStageType::RuntimeHandover => {
                self.enter_library_mode();
                return;
            }
        }

        self.particle_swarm.update(dt);
        self.rebuild_scene();
    }

    pub fn enter_library_mode(&mut self) {
        self.screen_mode = BootScreenMode::Library;
        self.scene.sdf.root = SdfNode::Union {
            children: library_screen_geometry(self.scene.sdf.seed),
        };
        self.scene.sdf.lighting.ambient_light = 0.35;
        self.scene.sdf.camera.position = Vec3::new(0.0, 7.0, 18.0);
        self.scene.sdf.camera.target = Vec3::new(0.0, 3.0, 0.0);
    }

    fn rebuild_scene(&mut self) {
        self.scene.sdf.root = SdfNode::Union {
            children: self.static_children.clone(),
        };
        self.particle_swarm.apply_to_scene(&mut self.scene);
    }
}

fn deep_space_environment(seed: u32) -> Vec<SdfNode> {
    let mut children = Vec::new();

    children.push(SdfNode::Transform {
        modifiers: vec![SdfModifier::Translate {
            offset: Vec3::new(-22.0, 8.0, -40.0),
        }],
        child: Box::new(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Sphere { radius: 8.5 },
                modifiers: vec![],
                material: SdfMaterial {
                    material_type: SdfMaterialType::SolidColor,
                    base_color: Vec3::new(0.18, 0.26, 0.42),
                    emissive_strength: 0.05,
                    ..SdfMaterial::default()
                },
                bounds_radius: Some(9.0),
            },
        }),
        bounds_radius: Some(9.0),
    });

    children.push(SdfNode::Transform {
        modifiers: vec![SdfModifier::Translate {
            offset: Vec3::new(28.0, -4.0, -55.0),
        }],
        child: Box::new(SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Sphere { radius: 12.0 },
                modifiers: vec![],
                material: SdfMaterial {
                    material_type: SdfMaterialType::SolidColor,
                    base_color: Vec3::new(0.34, 0.18, 0.22),
                    emissive_strength: 0.04,
                    ..SdfMaterial::default()
                },
                bounds_radius: Some(12.5),
            },
        }),
        bounds_radius: Some(12.5),
    });

    for i in 0..280 {
        let s = u64::from(seed) ^ i as u64;
        let x = sample(s.wrapping_mul(3), 130.0);
        let y = sample(s.wrapping_mul(5), 70.0);
        let z = -65.0 - sample(s.wrapping_mul(7), 110.0);
        children.push(SdfNode::Transform {
            modifiers: vec![SdfModifier::Translate {
                offset: Vec3::new(x, y, z),
            }],
            child: Box::new(SdfNode::Primitive {
                object: SdfObject {
                    primitive: SdfPrimitive::Sphere { radius: 0.16 },
                    modifiers: vec![],
                    material: SdfMaterial {
                        material_type: SdfMaterialType::SolidColor,
                        base_color: Vec3::new(0.85, 0.9, 1.0),
                        emissive_strength: 0.35,
                        ..SdfMaterial::default()
                    },
                    bounds_radius: Some(0.3),
                },
            }),
            bounds_radius: Some(0.3),
        });
    }

    children
}

fn library_screen_geometry(seed: u32) -> Vec<SdfNode> {
    let generator = TypographyGenerator::new(u64::from(seed));
    let mut scene = Scene {
        sdf: SdfScene {
            camera: SdfCamera {
                position: Vec3::new(0.0, 7.0, 18.0),
                target: Vec3::new(0.0, 3.0, 0.0),
                fov_degrees: 60.0,
                aspect_ratio: 16.0 / 9.0,
            },
            lighting: SdfLighting {
                ambient_light: 0.35,
                key_lights: vec![],
                fog_color: Vec3::new(0.04, 0.04, 0.06),
                fog_density: 0.012,
                fog_height_falloff: 0.08,
                volumetric: Default::default(),
            },
            seed,
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

    generator.apply_word_to_scene(&mut scene, "AUREX-X");

    let mut children = match scene.sdf.root {
        SdfNode::Union { children } => children,
        _ => vec![],
    };

    children.extend([
        menu_bar(
            Vec3::new(0.0, 0.5, -2.0),
            Vec3::new(6.0, 0.25, 0.4),
            Vec3::new(0.2, 0.35, 0.6),
        ),
        menu_bar(
            Vec3::new(0.0, -1.0, -2.0),
            Vec3::new(6.0, 0.25, 0.4),
            Vec3::new(0.22, 0.28, 0.48),
        ),
        menu_bar(
            Vec3::new(0.0, -2.5, -2.0),
            Vec3::new(6.0, 0.25, 0.4),
            Vec3::new(0.45, 0.18, 0.2),
        ),
    ]);

    children
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

fn sample(salt: u64, amplitude: f32) -> f32 {
    let mixed = splitmix64(salt);
    let unit = ((mixed >> 40) as u32) as f32 / (u32::MAX >> 8) as f32;
    (unit * 2.0 - 1.0) * amplitude
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}
