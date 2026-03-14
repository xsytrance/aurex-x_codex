use super::snapshot::RhythmFieldSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisualTheme {
    #[default]
    Electronic,
    Jazz,
    Ambient,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainLayerParams {
    pub amplitude_hint: f32,
    pub roughness_hint: f32,
}

impl Default for TerrainLayerParams {
    fn default() -> Self {
        Self {
            amplitude_hint: 0.5,
            roughness_hint: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StructureLayerParams {
    pub density: f32,
    pub emissive: f32,
}

impl Default for StructureLayerParams {
    fn default() -> Self {
        Self {
            density: 0.5,
            emissive: 0.35,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtmosphereLayerParams {
    pub hue_drift: f32,
    pub fog_density: f32,
}

impl Default for AtmosphereLayerParams {
    fn default() -> Self {
        Self {
            hue_drift: 0.5,
            fog_density: 0.45,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightingLayerParams {
    pub flash_envelope: f32,
    pub exposure: f32,
}

impl Default for LightingLayerParams {
    fn default() -> Self {
        Self {
            flash_envelope: 0.3,
            exposure: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleLayerParams {
    pub density_multiplier: f32,
    pub brightness: f32,
}

impl Default for ParticleLayerParams {
    fn default() -> Self {
        Self {
            density_multiplier: 0.55,
            brightness: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraLayerHints {
    pub drift: f32,
    pub fov_bias: f32,
}

impl Default for CameraLayerHints {
    fn default() -> Self {
        Self {
            drift: 0.5,
            fov_bias: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GeneratorStackOutput {
    pub terrain: TerrainLayerParams,
    pub structures: StructureLayerParams,
    pub atmosphere: AtmosphereLayerParams,
    pub lighting: LightingLayerParams,
    pub particles: ParticleLayerParams,
    pub camera_hints: CameraLayerHints,
}

#[derive(Debug, Clone, Copy)]
struct ThemeWeights {
    terrain: f32,
    structures: f32,
    atmosphere: f32,
    lighting: f32,
    particles: f32,
    camera: f32,
}

pub fn apply_rhythm_modulation(
    snapshot: &RhythmFieldSnapshot,
    stack_output: &GeneratorStackOutput,
    theme: VisualTheme,
) -> GeneratorStackOutput {
    let w = weights_for_theme(theme);

    let terrain = TerrainLayerParams {
        amplitude_hint: bounded_add(
            stack_output.terrain.amplitude_hint,
            snapshot.bass_energy,
            0.18 * w.terrain,
        ),
        roughness_hint: bounded_add(
            stack_output.terrain.roughness_hint,
            snapshot.mid_energy,
            0.10 * w.terrain,
        ),
    };

    let structures = StructureLayerParams {
        density: bounded_add(
            stack_output.structures.density,
            snapshot.intensity,
            0.08 * w.structures,
        ),
        emissive: bounded_add(
            stack_output.structures.emissive,
            snapshot.accent,
            0.25 * w.structures,
        ),
    };

    let atmosphere = AtmosphereLayerParams {
        hue_drift: bounded_add(
            stack_output.atmosphere.hue_drift,
            snapshot.bar_phase,
            0.22 * w.atmosphere,
        ),
        fog_density: bounded_add(
            stack_output.atmosphere.fog_density,
            snapshot.mid_energy,
            0.10 * w.atmosphere,
        ),
    };

    let lighting = LightingLayerParams {
        flash_envelope: bounded_add(
            stack_output.lighting.flash_envelope,
            snapshot.pulse,
            0.3 * w.lighting,
        ),
        exposure: bounded_add(
            stack_output.lighting.exposure,
            snapshot.intensity,
            0.12 * w.lighting,
        ),
    };

    let particles = ParticleLayerParams {
        density_multiplier: bounded_add(
            stack_output.particles.density_multiplier,
            snapshot.intensity,
            0.2 * w.particles,
        ),
        brightness: bounded_add(
            stack_output.particles.brightness,
            snapshot.high_energy,
            0.22 * w.particles,
        ),
    };

    let camera_hints = CameraLayerHints {
        drift: bounded_add(
            stack_output.camera_hints.drift,
            snapshot.beat_phase,
            0.05 * w.camera,
        ),
        fov_bias: bounded_add(
            stack_output.camera_hints.fov_bias,
            snapshot.pulse,
            0.04 * w.camera,
        ),
    };

    GeneratorStackOutput {
        terrain,
        structures,
        atmosphere,
        lighting,
        particles,
        camera_hints,
    }
}

fn bounded_add(base: f32, signal: f32, amount: f32) -> f32 {
    (base + signal.clamp(0.0, 1.0) * amount).clamp(0.0, 1.0)
}

fn weights_for_theme(theme: VisualTheme) -> ThemeWeights {
    match theme {
        VisualTheme::Electronic => ThemeWeights {
            terrain: 0.9,
            structures: 1.0,
            atmosphere: 0.75,
            lighting: 1.2,
            particles: 1.15,
            camera: 0.85,
        },
        VisualTheme::Jazz => ThemeWeights {
            terrain: 0.7,
            structures: 0.85,
            atmosphere: 1.2,
            lighting: 0.7,
            particles: 0.8,
            camera: 0.9,
        },
        VisualTheme::Ambient => ThemeWeights {
            terrain: 0.65,
            structures: 0.7,
            atmosphere: 1.05,
            lighting: 0.6,
            particles: 0.75,
            camera: 0.75,
        },
    }
}
