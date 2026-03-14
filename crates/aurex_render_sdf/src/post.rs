use crate::V3;

#[derive(Debug, Clone, Copy)]
pub struct PostProcessConfig {
    pub gamma: f32,
    pub vignette: f32,
    pub chromatic_aberration: f32,
    pub film_grain: f32,
    pub bloom_strength: f32,
    pub exposure_bias: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            gamma: 2.2,
            vignette: 0.2,
            chromatic_aberration: 0.1,
            film_grain: 0.03,
            bloom_strength: 0.2,
            exposure_bias: 0.0,
        }
    }
}

pub(crate) fn process_pixel(
    hdr: V3,
    emission: f32,
    uv: (f32, f32),
    seed: u32,
    config: PostProcessConfig,
) -> V3 {
    let exposure = adaptive_exposure(hdr, emission, config.exposure_bias);
    let mut c = hdr * exposure;
    c = aces_tonemap(c);
    c = apply_bloom_hint(c, emission, config.bloom_strength);
    c = apply_vignette(c, uv, config.vignette);
    c = apply_chromatic_hint(c, uv, config.chromatic_aberration);
    c = apply_film_grain(c, uv, seed, config.film_grain);
    apply_gamma(c, config.gamma)
}

fn adaptive_exposure(hdr: V3, emission: f32, bias: f32) -> f32 {
    let luma = hdr.x * 0.2126 + hdr.y * 0.7152 + hdr.z * 0.0722;
    let target = (0.7 / (0.15 + luma + emission * 0.3)).clamp(0.35, 1.8);
    target * (1.0 + bias)
}

fn aces_tonemap(c: V3) -> V3 {
    let a = 2.51;
    let b = 0.03;
    let c1 = 2.43;
    let d = 0.59;
    let e = 0.14;
    V3::new(
        ((c.x * (a * c.x + b)) / (c.x * (c1 * c.x + d) + e)).clamp(0.0, 1.0),
        ((c.y * (a * c.y + b)) / (c.y * (c1 * c.y + d) + e)).clamp(0.0, 1.0),
        ((c.z * (a * c.z + b)) / (c.z * (c1 * c.z + d) + e)).clamp(0.0, 1.0),
    )
}

fn apply_gamma(c: V3, gamma: f32) -> V3 {
    let inv = (1.0 / gamma.max(1e-3)).clamp(0.1, 2.0);
    V3::new(c.x.powf(inv), c.y.powf(inv), c.z.powf(inv)).clamp01()
}

fn apply_vignette(c: V3, uv: (f32, f32), amount: f32) -> V3 {
    let dx = uv.0 - 0.5;
    let dy = uv.1 - 0.5;
    let d = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
    let v = 1.0 - d * d * amount;
    c * v.clamp(0.0, 1.0)
}

fn apply_chromatic_hint(c: V3, uv: (f32, f32), amount: f32) -> V3 {
    let edge = ((uv.0 - 0.5).abs() + (uv.1 - 0.5).abs()).clamp(0.0, 1.0);
    let shift = edge * amount * 0.08;
    V3::new(
        (c.x + shift).clamp(0.0, 1.0),
        c.y,
        (c.z + shift * 0.4).clamp(0.0, 1.0),
    )
}

fn apply_film_grain(c: V3, uv: (f32, f32), seed: u32, amount: f32) -> V3 {
    let n = (((uv.0 * 9123.7 + uv.1 * 5729.3 + seed as f32 * 0.01).sin()) * 43_758.547).fract();
    let centered = (n - 0.5) * amount;
    (c + V3::splat(centered)).clamp01()
}

fn apply_bloom_hint(c: V3, emission: f32, amount: f32) -> V3 {
    c + V3::splat((emission * amount).clamp(0.0, 0.35))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_process_is_stable() {
        let cfg = PostProcessConfig::default();
        let a = process_pixel(V3::new(1.6, 0.8, 0.4), 0.9, (0.25, 0.75), 99, cfg);
        let b = process_pixel(V3::new(1.6, 0.8, 0.4), 0.9, (0.25, 0.75), 99, cfg);
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.z, b.z);
    }
}
