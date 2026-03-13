#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscillatorType {
    Sine,
    Triangle,
    Saw,
    Square,
    Noise,
    Supersaw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub value: f32,
    stage: EnvelopeStage,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.12,
            value: 0.0,
            stage: EnvelopeStage::Idle,
        }
    }
}

impl Envelope {
    pub fn from_adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let mut env = Self::default();
        env.attack = attack;
        env.decay = decay;
        env.sustain = sustain;
        env.release = release;
        env.value = 0.0;
        env
    }

    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => {
                self.value = 0.0;
            }
            EnvelopeStage::Attack => {
                self.value += dt / self.attack.max(1e-6);
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                let decay_step = dt / self.decay.max(1e-6);
                self.value += (self.sustain - 1.0) * decay_step;
                if self.value <= self.sustain {
                    self.value = self.sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.value = self.sustain;
            }
            EnvelopeStage::Release => {
                self.value -= dt / self.release.max(1e-6);
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
        self.value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FilterState {
    low: f32,
    high: f32,
    band: f32,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            low: 0.0,
            high: 0.0,
            band: 0.0,
        }
    }
}

impl FilterState {
    pub fn process(
        &mut self,
        filter: FilterType,
        input: f32,
        cutoff_hz: f32,
        resonance: f32,
        sample_rate: f32,
    ) -> f32 {
        let f = (2.0 * std::f32::consts::PI * cutoff_hz / sample_rate)
            .sin()
            .clamp(0.001, 0.99);
        self.low += f * self.band;
        self.high = input - self.low - resonance.clamp(0.0, 2.0) * self.band;
        self.band += f * self.high;

        match filter {
            FilterType::LowPass => self.low,
            FilterType::HighPass => self.high,
            FilterType::BandPass => self.band,
        }
    }
}

const DELAY_BUF_LEN: usize = 2048;
const CHORUS_BUF_LEN: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub struct DelayFx {
    buf: [f32; DELAY_BUF_LEN],
    idx: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl Default for DelayFx {
    fn default() -> Self {
        Self {
            buf: [0.0; DELAY_BUF_LEN],
            idx: 0,
            delay_samples: 600,
            feedback: 0.32,
            mix: 0.25,
        }
    }
}

impl DelayFx {
    pub fn configure(&mut self, delay_samples: usize, feedback: f32, mix: f32) {
        self.delay_samples = delay_samples.min(DELAY_BUF_LEN - 1).max(1);
        self.feedback = feedback.clamp(0.0, 0.98);
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let tap = (self.idx + DELAY_BUF_LEN - self.delay_samples) % DELAY_BUF_LEN;
        let delayed = self.buf[tap];
        self.buf[self.idx] = input + delayed * self.feedback;
        self.idx = (self.idx + 1) % DELAY_BUF_LEN;
        input * (1.0 - self.mix) + delayed * self.mix
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChorusFx {
    buf: [f32; CHORUS_BUF_LEN],
    idx: usize,
    lfo_phase: f32,
    depth_samples: f32,
    rate_hz: f32,
    mix: f32,
}

impl Default for ChorusFx {
    fn default() -> Self {
        Self {
            buf: [0.0; CHORUS_BUF_LEN],
            idx: 0,
            lfo_phase: 0.0,
            depth_samples: 12.0,
            rate_hz: 0.26,
            mix: 0.22,
        }
    }
}

impl ChorusFx {
    pub fn configure(&mut self, depth_samples: f32, rate_hz: f32, mix: f32) {
        self.depth_samples = depth_samples.clamp(1.0, 60.0);
        self.rate_hz = rate_hz.clamp(0.01, 4.0);
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        self.buf[self.idx] = input;
        let base_delay = 18.0;
        let lfo = (self.lfo_phase * std::f32::consts::TAU).sin();
        let offset = base_delay + self.depth_samples * (0.5 + 0.5 * lfo);
        let delay = offset as usize;
        let tap = (self.idx + CHORUS_BUF_LEN - delay.min(CHORUS_BUF_LEN - 1)) % CHORUS_BUF_LEN;
        let delayed = self.buf[tap];

        self.idx = (self.idx + 1) % CHORUS_BUF_LEN;
        self.lfo_phase = (self.lfo_phase + self.rate_hz / sample_rate).fract();

        input * (1.0 - self.mix) + delayed * self.mix
    }
}

pub fn saturate_soft(x: f32, drive: f32) -> f32 {
    (x * (1.0 + drive.max(0.0))).tanh()
}

#[derive(Debug, Clone, Copy)]
pub struct Instrument {
    pub oscillator: OscillatorType,
    pub filter: Option<FilterType>,
    pub envelope: Envelope,
    pub effect_flags: u32,
    pub gain: f32,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub drive: f32,
}

pub const FX_DELAY: u32 = 1 << 0;
pub const FX_CHORUS: u32 = 1 << 1;
pub const FX_SATURATION: u32 = 1 << 2;

impl Instrument {
    pub fn trance_bass() -> Self {
        Self {
            oscillator: OscillatorType::Triangle,
            filter: Some(FilterType::LowPass),
            envelope: Envelope {
                attack: 0.002,
                decay: 0.11,
                sustain: 0.0,
                release: 0.08,
                value: 0.0,
                stage: EnvelopeStage::Idle,
            },
            effect_flags: FX_SATURATION,
            gain: 0.95,
            cutoff_hz: 150.0,
            resonance: 0.62,
            drive: 0.35,
        }
    }

    pub fn supersaw_pad() -> Self {
        Self {
            oscillator: OscillatorType::Supersaw,
            filter: Some(FilterType::LowPass),
            envelope: Envelope {
                attack: 0.12,
                decay: 0.3,
                sustain: 0.68,
                release: 0.35,
                value: 0.0,
                stage: EnvelopeStage::Idle,
            },
            effect_flags: FX_CHORUS | FX_DELAY,
            gain: 0.7,
            cutoff_hz: 620.0,
            resonance: 0.2,
            drive: 0.0,
        }
    }

    pub fn analog_lead() -> Self {
        Self {
            oscillator: OscillatorType::Saw,
            filter: Some(FilterType::BandPass),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.16,
                sustain: 0.42,
                release: 0.18,
                value: 0.0,
                stage: EnvelopeStage::Idle,
            },
            effect_flags: FX_SATURATION,
            gain: 0.65,
            cutoff_hz: 1100.0,
            resonance: 0.55,
            drive: 0.18,
        }
    }

    pub fn noise_hat() -> Self {
        Self {
            oscillator: OscillatorType::Noise,
            filter: Some(FilterType::HighPass),
            envelope: Envelope {
                attack: 0.001,
                decay: 0.024,
                sustain: 0.0,
                release: 0.018,
                value: 0.0,
                stage: EnvelopeStage::Idle,
            },
            effect_flags: 0,
            gain: 0.56,
            cutoff_hz: 6200.0,
            resonance: 0.08,
            drive: 0.0,
        }
    }

    pub fn kick_drum() -> Self {
        Self {
            oscillator: OscillatorType::Sine,
            filter: Some(FilterType::LowPass),
            envelope: Envelope {
                attack: 0.001,
                decay: 0.12,
                sustain: 0.0,
                release: 0.08,
                value: 0.0,
                stage: EnvelopeStage::Idle,
            },
            effect_flags: FX_SATURATION,
            gain: 1.0,
            cutoff_hz: 220.0,
            resonance: 0.12,
            drive: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InstrumentVoice {
    pub instrument: Instrument,
    phase: f32,
    supersaw_phases: [f32; 7],
    noise_state: u32,
    pub env: Envelope,
    filter_state: FilterState,
    delay: DelayFx,
    chorus: ChorusFx,
}

impl InstrumentVoice {
    pub fn new(instrument: Instrument, noise_seed: u32) -> Self {
        let mut delay = DelayFx::default();
        delay.configure(680, 0.28, 0.22);
        let mut chorus = ChorusFx::default();
        chorus.configure(14.0, 0.21, 0.2);
        Self {
            env: instrument.envelope,
            instrument,
            phase: 0.0,
            supersaw_phases: [0.0; 7],
            noise_state: noise_seed.max(1),
            filter_state: FilterState::default(),
            delay,
            chorus,
        }
    }

    pub fn note_on(&mut self) {
        self.env.note_on();
    }

    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    pub fn sample(&mut self, freq_hz: f32, sample_rate: f32) -> f32 {
        let dt = 1.0 / sample_rate.max(1.0);
        let env = self.env.update(dt);
        let mut sample = sample_osc(
            self.instrument.oscillator,
            freq_hz,
            sample_rate,
            &mut self.phase,
            &mut self.noise_state,
            &mut self.supersaw_phases,
        );

        if let Some(filter) = self.instrument.filter {
            sample = self.filter_state.process(
                filter,
                sample,
                self.instrument.cutoff_hz,
                self.instrument.resonance,
                sample_rate,
            );
        }

        sample *= env * self.instrument.gain;

        if (self.instrument.effect_flags & FX_CHORUS) != 0 {
            sample = self.chorus.process(sample, sample_rate);
        }
        if (self.instrument.effect_flags & FX_DELAY) != 0 {
            sample = self.delay.process(sample);
        }
        if (self.instrument.effect_flags & FX_SATURATION) != 0 {
            sample = saturate_soft(sample, self.instrument.drive);
        }

        sample
    }
}

pub fn sample_osc(
    osc: OscillatorType,
    freq_hz: f32,
    sample_rate: f32,
    phase: &mut f32,
    noise_state: &mut u32,
    supersaw_phases: &mut [f32; 7],
) -> f32 {
    let inc = freq_hz / sample_rate.max(1.0);
    *phase = (*phase + inc).fract();

    match osc {
        OscillatorType::Sine => (*phase * std::f32::consts::TAU).sin(),
        OscillatorType::Triangle => 4.0 * (*phase - 0.5).abs() - 1.0,
        OscillatorType::Saw => *phase * 2.0 - 1.0,
        OscillatorType::Square => {
            if *phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        OscillatorType::Noise => {
            *noise_state = noise_state
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223);
            ((*noise_state >> 9) as f32 / (u32::MAX >> 9) as f32) * 2.0 - 1.0
        }
        OscillatorType::Supersaw => {
            let detunes = [-0.021_f32, -0.013, -0.008, 0.0, 0.008, 0.013, 0.021];
            let mut sum = 0.0;
            for (idx, detune) in detunes.iter().enumerate() {
                supersaw_phases[idx] = (supersaw_phases[idx] + inc * (1.0 + detune)).fract();
                sum += supersaw_phases[idx] * 2.0 - 1.0;
            }
            sum / 7.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Envelope, FilterState, FilterType, Instrument, InstrumentVoice, OscillatorType, sample_osc,
    };

    #[test]
    fn supersaw_is_deterministic() {
        let mut phase_a = 0.0;
        let mut phase_b = 0.0;
        let mut noise_a = 7;
        let mut noise_b = 7;
        let mut ss_a = [0.0; 7];
        let mut ss_b = [0.0; 7];
        let a = sample_osc(
            OscillatorType::Supersaw,
            220.0,
            48_000.0,
            &mut phase_a,
            &mut noise_a,
            &mut ss_a,
        );
        let b = sample_osc(
            OscillatorType::Supersaw,
            220.0,
            48_000.0,
            &mut phase_b,
            &mut noise_b,
            &mut ss_b,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn filter_processes_sample() {
        let mut fs = FilterState::default();
        let y = fs.process(FilterType::LowPass, 1.0, 200.0, 0.2, 48_000.0);
        assert!(y.is_finite());
    }

    #[test]
    fn instrument_voice_emits_signal_after_note_on() {
        let mut voice = InstrumentVoice::new(Instrument::trance_bass(), 11);
        voice.note_on();
        let mut energy = 0.0;
        for _ in 0..64 {
            energy += voice.sample(55.0, 48_000.0).abs();
        }
        assert!(energy > 0.0);
    }

    #[test]
    fn envelope_attack_reaches_up() {
        let mut env = Envelope::default();
        env.note_on();
        for _ in 0..100 {
            env.update(1.0 / 48_000.0);
        }
        assert!(env.value > 0.0);
    }
}
