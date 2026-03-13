use crate::CameraRig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualTheme {
    Reactor,
    Cathedral,
    DesertMonolith,
    StormField,
    NeonCity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryStyle {
    Monolith,
    SpireField,
    OrbitalRings,
    FractalLattice,
    CathedralArches,
    EnergyPillars,
    FloatingIslands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosphereType {
    Void,
    Mist,
    VolumetricFog,
    EnergyStorm,
    DustField,
    Aurora,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    NeonPulse,
    ReactorGlow,
    SunsetGradient,
    CrystalRefraction,
    LightningFlash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureSet {
    ReactorCore,
    Cathedral,
    DesertMonolith,
    NeonCity,
    StormField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPalette {
    pub primary: u32,
    pub secondary: u32,
    pub accent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldBlueprint {
    pub theme: VisualTheme,
    pub geometry_style: GeometryStyle,
    pub structure_set: StructureSet,
    pub atmosphere: AtmosphereType,
    pub lighting: LightingMode,
    pub color_palette: ColorPalette,
    pub camera_rig: CameraRig,
}

#[inline]
fn splitmix_u64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[inline]
fn pick_color(seed: u64) -> u32 {
    let r = (splitmix_u64(seed ^ 0x100) & 0xFF) as u32;
    let g = (splitmix_u64(seed ^ 0x200) & 0xFF) as u32;
    let b = (splitmix_u64(seed ^ 0x300) & 0xFF) as u32;
    (r << 16) | (g << 8) | b
}

pub fn generate_world_blueprint(seed: u64, theme: VisualTheme) -> WorldBlueprint {
    let structure_set = match theme {
        VisualTheme::Reactor => StructureSet::ReactorCore,
        VisualTheme::Cathedral => StructureSet::Cathedral,
        VisualTheme::DesertMonolith => StructureSet::DesertMonolith,
        VisualTheme::StormField => StructureSet::StormField,
        VisualTheme::NeonCity => StructureSet::NeonCity,
    };

    let geometry_style = match splitmix_u64(seed ^ 0xA100) % 7 {
        0 => GeometryStyle::Monolith,
        1 => GeometryStyle::SpireField,
        2 => GeometryStyle::OrbitalRings,
        3 => GeometryStyle::FractalLattice,
        4 => GeometryStyle::CathedralArches,
        5 => GeometryStyle::EnergyPillars,
        _ => GeometryStyle::FloatingIslands,
    };

    let atmosphere = match splitmix_u64(seed ^ 0xA200) % 6 {
        0 => AtmosphereType::Void,
        1 => AtmosphereType::Mist,
        2 => AtmosphereType::VolumetricFog,
        3 => AtmosphereType::EnergyStorm,
        4 => AtmosphereType::DustField,
        _ => AtmosphereType::Aurora,
    };

    let lighting = match theme {
        VisualTheme::Reactor => LightingMode::ReactorGlow,
        VisualTheme::Cathedral => LightingMode::CrystalRefraction,
        VisualTheme::DesertMonolith => LightingMode::SunsetGradient,
        VisualTheme::StormField => LightingMode::LightningFlash,
        VisualTheme::NeonCity => LightingMode::NeonPulse,
    };

    let camera_rig = match theme {
        VisualTheme::Reactor => CameraRig::ReactorDive,
        VisualTheme::Cathedral => CameraRig::Orbit,
        VisualTheme::DesertMonolith => CameraRig::Flyby,
        VisualTheme::StormField => CameraRig::PulseOrbit,
        VisualTheme::NeonCity => CameraRig::Flyby,
    };

    let color_palette = ColorPalette {
        primary: pick_color(seed ^ 0xB100),
        secondary: pick_color(seed ^ 0xB200),
        accent: pick_color(seed ^ 0xB300),
    };

    WorldBlueprint {
        theme,
        geometry_style,
        structure_set,
        atmosphere,
        lighting,
        color_palette,
        camera_rig,
    }
}

#[cfg(test)]
mod tests {
    use super::{VisualTheme, generate_world_blueprint};

    #[test]
    fn world_blueprint_is_deterministic() {
        let a = generate_world_blueprint(42, VisualTheme::Reactor);
        let b = generate_world_blueprint(42, VisualTheme::Reactor);
        assert_eq!(a, b);
    }

    #[test]
    fn theme_drives_structure_and_lighting() {
        let reactor = generate_world_blueprint(1, VisualTheme::Reactor);
        let cathedral = generate_world_blueprint(1, VisualTheme::Cathedral);
        assert_ne!(reactor.structure_set, cathedral.structure_set);
        assert_ne!(reactor.lighting, cathedral.lighting);
    }
}
