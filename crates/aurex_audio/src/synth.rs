use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OscillatorType {
    Sine,
    Square,
    Saw,
    Triangle,
    Noise,
    Fm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SynthNode {
    Oscillator {
        osc_type: OscillatorType,
        frequency: f32,
        amplitude: f32,
        phase: f32,
    },
    Noise {
        amplitude: f32,
        seed: u32,
    },
    FMOperator {
        carrier_freq: f32,
        mod_freq: f32,
        mod_index: f32,
        amplitude: f32,
    },
    Filter {
        cutoff: f32,
        resonance: f32,
        input: Box<SynthNode>,
    },
    Envelope {
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        gate: f32,
        input: Box<SynthNode>,
    },
    Mixer {
        inputs: Vec<SynthNode>,
        gain: f32,
    },
    Delay {
        delay_seconds: f32,
        feedback: f32,
        input: Box<SynthNode>,
    },
    Reverb {
        room_size: f32,
        damping: f32,
        input: Box<SynthNode>,
    },
    Distortion {
        drive: f32,
        mix: f32,
        input: Box<SynthNode>,
    },
    Chorus {
        depth: f32,
        rate: f32,
        mix: f32,
        input: Box<SynthNode>,
    },
}

pub fn sample_synth(node: &SynthNode, t: f32, sample_rate: f32, seed: u32) -> f32 {
    match node {
        SynthNode::Oscillator {
            osc_type,
            frequency,
            amplitude,
            phase,
        } => amplitude * sample_osc(*osc_type, t, *frequency, *phase, sample_rate, seed),
        SynthNode::Noise { amplitude, seed: s } => {
            amplitude * hash_noise((t * sample_rate) as i32, seed ^ *s)
        }
        SynthNode::FMOperator {
            carrier_freq,
            mod_freq,
            mod_index,
            amplitude,
        } => {
            let m = (std::f32::consts::TAU * mod_freq * t).sin() * mod_index;
            amplitude * (std::f32::consts::TAU * carrier_freq * t + m).sin()
        }
        SynthNode::Filter {
            cutoff,
            resonance,
            input,
        } => {
            let x = sample_synth(input, t, sample_rate, seed);
            let c = (cutoff / sample_rate).clamp(0.0, 0.5);
            (x * c * (1.0 + resonance.clamp(0.0, 2.0))).tanh()
        }
        SynthNode::Envelope {
            attack,
            decay,
            sustain,
            release,
            gate,
            input,
        } => {
            let x = sample_synth(input, t, sample_rate, seed);
            x * adsr(t, *attack, *decay, *sustain, *release, *gate)
        }
        SynthNode::Mixer { inputs, gain } => {
            let sum: f32 = inputs
                .iter()
                .map(|i| sample_synth(i, t, sample_rate, seed))
                .sum();
            sum * *gain / inputs.len().max(1) as f32
        }
        SynthNode::Delay {
            delay_seconds,
            feedback,
            input,
        } => {
            let dry = sample_synth(input, t, sample_rate, seed);
            let wet = sample_synth(input, (t - delay_seconds).max(0.0), sample_rate, seed);
            dry + wet * feedback
        }
        SynthNode::Reverb {
            room_size,
            damping,
            input,
        } => {
            let dry = sample_synth(input, t, sample_rate, seed);
            let tap1 = sample_synth(input, (t - 0.013 * room_size).max(0.0), sample_rate, seed);
            let tap2 = sample_synth(input, (t - 0.021 * room_size).max(0.0), sample_rate, seed);
            dry * (1.0 - damping) + (tap1 + tap2) * 0.5 * damping
        }
        SynthNode::Distortion { drive, mix, input } => {
            let x = sample_synth(input, t, sample_rate, seed);
            let d = (x * (1.0 + drive.max(0.0))).tanh();
            x * (1.0 - mix.clamp(0.0, 1.0)) + d * mix.clamp(0.0, 1.0)
        }
        SynthNode::Chorus {
            depth,
            rate,
            mix,
            input,
        } => {
            let x = sample_synth(input, t, sample_rate, seed);
            let mod_dt = depth * 0.005 * (std::f32::consts::TAU * rate * t).sin();
            let y = sample_synth(input, (t + mod_dt).max(0.0), sample_rate, seed);
            x * (1.0 - mix.clamp(0.0, 1.0)) + y * mix.clamp(0.0, 1.0)
        }
    }
}

fn sample_osc(
    osc: OscillatorType,
    t: f32,
    freq: f32,
    phase: f32,
    sample_rate: f32,
    seed: u32,
) -> f32 {
    let p = std::f32::consts::TAU * freq * t + phase;
    match osc {
        OscillatorType::Sine => p.sin(),
        OscillatorType::Square => {
            if p.sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        OscillatorType::Saw => 2.0 * (freq * t - (0.5 + freq * t).floor()),
        OscillatorType::Triangle => (2.0 / std::f32::consts::PI) * p.sin().asin(),
        OscillatorType::Noise => hash_noise((t * sample_rate.max(1.0)) as i32, seed),
        OscillatorType::Fm => (p + 2.0 * p.sin()).sin(),
    }
}

fn adsr(t: f32, a: f32, d: f32, s: f32, r: f32, gate: f32) -> f32 {
    if t < a.max(1e-6) {
        return t / a.max(1e-6);
    }
    if t < a + d.max(1e-6) {
        let k = (t - a) / d.max(1e-6);
        return 1.0 + (s - 1.0) * k;
    }
    if t < gate {
        return s;
    }
    let rel_t = (t - gate).max(0.0);
    (s * (1.0 - rel_t / r.max(1e-6))).max(0.0)
}

fn hash_noise(x: i32, seed: u32) -> f32 {
    let mut n = x ^ seed as i32;
    n = (n << 13) ^ n;
    let mixed = n
        .wrapping_mul(n.wrapping_mul(n).wrapping_mul(15_731).wrapping_add(789_221))
        .wrapping_add(1_376_312_589);
    let v = 1.0 - (((mixed & 0x7fff_ffff) as f32) / 1_073_741_824.0);
    v.clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{OscillatorType, SynthNode, sample_synth};

    #[test]
    fn noise_oscillator_depends_on_runtime_sample_rate() {
        let node = SynthNode::Oscillator {
            osc_type: OscillatorType::Noise,
            frequency: 220.0,
            amplitude: 1.0,
            phase: 0.0,
        };

        let t = 0.123;
        let at_44k = sample_synth(&node, t, 44_100.0, 99);
        let at_48k = sample_synth(&node, t, 48_000.0, 99);

        assert_ne!(at_44k, at_48k);
    }

    #[test]
    fn noise_oscillator_is_deterministic_for_same_inputs() {
        let node = SynthNode::Oscillator {
            osc_type: OscillatorType::Noise,
            frequency: 220.0,
            amplitude: 0.6,
            phase: 0.0,
        };

        let a = sample_synth(&node, 0.5, 48_000.0, 1234);
        let b = sample_synth(&node, 0.5, 48_000.0, 1234);

        assert_eq!(a, b);
    }
}
