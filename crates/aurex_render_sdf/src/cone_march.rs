#[derive(Debug, Clone, Copy)]
pub struct ConeMarchConfig {
    pub cone_step_multiplier: f32,
    pub cone_shadow_factor: f32,
    pub surface_thickness_estimation: f32,
    pub adaptive_step_scale: f32,
}

pub fn adaptive_step_scale(distance: f32, travel: f32, cfg: ConeMarchConfig) -> f32 {
    let far = (travel * 0.02).clamp(0.0, 1.0);
    let thick = cfg.surface_thickness_estimation.max(1e-4);
    let emptiness = (distance / thick).clamp(0.0, 8.0);
    (1.0 + far * cfg.adaptive_step_scale.max(0.0) + emptiness * 0.08)
        .clamp(0.2, cfg.cone_step_multiplier.max(0.2))
}

pub fn cone_step(distance: f32, travel: f32, cfg: ConeMarchConfig) -> f32 {
    distance.max(1e-4) * adaptive_step_scale(distance, travel, cfg)
}

pub fn shadow_cone_factor(distance: f32, cfg: ConeMarchConfig) -> f32 {
    (distance * cfg.cone_shadow_factor.max(0.05)).clamp(0.0, 1.0)
}
