use crate::{Rgba8, V3};
use aurex_scene::{
    TemporalBlendMode as SceneTemporalBlendMode, TemporalEffect as SceneTemporalEffect,
};

#[derive(Debug, Clone)]
pub(crate) struct TemporalFrame {
    pub width: u32,
    pub height: u32,
    pub color: Vec<V3>,
    pub depth: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TemporalBuffer {
    pub history: Vec<TemporalFrame>,
    pub max_history: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TemporalConfig {
    pub enabled: bool,
    pub history_depth: usize,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_depth: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TemporalBlendMode {
    AdditiveTrail,
    DecayTrail,
    MotionEcho,
    BeatEcho,
    ColorSmear,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TemporalEffect {
    pub blend_mode: TemporalBlendMode,
    pub decay_rate: f32,
    pub feedback_strength: f32,
    pub beat_sync: f32,
    pub color_shift: [f32; 3],
}

impl From<SceneTemporalBlendMode> for TemporalBlendMode {
    fn from(value: SceneTemporalBlendMode) -> Self {
        match value {
            SceneTemporalBlendMode::AdditiveTrail => Self::AdditiveTrail,
            SceneTemporalBlendMode::DecayTrail => Self::DecayTrail,
            SceneTemporalBlendMode::MotionEcho => Self::MotionEcho,
            SceneTemporalBlendMode::BeatEcho => Self::BeatEcho,
            SceneTemporalBlendMode::ColorSmear => Self::ColorSmear,
        }
    }
}

impl From<SceneTemporalEffect> for TemporalEffect {
    fn from(value: SceneTemporalEffect) -> Self {
        Self {
            blend_mode: value.blend_mode.into(),
            decay_rate: value.decay_rate,
            feedback_strength: value.feedback_strength,
            beat_sync: value.beat_sync,
            color_shift: [
                value.color_shift.x,
                value.color_shift.y,
                value.color_shift.z,
            ],
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_temporal_feedback(
    current: &[V3],
    depth: &[f32],
    width: u32,
    height: u32,
    buffer: &mut TemporalBuffer,
    config: TemporalConfig,
    effects: &[TemporalEffect],
    beat_phase: f32,
    current_measure: u32,
    harmonic_energy: f32,
    dominant_frequency: f32,
) -> Vec<V3> {
    if !config.enabled || effects.is_empty() {
        push_history(current, depth, width, height, buffer, config.history_depth);
        return current.to_vec();
    }

    let mut out = current.to_vec();

    for past in &buffer.history {
        if past.width != width || past.height != height {
            continue;
        }
        for fx in effects {
            apply_effect(
                &mut out,
                &past.color,
                depth,
                &past.depth,
                fx,
                beat_phase,
                current_measure,
                harmonic_energy,
                dominant_frequency,
            );
        }
    }

    push_history(current, depth, width, height, buffer, config.history_depth);
    out
}

fn push_history(
    current: &[V3],
    depth: &[f32],
    width: u32,
    height: u32,
    buffer: &mut TemporalBuffer,
    history_depth: usize,
) {
    let depth_cap = history_depth.max(1);
    buffer.max_history = depth_cap;
    buffer.history.insert(
        0,
        TemporalFrame {
            width,
            height,
            color: current.to_vec(),
            depth: depth.to_vec(),
        },
    );
    if buffer.history.len() > depth_cap {
        buffer.history.truncate(depth_cap);
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_effect(
    out: &mut [V3],
    prev_color: &[V3],
    curr_depth: &[f32],
    prev_depth: &[f32],
    fx: &TemporalEffect,
    beat_phase: f32,
    current_measure: u32,
    harmonic_energy: f32,
    dominant_frequency: f32,
) {
    let beat_gain = 1.0 + fx.beat_sync.max(0.0) * beat_phase;
    let measure_gate = if current_measure.is_multiple_of(2) {
        1.0
    } else {
        0.9
    };
    let harmonic = (0.6 + harmonic_energy * 0.4).clamp(0.4, 1.8);
    let freq_factor = (dominant_frequency / 440.0).clamp(0.3, 2.0);
    let strength =
        (fx.feedback_strength * beat_gain * measure_gate * harmonic * freq_factor).clamp(0.0, 2.0);
    let decay = fx.decay_rate.clamp(0.0, 1.0);

    for i in 0..out
        .len()
        .min(prev_color.len())
        .min(prev_depth.len())
        .min(curr_depth.len())
    {
        let depth_delta = (curr_depth[i] - prev_depth[i]).abs();
        let motion_weight = (depth_delta * 0.15).clamp(0.0, 1.0);
        let shift = V3::new(fx.color_shift[0], fx.color_shift[1], fx.color_shift[2]);
        let pc = prev_color[i];
        let contrib = match fx.blend_mode {
            TemporalBlendMode::AdditiveTrail => pc * strength,
            TemporalBlendMode::DecayTrail => pc * (strength * decay),
            TemporalBlendMode::MotionEcho => pc * (strength * (0.4 + motion_weight * 0.6)),
            TemporalBlendMode::BeatEcho => pc * (strength * beat_gain * 0.8),
            TemporalBlendMode::ColorSmear => (pc + shift) * (strength * 0.7),
        };
        out[i] = (out[i] * decay + contrib * (1.0 - decay)).clamp01();
    }
}

pub(crate) fn to_rgba8_pixels(colors: &[V3]) -> Vec<Rgba8> {
    colors
        .iter()
        .map(|c| Rgba8 {
            r: (c.x.clamp(0.0, 1.0) * 255.0) as u8,
            g: (c.y.clamp(0.0, 1.0) * 255.0) as u8,
            b: (c.z.clamp(0.0, 1.0) * 255.0) as u8,
            a: 255,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_history_is_bounded() {
        let mut buf = TemporalBuffer::default();
        let c = vec![V3::new(0.1, 0.2, 0.3); 4];
        let d = vec![1.0; 4];
        for _ in 0..10 {
            let _ = apply_temporal_feedback(
                &c,
                &d,
                2,
                2,
                &mut buf,
                TemporalConfig {
                    enabled: true,
                    history_depth: 3,
                },
                &[TemporalEffect {
                    blend_mode: TemporalBlendMode::AdditiveTrail,
                    decay_rate: 0.9,
                    feedback_strength: 0.4,
                    beat_sync: 0.0,
                    color_shift: [0.0, 0.0, 0.0],
                }],
                0.5,
                1,
                0.5,
                220.0,
            );
        }
        assert!(buf.history.len() <= 3);
    }
}
