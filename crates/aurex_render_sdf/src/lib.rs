pub mod noise;

use aurex_scene::{
    Scene, SdfMaterial, SdfMaterialType, SdfModifier, SdfPattern, SdfPrimitive, Vec3,
};
use bytemuck::{Pod, Zeroable};
use noise::{NoiseVec3, fbm, value_noise};

#[derive(Debug, Clone, Copy)]
pub struct RenderTime {
    pub seconds: f32,
}

impl Default for RenderTime {
    fn default() -> Self {
        Self { seconds: 0.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub max_steps: u32,
    pub max_distance: f32,
    pub surface_epsilon: f32,
    pub shadow_steps: u32,
    pub time: RenderTime,
    pub output_bloom_prepass: bool,
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
            time: RenderTime::default(),
            output_bloom_prepass: true,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq)]
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
    pub bloom_prepass: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialEvaluation {
    pub color: [f32; 3],
    pub emission: f32,
    pub roughness: f32,
}

pub fn render_sdf_scene(scene: &Scene) -> RenderedFrame {
    render_sdf_scene_with_config(scene, RenderConfig::default())
}

pub fn render_sdf_scene_with_config(scene: &Scene, config: RenderConfig) -> RenderedFrame {
    let mut pixels = Vec::with_capacity((config.width * config.height) as usize);
    let mut bloom_prepass = config
        .output_bloom_prepass
        .then(|| Vec::with_capacity((config.width * config.height) as usize));

    for y in 0..config.height {
        for x in 0..config.width {
            let ray = generate_camera_ray(scene, x, y, config.width, config.height);
            let (color, emission) = shade_ray(scene, ray.origin, ray.direction, config);
            pixels.push(to_rgba8(color));
            if let Some(bloom) = &mut bloom_prepass {
                bloom.push(emission.max(0.0));
            }
        }
    }

    RenderedFrame {
        width: config.width,
        height: config.height,
        pixels,
        bloom_prepass,
    }
}

pub fn wgpu_backend_marker() -> wgpu::Features {
    wgpu::Features::empty()
}

pub fn evaluate_material(
    material: &SdfMaterial,
    position: [f32; 3],
    normal: [f32; 3],
    time: RenderTime,
    scene_seed: u32,
) -> MaterialEvaluation {
    let p = V3::new(position[0], position[1], position[2]);
    let n = V3::new(normal[0], normal[1], normal[2]).normalized();
    let t = time.seconds;
    let seed = scene_seed as i32;
    let param =
        |key: &str, fallback: f32| material.parameters.get(key).copied().unwrap_or(fallback);

    let base = from_vec3(material.base_color);
    let pattern_mix = apply_pattern(material.pattern.clone(), p, t, seed);

    let (raw_color, emission_boost, roughness_mul) = match material.material_type {
        SdfMaterialType::SolidColor => (base, 0.0, 1.0),
        SdfMaterialType::NeonGrid => {
            let scale = param("grid_scale", 7.0);
            let pulse = param("pulse_speed", 3.5);
            let line_w = param("line_width", 0.08).clamp(0.01, 0.35);
            let gx = (p.x * scale + t * pulse).sin().abs();
            let gz = (p.z * scale + t * pulse * 0.7).sin().abs();
            let line = (1.0 - ((gx.min(gz) - line_w).max(0.0) / (1.0 - line_w))).clamp(0.0, 1.0);
            (base * (0.25 + 1.2 * line), 0.8 * line, 0.55)
        }
        SdfMaterialType::Plasma => {
            let speed = param("speed", 1.5);
            let freq = param("frequency", 2.2);
            let wave = (p.x * freq + t * speed).sin()
                + (p.y * (freq * 1.7) - t * speed * 0.6).sin()
                + (p.z * (freq * 1.3) + t * speed * 0.9).sin();
            let plasma = 0.5 + 0.5 * (wave / 3.0);
            let hue = V3::new(plasma, 1.0 - plasma, (plasma * 1.7).sin().abs());
            (base.hadamard(hue) * 1.2, 0.65 * plasma, 0.8)
        }
        SdfMaterialType::FractalMetal => {
            let n1 = fbm(
                NoiseVec3::new(p.x * 1.3 + t * 0.05, p.y * 1.3, p.z * 1.3),
                5,
                2.0,
                0.5,
                seed,
            );
            let fresnel = (1.0 - n.dot(V3::new(0.0, 0.0, -1.0)).abs()).powf(3.0);
            (
                base * (0.8 + 0.35 * n1.abs()) + V3::splat(fresnel * 0.35),
                0.05,
                0.35,
            )
        }
        SdfMaterialType::NoiseSurface => {
            let f = param("noise_frequency", 2.8);
            let n2 = fbm(
                NoiseVec3::new(p.x * f, p.y * f, p.z * f + t * 0.1),
                6,
                2.0,
                0.5,
                seed,
            );
            (base * (0.55 + 0.65 * n2.abs()), 0.0, 0.95)
        }
        SdfMaterialType::Holographic => {
            let view_angle = 0.5 + 0.5 * n.dot(V3::new(0.3, 0.9, 0.2)).clamp(-1.0, 1.0);
            let phase = t * param("shift_speed", 2.4) + p.y * 3.0;
            let rainbow = V3::new(
                (phase).sin() * 0.5 + 0.5,
                (phase + 2.094).sin() * 0.5 + 0.5,
                (phase + 4.188).sin() * 0.5 + 0.5,
            );
            (
                base.hadamard(rainbow) * (0.5 + view_angle),
                0.75 * view_angle,
                0.2,
            )
        }
        SdfMaterialType::Lava => {
            let flow = fbm(
                NoiseVec3::new(p.x * 2.4 + t * 0.35, p.y * 1.7, p.z * 2.4 - t * 0.2),
                5,
                2.0,
                0.55,
                seed,
            );
            let hot = (flow * 1.7).clamp(0.0, 1.0);
            let lava = V3::new(1.0, 0.3 + 0.6 * hot, 0.05 + 0.15 * hot);
            (base.hadamard(lava) * (0.5 + hot), 0.9 * hot, 0.7)
        }
        SdfMaterialType::Wireframe => {
            let scale = param("wire_scale", 8.0);
            let width = param("wire_width", 0.06);
            let wx = ((p.x * scale).fract() - 0.5).abs();
            let wy = ((p.y * scale).fract() - 0.5).abs();
            let wz = ((p.z * scale).fract() - 0.5).abs();
            let edge = ((width - wx.min(wy).min(wz)) / width.max(0.001)).clamp(0.0, 1.0);
            (base * (0.2 + 1.4 * edge), 0.5 * edge, 0.4)
        }
    };

    let col = raw_color * (0.7 + 0.3 * pattern_mix);
    MaterialEvaluation {
        color: [
            col.x.clamp(0.0, 1.0),
            col.y.clamp(0.0, 1.0),
            col.z.clamp(0.0, 1.0),
        ],
        emission: (material.emissive_strength + emission_boost).max(0.0),
        roughness: (material.roughness * roughness_mul).clamp(0.02, 1.0),
    }
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

fn shade_ray(scene: &Scene, origin: V3, direction: V3, config: RenderConfig) -> (V3, f32) {
    if let Some(hit) = march_scene(scene, origin, direction, config) {
        let object = &scene.sdf.objects[hit.object_index];
        let m = evaluate_material(
            &object.material,
            [hit.position.x, hit.position.y, hit.position.z],
            [hit.normal.x, hit.normal.y, hit.normal.z],
            config.time,
            scene.sdf.seed,
        );
        let base = V3::new(m.color[0], m.color[1], m.color[2]);
        let mut color = base * scene.sdf.lighting.ambient_light;

        for key in &scene.sdf.lighting.key_lights {
            let ldir = from_vec3(key.direction).normalized() * -1.0;
            let lambert = hit.normal.dot(ldir).max(0.0);
            let shadow = soft_shadow(scene, hit.position + hit.normal * 0.005, ldir, config);
            let half_vec = (ldir - direction).normalized();
            let specular = hit
                .normal
                .dot(half_vec)
                .max(0.0)
                .powf((1.0 - m.roughness) * 48.0 + 2.0);
            let light_color = from_vec3(key.color) * key.intensity;
            color = color
                + base.hadamard(light_color) * (lambert * shadow)
                + light_color * specular * (1.0 - m.roughness) * 0.25;
        }

        color = color + base * m.emission;
        return (color.clamp01(), m.emission);
    }

    let t = 0.5 * (direction.y + 1.0);
    (
        V3::new(0.05, 0.08, 0.12) * (1.0 - t) + V3::new(0.25, 0.3, 0.4) * t,
        0.0,
    )
}

fn apply_pattern(pattern: SdfPattern, p: V3, time: f32, seed: i32) -> f32 {
    match pattern {
        SdfPattern::None => 1.0,
        SdfPattern::Bands => (0.5 + 0.5 * (p.y * 8.0 + time).sin()).clamp(0.0, 1.0),
        SdfPattern::Rings => (0.5 + 0.5 * (p.length() * 10.0 - time * 1.4).sin()).clamp(0.0, 1.0),
        SdfPattern::Checker => {
            let c = ((p.x * 4.0).floor() as i32 + (p.z * 4.0).floor() as i32).abs();
            if c % 2 == 0 { 1.0 } else { 0.6 }
        }
        SdfPattern::Noise => (0.6
            + 0.4
                * value_noise(
                    NoiseVec3::new(p.x * 3.0 + time * 0.2, p.y * 3.0, p.z * 3.0),
                    seed,
                ))
        .clamp(0.0, 1.0),
    }
}

fn march_scene(scene: &Scene, origin: V3, direction: V3, config: RenderConfig) -> Option<Hit> {
    let mut t = 0.0;
    for _ in 0..config.max_steps {
        let p = origin + direction * t;
        let (distance, object_index) = scene_distance(scene, p, config.time.seconds);
        if distance < config.surface_epsilon {
            let normal =
                estimate_normal(scene, p, config.surface_epsilon * 2.0, config.time.seconds);
            return Some(Hit {
                position: p,
                normal,
                object_index,
            });
        }

        t += distance.max(config.surface_epsilon * 0.5);
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
        let (distance, _) = scene_distance(scene, p, config.time.seconds);
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

fn estimate_normal(scene: &Scene, p: V3, eps: f32, time: f32) -> V3 {
    let ex = V3::new(eps, 0.0, 0.0);
    let ey = V3::new(0.0, eps, 0.0);
    let ez = V3::new(0.0, 0.0, eps);

    let dx = scene_distance(scene, p + ex, time).0 - scene_distance(scene, p - ex, time).0;
    let dy = scene_distance(scene, p + ey, time).0 - scene_distance(scene, p - ey, time).0;
    let dz = scene_distance(scene, p + ez, time).0 - scene_distance(scene, p - ez, time).0;

    V3::new(dx, dy, dz).normalized()
}

fn scene_distance(scene: &Scene, point: V3, time: f32) -> (f32, usize) {
    let mut best = f32::INFINITY;
    let mut index = 0usize;

    for (i, object) in scene.sdf.objects.iter().enumerate() {
        let mut p = point;
        let mut distance_scale = 1.0;

        for modifier in &object.modifiers {
            apply_modifier(&mut p, &mut distance_scale, modifier, scene.sdf.seed, time);
        }

        let distance = eval_primitive(&object.primitive, p, scene.sdf.seed, time) * distance_scale;
        if distance < best {
            best = distance;
            index = i;
        }
    }

    (best, index)
}

fn apply_modifier(
    p: &mut V3,
    distance_scale: &mut f32,
    modifier: &SdfModifier,
    scene_seed: u32,
    time: f32,
) {
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
            let n = value_noise(
                NoiseVec3::new(
                    p.x * *frequency + time * 0.25,
                    p.y * *frequency,
                    p.z * *frequency,
                ),
                scene_seed as i32 + *seed as i32,
            );
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

fn eval_primitive(primitive: &SdfPrimitive, p: V3, scene_seed: u32, time: f32) -> f32 {
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
            let h = (pa.dot(ba) / ba.dot(ba).max(1e-6)).clamp(0.0, 1.0);
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
            let n = fbm(
                NoiseVec3::new(
                    p.x * *frequency + time * 0.15,
                    p.y * *frequency,
                    p.z * *frequency,
                ),
                5,
                2.0,
                0.5,
                scene_seed as i32 + *seed as i32,
            );
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

    0.5 * r.max(1e-6).ln() * r / dr.max(1e-6)
}

fn generate_camera_ray(scene: &Scene, x: u32, y: u32, width: u32, height: u32) -> Ray {
    let cam = &scene.sdf.camera;
    let origin = from_vec3(cam.position);
    let target = from_vec3(cam.target);

    let forward = (target - origin).normalized();
    let world_up = if forward.y.abs() > 0.999 {
        V3::new(0.0, 0.0, 1.0)
    } else {
        V3::new(0.0, 1.0, 0.0)
    };
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
    use super::{RenderConfig, RenderTime, evaluate_material, render_sdf_scene_with_config};
    use aurex_scene::{
        Scene, SdfCamera, SdfLighting, SdfMaterial, SdfMaterialType, SdfModifier, SdfObject,
        SdfPattern, SdfPrimitive, Vec3,
    };

    fn sample_scene() -> Scene {
        Scene {
            sdf: aurex_scene::SdfScene {
                seed: 2027,
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
                    material: SdfMaterial {
                        material_type: SdfMaterialType::NeonGrid,
                        base_color: Vec3::new(0.3, 0.95, 1.0),
                        emissive_strength: 0.7,
                        roughness: 0.2,
                        pattern: SdfPattern::Bands,
                        parameters: std::collections::BTreeMap::new(),
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
        assert_eq!(frame.bloom_prepass.as_ref().map(|v| v.len()), Some(64 * 36));
    }

    #[test]
    fn render_is_deterministic() {
        let config = RenderConfig {
            width: 40,
            height: 24,
            time: RenderTime { seconds: 4.0 },
            ..RenderConfig::default()
        };
        let a = render_sdf_scene_with_config(&sample_scene(), config);
        let b = render_sdf_scene_with_config(&sample_scene(), config);

        assert_eq!(a.pixels, b.pixels);
        assert_eq!(a.bloom_prepass, b.bloom_prepass);
    }

    #[test]
    fn evaluate_material_animated_changes_with_time() {
        let material = SdfMaterial {
            material_type: SdfMaterialType::Plasma,
            ..SdfMaterial::default()
        };
        let a = evaluate_material(
            &material,
            [0.2, 0.3, 0.4],
            [0.0, 1.0, 0.0],
            RenderTime { seconds: 0.0 },
            1,
        );
        let b = evaluate_material(
            &material,
            [0.2, 0.3, 0.4],
            [0.0, 1.0, 0.0],
            RenderTime { seconds: 2.0 },
            1,
        );

        assert_ne!(a.color, b.color);
    }
}
