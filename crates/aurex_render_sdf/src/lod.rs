use std::sync::atomic::{AtomicU64, Ordering};

static LOD_ACTIVATIONS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
pub struct LodConfig {
    pub distance_bias: f32,
    pub fractal_iteration_limit: u32,
    pub fractal_lod_scale: f32,
    pub detail_falloff: f32,
}

pub fn reset_lod_counters() {
    LOD_ACTIVATIONS.store(0, Ordering::Relaxed);
}

pub fn lod_activation_count() -> u64 {
    LOD_ACTIVATIONS.load(Ordering::Relaxed)
}

pub fn lod_iterations(base_iterations: u32, distance: f32, cfg: LodConfig) -> u32 {
    let bias = cfg.distance_bias.max(0.01);
    let d = (distance * bias).max(0.0);
    let reduction = 1.0 + d * cfg.fractal_lod_scale.max(0.01) * cfg.detail_falloff.max(0.01);
    let it = ((base_iterations as f32) / reduction).round() as u32;
    let clamped = it.max(2).min(cfg.fractal_iteration_limit.max(2));
    if clamped < base_iterations {
        LOD_ACTIVATIONS.fetch_add(1, Ordering::Relaxed);
    }
    clamped
}
