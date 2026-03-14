use serde::{Deserialize, Serialize};

use crate::{Scene, SdfPrimitive};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AutomationInput {
    pub time_seconds: f32,
    pub beat: f32,
    pub measure: f32,
    pub phrase: f32,
    pub tempo: f32,
    pub bass: f32,
    pub mid: f32,
    pub high: f32,
    pub dominant_frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationTrack {
    pub name: String,
    pub source: AutomationSource,
    pub curve: AutomationCurve,
    #[serde(default = "default_one")]
    pub amplitude: f32,
    #[serde(default)]
    pub offset: f32,
    #[serde(default = "default_one")]
    pub frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationBinding {
    pub target: AutomationTarget,
    pub track: AutomationTrack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutomationSource {
    Time,
    Beat,
    Measure,
    Phrase,
    Tempo,
    Bass,
    Mid,
    High,
    DominantFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutomationCurve {
    Linear,
    Smoothstep,
    Sine,
    Noise,
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutomationTarget {
    TunnelRadius,
    PatternScale,
    CameraRoll,
    LightingIntensity,
    MaterialEmissive,
    CameraFov,
}

impl AutomationTrack {
    pub fn sample(&self, input: AutomationInput, seed: u32) -> f32 {
        let src = match self.source {
            AutomationSource::Time => input.time_seconds,
            AutomationSource::Beat => input.beat,
            AutomationSource::Measure => input.measure,
            AutomationSource::Phrase => input.phrase,
            AutomationSource::Tempo => input.tempo / 120.0,
            AutomationSource::Bass => input.bass,
            AutomationSource::Mid => input.mid,
            AutomationSource::High => input.high,
            AutomationSource::DominantFrequency => {
                (input.dominant_frequency / 440.0).clamp(0.0, 4.0)
            }
        };

        let t = src * self.frequency;
        let c = match self.curve {
            AutomationCurve::Linear => t,
            AutomationCurve::Smoothstep => {
                let x = t.fract();
                x * x * (3.0 - 2.0 * x)
            }
            AutomationCurve::Sine => 0.5 + 0.5 * (t * std::f32::consts::TAU).sin(),
            AutomationCurve::Noise => hash_noise(t, seed),
            AutomationCurve::Exponential => t.abs().min(8.0).exp() / std::f32::consts::E.powf(8.0),
        };

        c * self.amplitude + self.offset
    }
}

pub fn apply_bindings(
    scene: &mut Scene,
    bindings: &[AutomationBinding],
    input: AutomationInput,
    seed: u32,
) {
    for binding in bindings {
        let value = binding.track.sample(input, seed);
        match binding.target {
            AutomationTarget::TunnelRadius => {
                if let Some(generator) = &mut scene.sdf.generator
                    && let crate::generators::SceneGenerator::Tunnel(tunnel) = generator
                {
                    tunnel.radius = (tunnel.radius + value * 0.15).max(0.2);
                }
                for o in &mut scene.sdf.objects {
                    if let SdfPrimitive::Torus { major_radius, .. } = &mut o.primitive {
                        *major_radius = (*major_radius + value * 0.15).max(0.2);
                    }
                }
            }
            AutomationTarget::PatternScale => {
                for o in &mut scene.sdf.objects {
                    o.material
                        .parameters
                        .insert("pattern_scale".into(), value.max(0.0));
                }
            }
            AutomationTarget::CameraRoll => {
                scene.sdf.camera.position.x += value * 0.03;
            }
            AutomationTarget::LightingIntensity => {
                for l in &mut scene.sdf.lighting.key_lights {
                    l.intensity *= (1.0 + value * 0.1).max(0.0);
                }
            }
            AutomationTarget::MaterialEmissive => {
                for o in &mut scene.sdf.objects {
                    o.material.emissive_strength =
                        (o.material.emissive_strength + value * 0.2).max(0.0);
                }
            }
            AutomationTarget::CameraFov => {
                scene.sdf.camera.fov_degrees =
                    (scene.sdf.camera.fov_degrees + value * 1.5).clamp(25.0, 110.0);
            }
        }
    }
}

fn hash_noise(t: f32, seed: u32) -> f32 {
    let v = (t * 37.1 + seed as f32 * 0.17).sin() * 43_758.547;
    v.fract().abs()
}

fn default_one() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automation_curve_stability() {
        let track = AutomationTrack {
            name: "bass-rise".into(),
            source: AutomationSource::Bass,
            curve: AutomationCurve::Noise,
            amplitude: 0.8,
            offset: 0.1,
            frequency: 1.2,
        };
        let input = AutomationInput {
            time_seconds: 1.0,
            beat: 2.0,
            measure: 1.0,
            phrase: 0.5,
            tempo: 140.0,
            bass: 0.9,
            mid: 0.3,
            high: 0.2,
            dominant_frequency: 220.0,
        };
        let a = track.sample(input, 99);
        let b = track.sample(input, 99);
        assert_eq!(a, b);
    }
}
