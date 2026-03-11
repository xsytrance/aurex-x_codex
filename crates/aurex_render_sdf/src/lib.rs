use aurex_scene::{Scene, SdfModifier, SdfObject, SdfPrimitive, Vec3};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub max_steps: u32,
    pub max_distance: f32,
    pub surface_epsilon: f32,
    pub shadow_steps: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 360,
            max_steps: 128,
            max_distance: 120.0,
            surface_epsilon: 0.001,
            shadow_steps: 48,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgba8>,
}

pub fn render_sdf_scene(scene: &Scene) -> RenderedFrame {
    render_sdf_scene_with_config(scene, RenderConfig::default())
}

pub fn render_sdf_scene_with_config(scene: &Scene, config: RenderConfig) -> RenderedFrame {
    let mut pixels = Vec::with_capacity((config.width * config.height) as usize);
    for y in 0..config.height {
        for x in 0..config.width {
            let ray = generate_camera_ray(scene, x, y, config.width, config.height);
            let color = shade_ray(scene, ray.origin, ray.direction, config);
            pixels.push(to_rgba8(color));
        }
    }

    RenderedFrame {
        width: config.width,
        height: config.height,
        pixels,
    }
}

pub fn wgpu_backend_marker() -> wgpu::Features {
    wgpu::Features::empty()
}

#[derive(Clone, Copy)]
struct Ray {
    origin: V3,
    direction: V3,
}

#[derive(Clone, Copy)]
struct Hit {
    position: V3,
    normal: V3,
    object_index: usize,
}

fn shade_ray(scene: &Scene, origin: V3, direction: V3, config: RenderConfig) -> V3 {
    if let Some(hit) = march_scene(scene, origin, direction, config) {
        let mut color = V3::splat(scene.sdf.lighting.ambient_light);
        let base = material_color(&scene.sdf.objects[hit.object_index]);

        for key in &scene.sdf.lighting.key_lights {
            let ldir = from_vec3(key.direction).normalized() * -1.0;
            let lambert = hit.normal.dot(ldir).max(0.0);
            let shadow = soft_shadow(scene, hit.position + hit.normal * 0.005, ldir, config);
            let light_color = from_vec3(key.color) * key.intensity * lambert * shadow;
            color = color + base.hadamard(light_color);
        }

        return color.clamp01();
    }

    let t = 0.5 * (direction.y + 1.0);
    V3::new(0.05, 0.08, 0.12) * (1.0 - t) + V3::new(0.25, 0.3, 0.4) * t
}

fn material_color(object: &SdfObject) -> V3 {
    from_vec3(object.material.color)
}

fn march_scene(scene: &Scene, origin: V3, direction: V3, config: RenderConfig) -> Option<Hit> {
    let mut t = 0.0;
    for _ in 0..config.max_steps {
        let p = origin + direction * t;
        let (distance, object_index) = scene_distance(scene, p);
        if distance < config.surface_epsilon {
            let normal = estimate_normal(scene, p, config.surface_epsilon * 2.0);
            return Some(Hit {
                position: p,
                normal,
                object_index,
            });
        }

        t += distance;
        if t > config.max_distance {
            return None;
        }
    }

    None
}

fn soft_shadow(scene: &Scene, origin: V3, direction: V3, config: RenderConfig) -> f32 {
    let mut t = 0.02;
    let mut visibility: f32 = 1.0;
    for _ in 0..config.shadow_steps {
        let p = origin + direction * t;
        let (distance, _) = scene_distance(scene, p);
        if distance < config.surface_epsilon {
            return 0.0;
        }
        visibility = visibility.min(12.0 * distance / t.max(0.001));
        t += distance.max(0.01);
        if t > config.max_distance {
            break;
        }
    }
    visibility.clamp(0.0, 1.0)
}

fn estimate_normal(scene: &Scene, p: V3, eps: f32) -> V3 {
    let ex = V3::new(eps, 0.0, 0.0);
    let ey = V3::new(0.0, eps, 0.0);
    let ez = V3::new(0.0, 0.0, eps);

    let dx = scene_distance(scene, p + ex).0 - scene_distance(scene, p - ex).0;
    let dy = scene_distance(scene, p + ey).0 - scene_distance(scene, p - ey).0;
    let dz = scene_distance(scene, p + ez).0 - scene_distance(scene, p - ez).0;

    V3::new(dx, dy, dz).normalized()
}

fn scene_distance(scene: &Scene, point: V3) -> (f32, usize) {
    let mut best = f32::INFINITY;
    let mut index = 0usize;

    for (i, object) in scene.sdf.objects.iter().enumerate() {
        let mut p = point;
        let mut distance_scale = 1.0;

        for modifier in &object.modifiers {
            apply_modifier(&mut p, &mut distance_scale, modifier);
        }

        let distance = eval_primitive(&object.primitive, p) * distance_scale;
        if distance < best {
            best = distance;
            index = i;
        }
    }

    (best, index)
}

fn apply_modifier(p: &mut V3, distance_scale: &mut f32, modifier: &SdfModifier) {
    match modifier {
        SdfModifier::Repeat { cell } => {
            let cell = from_vec3(*cell);
            p.x = repeat_axis(p.x, cell.x);
            p.y = repeat_axis(p.y, cell.y);
            p.z = repeat_axis(p.z, cell.z);
        }
        SdfModifier::Twist { strength } => {
            let angle = *strength * p.y;
            let (s, c) = angle.sin_cos();
            let x = p.x * c - p.z * s;
            let z = p.x * s + p.z * c;
            p.x = x;
            p.z = z;
        }
        SdfModifier::Bend { strength } => {
            let angle = *strength * p.x;
            let (s, c) = angle.sin_cos();
            let y = p.y * c - p.z * s;
            let z = p.y * s + p.z * c;
            p.y = y;
            p.z = z;
        }
        SdfModifier::Scale { factor } => {
            let safe = factor.abs().max(0.0001);
            *p = *p / safe;
            *distance_scale *= safe;
        }
        SdfModifier::Rotate { axis, radians } => {
            *p = rotate_axis_angle(*p, from_vec3(*axis).normalized(), -*radians);
        }
        SdfModifier::Translate { offset } => {
            *p = *p - from_vec3(*offset);
        }
        SdfModifier::NoiseDisplacement {
            amplitude,
            frequency,
            seed,
        } => {
            let n = noise3(*p * *frequency, *seed as i32);
            p.x += n * *amplitude;
            p.y += n * *amplitude;
            p.z += n * *amplitude;
        }
        SdfModifier::Mirror { normal, offset } => {
            let n = from_vec3(*normal).normalized();
            let side = p.dot(n) - *offset;
            if side < 0.0 {
                *p = *p - n * (2.0 * side);
            }
        }
    }
}

fn eval_primitive(primitive: &SdfPrimitive, p: V3) -> f32 {
    match primitive {
        SdfPrimitive::Sphere { radius } => p.length() - *radius,
        SdfPrimitive::Box { size } => {
            let b = from_vec3(*size);
            let q = p.abs() - b;
            q.max(V3::zero()).length() + q.max_component().min(0.0)
        }
        SdfPrimitive::Torus {
            major_radius,
            minor_radius,
        } => {
            let q = V3::new(V2::new(p.x, p.z).length() - *major_radius, p.y, 0.0);
            V2::new(q.x, q.y).length() - *minor_radius
        }
        SdfPrimitive::Plane { normal, offset } => p.dot(from_vec3(*normal).normalized()) + *offset,
        SdfPrimitive::Cylinder {
            radius,
            half_height,
        } => {
            let d = V2::new(
                V2::new(p.x, p.z).length() - *radius,
                p.y.abs() - *half_height,
            );
            d.max(V2::zero()).length() + d.max_component().min(0.0)
        }
        SdfPrimitive::Capsule { a, b, radius } => {
            let a = from_vec3(*a);
            let b = from_vec3(*b);
            let pa = p - a;
            let ba = b - a;
            let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
            (pa - ba * h).length() - *radius
        }
        SdfPrimitive::Mandelbulb {
            power,
            iterations,
            bailout,
        } => sdf_mandelbulb(p, *power, *iterations, *bailout),
        SdfPrimitive::NoiseField {
            radius,
            amplitude,
            frequency,
            seed,
        } => {
            let n = noise3(p * *frequency, *seed as i32);
            p.length() - *radius + n * *amplitude
        }
    }
}

fn sdf_mandelbulb(p: V3, power: f32, iterations: u32, bailout: f32) -> f32 {
    let mut z = p;
    let mut dr = 1.0;
    let mut r = 0.0;

    for _ in 0..iterations {
        r = z.length();
        if r > bailout {
            break;
        }

        let theta = (z.z / r.max(1e-6)).acos();
        let phi = z.y.atan2(z.x);
        dr = r.powf(power - 1.0) * power * dr + 1.0;

        let zr = r.powf(power);
        let theta = theta * power;
        let phi = phi * power;

        z = V3::new(
            theta.sin() * phi.cos(),
            phi.sin() * theta.sin(),
            theta.cos(),
        ) * zr
            + p;
    }

    0.5 * r.ln() * r / dr
}

fn generate_camera_ray(scene: &Scene, x: u32, y: u32, width: u32, height: u32) -> Ray {
    let cam = &scene.sdf.camera;
    let origin = from_vec3(cam.position);
    let target = from_vec3(cam.target);

    let forward = (target - origin).normalized();
    let world_up = V3::new(0.0, 1.0, 0.0);
    let right = forward.cross(world_up).normalized();
    let up = right.cross(forward).normalized();

    let nx = (x as f32 + 0.5) / width as f32;
    let ny = (y as f32 + 0.5) / height as f32;
    let sx = 2.0 * nx - 1.0;
    let sy = 1.0 - 2.0 * ny;

    let fov_scale = (cam.fov_degrees.to_radians() * 0.5).tan();
    let aspect = cam.aspect_ratio;
    let direction =
        (forward + right * (sx * aspect * fov_scale) + up * (sy * fov_scale)).normalized();

    Ray { origin, direction }
}

fn repeat_axis(value: f32, cell: f32) -> f32 {
    if cell.abs() < 1e-6 {
        return value;
    }
    (value + 0.5 * cell).rem_euclid(cell) - 0.5 * cell
}

fn rotate_axis_angle(p: V3, axis: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    p * c + axis.cross(p) * s + axis * axis.dot(p) * (1.0 - c)
}

fn noise3(p: V3, seed: i32) -> f32 {
    let x = (p.x * 127.1 + p.y * 311.7 + p.z * 74.7 + seed as f32 * 13.37).sin() * 43758.5453;
    (x.fract() * 2.0) - 1.0
}

fn to_rgba8(color: V3) -> Rgba8 {
    Rgba8 {
        r: (color.x.clamp(0.0, 1.0) * 255.0) as u8,
        g: (color.y.clamp(0.0, 1.0) * 255.0) as u8,
        b: (color.z.clamp(0.0, 1.0) * 255.0) as u8,
        a: 255,
    }
}

fn from_vec3(v: Vec3) -> V3 {
    V3::new(v.x, v.y, v.z)
}

#[derive(Clone, Copy, Debug)]
struct V2 {
    x: f32,
    y: f32,
}

impl V2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y))
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn max_component(self) -> f32 {
        self.x.max(self.y)
    }
}

#[derive(Clone, Copy, Debug)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

impl V3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn splat(v: f32) -> Self {
        Self::new(v, v, v)
    }

    fn zero() -> Self {
        Self::splat(0.0)
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalized(self) -> Self {
        self / self.length().max(1e-6)
    }

    fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y), self.z.max(rhs.z))
    }

    fn max_component(self) -> f32 {
        self.x.max(self.y).max(self.z)
    }

    fn hadamard(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }

    fn clamp01(self) -> Self {
        Self::new(
            self.x.clamp(0.0, 1.0),
            self.y.clamp(0.0, 1.0),
            self.z.clamp(0.0, 1.0),
        )
    }
}

impl std::ops::Add for V3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for V3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f32> for V3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl std::ops::Div<f32> for V3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderConfig, render_sdf_scene_with_config};
    use aurex_scene::{Scene, SdfCamera, SdfLighting, SdfModifier, SdfObject, SdfPrimitive, Vec3};

    fn sample_scene() -> Scene {
        Scene {
            sdf: aurex_scene::SdfScene {
                camera: SdfCamera {
                    position: Vec3::new(0.0, 0.0, -4.0),
                    target: Vec3::new(0.0, 0.0, 0.0),
                    fov_degrees: 60.0,
                    aspect_ratio: 16.0 / 9.0,
                },
                lighting: SdfLighting {
                    ambient_light: 0.15,
                    key_lights: vec![aurex_scene::KeyLight {
                        direction: Vec3::new(-0.4, -1.0, -0.5),
                        intensity: 1.1,
                        color: Vec3::new(1.0, 0.95, 0.9),
                    }],
                },
                objects: vec![SdfObject {
                    primitive: SdfPrimitive::Sphere { radius: 1.0 },
                    modifiers: vec![SdfModifier::Translate {
                        offset: Vec3::new(0.0, 0.0, 0.0),
                    }],
                    material: aurex_scene::SdfMaterial {
                        color: Vec3::new(0.7, 0.9, 1.0),
                    },
                }],
            },
        }
    }

    #[test]
    fn renders_expected_dimensions() {
        let frame = render_sdf_scene_with_config(
            &sample_scene(),
            RenderConfig {
                width: 64,
                height: 36,
                ..RenderConfig::default()
            },
        );

        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 36);
        assert_eq!(frame.pixels.len(), 64 * 36);
    }

    #[test]
    fn render_is_deterministic() {
        let config = RenderConfig {
            width: 40,
            height: 24,
            ..RenderConfig::default()
        };
        let a = render_sdf_scene_with_config(&sample_scene(), config);
        let b = render_sdf_scene_with_config(&sample_scene(), config);

        assert_eq!(a.pixels.len(), b.pixels.len());
        let checksum_a: u64 = a.pixels.iter().fold(0u64, |acc, px| {
            acc.wrapping_mul(1099511628211)
                .wrapping_add(px.r as u64 + ((px.g as u64) << 8) + ((px.b as u64) << 16))
        });
        let checksum_b: u64 = b.pixels.iter().fold(0u64, |acc, px| {
            acc.wrapping_mul(1099511628211)
                .wrapping_add(px.r as u64 + ((px.g as u64) << 8) + ((px.b as u64) << 16))
        });
        assert_eq!(checksum_a, checksum_b);
    }
}
