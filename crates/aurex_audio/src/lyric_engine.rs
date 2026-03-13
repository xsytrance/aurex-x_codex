use crate::style_profile::{StyleProfile, splitmix_u64};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    pub words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricSyllable {
    pub text: String,
    pub beat_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricTimeline {
    pub syllables: Vec<LyricSyllable>,
}

pub fn generate_lyrics(seed: u64, style: StyleProfile) -> Lyrics {
    let bank = word_bank(style.name);
    let template_count = 4usize;
    let mut lines = Vec::with_capacity(template_count);

    for idx in 0..template_count {
        let s = splitmix_u64(seed ^ ((idx as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)));
        let t = (s as usize) % 3;
        let n1 = bank[(splitmix_u64(s ^ 0xA11C_E501) as usize) % bank.len()];
        let n2 = bank[(splitmix_u64(s ^ 0xB22D_F702) as usize) % bank.len()];
        let line = match t {
            0 => format!("Feel the {} tonight", n1),
            1 => format!("We rise into the {}", n1),
            _ => format!("The {} calls my {}", n1, n2),
        };

        lines.push(LyricLine {
            words: line.split_whitespace().map(|w| w.to_string()).collect(),
        });
    }

    Lyrics { lines }
}

pub fn build_lyric_timeline(lyrics: &Lyrics, bpm: u32) -> LyricTimeline {
    let mut syllables = Vec::new();
    let mut beat_cursor = 0.0f32;
    let beat_step = if bpm >= 120 { 0.5 } else { 1.0 };

    for line in &lyrics.lines {
        for word in &line.words {
            let parts = split_word_into_syllables(word);
            for part in parts {
                syllables.push(LyricSyllable {
                    text: part,
                    beat_time: beat_cursor,
                });
                beat_cursor += beat_step;
            }
        }
        beat_cursor += beat_step;
    }

    LyricTimeline { syllables }
}

fn word_bank(style_name: &str) -> &'static [&'static str] {
    match style_name {
        "Electronic" => &[
            "pulse",
            "signal",
            "neon",
            "electric",
            "light",
            "frequency",
            "rise",
        ],
        "RnB" => &["heart", "touch", "feel", "night", "love", "breath"],
        "Jazz" => &["night", "blue", "rhythm", "groove", "sway"],
        "World" => &["spirit", "earth", "fire", "sky", "wind"],
        _ => &["echo", "glow", "dream", "wave", "spark", "horizon"],
    }
}

fn split_word_into_syllables(word: &str) -> Vec<String> {
    let mut out = Vec::new();
    let chars: Vec<char> = word.chars().collect();
    if chars.len() <= 3 {
        out.push(word.to_string());
        return out;
    }

    let mut start = 0usize;
    for i in 1..chars.len() {
        let prev_v = is_vowel(chars[i - 1]);
        let curr_v = is_vowel(chars[i]);
        if prev_v && !curr_v {
            let part: String = chars[start..=i].iter().collect();
            out.push(part);
            start = i + 1;
        }
    }

    if start < chars.len() {
        out.push(chars[start..].iter().collect());
    }

    if out.is_empty() {
        out.push(word.to_string());
    }

    out
}

fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

#[cfg(test)]
mod tests {
    use super::{build_lyric_timeline, generate_lyrics};
    use crate::style_profile::choose_style;

    #[test]
    fn lyric_generation_is_deterministic() {
        let style = choose_style(1);
        let a = generate_lyrics(77, style);
        let b = generate_lyrics(77, style);
        assert_eq!(a, b);
    }

    #[test]
    fn lyric_timeline_is_monotonic() {
        let style = choose_style(2);
        let lyrics = generate_lyrics(22, style);
        let tl = build_lyric_timeline(&lyrics, 128);
        assert!(!tl.syllables.is_empty());
        for pair in tl.syllables.windows(2) {
            assert!(pair[1].beat_time >= pair[0].beat_time);
        }
    }
}
