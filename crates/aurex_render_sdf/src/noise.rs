#[derive(Clone, Copy, Debug)]
pub struct NoiseVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl NoiseVec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

pub fn hash_noise(mut x: i32) -> f32 {
    x = ((x >> 13) ^ x).wrapping_mul(15731);
    x = (x.wrapping_mul(x).wrapping_mul(789221)).wrapping_add(1376312589);
    let n = (x & 0x7fff_ffff) as f32 / 2_147_483_647.0;
    n * 2.0 - 1.0
}

fn hash3i(x: i32, y: i32, z: i32, seed: i32) -> f32 {
    let h = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(z.wrapping_mul(2147483647))
        .wrapping_add(seed.wrapping_mul(1274126177));
    hash_noise(h)
}

fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn value_noise(p: NoiseVec3, seed: i32) -> f32 {
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let iz = p.z.floor() as i32;

    let fx = p.x - ix as f32;
    let fy = p.y - iy as f32;
    let fz = p.z - iz as f32;

    let u = fade(fx);
    let v = fade(fy);
    let w = fade(fz);

    let c000 = hash3i(ix, iy, iz, seed);
    let c100 = hash3i(ix + 1, iy, iz, seed);
    let c010 = hash3i(ix, iy + 1, iz, seed);
    let c110 = hash3i(ix + 1, iy + 1, iz, seed);
    let c001 = hash3i(ix, iy, iz + 1, seed);
    let c101 = hash3i(ix + 1, iy, iz + 1, seed);
    let c011 = hash3i(ix, iy + 1, iz + 1, seed);
    let c111 = hash3i(ix + 1, iy + 1, iz + 1, seed);

    let x00 = lerp(c000, c100, u);
    let x10 = lerp(c010, c110, u);
    let x01 = lerp(c001, c101, u);
    let x11 = lerp(c011, c111, u);

    let y0 = lerp(x00, x10, v);
    let y1 = lerp(x01, x11, v);

    lerp(y0, y1, w)
}

pub fn fbm(mut p: NoiseVec3, octaves: u32, lacunarity: f32, gain: f32, seed: i32) -> f32 {
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    let mut value = 0.0;

    for i in 0..octaves {
        let n = value_noise(
            NoiseVec3::new(p.x * frequency, p.y * frequency, p.z * frequency),
            seed.wrapping_add(i as i32 * 17),
        );
        value += n * amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
        p.x += 0.173;
        p.y += 0.137;
        p.z += 0.193;
    }

    value
}

#[cfg(test)]
mod tests {
    use super::{NoiseVec3, fbm, hash_noise, value_noise};

    #[test]
    fn noise_is_deterministic() {
        let a = value_noise(NoiseVec3::new(1.2, 3.4, 5.6), 44);
        let b = value_noise(NoiseVec3::new(1.2, 3.4, 5.6), 44);
        assert_eq!(a, b);
        assert_eq!(hash_noise(7), hash_noise(7));
        assert_eq!(
            fbm(NoiseVec3::new(0.4, 1.3, 2.2), 5, 2.0, 0.5, 91),
            fbm(NoiseVec3::new(0.4, 1.3, 2.2), 5, 2.0, 0.5, 91)
        );
    }
}
