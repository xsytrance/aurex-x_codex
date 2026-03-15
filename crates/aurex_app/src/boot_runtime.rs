use aurex_scene::{
    KeyLight, Scene, SdfCamera, SdfLighting, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode,
    SdfObject, SdfPrimitive, SdfScene, Vec3,
};

const MAX_BOOT_OBJECTS: usize = 100;
const BOOT_TO_LIBRARY_SECONDS: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootScreenMode {
    Cinematic,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootStage {
    SphereScene,
    Library,
}

pub struct BootRuntime {
    pub scene: Scene,
    pub boot_timer: f32,
    screen_mode: BootScreenMode,
    last_debug_second: i32,
}

impl BootRuntime {
    pub fn new(seed: u64) -> Self {
        let mut runtime = Self {
            scene: Scene {
                sdf: SdfScene {
                    camera: SdfCamera {
                        position: Vec3::new(0.0, 0.0, 12.0),
                        target: Vec3::new(0.0, 0.0, 0.0),
                        fov_degrees: 60.0,
                        aspect_ratio: 16.0 / 9.0,
                    },
                    lighting: SdfLighting {
                        ambient_light: 0.12,
                        key_lights: vec![KeyLight {
                            direction: Vec3::new(0.5, -1.0, -0.5),
                            intensity: 3.0,
                            color: Vec3::new(1.0, 0.98, 0.92),
                        }],
                        fog_color: Vec3::new(0.01, 0.01, 0.02),
                        fog_density: 0.001,
                        fog_height_falloff: 0.05,
                        volumetric: Default::default(),
                    },
                    seed: seed as u32,
                    objects: Vec::new(),
                    root: SdfNode::Empty,
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
            },
            boot_timer: 0.0,
            screen_mode: BootScreenMode::Cinematic,
            last_debug_second: -1,
        };

        runtime.rebuild_scene();
        runtime
    }

    pub fn screen_mode(&self) -> BootScreenMode {
        self.screen_mode
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let dt = delta_seconds.max(0.0);
        self.boot_timer += dt;

        if self.screen_mode == BootScreenMode::Cinematic
            && self.boot_timer >= BOOT_TO_LIBRARY_SECONDS
        {
            self.enter_library_mode();
        }

        self.update_camera();
        self.update_lighting();
        self.rebuild_scene();
        self.log_once_per_second();
    }

    pub fn enter_library_mode(&mut self) {
        self.screen_mode = BootScreenMode::Library;
    }

    fn stage(&self) -> BootStage {
        match self.screen_mode {
            BootScreenMode::Cinematic => BootStage::SphereScene,
            BootScreenMode::Library => BootStage::Library,
        }
    }

    fn update_camera(&mut self) {
        self.scene.sdf.camera.target = Vec3::new(0.0, 0.0, 0.0);
        self.scene.sdf.camera.position = match self.screen_mode {
            BootScreenMode::Cinematic => {
                let t = self.boot_timer;
                Vec3::new((t * 0.2).cos() * 1.0, (t * 0.17).sin() * 0.4, 12.0)
            }
            BootScreenMode::Library => Vec3::new(0.0, 0.0, 12.0),
        };
    }

    fn update_lighting(&mut self) {
        if let Some(light) = self.scene.sdf.lighting.key_lights.first_mut() {
            light.direction = Vec3::new(0.5, -1.0, -0.5);
            light.intensity = 3.0;
            light.color = Vec3::new(1.0, 0.98, 0.92);
        } else {
            self.scene.sdf.lighting.key_lights.push(KeyLight {
                direction: Vec3::new(0.5, -1.0, -0.5),
                intensity: 3.0,
                color: Vec3::new(1.0, 0.98, 0.92),
            });
        }

        self.scene.sdf.lighting.ambient_light = match self.screen_mode {
            BootScreenMode::Cinematic => 0.12,
            BootScreenMode::Library => 0.2,
        };
    }

    fn rebuild_scene(&mut self) {
        self.scene.sdf.objects.clear();

        let main_sphere = SdfObject {
            primitive: SdfPrimitive::Sphere { radius: 2.0 },
            modifiers: vec![],
            material: SdfMaterial {
                material_type: SdfMaterialType::SolidColor,
                base_color: Vec3::new(0.75, 0.82, 0.95),
                emissive_strength: 0.08,
                ..SdfMaterial::default()
            },
            bounds_radius: Some(2.1),
        };
        self.scene.sdf.objects.push(main_sphere.clone());

        if self.screen_mode == BootScreenMode::Library {
            self.scene.sdf.objects.push(SdfObject {
                primitive: SdfPrimitive::Box {
                    size: Vec3::new(6.0, 0.35, 0.2),
                },
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(0.0, -3.2, 0.0),
                }],
                material: SdfMaterial {
                    material_type: SdfMaterialType::SolidColor,
                    base_color: Vec3::new(0.2, 0.24, 0.32),
                    emissive_strength: 0.03,
                    ..SdfMaterial::default()
                },
                bounds_radius: Some(6.1),
            });
        }

        self.scene.sdf.root = SdfNode::Primitive {
            object: main_sphere,
        };

        if self.scene.sdf.objects.len() > MAX_BOOT_OBJECTS {
            self.scene.sdf.objects.truncate(MAX_BOOT_OBJECTS);
        }
    }

    fn log_once_per_second(&mut self) {
        let second = self.boot_timer.floor() as i32;
        if second != self.last_debug_second {
            self.last_debug_second = second;
            println!(
                "boot objects: {} stage: {:?}",
                self.scene.sdf.objects.len(),
                self.stage()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BOOT_TO_LIBRARY_SECONDS, BootRuntime, BootScreenMode};

    #[test]
    fn boot_runtime_populates_objects() {
        let runtime = BootRuntime::new(2026);
        assert!(!runtime.scene.sdf.objects.is_empty());
        assert!(runtime.scene.sdf.objects.len() <= 100);
    }

    #[test]
    fn boot_runtime_transitions_to_library_after_three_seconds() {
        let mut runtime = BootRuntime::new(2026);
        runtime.update(BOOT_TO_LIBRARY_SECONDS + 0.01);
        assert_eq!(runtime.screen_mode(), BootScreenMode::Library);
    }
}
