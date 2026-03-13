use aurex_render::rhythm_field::{
    AtmosphereLayerParams, CameraLayerHints, GeneratorStackOutput, LightingLayerParams,
    ParticleLayerParams, RhythmFieldSnapshot, SequencerState, StructureLayerParams,
    TerrainLayerParams, VisualTheme, apply_rhythm_modulation, sample_rhythm_field,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingStyle {
    Neon,
    Warm,
    Diffuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraStyle {
    Orbit,
    Drift,
    Float,
}

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
    pub world_blueprint: WorldBlueprint,
    pub generator_output: GeneratorStackOutput,
    pub rhythm_snapshot: RhythmFieldSnapshot,
    pub modulated_output: GeneratorStackOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PulseConfig {
    pub name: String,
    pub theme: VisualTheme,
    pub seed: u64,
    pub structure_density_hint: f32,
    pub lighting_style: LightingStyle,
    pub particle_intensity_hint: f32,
    pub camera_style: CameraStyle,
    pub rhythm_reactivity: f32,
}

pub struct PulseBuilder {
    config: PulseConfig,
}

impl PulseBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: PulseConfig {
                name: name.into(),
                theme: VisualTheme::Electronic,
                seed: 0,
                structure_density_hint: 0.5,
                lighting_style: LightingStyle::Diffuse,
                particle_intensity_hint: 0.5,
                camera_style: CameraStyle::Drift,
                rhythm_reactivity: 0.5,
            },
        }
    }

    pub fn theme(mut self, theme: VisualTheme) -> Self {
        self.config.theme = theme;
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = seed;
        self
    }

    pub fn structure_density(mut self, value: f32) -> Self {
        self.config.structure_density_hint = value.clamp(0.0, 1.0);
        self
    }

    pub fn lighting_style(mut self, style: LightingStyle) -> Self {
        self.config.lighting_style = style;
        self
    }

    pub fn particle_intensity(mut self, value: f32) -> Self {
        self.config.particle_intensity_hint = value.clamp(0.0, 1.0);
        self
    }

    pub fn camera_style(mut self, style: CameraStyle) -> Self {
        self.config.camera_style = style;
        self
    }

    pub fn rhythm_reactivity(mut self, value: f32) -> Self {
        self.config.rhythm_reactivity = value.clamp(0.0, 1.0);
        self
    }

    pub fn build(self) -> ExamplePulseConfig {
        let config = self.config;

        let world_blueprint = WorldBlueprint {
            name: config.name.clone(),
            theme: config.theme,
            palette_hint: palette_for_theme(config.theme).to_string(),
            camera_motion: camera_motion_label(config.camera_style).to_string(),
        };

        let generator_output = build_generator_output(&config);
        let sequencer_state = build_sequencer_state(&config);
        let rhythm_snapshot = sample_rhythm_field(
            config.seed,
            sample_time_for_theme(config.theme),
            sequencer_state,
        );

        let modulated_once =
            apply_rhythm_modulation(&rhythm_snapshot, &generator_output, config.theme);
        let modulated_output =
            blend_stack_output(generator_output, modulated_once, config.rhythm_reactivity);

        ExamplePulseConfig {
            pulse_name: config.name,
            world_blueprint,
            generator_output,
            rhythm_snapshot,
            modulated_output,
        }
    }
}

fn build_generator_output(config: &PulseConfig) -> GeneratorStackOutput {
    let density = config.structure_density_hint;
    let particle_intensity = config.particle_intensity_hint;

    let (flash_envelope, exposure) = match config.lighting_style {
        LightingStyle::Neon => (0.72, 0.72),
        LightingStyle::Warm => (0.26, 0.56),
        LightingStyle::Diffuse => (0.16, 0.44),
    };

    let (drift, fov_bias) = match config.camera_style {
        CameraStyle::Orbit => (0.46, 0.6),
        CameraStyle::Drift => (0.62, 0.5),
        CameraStyle::Float => (0.68, 0.44),
    };

    let atmosphere_base = match config.theme {
        VisualTheme::Electronic => (0.48, 0.3),
        VisualTheme::Jazz => (0.66, 0.74),
        VisualTheme::Ambient => (0.72, 0.82),
    };

    GeneratorStackOutput {
        terrain: TerrainLayerParams {
            amplitude_hint: (0.28 + density * 0.4).clamp(0.0, 1.0),
            roughness_hint: (0.24 + density * 0.34).clamp(0.0, 1.0),
        },
        structures: StructureLayerParams {
            density,
            emissive: (0.14 + density * 0.68).clamp(0.0, 1.0),
        },
        atmosphere: AtmosphereLayerParams {
            hue_drift: atmosphere_base.0,
            fog_density: atmosphere_base.1,
        },
        lighting: LightingLayerParams {
            flash_envelope,
            exposure,
        },
        particles: ParticleLayerParams {
            density_multiplier: particle_intensity,
            brightness: (0.1 + particle_intensity * 0.78).clamp(0.0, 1.0),
        },
        camera_hints: CameraLayerHints { drift, fov_bias },
    }
}

fn build_sequencer_state(config: &PulseConfig) -> SequencerState {
    match config.theme {
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

fn palette_for_theme(theme: VisualTheme) -> &'static str {
    match theme {
        VisualTheme::Electronic => "neon_cyan_magenta",
        VisualTheme::Jazz => "warm_amber_teal",
        VisualTheme::Ambient => "mist_blue_violet",
    }
}

fn camera_motion_label(style: CameraStyle) -> &'static str {
    match style {
        CameraStyle::Orbit => "slow_orbital_center",
        CameraStyle::Drift => "gentle_drift_pan",
        CameraStyle::Float => "slow_floating_motion",
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
    use super::{CameraStyle, LightingStyle, PulseBuilder};
    use aurex_render::rhythm_field::VisualTheme;

    #[test]
    fn pulse_builder_is_deterministic_for_same_config() {
        let a = PulseBuilder::new("Electronic Megacity")
            .theme(VisualTheme::Electronic)
            .seed(42)
            .structure_density(0.9)
            .particle_intensity(0.8)
            .lighting_style(LightingStyle::Neon)
            .camera_style(CameraStyle::Orbit)
            .rhythm_reactivity(1.0)
            .build();
        let b = PulseBuilder::new("Electronic Megacity")
            .theme(VisualTheme::Electronic)
            .seed(42)
            .structure_density(0.9)
            .particle_intensity(0.8)
            .lighting_style(LightingStyle::Neon)
            .camera_style(CameraStyle::Orbit)
            .rhythm_reactivity(1.0)
            .build();

        assert_eq!(a, b);
    }
}
