use aurex_audio::song_planner::generate_song_plan_for_style;

use crate::{
    determinism::splitmix_u64,
    experience_planner::{ExperiencePlan, VisualTheme, generate_experience},
    identity_engine::{IdentityProfile, StyleBias, ToneType, generate_identity},
};

#[derive(Debug, Clone, PartialEq)]
pub struct CreativeDirective {
    pub identity: IdentityProfile,
    pub experience: ExperiencePlan,
}

pub fn direct_experience(identity_seed: u64, experience_seed: u64) -> CreativeDirective {
    let identity = generate_identity(identity_seed);
    let mut experience = generate_experience(experience_seed);

    // Tone alignment: typography/visual mood
    match identity.tone {
        ToneType::Cyberpunk => {
            experience.visual_theme = VisualTheme::NeonCity;
            experience.typography_style.glow_strength += 0.25;
            experience.typography_style.distortion += 0.18;
        }
        ToneType::Cosmic => {
            experience.visual_theme = VisualTheme::Cathedral;
            experience.typography_style.glow_strength += 0.18;
        }
        ToneType::Mystical => {
            experience.visual_theme = VisualTheme::DesertMonolith;
            experience.typography_style.distortion += 0.08;
        }
        ToneType::Industrial => {
            experience.visual_theme = VisualTheme::StormField;
            experience.typography_style.distortion += 0.22;
        }
        ToneType::Ethereal => {
            experience.visual_theme = VisualTheme::Reactor;
            experience.typography_style.glow_strength += 0.2;
        }
    }

    experience.typography_style.glow_strength =
        experience.typography_style.glow_strength.clamp(0.0, 2.5);
    experience.typography_style.distortion = experience.typography_style.distortion.clamp(0.0, 1.0);

    // Genre bias alignment: target style family by semantic name (no index coupling)
    let style_name = match identity.genre_bias {
        StyleBias::Electronic => "Electronic",
        StyleBias::Jazz => "Jazz",
        StyleBias::World => "World",
        StyleBias::Classical => "Classical",
        StyleBias::Fusion => "Fusion",
    };
    let style_seed = splitmix_u64(experience_seed ^ 0xC0DE_0001);
    experience.song_plan = generate_song_plan_for_style(style_seed, style_name);

    // Identity stamp for cohesion
    experience.title = format!("{} // {}", identity.name, experience.title);

    CreativeDirective {
        identity,
        experience,
    }
}

#[cfg(test)]
mod tests {
    use super::direct_experience;
    use crate::identity_engine::StyleBias;

    #[test]
    fn creative_director_is_deterministic() {
        let a = direct_experience(11, 99);
        let b = direct_experience(11, 99);
        assert_eq!(a, b);
    }

    #[test]
    fn directive_title_contains_identity_name() {
        let d = direct_experience(42, 7);
        assert!(d.experience.title.contains(&d.identity.name));
    }

    #[test]
    fn genre_bias_maps_to_expected_style_family() {
        let style_for = |bias: StyleBias| {
            (0_u64..10_000)
                .find_map(|seed| {
                    let directive = direct_experience(seed, 999);
                    (directive.identity.genre_bias == bias)
                        .then_some(directive.experience.song_plan.style.name)
                })
                .expect("able to find seed for requested genre bias")
        };

        assert_eq!(style_for(StyleBias::Electronic), "Electronic");
        assert_eq!(style_for(StyleBias::Jazz), "Jazz");
        assert_eq!(style_for(StyleBias::World), "World");
        assert_eq!(style_for(StyleBias::Classical), "Classical");
        assert_eq!(style_for(StyleBias::Fusion), "Fusion");
    }
}
