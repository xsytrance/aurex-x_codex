use crate::style_profile::{
    ScaleType, StyleProfile, choose_style, choose_style_selection, splitmix_u64,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongSection {
    Intro,
    Verse,
    Chorus,
    Bridge,
    Breakdown,
    Drop,
    Outro,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongStructure {
    pub sections: Vec<SongSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chord {
    I,
    Ii,
    Iii,
    Iv,
    V,
    Vi,
    ViiDim,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordProgression {
    pub chords: Vec<Chord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongPlan {
    pub title: String,
    pub bpm: u32,
    pub scale: ScaleType,
    pub structure: SongStructure,
    pub chords: ChordProgression,
    pub style: StyleProfile,
}

pub fn generate_chord_progression(seed: u64, scale: ScaleType) -> ChordProgression {
    let mut mode = splitmix_u64(seed ^ 0xC401_D5A0_7A1E_9001) % 3;
    if matches!(scale, ScaleType::HarmonicMinor) {
        mode = 2;
    }

    let chords = match mode {
        0 => vec![Chord::Vi, Chord::Iv, Chord::I, Chord::V],
        1 => vec![Chord::Ii, Chord::V, Chord::I],
        _ => vec![Chord::I, Chord::ViiDim, Chord::Vi],
    };

    ChordProgression { chords }
}

pub fn generate_song_plan(seed: u64) -> SongPlan {
    let style_selection = choose_style_selection(seed);
    let style = choose_style(seed);
    let structure = structure_for_style(style.name);
    let chords = generate_chord_progression(seed ^ 0x55AA_11CC_7788_3300, style_selection.scale);

    SongPlan {
        title: generate_title(seed),
        bpm: style_selection.bpm.round().clamp(40.0, 240.0) as u32,
        scale: style_selection.scale,
        structure,
        chords,
        style,
    }
}

fn structure_for_style(style_name: &str) -> SongStructure {
    let sections = match style_name {
        "Electronic" => vec![
            SongSection::Intro,
            SongSection::Bridge,
            SongSection::Drop,
            SongSection::Breakdown,
            SongSection::Drop,
            SongSection::Outro,
        ],
        "Pop" => vec![
            SongSection::Intro,
            SongSection::Verse,
            SongSection::Chorus,
            SongSection::Verse,
            SongSection::Chorus,
            SongSection::Bridge,
            SongSection::Chorus,
            SongSection::Outro,
        ],
        "Jazz" => vec![
            SongSection::Intro,
            SongSection::Verse,
            SongSection::Bridge,
            SongSection::Bridge,
            SongSection::Verse,
            SongSection::Outro,
        ],
        "World" => vec![
            SongSection::Intro,
            SongSection::Verse,
            SongSection::Verse,
            SongSection::Bridge,
            SongSection::Verse,
            SongSection::Outro,
        ],
        _ => vec![
            SongSection::Intro,
            SongSection::Verse,
            SongSection::Chorus,
            SongSection::Bridge,
            SongSection::Outro,
        ],
    };

    SongStructure { sections }
}

fn generate_title(seed: u64) -> String {
    const FIRST: [&str; 10] = [
        "Signal", "Neon", "Ghost", "Pulse", "Echo", "Solar", "Velvet", "Midnight", "Crystal",
        "Circuit",
    ];
    const SECOND: [&str; 10] = [
        "Bloom",
        "Pilgrimage",
        "Frequency",
        "Cathedral",
        "Transit",
        "Ritual",
        "Drift",
        "Cascade",
        "Mirage",
        "Harbor",
    ];

    let i = (splitmix_u64(seed ^ 0xA1A2_A3A4_A5A6_A7A8) as usize) % FIRST.len();
    let j = (splitmix_u64(seed ^ 0xB1B2_B3B4_B5B6_B7B8) as usize) % SECOND.len();
    format!("{} {}", FIRST[i], SECOND[j])
}

#[cfg(test)]
mod tests {
    use super::{Chord, SongSection, generate_chord_progression, generate_song_plan};
    use crate::style_profile::ScaleType;

    #[test]
    fn chord_generation_is_deterministic() {
        let a = generate_chord_progression(7, ScaleType::Minor);
        let b = generate_chord_progression(7, ScaleType::Minor);
        assert_eq!(a, b);
    }

    #[test]
    fn known_templates_are_valid() {
        let p = generate_chord_progression(1, ScaleType::Major);
        assert!(!p.chords.is_empty());
        assert!(p.chords.iter().all(|c| matches!(
            c,
            Chord::I | Chord::Ii | Chord::Iii | Chord::Iv | Chord::V | Chord::Vi | Chord::ViiDim
        )));
    }

    #[test]
    fn song_plan_is_deterministic_and_structured() {
        let a = generate_song_plan(42);
        let b = generate_song_plan(42);
        assert_eq!(a, b);
        assert!(!a.title.is_empty());
        assert!(!a.structure.sections.is_empty());
        assert_eq!(a.structure.sections[0], SongSection::Intro);
        assert_eq!(
            a.structure.sections[a.structure.sections.len() - 1],
            SongSection::Outro
        );
    }
}
