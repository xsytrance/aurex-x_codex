use aurex_music::rhythm_field::RhythmField;
use aurex_render_sdf::diagnostics::FrameDiagnostics;

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
    pub last_frame: Option<FrameDiagnostics>,
}
