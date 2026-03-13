use aurex_music::rhythm_field::RhythmField;
use aurex_render_sdf::diagnostics::FrameDiagnostics;

use crate::resonance::PrimeFaction;

#[derive(Debug, Clone, Default)]
pub struct LifecycleTiming {
    pub load_ms: f64,
    pub initialize_ms: f64,
    pub update_ms: f64,
    pub render_ms: f64,
    pub shutdown_ms: f64,
}

#[derive(Debug, Clone, Default)]
pub struct PulseDiagnostics {
    pub lifecycle: LifecycleTiming,
    pub frames_rendered: u64,
    pub rhythm_field: Option<RhythmField>,
    pub rhythm_summary: Option<RhythmSummary>,
    pub dominant_prime: Option<PrimeFaction>,
    pub top_three_primes: Vec<PrimeFaction>,
    pub last_frame: Option<FrameDiagnostics>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RhythmSummary {
    pub beat_phase: f32,
    pub bar_index: u64,
    pub bass_energy: f32,
}
