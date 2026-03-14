use crate::V3;

#[derive(Debug, Clone, Copy)]
pub struct ParticleConfig {
    pub density: f32,
    pub intensity: f32,
}

pub(crate) fn particle_overlay(
    uv: (f32, f32),
    time: f32,
    seed: u32,
    audio_energy: f32,
    beat_phase: f32,
    harmonic: f32,
    cfg: ParticleConfig,
) -> V3 {
    let t = time * 0.7 + beat_phase * 0.5;
    let n = (((uv.0 * 1234.5 + uv.1 * 9876.5 + t + seed as f32 * 0.01).sin()) * 43_758.547).fract();
    let spark = (n - (1.0 - cfg.density.clamp(0.0, 1.0))).max(0.0) * 8.0;
    let glow = spark * cfg.intensity * (0.4 + audio_energy * 0.6 + harmonic * 0.3);
    V3::new(glow * 0.4, glow * 0.7, glow)
}
