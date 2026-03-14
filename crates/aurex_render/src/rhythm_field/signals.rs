use super::snapshot::RhythmFieldSnapshot;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SequencerState {
    pub bpm: f32,
    pub beat_index: u32,
    pub bar_index: u32,
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
}

impl Default for SequencerState {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            beat_index: 0,
            bar_index: 0,
            bass_energy: 0.4,
            mid_energy: 0.35,
            high_energy: 0.3,
        }
    }
}

pub fn sample_rhythm_field(
    seed: u64,
    time: f32,
    sequencer_state: SequencerState,
) -> RhythmFieldSnapshot {
    let bpm = sequencer_state.bpm.max(1.0);
    let beat_period = 60.0 / bpm;
    let beat_phase = (time / beat_period).fract().clamp(0.0, 1.0);

    let bars_from_time = time / (beat_period * 4.0);
    let bar_phase = (bars_from_time + sequencer_state.bar_index as f32)
        .fract()
        .clamp(0.0, 1.0);

    let pulse = (1.0 - (beat_phase - 0.5).abs() * 2.0).clamp(0.0, 1.0);

    let bass_energy = normalize01(sequencer_state.bass_energy);
    let mid_energy = normalize01(sequencer_state.mid_energy);
    let high_energy = normalize01(sequencer_state.high_energy);

    let seed_bias = seed_unit(seed, sequencer_state.beat_index);
    let accent_gate = if sequencer_state.beat_index.is_multiple_of(4) {
        1.0
    } else {
        0.45
    };
    let accent = (accent_gate * (0.65 + 0.35 * pulse) * (0.8 + 0.2 * seed_bias)).clamp(0.0, 1.0);

    let intensity =
        (bass_energy * 0.42 + mid_energy * 0.28 + high_energy * 0.2 + pulse * 0.1).clamp(0.0, 1.0);

    RhythmFieldSnapshot {
        beat_phase,
        bar_phase,
        pulse,
        bass_energy,
        mid_energy,
        high_energy,
        intensity,
        accent,
    }
    .clamped()
}

fn normalize01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn seed_unit(seed: u64, step: u32) -> f32 {
    let mut x = seed ^ ((step as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x as f64 / u64::MAX as f64) as f32
}
