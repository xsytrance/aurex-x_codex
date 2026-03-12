use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstrumentKind {
    SineSynth,
    NoiseSynth,
    PulseSynth,
    Percussion,
}

impl InstrumentKind {
    pub fn as_audio_instrument(&self) -> &'static str {
        match self {
            InstrumentKind::SineSynth => "sine_synth",
            InstrumentKind::NoiseSynth => "noise_synth",
            InstrumentKind::PulseSynth => "pulse_synth",
            InstrumentKind::Percussion => "percussion",
        }
    }
}
