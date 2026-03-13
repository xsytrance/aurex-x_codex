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
    pub idle_time_seconds: f32,
    pub warning_issued: bool,
    pub resonance_event_count: u32,
    pub resonance_event_ready: bool,
    pub prime_pulse_distance: f32,
    pub prime_pulse_layer: u32,
    pub prime_pulse_layers_unlocked: u32,
    pub prime_pulse_force_field_active: bool,
    pub prime_pulse_intensity: f32,
    pub prime_pulse_proximity: f32,
    pub last_frame: Option<FrameDiagnostics>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RhythmSummary {
    pub beat_phase: f32,
    pub bar_index: u64,
    pub bass_energy: f32,
}
