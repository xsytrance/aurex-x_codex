use crate::V3;

pub(crate) fn fold_scale_iterate(mut p: V3, iterations: u32, scale: f32) -> V3 {
    let s = scale.max(1e-3);
    for _ in 0..iterations.max(1) {
        p = V3::new(p.x.abs(), p.y.abs(), p.z.abs()) * s - V3::splat(s - 1.0);
    }
    p
}

pub(crate) fn recursive_symmetry(mut p: V3, depth: u32) -> V3 {
    for _ in 0..depth.max(1) {
        p = V3::new(p.x.abs(), p.y.abs(), p.z.abs());
        if p.x < p.y {
            std::mem::swap(&mut p.x, &mut p.y);
        }
        if p.x < p.z {
            std::mem::swap(&mut p.x, &mut p.z);
        }
    }
    p
}

pub(crate) fn kifs_fractal(p: V3, iterations: u32, scale: f32, bailout: f32) -> f32 {
    let mut z = p;
    let mut r = z.length();
    for _ in 0..iterations.max(2) {
        z = recursive_symmetry(fold_scale_iterate(z, 1, scale), 1) + p;
        r = z.length();
        if r > bailout {
            break;
        }
    }
    r - 1.0
}
