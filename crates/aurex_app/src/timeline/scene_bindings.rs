use super::scene_manager::SceneLayerState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneVisualProfile {
    pub geometry_density: f32,
    pub structure_scale: f32,
    pub structure_height: f32,
    pub structure_complexity: f32,
    pub particle_density: f32,
    pub fog_density: f32,
    pub glow_intensity: f32,
    pub starfield_enabled: bool,
    pub logo_enabled: bool,
}

impl Default for SceneVisualProfile {
    fn default() -> Self {
        Self {
            geometry_density: 0.5,
            structure_scale: 0.5,
            structure_height: 0.5,
            structure_complexity: 0.5,
            particle_density: 0.5,
            fog_density: 0.4,
            glow_intensity: 0.5,
            starfield_enabled: false,
            logo_enabled: false,
        }
    }
}

pub fn profile_for_scene(scene_id: &str) -> SceneVisualProfile {
    match scene_id {
        // megacity: dense, tall skyline structures
        "megacity_skyline" => SceneVisualProfile {
            geometry_density: 0.95,
            structure_scale: 0.78,
            structure_height: 0.94,
            structure_complexity: 0.86,
            particle_density: 0.28,
            fog_density: 0.24,
            glow_intensity: 0.88,
            starfield_enabled: false,
            logo_enabled: false,
        },
        // jazz: medium structure presence with heavy fog / warm lighting profile
        "jazz_lounge" => SceneVisualProfile {
            geometry_density: 0.52,
            structure_scale: 0.58,
            structure_height: 0.48,
            structure_complexity: 0.54,
            particle_density: 0.24,
            fog_density: 0.76,
            glow_intensity: 0.58,
            starfield_enabled: false,
            logo_enabled: false,
        },
        // ambient: minimal geometry, heavy atmosphere
        "ambient_mist" => SceneVisualProfile {
            geometry_density: 0.12,
            structure_scale: 0.32,
            structure_height: 0.22,
            structure_complexity: 0.28,
            particle_density: 0.18,
            fog_density: 0.90,
            glow_intensity: 0.34,
            starfield_enabled: true,
            logo_enabled: false,
        },
        // aurielle intro: low geometry, strong particles, light starfield
        "boot_pulse" => SceneVisualProfile {
            geometry_density: 0.30,
            structure_scale: 0.42,
            structure_height: 0.32,
            structure_complexity: 0.44,
            particle_density: 0.74,
            fog_density: 0.34,
            glow_intensity: 0.75,
            starfield_enabled: true,
            logo_enabled: false,
        },
        "aurex_logo" => SceneVisualProfile {
            geometry_density: 0.20,
            structure_scale: 0.36,
            structure_height: 0.22,
            structure_complexity: 0.35,
            particle_density: 0.14,
            fog_density: 0.24,
            glow_intensity: 0.85,
            starfield_enabled: false,
            logo_enabled: true,
        },
        "rings" => SceneVisualProfile {
            geometry_density: 0.64,
            structure_scale: 0.74,
            structure_height: 0.68,
            structure_complexity: 0.58,
            particle_density: 0.38,
            fog_density: 0.32,
            glow_intensity: 0.68,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "particle_swirl" => SceneVisualProfile {
            geometry_density: 0.30,
            structure_scale: 0.44,
            structure_height: 0.36,
            structure_complexity: 0.50,
            particle_density: 0.92,
            fog_density: 0.46,
            glow_intensity: 0.72,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "starfield_expansion" => SceneVisualProfile {
            geometry_density: 0.22,
            structure_scale: 0.38,
            structure_height: 0.30,
            structure_complexity: 0.34,
            particle_density: 0.68,
            fog_density: 0.30,
            glow_intensity: 0.78,
            starfield_enabled: true,
            logo_enabled: false,
        },
        "aurielle_reveal_scene" | "aurielle_reveal" => SceneVisualProfile {
            geometry_density: 0.26,
            structure_scale: 0.44,
            structure_height: 0.34,
            structure_complexity: 0.46,
            particle_density: 0.82,
            fog_density: 0.40,
            glow_intensity: 0.95,
            starfield_enabled: true,
            logo_enabled: true,
        },
        _ => SceneVisualProfile::default(),
    }
}

pub fn blend_scene_profiles(layers: &[SceneLayerState]) -> SceneVisualProfile {
    if layers.is_empty() {
        return SceneVisualProfile::default();
    }

    let total_weight = layers
        .iter()
        .map(|layer| layer.weight.max(0.0))
        .sum::<f32>()
        .max(1e-5);

    let mut geometry_density = 0.0;
    let mut structure_scale = 0.0;
    let mut structure_height = 0.0;
    let mut structure_complexity = 0.0;
    let mut particle_density = 0.0;
    let mut fog_density = 0.0;
    let mut glow_intensity = 0.0;
    let mut starfield_score = 0.0;
    let mut logo_score = 0.0;

    for layer in layers {
        let w = layer.weight.max(0.0) / total_weight;
        let profile = profile_for_scene(&layer.scene_id);
        geometry_density += profile.geometry_density * w;
        structure_scale += profile.structure_scale * w;
        structure_height += profile.structure_height * w;
        structure_complexity += profile.structure_complexity * w;
        particle_density += profile.particle_density * w;
        fog_density += profile.fog_density * w;
        glow_intensity += profile.glow_intensity * w;
        if profile.starfield_enabled {
            starfield_score += w;
        }
        if profile.logo_enabled {
            logo_score += w;
        }
    }

    SceneVisualProfile {
        geometry_density: geometry_density.clamp(0.0, 1.0),
        structure_scale: structure_scale.clamp(0.0, 1.0),
        structure_height: structure_height.clamp(0.0, 1.0),
        structure_complexity: structure_complexity.clamp(0.0, 1.0),
        particle_density: particle_density.clamp(0.0, 1.0),
        fog_density: fog_density.clamp(0.0, 1.0),
        glow_intensity: glow_intensity.clamp(0.0, 1.0),
        starfield_enabled: starfield_score >= 0.5,
        logo_enabled: logo_score >= 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::{blend_scene_profiles, profile_for_scene};
    use crate::timeline::scene_manager::SceneLayerState;

    #[test]
    fn scene_id_maps_to_expected_profile() {
        let logo = profile_for_scene("aurex_logo");
        assert!(logo.logo_enabled);
        assert!(!logo.starfield_enabled);
        assert!(logo.glow_intensity > 0.8);

        let stars = profile_for_scene("starfield_expansion");
        assert!(stars.starfield_enabled);
        assert!(stars.particle_density > 0.6);
        assert!(stars.structure_height < 0.4);
    }

    #[test]
    fn demo_profiles_are_distinct() {
        let megacity = profile_for_scene("megacity_skyline");
        let jazz = profile_for_scene("jazz_lounge");
        let ambient = profile_for_scene("ambient_mist");
        let aurielle = profile_for_scene("aurielle_reveal_scene");
        assert!(megacity.geometry_density > jazz.geometry_density);
        assert!(megacity.structure_height > jazz.structure_height);
        assert!(ambient.fog_density > jazz.fog_density);
        assert!(ambient.geometry_density < jazz.geometry_density);
        assert!(aurielle.particle_density > megacity.particle_density);
    }

    #[test]
    fn profile_blending_is_deterministic() {
        let layers = vec![
            SceneLayerState {
                scene_id: "rings".to_string(),
                layer: 1,
                weight: 0.4,
            },
            SceneLayerState {
                scene_id: "particle_swirl".to_string(),
                layer: 1,
                weight: 0.6,
            },
        ];

        let a = blend_scene_profiles(&layers);
        let b = blend_scene_profiles(&layers);
        assert_eq!(a, b);
        assert!(a.particle_density > 0.6);
        assert!(a.structure_height > 0.4);
    }
}
