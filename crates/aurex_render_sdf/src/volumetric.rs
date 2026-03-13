use crate::V3;

#[derive(Debug, Clone, Copy)]
pub struct VolumetricConfig {
    pub volumetric_density: f32,
    pub scatter_strength: f32,
    pub fog_color: [f32; 3],
    pub light_beam_strength: f32,
}

pub(crate) fn apply_volumetric(
    color: V3,
    distance: f32,
    harmonic_energy: f32,
    cfg: VolumetricConfig,
) -> V3 {
    let density = (cfg.volumetric_density * (1.0 + harmonic_energy * 0.4)).max(0.0);
    let trans = (-density * distance * 0.05).exp();
    let fog = V3::new(cfg.fog_color[0], cfg.fog_color[1], cfg.fog_color[2]);
    let shafts = cfg.scatter_strength.max(0.0) * cfg.light_beam_strength.max(0.0) * (1.0 - trans);
    (color * trans + fog * (1.0 - trans) + V3::splat(shafts * 0.08)).clamp01()
}
