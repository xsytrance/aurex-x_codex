use aurex_audio::song_planner::{SongPlan, generate_song_plan};
use aurex_render::typography::{TypographyStyle, choose_typography_style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualTheme {
    Reactor,
    Cathedral,
    DesertMonolith,
    StormField,
    NeonCity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExperiencePlan {
    pub title: String,
    pub duration_seconds: f32,
    pub song_plan: SongPlan,
    pub typography_style: TypographyStyle,
    pub visual_theme: VisualTheme,
}

pub fn generate_experience_title(seed: u64) -> String {
    const FIRST: [&str; 10] = [
        "Signal", "Neon", "Ghost", "Voltage", "Pulse", "Crystal", "Solar", "Echo", "Circuit",
        "Storm",
    ];
    const SECOND: [&str; 10] = [
        "Bloom",
        "Pilgrimage",
        "Frequency",
        "Cathedral",
        "Ascension",
        "Drift",
        "Harbor",
        "Mirage",
        "Transit",
        "Parallax",
    ];

    let a = (splitmix_u64(seed ^ 0xDEAD_BEEF_1000_0001) as usize) % FIRST.len();
    let b = (splitmix_u64(seed ^ 0xDEAD_BEEF_1000_0002) as usize) % SECOND.len();
    format!("{} {}", FIRST[a], SECOND[b])
}

pub fn generate_experience(seed: u64) -> ExperiencePlan {
    let duration_t = splitmix_f32(seed ^ 0xDEAD_BEEF_2000_0001);
    let duration_seconds = 30.0 + duration_t * 60.0;

    let visual_theme = match splitmix_u64(seed ^ 0xDEAD_BEEF_3000_0001) % 5 {
        0 => VisualTheme::Reactor,
        1 => VisualTheme::Cathedral,
        2 => VisualTheme::DesertMonolith,
        3 => VisualTheme::StormField,
        _ => VisualTheme::NeonCity,
    };

    ExperiencePlan {
        title: generate_experience_title(seed),
        duration_seconds,
        song_plan: generate_song_plan(seed ^ 0xDEAD_BEEF_4000_0001),
        typography_style: choose_typography_style(seed ^ 0xDEAD_BEEF_5000_0001),
        visual_theme,
    }
}

fn splitmix_u64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn splitmix_f32(seed: u64) -> f32 {
    (splitmix_u64(seed) as f64 / u64::MAX as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::{generate_experience, generate_experience_title};

    #[test]
    fn title_generation_is_deterministic() {
        let a = generate_experience_title(77);
        let b = generate_experience_title(77);
        assert_eq!(a, b);
    }

    #[test]
    fn experience_generation_is_deterministic_and_bounded() {
        let a = generate_experience(15);
        let b = generate_experience(15);
        assert_eq!(a, b);
        assert!((30.0..=90.0).contains(&a.duration_seconds));
        assert!(!a.title.is_empty());
    }
}
