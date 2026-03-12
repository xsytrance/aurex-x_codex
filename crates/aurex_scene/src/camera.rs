use serde::{Deserialize, Serialize};

use crate::{SdfCamera, Vec3};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RhythmSync {
    #[serde(default)]
    pub tempo_sync: f32,
    #[serde(default)]
    pub beat_shake: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrbitCamera {
    pub center: Vec3,
    #[serde(default = "default_radius")]
    pub radius: f32,
    #[serde(default = "default_orbit_speed")]
    pub speed: f32,
    #[serde(default)]
    pub height: f32,
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    #[serde(default)]
    pub roll: f32,
    #[serde(default)]
    pub rhythm: RhythmSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlythroughCamera {
    pub start: Vec3,
    pub end: Vec3,
    #[serde(default)]
    pub target: Vec3,
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    #[serde(default)]
    pub roll: f32,
    #[serde(default)]
    pub rhythm: RhythmSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetTrackingCamera {
    pub position: Vec3,
    pub target: Vec3,
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    #[serde(default)]
    pub roll: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BezierPathCamera {
    pub control_points: Vec<Vec3>,
    pub target: Vec3,
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    #[serde(default)]
    pub roll: f32,
    #[serde(default)]
    pub rhythm: RhythmSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RhythmCamera {
    pub position: Vec3,
    pub target: Vec3,
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    #[serde(default)]
    pub roll: f32,
    #[serde(default)]
    pub tempo_sync: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CameraRig {
    Orbit(OrbitCamera),
    Flythrough(FlythroughCamera),
    TargetTracking(TargetTrackingCamera),
    BezierPath(BezierPathCamera),
    Rhythm(RhythmCamera),
}

#[derive(Debug, Clone, Copy)]
pub struct CameraSyncInput {
    pub beat: f32,
    pub phrase: f32,
    pub tempo: f32,
}

impl CameraRig {
    pub fn sample(
        &self,
        base: &SdfCamera,
        time: f32,
        duration: f32,
        sync: CameraSyncInput,
    ) -> SdfCamera {
        match self {
            CameraRig::Orbit(c) => c.sample(base, time, duration, sync),
            CameraRig::Flythrough(c) => c.sample(base, time, duration, sync),
            CameraRig::TargetTracking(c) => c.sample(base),
            CameraRig::BezierPath(c) => c.sample(base, time, duration, sync),
            CameraRig::Rhythm(c) => c.sample(base, time, sync),
        }
    }
}

impl OrbitCamera {
    fn sample(
        &self,
        base: &SdfCamera,
        time: f32,
        duration: f32,
        sync: CameraSyncInput,
    ) -> SdfCamera {
        let phase = (time / duration.max(1e-6))
            * self.speed
            * (1.0 + self.rhythm.tempo_sync * sync.tempo * 0.01);
        let ang = phase * std::f32::consts::TAU;
        let shake = self.rhythm.beat_shake * sync.beat;
        SdfCamera {
            position: Vec3::new(
                self.center.x + ang.cos() * self.radius,
                self.center.y + self.height + shake,
                self.center.z + ang.sin() * self.radius,
            ),
            target: self.center,
            fov_degrees: self.fov_degrees,
            aspect_ratio: base.aspect_ratio,
        }
    }
}

impl FlythroughCamera {
    fn sample(
        &self,
        base: &SdfCamera,
        time: f32,
        duration: f32,
        sync: CameraSyncInput,
    ) -> SdfCamera {
        let t = (time / duration.max(1e-6)).clamp(0.0, 1.0);
        let speedup = (self.rhythm.tempo_sync * sync.tempo * 0.005).clamp(0.0, 0.5);
        let alpha = (t * (1.0 + speedup)).clamp(0.0, 1.0);
        SdfCamera {
            position: self.start.lerp(self.end, alpha),
            target: self.target,
            fov_degrees: self.fov_degrees,
            aspect_ratio: base.aspect_ratio,
        }
    }
}

impl TargetTrackingCamera {
    fn sample(&self, base: &SdfCamera) -> SdfCamera {
        SdfCamera {
            position: self.position,
            target: self.target,
            fov_degrees: self.fov_degrees,
            aspect_ratio: base.aspect_ratio,
        }
    }
}

impl BezierPathCamera {
    fn sample(
        &self,
        base: &SdfCamera,
        time: f32,
        duration: f32,
        sync: CameraSyncInput,
    ) -> SdfCamera {
        let t = ((time / duration.max(1e-6)) + self.rhythm.tempo_sync * sync.tempo * 0.001)
            .clamp(0.0, 1.0);
        let pos = sample_bezier(&self.control_points, t);
        SdfCamera {
            position: pos,
            target: self.target,
            fov_degrees: self.fov_degrees,
            aspect_ratio: base.aspect_ratio,
        }
    }
}

impl RhythmCamera {
    fn sample(&self, base: &SdfCamera, time: f32, sync: CameraSyncInput) -> SdfCamera {
        let wobble = (time * (1.5 + self.tempo_sync * sync.tempo * 0.01)).sin() * 0.25;
        SdfCamera {
            position: Vec3::new(
                self.position.x,
                self.position.y + wobble + sync.beat * 0.08,
                self.position.z,
            ),
            target: Vec3::new(
                self.target.x,
                self.target.y + sync.phrase * 0.1,
                self.target.z,
            ),
            fov_degrees: self.fov_degrees,
            aspect_ratio: base.aspect_ratio,
        }
    }
}

fn sample_bezier(control_points: &[Vec3], t: f32) -> Vec3 {
    match control_points {
        [] => Vec3::new(0.0, 0.0, -5.0),
        [p] => *p,
        _ => {
            let mut points = control_points.to_vec();
            while points.len() > 1 {
                let mut next = Vec::with_capacity(points.len() - 1);
                for pair in points.windows(2) {
                    next.push(pair[0].lerp(pair[1], t));
                }
                points = next;
            }
            points[0]
        }
    }
}

fn default_radius() -> f32 {
    6.0
}
fn default_orbit_speed() -> f32 {
    1.0
}
fn default_fov() -> f32 {
    60.0
}

impl Default for RhythmSync {
    fn default() -> Self {
        Self {
            tempo_sync: 0.0,
            beat_shake: 0.0,
        }
    }
}

pub fn estimate_framing_distance(scene_scale: f32, fov_degrees: f32) -> f32 {
    let half_fov = (fov_degrees.to_radians() * 0.5).tan().max(1e-4);
    (scene_scale / half_fov).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_sampling_is_deterministic() {
        let rig = CameraRig::BezierPath(BezierPathCamera {
            control_points: vec![
                Vec3::new(0.0, 0.0, -8.0),
                Vec3::new(1.0, 0.5, -6.0),
                Vec3::new(0.0, 1.0, -4.0),
            ],
            target: Vec3::new(0.0, 0.0, 0.0),
            fov_degrees: 55.0,
            roll: 0.0,
            rhythm: RhythmSync::default(),
        });
        let base = SdfCamera {
            position: Vec3::new(0.0, 0.0, -5.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            fov_degrees: 60.0,
            aspect_ratio: 1.77,
        };
        let a = rig.sample(
            &base,
            1.5,
            6.0,
            CameraSyncInput {
                beat: 0.1,
                phrase: 0.2,
                tempo: 140.0,
            },
        );
        let b = rig.sample(
            &base,
            1.5,
            6.0,
            CameraSyncInput {
                beat: 0.1,
                phrase: 0.2,
                tempo: 140.0,
            },
        );
        assert_eq!(a, b);
    }
}
