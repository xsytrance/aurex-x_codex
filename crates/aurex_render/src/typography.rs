#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphStyle {
    Neon,
    Pulse,
    Crystal,
    Circuit,
    Rune,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyStyle {
    pub glyph_style: GlyphStyle,
    pub glow_strength: f32,
    pub distortion: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricRenderEvent {
    pub text: String,
    pub position: [f32; 2],
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimedLyricRenderEvent {
    pub beat_time: f32,
    pub event: LyricRenderEvent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyReactiveState {
    pub scale_boost: f32,
    pub glow_boost: f32,
    pub ambient_boost: f32,
    pub letter_motion: f32,
    pub spark_intensity: f32,
}

impl Default for TypographyReactiveState {
    fn default() -> Self {
        Self {
            scale_boost: 0.0,
            glow_boost: 0.0,
            ambient_boost: 0.0,
            letter_motion: 0.0,
            spark_intensity: 0.0,
        }
    }
}

impl TypographyReactiveState {
    pub fn advance_frame(&mut self) {
        self.scale_boost *= 0.82;
        self.glow_boost *= 0.88;
        self.ambient_boost *= 0.92;
        self.letter_motion *= 0.84;
        self.spark_intensity *= 0.68;
    }
}

pub fn choose_typography_style(seed: u64) -> TypographyStyle {
    let style = match splitmix_u64(seed ^ 0x5A51_7EED_2024_0001) % 5 {
        0 => GlyphStyle::Neon,
        1 => GlyphStyle::Pulse,
        2 => GlyphStyle::Crystal,
        3 => GlyphStyle::Circuit,
        _ => GlyphStyle::Rune,
    };

    let glow = 0.7 + splitmix_f32(seed ^ 0x5A51_7EED_2024_1001) * 0.8;
    let distortion = 0.05 + splitmix_f32(seed ^ 0x5A51_7EED_2024_2001) * 0.35;

    TypographyStyle {
        glyph_style: style,
        glow_strength: glow,
        distortion,
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
    use super::choose_typography_style;

    #[test]
    fn typography_choice_is_deterministic() {
        let a = choose_typography_style(88);
        let b = choose_typography_style(88);
        assert_eq!(a, b);
    }
}
