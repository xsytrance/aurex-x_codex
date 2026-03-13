use super::config::{
    AtmosphereType, CameraRig, GeometryStyle, LightingMode, PulseConfig, StructureSet,
};
use crate::pulse_sequence::{PulseSequence, apply_phase_overrides};
use aurex_render::rhythm_field::{
    AtmosphereLayerParams, CameraLayerHints, GeneratorStackOutput, LightingLayerParams,
    ParticleLayerParams, RhythmFieldSnapshot, SequencerState, StructureLayerParams,
    TerrainLayerParams, VisualTheme, apply_rhythm_modulation, sample_rhythm_field,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WorldBlueprint {
    pub name: String,
    pub theme: VisualTheme,
    pub palette_hint: String,
    pub camera_motion: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExamplePulseConfig {
    pub pulse_name: String,
    pub pulse_config: PulseConfig,
    pub world_blueprint: WorldBlueprint,
    pub generator_output: GeneratorStackOutput,
    pub rhythm_snapshot: RhythmFieldSnapshot,
    pub modulated_output: GeneratorStackOutput,
    pub sequence: Option<PulseSequence>,
    pub current_phase_name: Option<String>,
    pub sequence_duration_seconds: Option<f32>,
}

pub struct PulseBuilder {
    config: PulseConfig,
    sequence: Option<PulseSequence>,
}

impl PulseBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: PulseConfig {
                name: name.into(),
                ..PulseConfig::default()
            },
            sequence: None,
        }
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = seed;
        self
    }

    pub fn theme(mut self, theme: VisualTheme) -> Self {
        self.config.theme = theme;
        self
    }

    pub fn geometry_style(mut self, style: GeometryStyle) -> Self {
        self.config.geometry_style = style;
        self
    }

    pub fn atmosphere(mut self, atmosphere: AtmosphereType) -> Self {
        self.config.atmosphere_type = atmosphere;
        self
    }

    pub fn lighting(mut self, mode: LightingMode) -> Self {
        self.config.lighting_mode = mode;
        self
    }

    pub fn structures(mut self, set: StructureSet) -> Self {
        self.config.structure_set = set;
        self
    }

    pub fn color_palette(mut self, palette: impl Into<String>) -> Self {
        self.config.color_palette = palette.into();
        self
    }

    pub fn camera_rig(mut self, rig: CameraRig) -> Self {
        self.config.camera_rig = rig;
        self
    }

    pub fn rhythm_intensity(mut self, value: f32) -> Self {
        self.config.rhythm_intensity = value.clamp(0.0, 1.0);
        self
    }

    pub fn particle_density_multiplier(mut self, value: f32) -> Self {
        self.config.particle_density_multiplier = value.clamp(0.0, 1.0);
        self
    }

    pub fn sequence(mut self, sequence: PulseSequence) -> Self {
        self.sequence = Some(sequence);
        self
    }

    pub fn build(self) -> ExamplePulseConfig {
        let mut config = self.config;
        let sequence = self.sequence;

        let base_time = sample_time_for_theme(config.theme);
        let mut current_phase_name = None;
        let mut sequence_duration_seconds = None;

        if let Some(seq) = &sequence {
            let phase = seq.phase_at_time(base_time);
            current_phase_name = Some(phase.name.clone());
            sequence_duration_seconds = Some(seq.total_duration());
            config = apply_phase_overrides(&config, phase);
        }

        let world_blueprint = WorldBlueprint {
            name: config.name.clone(),
            theme: config.theme,
            palette_hint: config.color_palette.clone(),
            camera_motion: camera_motion_label(config.camera_rig).to_string(),
        };

        let generator_output = build_generator_output(&config);
        let sequencer_state = build_sequencer_state(config.theme);
        let rhythm_snapshot = sample_rhythm_field(config.seed, base_time, sequencer_state);

        let fully_modulated =
            apply_rhythm_modulation(&rhythm_snapshot, &generator_output, config.theme);
        let modulated_output =
            blend_stack_output(generator_output, fully_modulated, config.rhythm_intensity);

        ExamplePulseConfig {
            pulse_name: config.name.clone(),
            pulse_config: config,
            world_blueprint,
            generator_output,
            rhythm_snapshot,
            modulated_output,
            sequence,
            current_phase_name,
            sequence_duration_seconds,
        }
    }
}

fn build_generator_output(config: &PulseConfig) -> GeneratorStackOutput {
    let density = structure_density_for_set(config.structure_set);
    let particle_intensity = config.particle_density_multiplier;

    let terrain = match config.geometry_style {
        GeometryStyle::City => TerrainLayerParams {
            amplitude_hint: 0.62,
            roughness_hint: 0.58,
        },
        GeometryStyle::Lounge => TerrainLayerParams {
            amplitude_hint: 0.42,
            roughness_hint: 0.34,
        },
        GeometryStyle::Dreamscape => TerrainLayerParams {
            amplitude_hint: 0.34,
            roughness_hint: 0.28,
        },
    };

    let atmosphere = match config.atmosphere_type {
        AtmosphereType::Clear => AtmosphereLayerParams {
            hue_drift: 0.48,
            fog_density: 0.3,
        },
        AtmosphereType::Hazy => AtmosphereLayerParams {
            hue_drift: 0.66,
            fog_density: 0.74,
        },
        AtmosphereType::Foggy => AtmosphereLayerParams {
            hue_drift: 0.72,
            fog_density: 0.82,
        },
    };

    let lighting = match config.lighting_mode {
        LightingMode::NeonPulse => LightingLayerParams {
            flash_envelope: 0.8,
            exposure: 0.7,
        },
        LightingMode::WarmGlow => LightingLayerParams {
            flash_envelope: 0.24,
            exposure: 0.56,
        },
        LightingMode::DiffuseSoft => LightingLayerParams {
            flash_envelope: 0.15,
            exposure: 0.42,
        },
    };

    let camera_hints = match config.camera_rig {
        CameraRig::Orbit => CameraLayerHints {
            drift: 0.45,
            fov_bias: 0.6,
        },
        CameraRig::Drift => CameraLayerHints {
            drift: 0.62,
            fov_bias: 0.5,
        },
        CameraRig::Float => CameraLayerHints {
            drift: 0.68,
            fov_bias: 0.44,
        },
    };

    GeneratorStackOutput {
        terrain,
        structures: StructureLayerParams {
            density,
            emissive: (0.14 + density * 0.68).clamp(0.0, 1.0),
        },
        atmosphere,
        lighting,
        particles: ParticleLayerParams {
            density_multiplier: particle_intensity,
            brightness: (0.1 + particle_intensity * 0.78).clamp(0.0, 1.0),
        },
        camera_hints,
    }
}

fn blend_stack_output(
    base: GeneratorStackOutput,
    modulated: GeneratorStackOutput,
    reactivity: f32,
) -> GeneratorStackOutput {
    let lerp = |a: f32, b: f32| a + (b - a) * reactivity;

    GeneratorStackOutput {
        terrain: TerrainLayerParams {
            amplitude_hint: lerp(
                base.terrain.amplitude_hint,
                modulated.terrain.amplitude_hint,
            ),
            roughness_hint: lerp(
                base.terrain.roughness_hint,
                modulated.terrain.roughness_hint,
            ),
        },
        structures: StructureLayerParams {
            density: lerp(base.structures.density, modulated.structures.density),
            emissive: lerp(base.structures.emissive, modulated.structures.emissive),
        },
        atmosphere: AtmosphereLayerParams {
            hue_drift: lerp(base.atmosphere.hue_drift, modulated.atmosphere.hue_drift),
            fog_density: lerp(
                base.atmosphere.fog_density,
                modulated.atmosphere.fog_density,
            ),
        },
        lighting: LightingLayerParams {
            flash_envelope: lerp(
                base.lighting.flash_envelope,
                modulated.lighting.flash_envelope,
            ),
            exposure: lerp(base.lighting.exposure, modulated.lighting.exposure),
        },
        particles: ParticleLayerParams {
            density_multiplier: lerp(
                base.particles.density_multiplier,
                modulated.particles.density_multiplier,
            ),
            brightness: lerp(base.particles.brightness, modulated.particles.brightness),
        },
        camera_hints: CameraLayerHints {
            drift: lerp(base.camera_hints.drift, modulated.camera_hints.drift),
            fov_bias: lerp(base.camera_hints.fov_bias, modulated.camera_hints.fov_bias),
        },
    }
}

fn structure_density_for_set(set: StructureSet) -> f32 {
    match set {
        StructureSet::Dense => 0.86,
        StructureSet::Sparse => 0.36,
        StructureSet::Minimal => 0.14,
    }
}

fn build_sequencer_state(theme: VisualTheme) -> SequencerState {
    match theme {
        VisualTheme::Electronic => SequencerState {
            bpm: 132.0,
            beat_index: 12,
            bar_index: 3,
            bass_energy: 0.76,
            mid_energy: 0.58,
            high_energy: 0.71,
        },
        VisualTheme::Jazz => SequencerState {
            bpm: 96.0,
            beat_index: 9,
            bar_index: 4,
            bass_energy: 0.38,
            mid_energy: 0.64,
            high_energy: 0.29,
        },
        VisualTheme::Ambient => SequencerState {
            bpm: 72.0,
            beat_index: 6,
            bar_index: 2,
            bass_energy: 0.27,
            mid_energy: 0.34,
            high_energy: 0.22,
        },
    }
}

fn camera_motion_label(style: CameraRig) -> &'static str {
    match style {
        CameraRig::Orbit => "slow_orbital_center",
        CameraRig::Drift => "gentle_drift_pan",
        CameraRig::Float => "slow_floating_motion",
    }
}

fn sample_time_for_theme(theme: VisualTheme) -> f32 {
    match theme {
        VisualTheme::Electronic => 2.75,
        VisualTheme::Jazz => 5.125,
        VisualTheme::Ambient => 8.0,
    }
}

#[cfg(test)]
mod tests {
    use super::PulseBuilder;
    use crate::pulse_builder::config::{
        AtmosphereType, CameraRig, GeometryStyle, LightingMode, StructureSet,
    };
    use crate::pulse_sequence::{PulsePhaseOverrides, PulseSequence};
    use aurex_render::rhythm_field::VisualTheme;

    #[test]
    fn pulse_builder_is_deterministic_for_same_config() {
        let build = || {
            PulseBuilder::new("Electronic Megacity")
                .seed(42)
                .theme(VisualTheme::Electronic)
                .geometry_style(GeometryStyle::City)
                .structures(StructureSet::Dense)
                .lighting(LightingMode::NeonPulse)
                .atmosphere(AtmosphereType::Clear)
                .color_palette("neon_cyan_magenta")
                .camera_rig(CameraRig::Orbit)
                .rhythm_intensity(1.0)
                .particle_density_multiplier(0.82)
                .build()
        };

        assert_eq!(build(), build());
    }

    #[test]
    fn sequence_override_application_is_predictable() {
        let sequence = PulseSequence::new()
            .add_phase("Silence", 1.0)
            .add_phase_with_overrides(
                "Reveal",
                5.0,
                PulsePhaseOverrides {
                    lighting_override: Some(LightingMode::WarmGlow),
                    atmosphere_override: None,
                    particle_override: Some(0.4),
                    camera_override: None,
                    rhythm_intensity_override: Some(0.2),
                    structure_override: Some(StructureSet::Sparse),
                },
            );

        let pulse = PulseBuilder::new("Seq Demo")
            .seed(9)
            .theme(VisualTheme::Electronic)
            .geometry_style(GeometryStyle::City)
            .structures(StructureSet::Dense)
            .lighting(LightingMode::NeonPulse)
            .atmosphere(AtmosphereType::Clear)
            .camera_rig(CameraRig::Orbit)
            .rhythm_intensity(1.0)
            .particle_density_multiplier(0.82)
            .sequence(sequence)
            .build();

        assert_eq!(pulse.current_phase_name.as_deref(), Some("Reveal"));
        assert_eq!(pulse.pulse_config.lighting_mode, LightingMode::WarmGlow);
        assert_eq!(pulse.pulse_config.structure_set, StructureSet::Sparse);
        assert!(pulse.pulse_config.rhythm_intensity < 1.0);
        assert!(pulse.sequence_duration_seconds.unwrap_or_default() > 0.0);
    }
}
