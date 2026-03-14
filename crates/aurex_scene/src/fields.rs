use crate::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SceneField {
    Noise(NoiseField),
    Flow(FlowField),
    Pulse(PulseField),
    Audio(AudioField),
    Rhythm(RhythmField),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct NoiseField {
    pub scale: f32,
    pub strength: f32,
    pub octaves: u32,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct FlowField {
    pub direction: Vec3,
    pub turbulence: f32,
    pub strength: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PulseField {
    pub origin: Vec3,
    pub frequency: f32,
    pub amplitude: f32,
    pub falloff: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RhythmField {
    pub beat_strength: f32,
    pub measure_strength: f32,
    pub phrase_strength: f32,
    pub tempo: f32,
}

pub fn sample_rhythm(field: RhythmField, time: f32) -> f32 {
    let beat_pos = time / (60.0 / field.tempo.max(1.0));
    let beat_phase = beat_pos.fract();
    let measure_phase = (beat_pos / 4.0).fract();
    let phrase_phase = (beat_pos / 16.0).fract();
    let beat = (1.0 - beat_phase).powf(2.0) * field.beat_strength;
    let measure = (1.0 - measure_phase).powf(2.0) * field.measure_strength;
    let phrase = (1.0 - phrase_phase).powf(2.0) * field.phrase_strength;
    beat + measure + phrase
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioBand {
    Kick,
    Snare,
    Bass,
    Mid,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AudioField {
    pub band: AudioBand,
    pub strength: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldSample {
    pub scalar: f32,
    pub vector: Vec3,
    pub energy: f32,
}

impl FieldSample {
    pub const fn zero() -> Self {
        Self {
            scalar: 0.0,
            vector: Vec3::new(0.0, 0.0, 0.0),
            energy: 0.0,
        }
    }

    pub fn combine(self, rhs: Self) -> Self {
        Self {
            scalar: self.scalar + rhs.scalar,
            vector: Vec3::new(
                self.vector.x + rhs.vector.x,
                self.vector.y + rhs.vector.y,
                self.vector.z + rhs.vector.z,
            ),
            energy: self.energy + rhs.energy,
        }
    }
}

pub fn sample_field(field: &SceneField, position: Vec3, time: f32, scene_seed: u32) -> FieldSample {
    match field {
        SceneField::Noise(f) => sample_noise_field(*f, position, time, scene_seed),
        SceneField::Flow(f) => sample_flow_field(*f, position, time, scene_seed),
        SceneField::Pulse(f) => sample_pulse_field(*f, position, time),
        SceneField::Audio(f) => sample_audio_field(*f, position, time, scene_seed),
        SceneField::Rhythm(f) => sample_rhythm_field(*f, position, time),
    }
}

pub fn sample_fields(
    fields: &[SceneField],
    position: Vec3,
    time: f32,
    scene_seed: u32,
) -> FieldSample {
    fields.iter().fold(FieldSample::zero(), |acc, f| {
        acc.combine(sample_field(f, position, time, scene_seed))
    })
}

fn sample_noise_field(field: NoiseField, position: Vec3, time: f32, seed: u32) -> FieldSample {
    let mut amp = 1.0;
    let mut freq = field.scale.max(0.001);
    let mut v = 0.0;
    for i in 0..field.octaves.max(1) {
        let n = hash3(
            position.x * freq + time * field.speed,
            position.y * freq,
            position.z * freq,
            seed.wrapping_add(i * 31),
        );
        v += n * amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    let scalar = v * field.strength;
    FieldSample {
        scalar,
        vector: Vec3::new(scalar * 0.7, scalar * 0.3, -scalar * 0.6),
        energy: scalar.abs(),
    }
}

fn sample_flow_field(field: FlowField, position: Vec3, time: f32, seed: u32) -> FieldSample {
    let t = hash3(
        position.x * 0.43 + time,
        position.y * 0.51,
        position.z * 0.39,
        seed,
    ) * field.turbulence;
    let dir = field.direction;
    let vec = Vec3::new(
        (dir.x + t).sin() * field.strength,
        (dir.y + t * 0.7).sin() * field.strength,
        (dir.z - t * 1.2).sin() * field.strength,
    );
    FieldSample {
        scalar: (vec.x + vec.y + vec.z) / 3.0,
        vector: vec,
        energy: (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt(),
    }
}

fn sample_pulse_field(field: PulseField, position: Vec3, time: f32) -> FieldSample {
    let dx = position.x - field.origin.x;
    let dy = position.y - field.origin.y;
    let dz = position.z - field.origin.z;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let wave = (dist * 2.0 - time * field.frequency * std::f32::consts::TAU).sin();
    let atten = (-field.falloff.max(0.0) * dist).exp();
    let e = wave.abs() * atten * field.amplitude;
    FieldSample {
        scalar: wave * atten * field.amplitude,
        vector: Vec3::new(dx, dy, dz),
        energy: e,
    }
}

fn sample_rhythm_field(field: RhythmField, position: Vec3, time: f32) -> FieldSample {
    let r = sample_rhythm(field, time);
    let d = (position.x * position.x + position.z * position.z).sqrt();
    let radial = (1.0 - d / 25.0).clamp(0.0, 1.0);
    let e = r * radial;
    FieldSample {
        scalar: e,
        vector: Vec3::new(0.0, e * 0.8, 0.0),
        energy: e.abs(),
    }
}

fn sample_audio_field(field: AudioField, position: Vec3, time: f32, seed: u32) -> FieldSample {
    let band_phase = match field.band {
        AudioBand::Kick => 1.0,
        AudioBand::Snare => 2.1,
        AudioBand::Bass => 0.5,
        AudioBand::Mid => 1.6,
        AudioBand::High => 3.2,
    };
    let dist = (position.x * position.x + position.y * position.y + position.z * position.z).sqrt();
    let radial = (1.0 - dist / field.radius.max(0.001)).clamp(0.0, 1.0);
    let signal = (time * band_phase * std::f32::consts::TAU
        + hash3(position.x, position.y, position.z, seed))
    .sin()
    .abs();
    let e = signal * radial * field.strength;
    FieldSample {
        scalar: e,
        vector: Vec3::new(0.0, e * 0.5, 0.0),
        energy: e,
    }
}

fn hash3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let v = (x * 127.1 + y * 311.7 + z * 74.7 + seed as f32 * 19.19).sin() * 43_758.547;
    v.fract() * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::{
        AudioBand, AudioField, FieldSample, FlowField, NoiseField, PulseField, RhythmField,
        SceneField, sample_fields,
    };
    use crate::Vec3;

    #[test]
    fn field_sampling_is_deterministic() {
        let fields = vec![
            SceneField::Noise(NoiseField {
                scale: 1.5,
                strength: 0.4,
                octaves: 4,
                speed: 1.0,
            }),
            SceneField::Pulse(PulseField {
                origin: Vec3::new(0.0, 0.0, 0.0),
                frequency: 2.0,
                amplitude: 1.0,
                falloff: 0.2,
            }),
            SceneField::Flow(FlowField {
                direction: Vec3::new(1.0, 0.2, 0.4),
                turbulence: 0.5,
                strength: 0.6,
            }),
            SceneField::Audio(AudioField {
                band: AudioBand::Kick,
                strength: 0.8,
                radius: 10.0,
            }),
            SceneField::Rhythm(RhythmField {
                beat_strength: 1.0,
                measure_strength: 0.5,
                phrase_strength: 0.3,
                tempo: 140.0,
            }),
        ];
        let a = sample_fields(&fields, Vec3::new(1.0, 0.5, -2.0), 1.2, 42);
        let b = sample_fields(&fields, Vec3::new(1.0, 0.5, -2.0), 1.2, 42);
        assert_eq!(a, b);
        assert!(a != FieldSample::zero());
    }
}
