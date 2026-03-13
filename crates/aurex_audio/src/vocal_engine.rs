use crate::{
    VoiceSynthConfig,
    runtime_toolkit::{
        ChorusFx, DelayFx, Envelope, FX_CHORUS, FX_DELAY, FX_SATURATION, OscillatorType,
        sample_osc, saturate_soft,
    },
    voice::{Phoneme, VoicePreset},
};

pub const CHANT_PHONEMES: [&str; 5] = ["AH", "OH", "YA", "NA", "HE"];
pub const SCAT_PHONEMES: [&str; 5] = ["BA", "DA", "DOO", "BEE", "SKA"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocalType {
    Chant,
    ChoirPad,
    RnbSynth,
    Robot,
    Scat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocalPhoneme {
    AH,
    OH,
    YA,
    NA,
    HE,
    BA,
    DA,
    DOO,
    BEE,
    SKA,
    EE,
    OO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phrase {
    pub phonemes: [VocalPhoneme; 16],
    pub len: usize,
}

impl Phrase {
    pub fn iter(&self) -> impl Iterator<Item = VocalPhoneme> + '_ {
        self.phonemes.iter().take(self.len).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Formant {
    pub frequency: f32,
    pub bandwidth: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vowel {
    AH,
    OO,
    EE,
    OH,
}

#[derive(Debug, Clone, Copy)]
struct Resonator {
    low: f32,
    band: f32,
}

impl Default for Resonator {
    fn default() -> Self {
        Self {
            low: 0.0,
            band: 0.0,
        }
    }
}

impl Resonator {
    fn process(&mut self, input: f32, formant: Formant, sample_rate: f32) -> f32 {
        let f = (2.0 * std::f32::consts::PI * formant.frequency / sample_rate)
            .sin()
            .clamp(0.001, 0.98);
        let q = (formant.bandwidth / formant.frequency.max(1.0)).clamp(0.01, 1.0);
        let high = input - self.low - self.band * q;
        self.band += f * high;
        self.low += f * self.band;
        self.band
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FormantFilterState {
    r1: Resonator,
    r2: Resonator,
    r3: Resonator,
}

impl Default for FormantFilterState {
    fn default() -> Self {
        Self {
            r1: Resonator::default(),
            r2: Resonator::default(),
            r3: Resonator::default(),
        }
    }
}

impl FormantFilterState {
    pub fn process(&mut self, input: f32, formants: [Formant; 3], sample_rate: f32) -> f32 {
        let f1 = self.r1.process(input, formants[0], sample_rate);
        let f2 = self.r2.process(input, formants[1], sample_rate);
        let f3 = self.r3.process(input, formants[2], sample_rate);
        (f1 * 0.55 + f2 * 0.3 + f3 * 0.15).clamp(-1.0, 1.0)
    }
}

pub fn vowel_formants(vowel: Vowel) -> [Formant; 3] {
    match vowel {
        Vowel::AH => [
            Formant {
                frequency: 800.0,
                bandwidth: 80.0,
            },
            Formant {
                frequency: 1200.0,
                bandwidth: 90.0,
            },
            Formant {
                frequency: 2500.0,
                bandwidth: 120.0,
            },
        ],
        Vowel::OO => [
            Formant {
                frequency: 350.0,
                bandwidth: 70.0,
            },
            Formant {
                frequency: 800.0,
                bandwidth: 80.0,
            },
            Formant {
                frequency: 2200.0,
                bandwidth: 120.0,
            },
        ],
        Vowel::EE => [
            Formant {
                frequency: 300.0,
                bandwidth: 60.0,
            },
            Formant {
                frequency: 2200.0,
                bandwidth: 100.0,
            },
            Formant {
                frequency: 3000.0,
                bandwidth: 130.0,
            },
        ],
        Vowel::OH => [
            Formant {
                frequency: 570.0,
                bandwidth: 80.0,
            },
            Formant {
                frequency: 900.0,
                bandwidth: 90.0,
            },
            Formant {
                frequency: 2400.0,
                bandwidth: 120.0,
            },
        ],
    }
}

pub fn generate_phrase(seed: u64, phonemes: &[&str]) -> Phrase {
    let mut out = [VocalPhoneme::AH; 16];
    let len = 8usize.min(out.len()).max(1);
    for (idx, slot) in out.iter_mut().take(len).enumerate() {
        let step = (idx as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let step_seed = splitmix_u64(seed ^ step);
        let p = phonemes[(step_seed as usize) % phonemes.len().max(1)];
        *slot = parse_phoneme(p);
    }
    Phrase { phonemes: out, len }
}

#[derive(Debug, Clone, Copy)]
pub struct VocalVoice {
    pub vocal_type: VocalType,
    phase: f32,
    supersaw_phases: [f32; 7],
    noise_state: u32,
    formant: FormantFilterState,
    env: Envelope,
    delay: DelayFx,
    chorus: ChorusFx,
    effect_flags: u32,
    drive: f32,
}

impl VocalVoice {
    pub fn new(vocal_type: VocalType, seed: u32) -> Self {
        let mut delay = DelayFx::default();
        delay.configure(520, 0.24, 0.18);
        let mut chorus = ChorusFx::default();
        chorus.configure(11.0, 0.19, 0.2);

        let (env, effect_flags, drive) = match vocal_type {
            VocalType::Chant => (Envelope::from_adsr(0.02, 0.2, 0.6, 0.2), 0, 0.0),
            VocalType::ChoirPad => (
                Envelope::from_adsr(0.18, 0.35, 0.72, 0.42),
                FX_CHORUS | FX_DELAY,
                0.0,
            ),
            VocalType::RnbSynth => (Envelope::from_adsr(0.08, 0.24, 0.65, 0.24), FX_CHORUS, 0.0),
            VocalType::Robot => (
                Envelope::from_adsr(0.004, 0.08, 0.32, 0.07),
                FX_SATURATION,
                0.35,
            ),
            VocalType::Scat => (Envelope::from_adsr(0.003, 0.06, 0.2, 0.05), 0, 0.0),
        };

        Self {
            vocal_type,
            phase: 0.0,
            supersaw_phases: [0.0; 7],
            noise_state: seed.max(1),
            formant: FormantFilterState::default(),
            env,
            delay,
            chorus,
            effect_flags,
            drive,
        }
    }

    pub fn note_on(&mut self) {
        self.env.note_on();
    }

    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    pub fn sample(&mut self, freq_hz: f32, phoneme: VocalPhoneme, sample_rate: f32) -> f32 {
        let osc = match self.vocal_type {
            VocalType::Chant => OscillatorType::Triangle,
            VocalType::ChoirPad => OscillatorType::Supersaw,
            VocalType::RnbSynth => OscillatorType::Saw,
            VocalType::Robot => OscillatorType::Square,
            VocalType::Scat => OscillatorType::Triangle,
        };

        let dt = 1.0 / sample_rate.max(1.0);
        let env = self.env.update(dt);
        let raw = sample_osc(
            osc,
            freq_hz,
            sample_rate,
            &mut self.phase,
            &mut self.noise_state,
            &mut self.supersaw_phases,
        );

        let vowel = phoneme_to_vowel(phoneme, self.vocal_type);
        let mut y = self
            .formant
            .process(raw, vowel_formants(vowel), sample_rate);
        y *= env;

        if (self.effect_flags & FX_CHORUS) != 0 {
            y = self.chorus.process(y, sample_rate);
        }
        if (self.effect_flags & FX_DELAY) != 0 {
            y = self.delay.process(y);
        }
        if (self.effect_flags & FX_SATURATION) != 0 {
            y = saturate_soft(y, self.drive);
        }

        y
    }
}

pub fn default_phrase_for(vocal_type: VocalType, seed: u64) -> Phrase {
    match vocal_type {
        VocalType::Scat => generate_phrase(seed, &SCAT_PHONEMES),
        VocalType::Chant => generate_phrase(seed, &CHANT_PHONEMES),
        VocalType::ChoirPad => generate_phrase(seed ^ 0xA7, &["AH", "OH", "AH", "OO"]),
        VocalType::RnbSynth => generate_phrase(seed ^ 0x5C, &["EE", "OH", "AH", "OO"]),
        VocalType::Robot => generate_phrase(seed ^ 0x13, &["EE", "AH", "OH", "OO"]),
    }
}

pub fn vocal_type_to_config(vocal_type: VocalType, seed: u64) -> VoiceSynthConfig {
    let phrase = default_phrase_for(vocal_type, seed);
    let sequence = phrase.iter().map(to_voice_phoneme).collect::<Vec<_>>();

    let (preset, base_pitch_hz, phoneme_duration) = match vocal_type {
        VocalType::Chant => (VoicePreset::Choir, 148.0, 0.28),
        VocalType::ChoirPad => (VoicePreset::Choir, 170.0, 0.34),
        VocalType::RnbSynth => (VoicePreset::Female, 196.0, 0.22),
        VocalType::Robot => (VoicePreset::Robot, 188.0, 0.17),
        VocalType::Scat => (VoicePreset::Alien, 210.0, 0.12),
    };

    VoiceSynthConfig {
        preset,
        phonemes: sequence,
        base_pitch_hz,
        phoneme_duration,
    }
}

fn parse_phoneme(src: &str) -> VocalPhoneme {
    match src {
        "AH" => VocalPhoneme::AH,
        "OH" => VocalPhoneme::OH,
        "YA" => VocalPhoneme::YA,
        "NA" => VocalPhoneme::NA,
        "HE" => VocalPhoneme::HE,
        "BA" => VocalPhoneme::BA,
        "DA" => VocalPhoneme::DA,
        "DOO" => VocalPhoneme::DOO,
        "BEE" => VocalPhoneme::BEE,
        "SKA" => VocalPhoneme::SKA,
        "EE" => VocalPhoneme::EE,
        "OO" => VocalPhoneme::OO,
        _ => VocalPhoneme::AH,
    }
}

fn to_voice_phoneme(p: VocalPhoneme) -> Phoneme {
    match p {
        VocalPhoneme::AH | VocalPhoneme::NA | VocalPhoneme::BA | VocalPhoneme::DA => Phoneme::AH,
        VocalPhoneme::OH | VocalPhoneme::HE | VocalPhoneme::SKA => Phoneme::OH,
        VocalPhoneme::YA | VocalPhoneme::BEE | VocalPhoneme::EE => Phoneme::EE,
        VocalPhoneme::DOO | VocalPhoneme::OO => Phoneme::OO,
    }
}

fn phoneme_to_vowel(p: VocalPhoneme, vocal_type: VocalType) -> Vowel {
    match vocal_type {
        VocalType::Chant => match p {
            VocalPhoneme::AH | VocalPhoneme::NA => Vowel::AH,
            _ => Vowel::OH,
        },
        VocalType::ChoirPad => Vowel::AH,
        VocalType::RnbSynth => match p {
            VocalPhoneme::EE | VocalPhoneme::BEE | VocalPhoneme::YA => Vowel::EE,
            _ => Vowel::OH,
        },
        VocalType::Robot => match p {
            VocalPhoneme::EE | VocalPhoneme::BEE => Vowel::EE,
            _ => Vowel::OH,
        },
        VocalType::Scat => match p {
            VocalPhoneme::DOO | VocalPhoneme::OO => Vowel::OO,
            VocalPhoneme::BEE | VocalPhoneme::EE => Vowel::EE,
            _ => Vowel::AH,
        },
    }
}

fn splitmix_u64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::{
        CHANT_PHONEMES, VocalType, VocalVoice, default_phrase_for, generate_phrase,
        vocal_type_to_config,
    };

    #[test]
    fn phrase_generation_is_deterministic() {
        let a = generate_phrase(88, &CHANT_PHONEMES);
        let b = generate_phrase(88, &CHANT_PHONEMES);
        assert_eq!(a, b);
    }

    #[test]
    fn vocal_type_maps_to_non_empty_config() {
        let cfg = vocal_type_to_config(VocalType::Scat, 19);
        assert!(!cfg.phonemes.is_empty());
        assert!(cfg.base_pitch_hz > 0.0);
    }

    #[test]
    fn default_phrase_varies_by_type() {
        let chant = default_phrase_for(VocalType::Chant, 12);
        let scat = default_phrase_for(VocalType::Scat, 12);
        assert_ne!(chant, scat);
    }

    #[test]
    fn vocal_voice_sampling_is_deterministic() {
        let phrase = default_phrase_for(VocalType::Robot, 1);
        let phoneme = phrase.phonemes[0];
        let mut a = VocalVoice::new(VocalType::Robot, 7);
        let mut b = VocalVoice::new(VocalType::Robot, 7);
        a.note_on();
        b.note_on();
        let sa = a.sample(190.0, phoneme, 48_000.0);
        let sb = b.sample(190.0, phoneme, 48_000.0);
        assert_eq!(sa, sb);
    }
}
