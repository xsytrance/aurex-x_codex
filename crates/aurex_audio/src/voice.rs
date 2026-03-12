use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phoneme {
    AH,
    EH,
    OH,
    OO,
    EE,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct FormantFilter {
    pub f1: f32,
    pub f2: f32,
    pub f3: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoicePreset {
    Robot,
    Female,
    Male,
    Choir,
    Alien,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceSynth {
    pub preset: VoicePreset,
    pub sequence: Vec<Phoneme>,
    pub base_pitch_hz: f32,
    pub phoneme_duration: f32,
}

impl VoiceSynth {
    pub fn sample(&self, t: f32) -> f32 {
        if self.sequence.is_empty() {
            return 0.0;
        }
        let idx = ((t / self.phoneme_duration.max(1e-6)) as usize) % self.sequence.len();
        let phoneme = self.sequence[idx];
        let formant = formant_for(phoneme, self.preset);
        let source = (std::f32::consts::TAU * self.base_pitch_hz * t).sin();
        apply_formant(source, formant, t)
    }
}

pub fn phonemes_for_word(word: &str) -> Vec<Phoneme> {
    match word.to_uppercase().as_str() {
        "HELLO" => vec![Phoneme::EH, Phoneme::OH],
        "AUREX" => vec![Phoneme::AH, Phoneme::OO, Phoneme::EH],
        _ => vec![Phoneme::AH],
    }
}

fn formant_for(p: Phoneme, preset: VoicePreset) -> FormantFilter {
    let base = match p {
        Phoneme::AH => FormantFilter {
            f1: 800.0,
            f2: 1200.0,
            f3: 2500.0,
        },
        Phoneme::EH => FormantFilter {
            f1: 530.0,
            f2: 1840.0,
            f3: 2480.0,
        },
        Phoneme::OH => FormantFilter {
            f1: 570.0,
            f2: 840.0,
            f3: 2410.0,
        },
        Phoneme::OO => FormantFilter {
            f1: 300.0,
            f2: 870.0,
            f3: 2240.0,
        },
        Phoneme::EE => FormantFilter {
            f1: 270.0,
            f2: 2290.0,
            f3: 3010.0,
        },
    };

    let scale = match preset {
        VoicePreset::Robot => 0.9,
        VoicePreset::Female => 1.2,
        VoicePreset::Male => 1.0,
        VoicePreset::Choir => 1.05,
        VoicePreset::Alien => 1.35,
    };

    FormantFilter {
        f1: base.f1 * scale,
        f2: base.f2 * scale,
        f3: base.f3 * scale,
    }
}

fn apply_formant(source: f32, f: FormantFilter, t: f32) -> f32 {
    let o1 = (std::f32::consts::TAU * f.f1 * t).sin();
    let o2 = (std::f32::consts::TAU * f.f2 * t).sin();
    let o3 = (std::f32::consts::TAU * f.f3 * t).sin();
    source * 0.4 + o1 * 0.3 + o2 * 0.2 + o3 * 0.1
}

#[cfg(test)]
mod tests {
    use super::{Phoneme, VoicePreset, VoiceSynth};

    #[test]
    fn voice_synth_is_deterministic_for_same_inputs() {
        let voice = VoiceSynth {
            preset: VoicePreset::Alien,
            sequence: vec![Phoneme::AH, Phoneme::EE, Phoneme::OH],
            base_pitch_hz: 180.0,
            phoneme_duration: 0.2,
        };

        let a = voice.sample(0.27);
        let b = voice.sample(0.27);
        assert_eq!(a, b);
    }

    #[test]
    fn different_presets_produce_different_timbre_samples() {
        let female = VoiceSynth {
            preset: VoicePreset::Female,
            sequence: vec![Phoneme::EE],
            base_pitch_hz: 220.0,
            phoneme_duration: 0.25,
        };
        let alien = VoiceSynth {
            preset: VoicePreset::Alien,
            sequence: vec![Phoneme::EE],
            base_pitch_hz: 220.0,
            phoneme_duration: 0.25,
        };

        let t = 0.0137;
        assert_ne!(female.sample(t), alien.sample(t));
    }
}
