use aurex_scene::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub fn repeat_grid(p: Vec3, cell_size: Vec3) -> Vec3 {
    Vec3 {
        x: repeat_axis_scalar(p.x, cell_size.x),
        y: repeat_axis_scalar(p.y, cell_size.y),
        z: repeat_axis_scalar(p.z, cell_size.z),
    }
}

pub fn repeat_axis(p: Vec3, spacing: f32, axis: Axis) -> Vec3 {
    let mut out = p;
    match axis {
        Axis::X => out.x = repeat_axis_scalar(out.x, spacing),
        Axis::Y => out.y = repeat_axis_scalar(out.y, spacing),
        Axis::Z => out.z = repeat_axis_scalar(out.z, spacing),
    }
    out
}

pub fn repeat_polar(p: Vec3, sectors: u32) -> Vec3 {
    let n = sectors.max(1) as f32;
    let sector_angle = std::f32::consts::TAU / n;
    let r = (p.x * p.x + p.z * p.z).sqrt();
    let a = p.z.atan2(p.x);
    let folded = (a + 0.5 * sector_angle).rem_euclid(sector_angle) - 0.5 * sector_angle;
    Vec3 {
        x: folded.cos() * r,
        y: p.y,
        z: folded.sin() * r,
    }
}

pub fn repeat_sphere(p: Vec3, radius: f32) -> Vec3 {
    let r = radius.abs().max(1e-6);
    let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
    if len < 1e-6 {
        return p;
    }
    let k = (len / r).round().max(1.0);
    let target = (len - k * r).abs();
    let scale = target / len;
    Vec3 {
        x: p.x * scale,
        y: p.y * scale,
        z: p.z * scale,
    }
}

pub fn fold_space(p: Vec3) -> Vec3 {
    Vec3 {
        x: p.x.abs(),
        y: p.y.abs(),
        z: p.z.abs(),
    }
}

pub fn kaleidoscope_fold(p: Vec3, segments: u32) -> Vec3 {
    let mirrored = mirror_fold(p);
    repeat_polar(mirrored, segments)
}

pub fn mirror_fold(p: Vec3) -> Vec3 {
    Vec3 {
        x: p.x.abs(),
        y: p.y,
        z: p.z.abs(),
    }
}

fn repeat_axis_scalar(value: f32, spacing: f32) -> f32 {
    let s = spacing.abs();
    if s < 1e-6 {
        return value;
    }
    (value + 0.5 * s).rem_euclid(s) - 0.5 * s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_repeat_is_deterministic() {
        let p = Vec3 {
            x: 12.4,
            y: -3.7,
            z: 8.9,
        };
        let c = Vec3 {
            x: 4.0,
            y: 2.0,
            z: 3.0,
        };
        assert_eq!(repeat_grid(p, c), repeat_grid(p, c));
    }

    #[test]
    fn mirror_fold_keeps_positive_xz() {
        let p = Vec3 {
            x: -2.0,
            y: -1.0,
            z: -3.0,
        };
        let m = mirror_fold(p);
        assert!(m.x >= 0.0 && m.z >= 0.0);
        assert_eq!(m.y, -1.0);
    }
}
