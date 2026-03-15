use aurex_scene::{Scene, Vec3};

use crate::demo_sequencer::DemoStageType;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraRig {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub time: f32,
}

impl CameraRig {
    pub fn new() -> Self {
        Self {
            position: [0.0, 10.0, 20.0],
            target: [0.0, 0.0, 0.0],
            orbit_radius: 20.0,
            orbit_speed: 0.2,
            time: 0.0,
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.time = (self.time + delta_seconds.max(0.0)).max(0.0);
        let orbit_time = self.time * self.orbit_speed;

        self.position[0] = orbit_time.cos() * self.orbit_radius;
        self.position[2] = orbit_time.sin() * self.orbit_radius;
    }

    pub fn apply_to_scene(&self, scene: &mut Scene) {
        scene.sdf.camera.position = Vec3::new(self.position[0], self.position[1], self.position[2]);
        scene.sdf.camera.target = Vec3::new(self.target[0], self.target[1], self.target[2]);
    }

    pub fn apply_stage_profile(&mut self, stage: DemoStageType) {
        match stage {
            DemoStageType::Bootstrap => {
                self.orbit_speed = 0.12;
                self.orbit_radius = 20.0;
                self.target = [0.0, 0.0, 0.0];
            }
            DemoStageType::ParticleFormation => {
                self.orbit_speed = 0.22;
                self.orbit_radius = 18.0;
                self.target = [0.0, 0.5, 0.0];
            }
            DemoStageType::LogoAssembly => {
                self.orbit_speed = 0.18;
                self.orbit_radius = 24.0;
                self.target = [0.0, 1.0, 0.0];
            }
            DemoStageType::LogoReveal => {
                self.orbit_speed = 0.1;
                self.orbit_radius = (self.orbit_radius - 0.25).max(12.0);
                self.position[1] = (self.position[1] - 0.05).max(6.5);
                self.target = [0.0, 1.25, 0.0];
            }
            DemoStageType::EnergyPulse => {
                self.orbit_speed = 0.34;
                self.orbit_radius = 16.0;
                self.position[1] = 9.5;
                self.target = [0.0, 0.75, 0.0];
            }
            DemoStageType::SceneCollapse => {
                self.orbit_speed = 0.42;
                self.orbit_radius = (self.orbit_radius + 0.8).min(34.0);
                self.position[1] = (self.position[1] + 0.08).min(18.0);
                self.target = [0.0, -0.5, 0.0];
            }
            DemoStageType::RuntimeHandover => {
                self.orbit_speed = 0.16;
                self.orbit_radius = 20.0;
                self.position[1] = 10.0;
                self.target = [0.0, 0.0, 0.0];
            }
        }
    }
}

impl Default for CameraRig {
    fn default() -> Self {
        Self::new()
    }
}
