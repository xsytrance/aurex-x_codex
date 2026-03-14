use super::scene_manager::SceneLayerState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneVisualProfile {
    pub geometry_density: f32,
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
        "megacity_skyline" => SceneVisualProfile {
            geometry_density: 0.92,
            particle_density: 0.28,
            fog_density: 0.22,
            glow_intensity: 0.88,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "jazz_lounge" => SceneVisualProfile {
            geometry_density: 0.42,
            particle_density: 0.22,
            fog_density: 0.68,
            glow_intensity: 0.52,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "ambient_mist" => SceneVisualProfile {
            geometry_density: 0.18,
            particle_density: 0.16,
            fog_density: 0.86,
            glow_intensity: 0.34,
            starfield_enabled: true,
            logo_enabled: false,
        },
        "boot_pulse" => SceneVisualProfile {
            geometry_density: 0.35,
            particle_density: 0.2,
            fog_density: 0.3,
            glow_intensity: 0.75,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "aurex_logo" => SceneVisualProfile {
            geometry_density: 0.25,
            particle_density: 0.1,
            fog_density: 0.2,
            glow_intensity: 0.85,
            starfield_enabled: false,
            logo_enabled: true,
        },
        "rings" => SceneVisualProfile {
            geometry_density: 0.6,
            particle_density: 0.35,
            fog_density: 0.32,
            glow_intensity: 0.68,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "particle_swirl" => SceneVisualProfile {
            geometry_density: 0.5,
            particle_density: 0.9,
            fog_density: 0.45,
            glow_intensity: 0.72,
            starfield_enabled: false,
            logo_enabled: false,
        },
        "starfield_expansion" => SceneVisualProfile {
            geometry_density: 0.45,
            particle_density: 0.7,
            fog_density: 0.25,
            glow_intensity: 0.78,
            starfield_enabled: true,
            logo_enabled: false,
        },
        "aurielle_reveal_scene" | "aurielle_reveal" => SceneVisualProfile {
            geometry_density: 0.82,
            particle_density: 0.58,
            fog_density: 0.38,
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
    let mut particle_density = 0.0;
    let mut fog_density = 0.0;
    let mut glow_intensity = 0.0;
    let mut starfield_score = 0.0;
    let mut logo_score = 0.0;

    for layer in layers {
        let w = layer.weight.max(0.0) / total_weight;
        let profile = profile_for_scene(&layer.scene_id);
        geometry_density += profile.geometry_density * w;
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
    }

    #[test]
    fn demo_profiles_are_distinct() {
        let megacity = profile_for_scene("megacity_skyline");
        let jazz = profile_for_scene("jazz_lounge");
        let ambient = profile_for_scene("ambient_mist");
        assert!(megacity.geometry_density > jazz.geometry_density);
        assert!(ambient.fog_density > jazz.fog_density);
        assert!(jazz.glow_intensity != ambient.glow_intensity);
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
    }
}
