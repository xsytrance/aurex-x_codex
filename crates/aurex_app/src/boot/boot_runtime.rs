use aurex_scene::{KeyLight, Scene, SdfCamera, SdfLighting, SdfNode, SdfScene, Vec3};

use crate::boot::boot_scene::{ensure_minimal_boot_lighting, rebuild_minimal_boot_scene};

const BOOT_TO_LIBRARY_SECONDS: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootScreenMode {
    Boot,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    SphereScene,
    LibraryScene,
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
                        position: Vec3::new(0.0, 0.0, 10.0),
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
                        fog_color: Vec3::new(0.0, 0.0, 0.0),
                        fog_density: 0.0,
                        fog_height_falloff: 0.0,
                        volumetric: Default::default(),
                    },
                    seed: seed as u32,
                    objects: vec![],
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
            screen_mode: BootScreenMode::Boot,
            last_debug_second: -1,
        };
        runtime.rebuild_scene();
        runtime
    }

    pub fn screen_mode(&self) -> BootScreenMode {
        self.screen_mode
    }

    pub fn stage(&self) -> BootStage {
        match self.screen_mode {
            BootScreenMode::Boot => BootStage::SphereScene,
            BootScreenMode::Library => BootStage::LibraryScene,
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let dt = delta_seconds.max(0.0);
        self.boot_timer += dt;

        if self.screen_mode == BootScreenMode::Boot && self.boot_timer >= BOOT_TO_LIBRARY_SECONDS {
            self.screen_mode = BootScreenMode::Library;
        }

        self.update_camera();
        self.update_lighting();
        self.rebuild_scene();
        self.log_once_per_second();
    }

    fn update_camera(&mut self) {
        self.scene.sdf.camera.position = Vec3::new(0.0, 0.0, 10.0);
        self.scene.sdf.camera.target = Vec3::new(0.0, 0.0, 0.0);
    }

    fn update_lighting(&mut self) {
        ensure_minimal_boot_lighting(&mut self.scene);
        self.scene.sdf.lighting.ambient_light = match self.screen_mode {
            BootScreenMode::Boot => 0.12,
            BootScreenMode::Library => 0.2,
        };
    }

    fn rebuild_scene(&mut self) {
        rebuild_minimal_boot_scene(&mut self.scene, self.screen_mode == BootScreenMode::Library);
    }

    fn log_once_per_second(&mut self) {
        let second = self.boot_timer.floor() as i32;
        if second != self.last_debug_second {
            self.last_debug_second = second;
            println!("boot objects: {}", self.scene.sdf.objects.len());
            println!("boot stage: {:?}", self.stage());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BootRuntime, BootScreenMode};

    #[test]
    fn boot_runtime_has_minimal_object_count() {
        let runtime = BootRuntime::new(2026);
        assert!(!runtime.scene.sdf.objects.is_empty());
        assert!(runtime.scene.sdf.objects.len() <= 10);
    }

    #[test]
    fn boot_runtime_enters_library_at_ten_seconds() {
        let mut runtime = BootRuntime::new(2026);
        runtime.update(10.1);
        assert_eq!(runtime.screen_mode(), BootScreenMode::Library);
    }
}
