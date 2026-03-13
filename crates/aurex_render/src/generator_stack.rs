use crate::CameraRig;
use crate::world_generator::{
    AtmosphereType, ColorPalette, GeometryStyle, LightingMode, StructureSet, WorldBlueprint,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainLayerParams {
    pub spatial_frequency: f32,
    pub elevation_amplitude: f32,
    pub radial_bias: f32,
    pub ridge_sharpness: f32,
    pub base_color: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureLayerParams {
    pub set: StructureSet,
    pub density: f32,
    pub verticality: f32,
    pub spacing: f32,
    pub accent_color: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtmosphereLayerParams {
    pub kind: AtmosphereType,
    pub fog_density: f32,
    pub haze: f32,
    pub sky_energy: f32,
    pub volumetric: f32,
    pub tint_color: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightingLayerParams {
    pub mode: LightingMode,
    pub key_intensity: f32,
    pub fill_intensity: f32,
    pub pulse_amount: f32,
    pub temperature_shift: f32,
    pub primary_color: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleLayerParams {
    pub spawn_rate: f32,
    pub drift: f32,
    pub turbulence: f32,
    pub emissive: f32,
    pub color: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLayerHints {
    pub preferred_rig: CameraRig,
    pub motion_intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratorStackOutput {
    pub terrain: TerrainLayerParams,
    pub structures: StructureLayerParams,
    pub atmosphere: AtmosphereLayerParams,
    pub lighting: LightingLayerParams,
    pub particles: ParticleLayerParams,
    pub camera_hints: CameraLayerHints,
}

#[inline]
fn splitmix_u64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[inline]
fn scalar(seed: u64) -> f32 {
    let raw = (splitmix_u64(seed) >> 40) as u32;
    raw as f32 / ((1_u32 << 24) - 1) as f32
}

#[inline]
fn range(seed: u64, min: f32, max: f32) -> f32 {
    min + scalar(seed) * (max - min)
}

fn terrain_from_geometry(
    seed: u64,
    style: GeometryStyle,
    palette: ColorPalette,
) -> TerrainLayerParams {
    match style {
        GeometryStyle::Monolith => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1101, 0.08, 0.16),
            elevation_amplitude: range(seed ^ 0x1102, 0.2, 0.4),
            radial_bias: range(seed ^ 0x1103, 0.65, 0.95),
            ridge_sharpness: range(seed ^ 0x1104, 0.7, 1.0),
            base_color: palette.primary,
        },
        GeometryStyle::SpireField => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1201, 0.2, 0.35),
            elevation_amplitude: range(seed ^ 0x1202, 0.45, 0.8),
            radial_bias: range(seed ^ 0x1203, 0.3, 0.55),
            ridge_sharpness: range(seed ^ 0x1204, 0.6, 0.95),
            base_color: palette.secondary,
        },
        GeometryStyle::OrbitalRings => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1301, 0.12, 0.2),
            elevation_amplitude: range(seed ^ 0x1302, 0.25, 0.5),
            radial_bias: range(seed ^ 0x1303, 0.75, 1.1),
            ridge_sharpness: range(seed ^ 0x1304, 0.4, 0.75),
            base_color: palette.accent,
        },
        GeometryStyle::FractalLattice => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1401, 0.25, 0.5),
            elevation_amplitude: range(seed ^ 0x1402, 0.3, 0.65),
            radial_bias: range(seed ^ 0x1403, 0.4, 0.7),
            ridge_sharpness: range(seed ^ 0x1404, 0.7, 1.2),
            base_color: palette.primary,
        },
        GeometryStyle::CathedralArches => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1501, 0.09, 0.2),
            elevation_amplitude: range(seed ^ 0x1502, 0.4, 0.7),
            radial_bias: range(seed ^ 0x1503, 0.5, 0.85),
            ridge_sharpness: range(seed ^ 0x1504, 0.5, 0.8),
            base_color: palette.secondary,
        },
        GeometryStyle::EnergyPillars => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1601, 0.2, 0.35),
            elevation_amplitude: range(seed ^ 0x1602, 0.6, 1.0),
            radial_bias: range(seed ^ 0x1603, 0.45, 0.8),
            ridge_sharpness: range(seed ^ 0x1604, 0.65, 1.0),
            base_color: palette.accent,
        },
        GeometryStyle::FloatingIslands => TerrainLayerParams {
            spatial_frequency: range(seed ^ 0x1701, 0.18, 0.3),
            elevation_amplitude: range(seed ^ 0x1702, 0.5, 0.9),
            radial_bias: range(seed ^ 0x1703, 0.2, 0.5),
            ridge_sharpness: range(seed ^ 0x1704, 0.35, 0.65),
            base_color: palette.primary,
        },
    }
}

fn structures_from_set(
    seed: u64,
    set: StructureSet,
    palette: ColorPalette,
) -> StructureLayerParams {
    match set {
        StructureSet::ReactorCore => StructureLayerParams {
            set,
            density: range(seed ^ 0x2101, 0.55, 0.8),
            verticality: range(seed ^ 0x2102, 0.7, 1.0),
            spacing: range(seed ^ 0x2103, 0.12, 0.24),
            accent_color: palette.accent,
        },
        StructureSet::Cathedral => StructureLayerParams {
            set,
            density: range(seed ^ 0x2201, 0.3, 0.5),
            verticality: range(seed ^ 0x2202, 0.8, 1.1),
            spacing: range(seed ^ 0x2203, 0.25, 0.4),
            accent_color: palette.secondary,
        },
        StructureSet::DesertMonolith => StructureLayerParams {
            set,
            density: range(seed ^ 0x2301, 0.15, 0.35),
            verticality: range(seed ^ 0x2302, 0.45, 0.75),
            spacing: range(seed ^ 0x2303, 0.45, 0.75),
            accent_color: palette.primary,
        },
        StructureSet::NeonCity => StructureLayerParams {
            set,
            density: range(seed ^ 0x2401, 0.6, 0.9),
            verticality: range(seed ^ 0x2402, 0.65, 1.0),
            spacing: range(seed ^ 0x2403, 0.15, 0.3),
            accent_color: palette.accent,
        },
        StructureSet::StormField => StructureLayerParams {
            set,
            density: range(seed ^ 0x2501, 0.25, 0.45),
            verticality: range(seed ^ 0x2502, 0.55, 0.85),
            spacing: range(seed ^ 0x2503, 0.35, 0.6),
            accent_color: palette.secondary,
        },
    }
}

fn atmosphere_from_type(
    seed: u64,
    kind: AtmosphereType,
    palette: ColorPalette,
) -> AtmosphereLayerParams {
    match kind {
        AtmosphereType::Void => AtmosphereLayerParams {
            kind,
            fog_density: range(seed ^ 0x3101, 0.0, 0.08),
            haze: range(seed ^ 0x3102, 0.0, 0.1),
            sky_energy: range(seed ^ 0x3103, 0.25, 0.45),
            volumetric: range(seed ^ 0x3104, 0.0, 0.1),
            tint_color: palette.primary,
        },
        AtmosphereType::Mist => AtmosphereLayerParams {
            kind,
            fog_density: range(seed ^ 0x3201, 0.25, 0.45),
            haze: range(seed ^ 0x3202, 0.25, 0.5),
            sky_energy: range(seed ^ 0x3203, 0.3, 0.6),
            volumetric: range(seed ^ 0x3204, 0.2, 0.4),
            tint_color: palette.secondary,
        },
        AtmosphereType::VolumetricFog => AtmosphereLayerParams {
            kind,
            fog_density: range(seed ^ 0x3301, 0.45, 0.75),
            haze: range(seed ^ 0x3302, 0.35, 0.7),
            sky_energy: range(seed ^ 0x3303, 0.25, 0.5),
            volumetric: range(seed ^ 0x3304, 0.55, 0.9),
            tint_color: palette.accent,
        },
        AtmosphereType::EnergyStorm => AtmosphereLayerParams {
            kind,
            fog_density: range(seed ^ 0x3401, 0.3, 0.6),
            haze: range(seed ^ 0x3402, 0.3, 0.55),
            sky_energy: range(seed ^ 0x3403, 0.7, 1.0),
            volumetric: range(seed ^ 0x3404, 0.45, 0.8),
            tint_color: palette.accent,
        },
        AtmosphereType::DustField => AtmosphereLayerParams {
            kind,
            fog_density: range(seed ^ 0x3501, 0.2, 0.5),
            haze: range(seed ^ 0x3502, 0.4, 0.8),
            sky_energy: range(seed ^ 0x3503, 0.25, 0.55),
            volumetric: range(seed ^ 0x3504, 0.05, 0.25),
            tint_color: palette.primary,
        },
        AtmosphereType::Aurora => AtmosphereLayerParams {
            kind,
            fog_density: range(seed ^ 0x3601, 0.08, 0.2),
            haze: range(seed ^ 0x3602, 0.2, 0.4),
            sky_energy: range(seed ^ 0x3603, 0.65, 1.0),
            volumetric: range(seed ^ 0x3604, 0.2, 0.45),
            tint_color: palette.secondary,
        },
    }
}

fn lighting_from_mode(seed: u64, mode: LightingMode, palette: ColorPalette) -> LightingLayerParams {
    match mode {
        LightingMode::NeonPulse => LightingLayerParams {
            mode,
            key_intensity: range(seed ^ 0x4101, 0.8, 1.2),
            fill_intensity: range(seed ^ 0x4102, 0.2, 0.45),
            pulse_amount: range(seed ^ 0x4103, 0.55, 0.9),
            temperature_shift: range(seed ^ 0x4104, -0.15, 0.1),
            primary_color: palette.accent,
        },
        LightingMode::ReactorGlow => LightingLayerParams {
            mode,
            key_intensity: range(seed ^ 0x4201, 0.7, 1.0),
            fill_intensity: range(seed ^ 0x4202, 0.35, 0.65),
            pulse_amount: range(seed ^ 0x4203, 0.45, 0.75),
            temperature_shift: range(seed ^ 0x4204, -0.05, 0.2),
            primary_color: palette.primary,
        },
        LightingMode::SunsetGradient => LightingLayerParams {
            mode,
            key_intensity: range(seed ^ 0x4301, 0.55, 0.85),
            fill_intensity: range(seed ^ 0x4302, 0.25, 0.5),
            pulse_amount: range(seed ^ 0x4303, 0.1, 0.3),
            temperature_shift: range(seed ^ 0x4304, 0.25, 0.6),
            primary_color: palette.secondary,
        },
        LightingMode::CrystalRefraction => LightingLayerParams {
            mode,
            key_intensity: range(seed ^ 0x4401, 0.6, 0.95),
            fill_intensity: range(seed ^ 0x4402, 0.3, 0.55),
            pulse_amount: range(seed ^ 0x4403, 0.15, 0.35),
            temperature_shift: range(seed ^ 0x4404, -0.2, 0.05),
            primary_color: palette.secondary,
        },
        LightingMode::LightningFlash => LightingLayerParams {
            mode,
            key_intensity: range(seed ^ 0x4501, 0.9, 1.4),
            fill_intensity: range(seed ^ 0x4502, 0.1, 0.35),
            pulse_amount: range(seed ^ 0x4503, 0.65, 1.0),
            temperature_shift: range(seed ^ 0x4504, -0.35, -0.05),
            primary_color: palette.accent,
        },
    }
}

fn particles_from_blueprint(seed: u64, blueprint: &WorldBlueprint) -> ParticleLayerParams {
    let base = match blueprint.atmosphere {
        AtmosphereType::Void => 0.05,
        AtmosphereType::Mist => 0.25,
        AtmosphereType::VolumetricFog => 0.45,
        AtmosphereType::EnergyStorm => 0.65,
        AtmosphereType::DustField => 0.5,
        AtmosphereType::Aurora => 0.35,
    };
    let lighting_bias = match blueprint.lighting {
        LightingMode::NeonPulse => 0.2,
        LightingMode::ReactorGlow => 0.14,
        LightingMode::SunsetGradient => 0.06,
        LightingMode::CrystalRefraction => 0.1,
        LightingMode::LightningFlash => 0.24,
    };

    ParticleLayerParams {
        spawn_rate: (base + lighting_bias + range(seed ^ 0x5101, 0.0, 0.2)).clamp(0.0, 1.2),
        drift: range(seed ^ 0x5102, 0.1, 0.9),
        turbulence: range(seed ^ 0x5103, 0.05, 0.85),
        emissive: range(seed ^ 0x5104, 0.2, 1.0),
        color: blueprint.color_palette.accent,
    }
}

pub fn generate_stack_output(seed: u64, blueprint: &WorldBlueprint) -> GeneratorStackOutput {
    let terrain = terrain_from_geometry(
        seed ^ 0xA001,
        blueprint.geometry_style,
        blueprint.color_palette,
    );
    let structures = structures_from_set(
        seed ^ 0xA002,
        blueprint.structure_set,
        blueprint.color_palette,
    );
    let atmosphere =
        atmosphere_from_type(seed ^ 0xA003, blueprint.atmosphere, blueprint.color_palette);
    let lighting = lighting_from_mode(seed ^ 0xA004, blueprint.lighting, blueprint.color_palette);
    let particles = particles_from_blueprint(seed ^ 0xA005, blueprint);

    GeneratorStackOutput {
        terrain,
        structures,
        atmosphere,
        lighting,
        particles,
        camera_hints: CameraLayerHints {
            preferred_rig: blueprint.camera_rig,
            motion_intensity: range(seed ^ 0xA006, 0.15, 0.9),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::generate_stack_output;
    use crate::world_generator::{VisualTheme, generate_world_blueprint};

    #[test]
    fn stack_output_is_deterministic() {
        let blueprint = generate_world_blueprint(101, VisualTheme::Reactor);
        let a = generate_stack_output(55, &blueprint);
        let b = generate_stack_output(55, &blueprint);
        assert_eq!(a, b);
    }

    #[test]
    fn stack_layers_change_with_different_blueprints() {
        let reactor = generate_world_blueprint(9, VisualTheme::Reactor);
        let cathedral = generate_world_blueprint(9, VisualTheme::Cathedral);

        let reactor_stack = generate_stack_output(33, &reactor);
        let cathedral_stack = generate_stack_output(33, &cathedral);

        assert_ne!(reactor_stack.structures.set, cathedral_stack.structures.set);
        assert_ne!(reactor_stack.lighting.mode, cathedral_stack.lighting.mode);
        assert_ne!(
            reactor_stack.structures.accent_color,
            cathedral_stack.structures.accent_color
        );
    }
}
