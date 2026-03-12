use crate::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HarmonicBand {
    Bass,
    Mid,
    High,
    Melody,
    Chords,
    Full,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct HarmonicField {
    pub center: Vec3,
    pub radius: f32,
    pub falloff: f32,
    pub strength: f32,
    pub band: HarmonicBand,
}

impl HarmonicField {
    pub fn sample(&self, position: Vec3, band_energy: f32) -> f32 {
        let dx = position.x - self.center.x;
        let dy = position.y - self.center.y;
        let dz = position.z - self.center.z;
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        let radial = (1.0 - d / self.radius.max(0.001)).clamp(0.0, 1.0);
        radial.powf(1.0 + self.falloff.max(0.0)) * band_energy.max(0.0) * self.strength
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct HarmonicBinding {
    pub band: HarmonicBand,
    #[serde(default = "default_binding_strength")]
    pub strength: f32,
}

fn default_binding_strength() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneHarmonicsConfig {
    #[serde(default)]
    pub geometry: Option<HarmonicBinding>,
    #[serde(default)]
    pub materials: Option<HarmonicBinding>,
    #[serde(default)]
    pub particles: Option<HarmonicBinding>,
    #[serde(default)]
    pub fields: Vec<HarmonicField>,
}
