use crate::pulse_builder::config::{
    AtmosphereType, CameraRig, LightingMode, PulseConfig, StructureSet,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PulsePhaseOverrides {
    pub lighting_override: Option<LightingMode>,
    pub atmosphere_override: Option<AtmosphereType>,
    pub particle_override: Option<f32>,
    pub camera_override: Option<CameraRig>,
    pub rhythm_intensity_override: Option<f32>,
    pub structure_override: Option<StructureSet>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PulsePhase {
    pub name: String,
    pub duration_seconds: f32,
    pub overrides: PulsePhaseOverrides,
}

impl PulsePhase {
    pub fn new(name: impl Into<String>, duration_seconds: f32) -> Self {
        Self {
            name: name.into(),
            duration_seconds: duration_seconds.max(0.001),
            overrides: PulsePhaseOverrides::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: PulsePhaseOverrides) -> Self {
        self.overrides = overrides;
        self
    }
}

pub fn apply_phase_overrides(base: &PulseConfig, phase: &PulsePhase) -> PulseConfig {
    let mut out = base.clone();
    let overrides = &phase.overrides;

    if let Some(lighting) = overrides.lighting_override {
        out.lighting_mode = lighting;
    }
    if let Some(atmosphere) = overrides.atmosphere_override {
        out.atmosphere_type = atmosphere;
    }
    if let Some(particle) = overrides.particle_override {
        out.particle_density_multiplier =
            ((out.particle_density_multiplier * 0.7) + particle * 0.3).clamp(0.0, 1.0);
    }
    if let Some(camera) = overrides.camera_override {
        out.camera_rig = camera;
    }
    if let Some(rhythm) = overrides.rhythm_intensity_override {
        out.rhythm_intensity = ((out.rhythm_intensity * 0.7) + rhythm * 0.3).clamp(0.0, 1.0);
    }
    if let Some(structures) = overrides.structure_override {
        out.structure_set = structures;
    }

    out
}
