pub mod cache;
pub mod cone_march;
pub mod diagnostics;
pub mod domain;
pub mod fractals;
pub mod gpu;
pub mod lod;
pub mod noise;
pub mod particles;
pub mod post;
pub mod stages;
pub mod temporal;
pub mod volumetric;

use aurex_audio::{analysis::AudioFeatures, analyze_procedural_audio};
use aurex_scene::{
    AudioSyncHook, RhythmParticleMode, Scene, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode,
    SdfObject, SdfPattern, SdfPrimitive, TimelineValue, Vec3,
    automation::{self, AutomationInput},
    camera::CameraSyncInput,
    director::CameraDirector,
    director_rules::DirectorRuleSet,
    effect_graph,
    fields::{self, FieldSample},
    generators::{self, SceneGenerator},
    harmonics::HarmonicBand,
    patterns::{PatternContext, PatternNetwork, sample_network},
    transition::TransitionContext,
};
use bytemuck::{Pod, Zeroable};
use cache::{EffectGraphEvalCache, FieldSampleCache, PatternSampleCache};
use cone_march::{ConeMarchConfig, cone_step, shadow_cone_factor};
use diagnostics::{CacheStats, FrameDiagnostics};
use domain::{
    Axis, fold_space, kaleidoscope_fold, mirror_fold, repeat_axis as domain_repeat_axis,
    repeat_grid as domain_repeat_grid, repeat_polar as domain_repeat_polar, repeat_sphere,
};
use fractals::kifs_fractal;
use lod::{LodConfig, lod_activation_count, lod_iterations, reset_lod_counters};
use noise::{NoiseVec3, fbm, value_noise};
use particles::{ParticleConfig, particle_overlay};
use post::{PostProcessConfig, process_pixel};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use temporal::{
    TemporalBuffer, TemporalConfig, TemporalEffect, apply_temporal_feedback, to_rgba8_pixels,
};
use volumetric::{VolumetricConfig, apply_volumetric};

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
pub struct QualitySettings {
    pub pattern_quality: f32,
    pub field_quality: f32,
    pub volumetric_quality: f32,
    pub raymarch_quality: f32,
    pub transition_quality: f32,
    pub post_quality: f32,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            pattern_quality: 1.0,
            field_quality: 1.0,
            volumetric_quality: 1.0,
            raymarch_quality: 1.0,
            transition_quality: 1.0,
            post_quality: 1.0,
        }
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
    pub shadow_softness: f32,
    pub ao_samples: u32,
    pub ao_strength: f32,
    pub enable_soft_shadows: bool,
    pub enable_ambient_occlusion: bool,
    pub enable_fog: bool,
    pub enable_scattering: bool,
    pub time: RenderTime,
    pub output_bloom_prepass: bool,
    pub adaptive_raymarch: bool,
    pub min_step_scale: f32,
    pub max_step_scale: f32,
    pub far_field_boost: f32,
    pub cone_step_multiplier: f32,
    pub cone_shadow_factor: f32,
    pub surface_thickness_estimation: f32,
    pub adaptive_step_scale: f32,
    pub distance_bias: f32,
    pub fractal_iteration_limit: u32,
    pub fractal_lod_scale: f32,
    pub detail_falloff: f32,
    pub volumetric_density: f32,
    pub scatter_strength: f32,
    pub light_beam_strength: f32,
    pub quality: QualitySettings,
    pub output_diagnostics: bool,
    pub post: PostProcessConfig,
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
            shadow_softness: 12.0,
            ao_samples: 6,
            ao_strength: 0.85,
            enable_soft_shadows: true,
            enable_ambient_occlusion: true,
            enable_fog: true,
            enable_scattering: true,
            time: RenderTime::default(),
            output_bloom_prepass: true,
            adaptive_raymarch: true,
            min_step_scale: 0.4,
            max_step_scale: 1.8,
            far_field_boost: 1.5,
            cone_step_multiplier: 1.25,
            cone_shadow_factor: 0.8,
            surface_thickness_estimation: 0.02,
            adaptive_step_scale: 1.0,
            distance_bias: 1.0,
            fractal_iteration_limit: 14,
            fractal_lod_scale: 0.9,
            detail_falloff: 0.7,
            volumetric_density: 0.08,
            scatter_strength: 0.45,
            light_beam_strength: 0.5,
            quality: QualitySettings::default(),
            output_diagnostics: false,
            post: PostProcessConfig::default(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPipelineStage {
    Geometry,
    MaterialShading,
    PatternSampling,
    Particles,
    PostProcessing,
    TemporalFeedback,
}

pub const RENDER_PIPELINE_STAGES: [RenderPipelineStage; 6] = [
    RenderPipelineStage::Geometry,
    RenderPipelineStage::MaterialShading,
    RenderPipelineStage::PatternSampling,
    RenderPipelineStage::Particles,
    RenderPipelineStage::PostProcessing,
    RenderPipelineStage::TemporalFeedback,
];

#[derive(Debug, Clone, Copy)]
pub struct MaterialEvaluation {
    pub color: [f32; 3],
    pub emission: f32,
    pub roughness: f32,
}

#[derive(Debug, Clone, Default)]
pub struct RendererState {
    pub effect_graph_cache: EffectGraphEvalCache,
    pub(crate) temporal_buffer: TemporalBuffer,
}

fn shared_renderer_state() -> &'static Mutex<RendererState> {
    static RENDERER_STATE: OnceLock<Mutex<RendererState>> = OnceLock::new();
    RENDERER_STATE.get_or_init(|| Mutex::new(RendererState::default()))
}

pub fn render_sdf_scene(scene: &Scene, time: RenderTime) -> RenderedFrame {
    render_sdf_scene_with_config(
        scene,
        RenderConfig {
            time,
            ..RenderConfig::default()
        },
    )
}

pub fn render_sdf_scene_with_config(scene: &Scene, config: RenderConfig) -> RenderedFrame {
    let (frame, _) = render_sdf_scene_with_diagnostics(scene, config);
    frame
}

pub fn render_sdf_scene_with_diagnostics(
    scene: &Scene,
    config: RenderConfig,
) -> (RenderedFrame, FrameDiagnostics) {
    let mut state = shared_renderer_state()
        .lock()
        .expect("renderer state mutex should not be poisoned");
    render_sdf_scene_with_state_and_diagnostics(scene, config, &mut state)
}

pub fn render_sdf_scene_with_state_and_diagnostics(
    scene: &Scene,
    config: RenderConfig,
    state: &mut RendererState,
) -> (RenderedFrame, FrameDiagnostics) {
    let frame_start = Instant::now();
    let mut diagnostics = FrameDiagnostics::default();
    reset_lod_counters();
    diagnostics.stages.extend([
        "ScenePreprocess",
        "EffectGraphEvaluation",
        "GeometrySdf",
        "MaterialPattern",
        "LightingAtmosphere",
        "Particles",
        "PostProcessing",
        "TemporalFeedback",
    ]);

    let animated_scene = scene_at_time(
        scene,
        config.time,
        Some(&mut diagnostics),
        &mut state.effect_graph_cache,
    );
    let mut post_colors = Vec::with_capacity((config.width * config.height) as usize);
    let mut depth_buffer = Vec::with_capacity((config.width * config.height) as usize);
    let mut bloom_prepass = config
        .output_bloom_prepass
        .then(|| Vec::with_capacity((config.width * config.height) as usize));

    let mut pattern_cache = PatternSampleCache::default();
    let mut field_cache = FieldSampleCache::default();

    let mut geometry_ns = 0_u128;
    let mut cone_step_reduction_acc = 0.0_f64;
    let mut material_pattern_ns = 0_u128;
    let mut lighting_atmosphere_ns = 0_u128;
    let mut particles_ns = 0_u128;
    let mut post_ns = 0_u128;

    for y in 0..config.height {
        for x in 0..config.width {
            let ray = generate_camera_ray(&animated_scene, x, y, config.width, config.height);
            let (
                color,
                emission,
                march_steps,
                sample_position,
                sample_depth,
                step_reduction,
                stage_times,
            ) = shade_ray(&animated_scene, ray.origin, ray.direction, config);

            geometry_ns += stage_times.geometry_ns;
            material_pattern_ns += stage_times.material_pattern_ns;
            lighting_atmosphere_ns += stage_times.lighting_atmosphere_ns;
            cone_step_reduction_acc += step_reduction as f64;

            let key = cache::SampleKey::from_world(
                sample_position,
                config.time.seconds,
                animated_scene.sdf.seed,
            );
            if pattern_cache.get(key).is_none() {
                pattern_cache.insert(key, emission);
            }
            if field_cache.get(key).is_none() {
                field_cache.insert(
                    key,
                    [color.x, color.y, color.z, emission, march_steps as f32],
                );
            }

            let uv = (
                (x as f32 + 0.5) / config.width as f32,
                (y as f32 + 0.5) / config.height as f32,
            );
            let particles_start = Instant::now();
            let audio = analyze_procedural_audio_opt(&animated_scene, config.time.seconds);
            let particle = particle_overlay(
                uv,
                config.time.seconds,
                animated_scene.sdf.seed,
                audio.0,
                audio.1,
                audio.2,
                ParticleConfig {
                    density: (0.02 + audio.0 * 0.25).clamp(0.0, 1.0),
                    intensity: 0.6,
                },
            );
            particles_ns += particles_start.elapsed().as_nanos();

            let post_start = Instant::now();
            let post_color = process_pixel(
                (color + particle).clamp01(),
                emission,
                uv,
                animated_scene.sdf.seed,
                PostProcessConfig {
                    vignette: config.post.vignette * config.quality.post_quality,
                    film_grain: config.post.film_grain * config.quality.post_quality,
                    ..config.post
                },
            );
            post_ns += post_start.elapsed().as_nanos();
            post_colors.push(post_color);
            depth_buffer.push(sample_depth);
            if let Some(bloom) = &mut bloom_prepass {
                bloom.push(emission.max(0.0));
            }
            diagnostics.stats.rays_traced += 1;
            diagnostics.stats.raymarch_steps_total += march_steps as u64;
        }
    }

    diagnostics.add_stage_duration("GeometrySdf", geometry_ns as f64 / 1_000_000.0);
    diagnostics.add_stage_duration("MaterialPattern", material_pattern_ns as f64 / 1_000_000.0);
    diagnostics.add_stage_duration(
        "LightingAtmosphere",
        lighting_atmosphere_ns as f64 / 1_000_000.0,
    );
    diagnostics.add_stage_duration("Particles", particles_ns as f64 / 1_000_000.0);
    diagnostics.add_stage_duration("PostProcessing", post_ns as f64 / 1_000_000.0);

    let temporal_start = Instant::now();
    let temporal_effects: Vec<TemporalEffect> = animated_scene
        .sdf
        .temporal_effects
        .iter()
        .cloned()
        .map(TemporalEffect::from)
        .collect();
    let (beat_phase, current_measure, harmonic_energy, dominant_frequency) =
        if let Some(audio) = &animated_scene.sdf.audio {
            let af = analyze_procedural_audio(audio, config.time.seconds);
            let harmonic =
                (af.harmonic_ratios[0] + af.harmonic_ratios[1] + af.harmonic_ratios[2]) / 3.0;
            (
                af.beat_phase,
                af.current_measure,
                harmonic,
                af.dominant_frequency,
            )
        } else {
            (0.0, 0_u32, 0.0, 220.0)
        };
    let final_colors = apply_temporal_feedback(
        &post_colors,
        &depth_buffer,
        config.width,
        config.height,
        &mut state.temporal_buffer,
        TemporalConfig::default(),
        &temporal_effects,
        beat_phase,
        current_measure,
        harmonic_energy,
        dominant_frequency,
    );
    diagnostics.add_stage_duration(
        "TemporalFeedback",
        temporal_start.elapsed().as_secs_f64() * 1000.0,
    );
    let pixels = to_rgba8_pixels(&final_colors);

    let effect_graph_evals = diagnostics.stats.cache.effect_graph_evals;
    diagnostics.stats.temporal_buffer_size = (state.temporal_buffer.history.len() as u64)
        * (config.width as u64)
        * (config.height as u64);
    diagnostics.stats.temporal_history_depth = state.temporal_buffer.history.len() as u32;

    diagnostics.stats.average_step_reduction = if diagnostics.stats.rays_traced > 0 {
        cone_step_reduction_acc / diagnostics.stats.rays_traced as f64
    } else {
        0.0
    };

    diagnostics.stats.cache = CacheStats {
        pattern_hits: pattern_cache.hits,
        pattern_misses: pattern_cache.misses,
        field_hits: field_cache.hits,
        field_misses: field_cache.misses,
        effect_graph_evals,
    };

    diagnostics.total_frame_time_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
    diagnostics.stats.stage_time_ms_total = diagnostics.total_frame_time_ms;
    diagnostics.stats.lod_activation_count = lod_activation_count();
    diagnostics.finalize_stage_percentages();

    (
        RenderedFrame {
            width: config.width,
            height: config.height,
            pixels,
            bloom_prepass,
        },
        diagnostics,
    )
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

    let rhythm_t = param("rhythm_time", t);
    let base = from_vec3(material.base_color);
    let pattern_mix = apply_pattern(material.pattern.clone(), p, rhythm_t, seed);

    let (raw_color, emission_boost, roughness_mul) = match material.material_type {
        SdfMaterialType::SolidColor => (base, 0.0, 1.0),
        SdfMaterialType::NeonGrid => {
            let scale = param("grid_scale", 7.0);
            let pulse = param("pulse_speed", 3.5);
            let line_w = param("line_width", 0.08).clamp(0.01, 0.35);
            let gx = (p.x * scale + rhythm_t * pulse).sin().abs();
            let gz = (p.z * scale + rhythm_t * pulse * 0.7).sin().abs();
            let line = (1.0 - ((gx.min(gz) - line_w).max(0.0) / (1.0 - line_w))).clamp(0.0, 1.0);
            (base * (0.25 + 1.2 * line), 0.8 * line, 0.55)
        }
        SdfMaterialType::Plasma => {
            let speed = param("speed", 1.5);
            let freq = param("frequency", 2.2);
            let wave = (p.x * freq + rhythm_t * speed).sin()
                + (p.y * (freq * 1.7) - rhythm_t * speed * 0.6).sin()
                + (p.z * (freq * 1.3) + rhythm_t * speed * 0.9).sin();
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
        SdfMaterialType::SpectralReactive => {
            let low = param("harmonic_low", 0.0);
            let mid = param("harmonic_mid", 0.0);
            let high = param("harmonic_high", 0.0);
            let harmonic_energy = param("harmonic_energy", 0.0);
            let dominant = param("dominant_frequency", 220.0).max(1.0);
            let beat_phase = param("rhythm_beat_phase", 0.5);
            let pitch_phase = (rhythm_t * dominant * 0.005).sin().abs();
            let neon_flicker = (high * 1.7
                + (rhythm_t * (16.0 + dominant * 0.01)).sin().abs()
                + (1.0 - beat_phase) * 0.25)
                .clamp(0.0, 1.8);
            let bass_pulse = (low * 1.2 + (rhythm_t * 2.0).sin().abs() * low * 0.6).clamp(0.0, 1.8);
            let surface_distort = fbm(
                NoiseVec3::new(
                    p.x * (2.0 + high),
                    p.y * (2.0 + mid),
                    p.z * (2.0 + high) + rhythm_t * 0.7,
                ),
                4,
                2.0,
                0.5,
                seed,
            )
            .abs();
            let hue = V3::new(
                (0.2 + 0.8 * high + pitch_phase * 0.3).clamp(0.0, 1.0),
                (0.2 + 0.8 * mid + surface_distort * 0.2).clamp(0.0, 1.0),
                (0.3 + 0.7 * low + (1.0 - pitch_phase) * 0.2).clamp(0.0, 1.0),
            );
            (
                base.hadamard(hue) * (0.7 + harmonic_energy * 0.4 + surface_distort * 0.2),
                (0.2 + neon_flicker * 0.5 + bass_pulse * 0.35).clamp(0.0, 2.0),
                (0.25 + 0.5 * (1.0 - surface_distort)).clamp(0.05, 0.95),
            )
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

    let mut pattern_color = V3::new(1.0, 1.0, 1.0);
    let mut pattern_emission = 0.0;
    let mut pattern_rough_mul = 1.0;

    if let Some(net) = &material.pattern_network {
        let ctx = PatternContext {
            low_freq_energy: param("pattern_low", 0.0),
            mid_freq_energy: param("pattern_mid", 0.0),
            high_freq_energy: param("pattern_high", 0.0),
            dominant_frequency: param("pattern_dominant", 0.0),
            current_beat: param("pattern_beat", 0.0).max(0.0) as u32,
            current_measure: param("pattern_measure", 0.0).max(0.0) as u32,
            current_phrase: param("pattern_phrase", 0.0).max(0.0) as u32,
            beat_phase: param("pattern_beat_phase", 0.0).clamp(0.0, 1.0),
            tempo: param("pattern_tempo", 120.0),
        };
        let ps = sample_network(
            net,
            Vec3::new(p.x, p.y, p.z),
            Vec3::new(p.x, p.y, p.z),
            Vec3::new(p.x.fract(), p.y.fract(), 0.0),
            rhythm_t,
            scene_seed,
            ctx,
        );
        pattern_color = V3::new(
            (0.6 + ps.value * 0.6).clamp(0.0, 1.2),
            (0.7 + ps.value * 0.5).clamp(0.0, 1.2),
            (0.8 + ps.value * 0.4).clamp(0.0, 1.2),
        );
        pattern_emission = ps.value * 0.35 + ps.distortion.abs() * 0.18;
        pattern_rough_mul = (1.0 - ps.value * 0.3).clamp(0.2, 1.0);
    }

    let col = raw_color.hadamard(pattern_color) * (0.7 + 0.3 * pattern_mix);
    MaterialEvaluation {
        color: [
            col.x.clamp(0.0, 1.0),
            col.y.clamp(0.0, 1.0),
            col.z.clamp(0.0, 1.0),
        ],
        emission: (material.emissive_strength + emission_boost + pattern_emission).max(0.0),
        roughness: (material.roughness * roughness_mul * pattern_rough_mul).clamp(0.02, 1.0),
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: V3,
    direction: V3,
}

#[derive(Clone)]
struct Hit {
    position: V3,
    normal: V3,
    material: SdfMaterial,
    distance_traveled: f32,
    distance_glow: f32,
    march_steps: u32,
    step_reduction_estimate: f32,
}

#[derive(Clone, Copy, Default)]
struct RayStageDurations {
    geometry_ns: u128,
    material_pattern_ns: u128,
    lighting_atmosphere_ns: u128,
}

#[derive(Clone)]
struct NodeEval {
    distance: f32,
    material: SdfMaterial,
}

fn scene_at_time(
    scene: &Scene,
    time: RenderTime,
    mut diagnostics: Option<&mut FrameDiagnostics>,
    effect_graph_cache: &mut EffectGraphEvalCache,
) -> Scene {
    let preprocess_start = Instant::now();
    let mut animated = scene.clone();
    let mut t = time.seconds;

    let af_preview = if let Some(cfg) = &animated.sdf.audio {
        analyze_procedural_audio(cfg, t)
    } else {
        AudioFeatures {
            kick_energy: 0.0,
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            low_freq_energy: 0.0,
            mid_freq_energy: 0.0,
            high_freq_energy: 0.0,
            dominant_frequency: 0.0,
            harmonic_ratios: [0.0, 0.0, 0.0],
            current_beat: 0,
            current_measure: 0,
            current_phrase: 0,
            beat_phase: 0.0,
            spectral_centroid: 0.0,
            tempo: 120.0,
        }
    };

    if let Some(demo_sequence) = animated.sdf.demo_sequence.clone() {
        let rule_set = DirectorRuleSet::default();
        if let Some(blended) = demo_sequence.blend_scene_at_time(
            &animated,
            t,
            TransitionContext {
                seed: animated.sdf.seed,
                time_seconds: t,
                beat: af_preview.current_beat as f32 + af_preview.beat_phase,
                measure: af_preview.current_measure as f32,
                phrase: af_preview.current_phrase as f32,
                tempo: af_preview.tempo,
                low_freq_energy: af_preview.low_freq_energy,
                mid_freq_energy: af_preview.mid_freq_energy,
                high_freq_energy: af_preview.high_freq_energy,
                dominant_frequency: af_preview.dominant_frequency,
            },
            &rule_set,
        ) {
            animated = blended;
        } else {
            demo_sequence.apply_at_time(&mut animated, t);
        }
    }

    if let Some(graph) = animated.sdf.effect_graph.clone() {
        let eg_start = Instant::now();
        let seed = animated.sdf.seed;
        if !effect_graph_cache.should_reuse(seed, t) {
            graph.evaluate_scene(
                &mut animated,
                effect_graph::EffectContext {
                    time_seconds: t,
                    seed,
                    bass_energy: af_preview.bass_energy,
                    mid_energy: af_preview.mid_energy,
                    high_energy: af_preview.high_energy,
                    tempo: af_preview.tempo,
                    beat_phase: af_preview.beat_phase,
                },
            );
            effect_graph_cache.mark_evaluated(seed, t);
            if let Some(d) = diagnostics.as_deref_mut() {
                d.stats.cache.effect_graph_evals += 1;
            }
        }
        if let Some(d) = diagnostics.as_deref_mut() {
            d.add_stage_duration(
                "EffectGraphEvaluation",
                eg_start.elapsed().as_secs_f64() * 1000.0,
            );
        }
    }

    if let Some(timeline) = animated.sdf.timeline.clone() {
        t = timeline.normalized_time(time.seconds);

        if let Some(path) = &timeline.camera_path {
            animated.sdf.camera = path.sample(t, &animated.sdf.camera, timeline.duration);
        }

        if let Some(rig) = &timeline.cinematic_camera {
            let sync = CameraSyncInput {
                beat: timeline.event_strength(AudioSyncHook::Kick, t),
                phrase: timeline.event_strength(AudioSyncHook::Snare, t),
                tempo: 120.0,
            };
            animated.sdf.camera = rig.sample(&animated.sdf.camera, t, timeline.duration, sync);
        } else if let Some(sequence) = &timeline.shot_sequence {
            let director = CameraDirector::default();
            if let Some(shot) = director.shot_for_time(sequence, t) {
                let sync = CameraSyncInput {
                    beat: timeline.event_strength(AudioSyncHook::Kick, t),
                    phrase: timeline.event_strength(AudioSyncHook::Bass, t),
                    tempo: 120.0,
                };
                animated.sdf.camera = shot.camera.sample(
                    &animated.sdf.camera,
                    t - shot.start,
                    (shot.end - shot.start).max(1e-3),
                    sync,
                );
            }
        }

        if let Some(TimelineValue::Vec3 { value }) =
            timeline.sample_keyframe_value("camera.position", t)
        {
            animated.sdf.camera.position = value;
        }
        if let Some(TimelineValue::Vec3 { value }) =
            timeline.sample_keyframe_value("camera.target", t)
        {
            animated.sdf.camera.target = value;
        }

        if let Some(TimelineValue::Float { value }) =
            timeline.sample_keyframe_value("light.intensity", t)
        {
            for l in &mut animated.sdf.lighting.key_lights {
                l.intensity = value;
            }
        }

        if let Some(TimelineValue::Float { value }) =
            timeline.sample_keyframe_value("material.emissive_strength", t)
        {
            apply_to_all_materials(
                &mut animated.sdf.root,
                &mut animated.sdf.objects,
                &mut |m| {
                    m.emissive_strength = value;
                },
            );
        }

        if let Some(TimelineValue::Float { value }) =
            timeline.sample_keyframe_value("tunnel.radius", t)
        {
            apply_to_primitives(
                &mut animated.sdf.root,
                &mut animated.sdf.objects,
                &mut |p| {
                    if let SdfPrimitive::Torus { major_radius, .. } = p {
                        *major_radius = value;
                    }
                },
            );
        }

        if let Some(generator) = &mut animated.sdf.generator {
            apply_generator_keyframes(generator, &timeline, t);
        }

        apply_field_keyframes(&mut animated, &timeline, t);

        let kick = timeline.event_strength(AudioSyncHook::Kick, t);
        let snare = timeline.event_strength(AudioSyncHook::Snare, t);
        let bass = timeline.event_strength(AudioSyncHook::Bass, t);

        if kick > 0.0 {
            animated.sdf.camera.position.y += 0.08 * kick;
        }
        if snare > 0.0 {
            apply_to_primitives(
                &mut animated.sdf.root,
                &mut animated.sdf.objects,
                &mut |p| {
                    if let SdfPrimitive::Torus { minor_radius, .. } = p {
                        *minor_radius *= 1.0 + 0.25 * snare;
                    }
                },
            );
        }
        if bass > 0.0 {
            apply_to_primitives(
                &mut animated.sdf.root,
                &mut animated.sdf.objects,
                &mut |p| {
                    if let SdfPrimitive::Mandelbulb { bailout, .. } = p {
                        *bailout *= 1.0 + 0.4 * bass;
                    }
                },
            );
        }
    }

    let af = apply_audio_to_scene(&mut animated, t);
    apply_harmonics_to_scene(&mut animated, af, t);

    if !animated.sdf.automation_tracks.is_empty() {
        let bindings = animated.sdf.automation_tracks.clone();
        let seed = animated.sdf.seed;
        automation::apply_bindings(
            &mut animated,
            &bindings,
            AutomationInput {
                time_seconds: t,
                beat: af.current_beat as f32 + af.beat_phase,
                measure: af.current_measure as f32,
                phrase: af.current_phrase as f32,
                tempo: af.tempo,
                bass: af.bass_energy,
                mid: af.mid_energy,
                high: af.high_energy,
                dominant_frequency: af.dominant_frequency,
            },
            seed,
        );
    }

    let rhythm_t = apply_rhythm_space_to_scene(&mut animated, af, t);
    apply_scene_pattern_networks(&mut animated, af, rhythm_t);

    if af.kick_energy > 0.0 || af.bass_energy > 0.0 || af.high_energy > 0.0 {
        animated
            .sdf
            .fields
            .push(aurex_scene::fields::SceneField::Audio(
                aurex_scene::fields::AudioField {
                    band: aurex_scene::fields::AudioBand::Kick,
                    strength: af.kick_energy.max(af.bass_energy),
                    radius: 30.0,
                },
            ));
    }

    if let Some(generator) = &animated.sdf.generator {
        animated.sdf.root = generators::expand_generator(
            generator,
            animated.sdf.seed,
            rhythm_t,
            &animated.sdf.fields,
        );
    }

    if let Some(d) = diagnostics.as_deref_mut() {
        d.add_stage_duration(
            "ScenePreprocess",
            preprocess_start.elapsed().as_secs_f64() * 1000.0,
        );
    }

    animated
}

fn apply_generator_keyframes(
    generator: &mut SceneGenerator,
    timeline: &aurex_scene::SceneTimeline,
    t: f32,
) {
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("generator.tunnel.radius", t)
    {
        if let SceneGenerator::Tunnel(g) = generator {
            g.radius = value;
        }
    }
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("generator.tunnel.twist", t)
    {
        if let SceneGenerator::Tunnel(g) = generator {
            g.twist = value;
        }
    }
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("generator.temple.fractal_scale", t)
    {
        if let SceneGenerator::FractalTemple(g) = generator {
            g.fractal_scale = value;
        }
    }
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("generator.circuit.component_density", t)
    {
        if let SceneGenerator::CircuitBoard(g) = generator {
            g.component_density = value;
        }
    }
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("generator.galaxy.rotation_speed", t)
    {
        if let SceneGenerator::ParticleGalaxy(g) = generator {
            g.rotation_speed = value;
        }
    }
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("generator.galaxy.radius", t)
    {
        if let SceneGenerator::ParticleGalaxy(g) = generator {
            g.radius = value;
        }
    }
}

fn apply_field_keyframes(scene: &mut Scene, timeline: &aurex_scene::SceneTimeline, t: f32) {
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("field.noise.strength", t)
    {
        for f in &mut scene.sdf.fields {
            if let aurex_scene::fields::SceneField::Noise(n) = f {
                n.strength = value;
            }
        }
    }
    if let Some(TimelineValue::Float { value }) =
        timeline.sample_keyframe_value("field.pulse.frequency", t)
    {
        for f in &mut scene.sdf.fields {
            if let aurex_scene::fields::SceneField::Pulse(p) = f {
                p.frequency = value;
            }
        }
    }
    if let Some(TimelineValue::Vec3 { value }) =
        timeline.sample_keyframe_value("field.flow.direction", t)
    {
        for f in &mut scene.sdf.fields {
            if let aurex_scene::fields::SceneField::Flow(fl) = f {
                fl.direction = value;
            }
        }
    }
}

fn apply_audio_to_scene(scene: &mut Scene, t: f32) -> AudioFeatures {
    let features = if let Some(cfg) = &scene.sdf.audio {
        analyze_procedural_audio(cfg, t)
    } else {
        AudioFeatures {
            kick_energy: 0.0,
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            low_freq_energy: 0.0,
            mid_freq_energy: 0.0,
            high_freq_energy: 0.0,
            dominant_frequency: 0.0,
            harmonic_ratios: [0.0, 0.0, 0.0],
            current_beat: 0,
            current_measure: 0,
            current_phrase: 0,
            beat_phase: 0.0,
            spectral_centroid: 0.0,
            tempo: 120.0,
        }
    };

    if !scene.sdf.lighting.key_lights.is_empty() {
        let boost = 1.0 + features.kick_energy * 0.15 + features.high_energy * 0.06;
        for l in &mut scene.sdf.lighting.key_lights {
            l.intensity *= boost;
        }
    }

    scene.sdf.lighting.fog_density += features.bass_energy * 0.01;
    scene.sdf.camera.position.y += features.kick_energy * 0.03;
    scene.sdf.camera.target.y += features.mid_energy * 0.02;

    features
}

fn harmonic_energy_for_band(features: AudioFeatures, band: HarmonicBand) -> f32 {
    match band {
        HarmonicBand::Bass => features.low_freq_energy.max(features.bass_energy),
        HarmonicBand::Mid => features.mid_freq_energy.max(features.mid_energy),
        HarmonicBand::High => features.high_freq_energy.max(features.high_energy),
        HarmonicBand::Melody => {
            (features.mid_freq_energy * 0.8 + features.high_freq_energy * 0.2)
                * (1.0 + features.harmonic_ratios[0] * 0.15)
        }
        HarmonicBand::Chords => {
            (features.low_freq_energy + features.mid_freq_energy + features.high_freq_energy)
                * (0.2 + features.harmonic_ratios[2] * 0.08)
        }
        HarmonicBand::Full => {
            features.low_freq_energy + features.mid_freq_energy + features.high_freq_energy
        }
    }
}

fn apply_harmonics_to_scene(scene: &mut Scene, features: AudioFeatures, t: f32) {
    let Some(cfg) = scene.sdf.harmonics.clone() else {
        return;
    };

    if let Some(geom) = cfg.geometry {
        let e = harmonic_energy_for_band(features, geom.band) * geom.strength;
        if let Some(generator) = &mut scene.sdf.generator {
            match generator {
                SceneGenerator::Tunnel(g) => {
                    g.radius *= 1.0 + e * 0.08;
                    g.twist *= 1.0 + e * 0.12;
                }
                SceneGenerator::FractalTemple(g) => {
                    g.fractal_scale *= 1.0 + e * 0.15;
                    g.pillar_height *= 1.0 + e * 0.06;
                }
                SceneGenerator::CircuitBoard(g) => {
                    g.height_variation *= 1.0 + e * 0.2;
                    g.trace_width *= 1.0 + e * 0.08;
                }
                SceneGenerator::ParticleGalaxy(g) => {
                    g.radius *= 1.0 + e * 0.1;
                    g.noise_spread *= 1.0 + e * 0.15;
                }
                SceneGenerator::HarmonicParticleField(g) => {
                    g.radius *= 1.0 + e * 0.16;
                    g.thickness *= 1.0 + e * 0.1;
                }
                _ => {}
            }
        }

        if !cfg.fields.is_empty() {
            for hf in cfg.fields {
                let h = harmonic_energy_for_band(features, hf.band) * hf.strength;
                let sampled = hf.sample(scene.sdf.camera.position, h);
                scene.sdf.camera.position.y += sampled * 0.02 * (1.0 + (t * 0.5).sin().abs());
            }
        }
    }

    if let Some(mat) = cfg.materials {
        let low = features.low_freq_energy * mat.strength;
        let mid = features.mid_freq_energy * mat.strength;
        let high = features.high_freq_energy * mat.strength;
        let band_energy = harmonic_energy_for_band(features, mat.band);
        apply_to_all_materials(&mut scene.sdf.root, &mut scene.sdf.objects, &mut |m| {
            m.parameters.insert("harmonic_low".into(), low);
            m.parameters.insert("harmonic_mid".into(), mid);
            m.parameters.insert("harmonic_high".into(), high);
            m.parameters.insert("harmonic_energy".into(), band_energy);
            m.parameters
                .insert("dominant_frequency".into(), features.dominant_frequency);
        });
    }

    if let Some(p) = cfg.particles {
        let e = harmonic_energy_for_band(features, p.band) * p.strength;
        scene
            .sdf
            .fields
            .push(aurex_scene::fields::SceneField::Audio(
                aurex_scene::fields::AudioField {
                    band: aurex_scene::fields::AudioBand::High,
                    strength: e.max(0.01),
                    radius: 20.0 + e * 50.0,
                },
            ));
    }
}

fn apply_rhythm_space_to_scene(scene: &mut Scene, features: AudioFeatures, t: f32) -> f32 {
    let Some(cfg) = scene.sdf.rhythm.clone() else {
        return t;
    };

    let mut rhythm_time = t;
    if let Some(warp) = cfg.time_warp {
        let mut scaled = t * warp.time_scale.max(0.01);
        scaled -= warp.time_delay.max(0.0);
        if warp.time_reverse {
            scaled = -scaled;
        }
        let echo = (t - warp.time_delay.max(0.0)).max(0.0) * warp.time_echo.clamp(0.0, 1.0);
        rhythm_time = scaled + echo;
    }

    if cfg.beat_geometry {
        let beat_pulse = (1.0 - features.beat_phase).powf(2.0);
        if let Some(generator) = &mut scene.sdf.generator {
            match generator {
                SceneGenerator::Tunnel(g) => {
                    g.radius *= 1.0 + beat_pulse * 0.08;
                }
                SceneGenerator::FractalTemple(g) => {
                    let measure_gate = if features.current_beat % 4 == 0 {
                        1.0
                    } else {
                        0.0
                    };
                    let phrase_gate = if features.current_beat % 16 == 0 {
                        1.0
                    } else {
                        0.0
                    };
                    g.pillar_height *= 1.0 + measure_gate * 0.08 + phrase_gate * 0.18;
                    g.fractal_scale *= 1.0 + phrase_gate * 0.1;
                }
                SceneGenerator::CircuitBoard(g) => {
                    let snare_like = (features.mid_energy + features.high_energy) * 0.5;
                    g.trace_width *= 1.0 + snare_like * 0.06;
                    g.height_variation *= 1.0 + snare_like * 0.1;
                }
                SceneGenerator::ParticleGalaxy(g) => {
                    g.rotation_speed *= 1.0 + beat_pulse * 0.08;
                }
                SceneGenerator::HarmonicParticleField(g) => {
                    g.thickness *= 1.0 + beat_pulse * 0.1;
                }
                _ => {}
            }
        }
    }

    if cfg.echo_effect {
        let beat_pulse = (1.0 - features.beat_phase).powf(2.0);
        if beat_pulse > 0.72 {
            let echoed = scene.sdf.root.clone();
            scene.sdf.root = SdfNode::Group {
                children: vec![
                    scene.sdf.root.clone(),
                    SdfNode::Transform {
                        modifiers: vec![SdfModifier::Translate {
                            offset: Vec3::new(0.0, 0.0, 0.2 + beat_pulse * 0.8),
                        }],
                        bounds_radius: Some(128.0),
                        child: Box::new(echoed),
                    },
                ],
            };
        }
    }

    if let Some(mode) = cfg.particle_mode {
        let (band, strength) = match mode {
            RhythmParticleMode::Bass => (
                aurex_scene::fields::AudioBand::Bass,
                (1.0 - features.beat_phase) * 1.2,
            ),
            RhythmParticleMode::Snare => (
                aurex_scene::fields::AudioBand::Mid,
                (features.current_beat % 2) as f32 * 0.8,
            ),
            RhythmParticleMode::Melody => (
                aurex_scene::fields::AudioBand::High,
                features.high_freq_energy * 0.9,
            ),
        };
        scene
            .sdf
            .fields
            .push(aurex_scene::fields::SceneField::Audio(
                aurex_scene::fields::AudioField {
                    band,
                    strength: strength.max(0.01),
                    radius: 16.0 + features.current_measure as f32 * 0.25,
                },
            ));
        scene
            .sdf
            .fields
            .push(aurex_scene::fields::SceneField::Rhythm(
                aurex_scene::fields::RhythmField {
                    beat_strength: (1.0 - features.beat_phase).max(0.0),
                    measure_strength: if features.current_beat % 4 == 0 {
                        1.0
                    } else {
                        0.35
                    },
                    phrase_strength: if features.current_beat % 16 == 0 {
                        1.0
                    } else {
                        0.2
                    },
                    tempo: features.tempo.max(1.0),
                },
            ));
    }

    apply_to_all_materials(&mut scene.sdf.root, &mut scene.sdf.objects, &mut |m| {
        m.parameters
            .insert("rhythm_beat_phase".into(), features.beat_phase);
        m.parameters
            .insert("rhythm_measure".into(), features.current_measure as f32);
        m.parameters
            .insert("rhythm_phrase".into(), features.current_phrase as f32);
        m.parameters.insert("rhythm_time".into(), rhythm_time);
    });

    scene.sdf.camera.position.y += (1.0 - features.beat_phase) * 0.04;
    rhythm_time
}

fn audio_pattern_context(features: AudioFeatures) -> PatternContext {
    PatternContext {
        low_freq_energy: features.low_freq_energy,
        mid_freq_energy: features.mid_freq_energy,
        high_freq_energy: features.high_freq_energy,
        dominant_frequency: features.dominant_frequency,
        current_beat: features.current_beat,
        current_measure: features.current_measure,
        current_phrase: features.current_phrase,
        beat_phase: features.beat_phase,
        tempo: features.tempo,
    }
}

fn apply_scene_pattern_networks(scene: &mut Scene, features: AudioFeatures, t: f32) {
    if scene.sdf.patterns.is_empty() {
        return;
    }
    let net: PatternNetwork = scene.sdf.patterns[0].clone();
    let ctx = audio_pattern_context(features);
    apply_to_all_materials(&mut scene.sdf.root, &mut scene.sdf.objects, &mut |m| {
        if m.pattern_network.is_none() {
            m.pattern_network = Some(net.clone());
        }
        m.parameters
            .insert("pattern_low".into(), ctx.low_freq_energy);
        m.parameters
            .insert("pattern_mid".into(), ctx.mid_freq_energy);
        m.parameters
            .insert("pattern_high".into(), ctx.high_freq_energy);
        m.parameters
            .insert("pattern_dominant".into(), ctx.dominant_frequency);
        m.parameters
            .insert("pattern_beat".into(), ctx.current_beat as f32);
        m.parameters
            .insert("pattern_measure".into(), ctx.current_measure as f32);
        m.parameters
            .insert("pattern_phrase".into(), ctx.current_phrase as f32);
        m.parameters
            .insert("pattern_beat_phase".into(), ctx.beat_phase.clamp(0.0, 1.0));
        m.parameters.insert("pattern_tempo".into(), ctx.tempo);
        m.parameters.insert("rhythm_time".into(), t);
    });
}

fn sample_scene_fields(scene: &Scene, p: V3, time: f32) -> FieldSample {
    fields::sample_fields(
        &scene.sdf.fields,
        Vec3::new(p.x, p.y, p.z),
        time,
        scene.sdf.seed,
    )
}

fn apply_to_all_materials(
    root: &mut SdfNode,
    objects: &mut [SdfObject],
    f: &mut dyn FnMut(&mut SdfMaterial),
) {
    for o in objects {
        f(&mut o.material);
    }
    apply_to_node_materials(root, f);
}

fn apply_to_node_materials(node: &mut SdfNode, f: &mut dyn FnMut(&mut SdfMaterial)) {
    match node {
        SdfNode::Primitive { object } => f(&mut object.material),
        SdfNode::Group { children }
        | SdfNode::Union { children }
        | SdfNode::SmoothUnion { children, .. }
        | SdfNode::Intersect { children }
        | SdfNode::Blend { children, .. } => {
            for child in children {
                apply_to_node_materials(child, f);
            }
        }
        SdfNode::Subtract { base, subtract } => {
            apply_to_node_materials(base, f);
            for n in subtract {
                apply_to_node_materials(n, f);
            }
        }
        SdfNode::Transform { child, .. } => apply_to_node_materials(child, f),
        SdfNode::Empty => {}
    }
}

fn apply_to_primitives(
    root: &mut SdfNode,
    objects: &mut [SdfObject],
    f: &mut dyn FnMut(&mut SdfPrimitive),
) {
    for o in objects {
        f(&mut o.primitive);
    }
    apply_to_node_primitives(root, f);
}

fn apply_to_node_primitives(node: &mut SdfNode, f: &mut dyn FnMut(&mut SdfPrimitive)) {
    match node {
        SdfNode::Primitive { object } => f(&mut object.primitive),
        SdfNode::Group { children }
        | SdfNode::Union { children }
        | SdfNode::SmoothUnion { children, .. }
        | SdfNode::Intersect { children }
        | SdfNode::Blend { children, .. } => {
            for child in children {
                apply_to_node_primitives(child, f);
            }
        }
        SdfNode::Subtract { base, subtract } => {
            apply_to_node_primitives(base, f);
            for n in subtract {
                apply_to_node_primitives(n, f);
            }
        }
        SdfNode::Transform { child, .. } => apply_to_node_primitives(child, f),
        SdfNode::Empty => {}
    }
}

fn shade_ray(
    scene: &Scene,
    origin: V3,
    direction: V3,
    config: RenderConfig,
) -> (V3, f32, u32, V3, f32, f32, RayStageDurations) {
    let mut timings = RayStageDurations::default();
    let geometry_start = Instant::now();
    if let Some(hit) = march_scene(scene, origin, direction, config) {
        timings.geometry_ns += geometry_start.elapsed().as_nanos();
        let material_start = Instant::now();
        let m = evaluate_material(
            &hit.material,
            [hit.position.x, hit.position.y, hit.position.z],
            [hit.normal.x, hit.normal.y, hit.normal.z],
            config.time,
            scene.sdf.seed,
        );
        let field = sample_scene_fields(scene, hit.position, config.time.seconds);
        timings.material_pattern_ns += material_start.elapsed().as_nanos();
        let light_start = Instant::now();
        let base = V3::new(m.color[0], m.color[1], m.color[2]) * (1.0 + field.scalar * 0.08);
        let mut color = base * (scene.sdf.lighting.ambient_light + field.energy * 0.03);

        let ao = if config.enable_ambient_occlusion {
            ambient_occlusion(scene, hit.position, hit.normal, config)
        } else {
            1.0
        };

        for key in &scene.sdf.lighting.key_lights {
            let ldir = from_vec3(key.direction).normalized() * -1.0;
            let lambert = hit.normal.dot(ldir).max(0.0);
            let shadow = if config.enable_soft_shadows {
                soft_shadow(scene, hit.position + hit.normal * 0.005, ldir, config)
            } else {
                1.0
            };
            let half_vec = (ldir - direction).normalized();
            let specular = hit
                .normal
                .dot(half_vec)
                .max(0.0)
                .powf((1.0 - m.roughness) * 48.0 + 2.0);
            let light_color = from_vec3(key.color) * key.intensity;
            color = color
                + base.hadamard(light_color) * (lambert * shadow * ao)
                + light_color * specular * (1.0 - m.roughness) * 0.25 * shadow;
        }

        color = color + base * (m.emission + field.energy * 0.15);
        color = color + base * hit.distance_glow * (0.25 + field.energy * 0.08);

        if config.enable_scattering {
            color =
                color + light_scattering(scene, origin, direction, hit.distance_traveled, config);
        }

        if config.enable_fog {
            color = apply_fog(scene, color, hit.position, hit.distance_traveled);
        }

        let audio = analyze_procedural_audio_opt(scene, config.time.seconds);
        color = apply_volumetric(
            color,
            hit.distance_traveled,
            audio.2,
            VolumetricConfig {
                volumetric_density: scene
                    .sdf
                    .lighting
                    .volumetric
                    .beam_density
                    .max(config.volumetric_density),
                scatter_strength: scene
                    .sdf
                    .lighting
                    .volumetric
                    .beam_falloff
                    .max(config.scatter_strength),
                fog_color: [
                    scene.sdf.lighting.fog_color.x,
                    scene.sdf.lighting.fog_color.y,
                    scene.sdf.lighting.fog_color.z,
                ],
                light_beam_strength: scene
                    .sdf
                    .lighting
                    .volumetric
                    .shaft_intensity
                    .max(config.light_beam_strength),
            },
        );

        timings.lighting_atmosphere_ns += light_start.elapsed().as_nanos();

        let emission_out = (m.emission + hit.distance_glow).max(0.0);
        return (
            color.clamp01(),
            emission_out,
            hit.march_steps,
            hit.position,
            hit.distance_traveled,
            hit.step_reduction_estimate,
            timings,
        );
    }

    timings.geometry_ns += geometry_start.elapsed().as_nanos();
    let sky_start = Instant::now();
    let t = 0.5 * (direction.y + 1.0);
    let mut sky = V3::new(0.05, 0.08, 0.12) * (1.0 - t) + V3::new(0.25, 0.3, 0.4) * t;
    if config.enable_scattering {
        sky = sky
            + light_scattering(scene, origin, direction, config.max_distance * 0.3, config) * 0.5;
    }
    timings.lighting_atmosphere_ns += sky_start.elapsed().as_nanos();
    let miss_position = origin + direction * config.max_distance;
    (
        sky.clamp01(),
        0.0,
        0,
        miss_position,
        config.max_distance,
        0.0,
        timings,
    )
}

fn apply_fog(scene: &Scene, surface_color: V3, position: V3, distance: f32) -> V3 {
    let fog_color = from_vec3(scene.sdf.lighting.fog_color);
    let fields = sample_scene_fields(scene, position, distance * 0.03);
    let density = (scene.sdf.lighting.fog_density + fields.energy * 0.05).max(0.0);
    let height = scene.sdf.lighting.fog_height_falloff.max(0.0);
    let height_term = (-height * position.y.max(0.0)).exp();
    let fog = 1.0 - (-density * distance * height_term).exp();
    surface_color * (1.0 - fog) + fog_color * fog
}

fn ambient_occlusion(scene: &Scene, position: V3, normal: V3, config: RenderConfig) -> f32 {
    let mut occ = 0.0;
    let samples = config.ao_samples.max(1);
    for i in 1..=samples {
        let expected = i as f32 * 0.08;
        let p = position + normal * expected;
        let d = scene_distance(scene, p, config.time.seconds, config).distance;
        occ += (expected - d).max(0.0) / expected;
    }
    (1.0 - (occ / samples as f32) * config.ao_strength).clamp(0.0, 1.0)
}

fn light_scattering(
    scene: &Scene,
    origin: V3,
    direction: V3,
    max_t: f32,
    config: RenderConfig,
) -> V3 {
    let v = &scene.sdf.lighting.volumetric;
    let steps = v.scattering_steps.max(2);
    let mut accum = V3::zero();
    let mut t = 0.1;
    let step = (max_t / steps as f32).max(0.05);
    for _ in 0..steps {
        let p = origin + direction * t;
        let transmittance =
            (-(scene.sdf.lighting.fog_density.max(0.0) + v.beam_density.max(0.0)) * t).exp();
        for key in &scene.sdf.lighting.key_lights {
            let ldir = from_vec3(key.direction).normalized() * -1.0;
            let phase = direction
                .dot(ldir)
                .max(0.0)
                .powf(config.shadow_softness.max(1.0) * (0.2 + v.beam_falloff.max(0.0)));
            let f = sample_scene_fields(scene, p, config.time.seconds);
            let c = from_vec3(key.color)
                * key.intensity
                * phase
                * transmittance
                * (0.02 + f.energy * 0.01)
                * (1.0 + v.shaft_intensity.max(0.0));
            accum = accum + c;
        }
        t += step;
        if t > max_t {
            break;
        }
    }
    accum
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
    let mut glow = 0.0;
    let mut march_steps = 0;
    let mut reduction_acc = 0.0_f32;
    for _ in 0..config.max_steps {
        march_steps += 1;
        let p = origin + direction * t;
        let eval = scene_distance(scene, p, config.time.seconds, config);
        let attenuation = (-t * 0.08).exp();
        glow += eval.material.emissive_strength.max(0.0) * attenuation * 0.01;
        if eval.distance < config.surface_epsilon {
            let normal = estimate_normal(
                scene,
                p,
                config.surface_epsilon * 2.0,
                config.time.seconds,
                config,
            );
            return Some(Hit {
                position: p,
                normal,
                material: eval.material,
                distance_traveled: t,
                distance_glow: glow,
                march_steps,
                step_reduction_estimate: if march_steps > 0 {
                    reduction_acc / march_steps as f32
                } else {
                    0.0
                },
            });
        }

        let base_step = eval
            .distance
            .max(config.surface_epsilon * config.min_step_scale.max(0.05));
        let far_factor = if config.adaptive_raymarch {
            (1.0 + (t / config.max_distance.max(1.0)) * config.far_field_boost).clamp(
                config.min_step_scale.max(0.05),
                config.max_step_scale.max(config.min_step_scale.max(0.05)),
            )
        } else {
            1.0
        };
        let adaptive_step = (base_step * far_factor * config.quality.raymarch_quality.max(0.25))
            .max(config.surface_epsilon * 0.25);
        let cone_cfg = ConeMarchConfig {
            cone_step_multiplier: config.cone_step_multiplier,
            cone_shadow_factor: config.cone_shadow_factor,
            surface_thickness_estimation: config.surface_thickness_estimation,
            adaptive_step_scale: config.adaptive_step_scale,
        };
        let stepped = cone_step(eval.distance, t, cone_cfg).max(adaptive_step);
        reduction_acc += ((stepped - adaptive_step) / adaptive_step.max(1e-4))
            .abs()
            .clamp(0.0, 1.0);
        t += stepped;
        if t > config.max_distance {
            return None;
        }
    }

    None
}

fn soft_shadow(scene: &Scene, origin: V3, direction: V3, config: RenderConfig) -> f32 {
    let mut t = 0.02;
    let mut shadow: f32 = 1.0;
    for _ in 0..config.shadow_steps {
        let p = origin + direction * t;
        let eval = scene_distance(scene, p, config.time.seconds, config);
        if eval.distance < config.surface_epsilon {
            return 0.0;
        }
        let cone_cfg = ConeMarchConfig {
            cone_step_multiplier: config.cone_step_multiplier,
            cone_shadow_factor: config.cone_shadow_factor,
            surface_thickness_estimation: config.surface_thickness_estimation,
            adaptive_step_scale: config.adaptive_step_scale,
        };
        shadow = shadow.min(
            config.shadow_softness * shadow_cone_factor(eval.distance, cone_cfg) / t.max(0.001),
        );
        t += cone_step(eval.distance, t, cone_cfg).max(0.01);
        if t > config.max_distance {
            break;
        }
    }
    shadow.clamp(0.0, 1.0)
}

fn estimate_normal(scene: &Scene, p: V3, eps: f32, time: f32, config: RenderConfig) -> V3 {
    let ex = V3::new(eps, 0.0, 0.0);
    let ey = V3::new(0.0, eps, 0.0);
    let ez = V3::new(0.0, 0.0, eps);

    let dx = scene_distance(scene, p + ex, time, config).distance
        - scene_distance(scene, p - ex, time, config).distance;
    let dy = scene_distance(scene, p + ey, time, config).distance
        - scene_distance(scene, p - ey, time, config).distance;
    let dz = scene_distance(scene, p + ez, time, config).distance
        - scene_distance(scene, p - ez, time, config).distance;

    V3::new(dx, dy, dz).normalized()
}

fn scene_distance(scene: &Scene, point: V3, time: f32, config: RenderConfig) -> NodeEval {
    let field = sample_scene_fields(scene, point, time);
    let warped = point + V3::new(field.vector.x, field.vector.y, field.vector.z) * 0.08;
    if !matches!(scene.sdf.root, SdfNode::Empty) {
        evaluate_node(&scene.sdf.root, warped, scene.sdf.seed, time, None, config)
    } else {
        let mut best = NodeEval {
            distance: f32::INFINITY,
            material: SdfMaterial::default(),
        };
        for object in &scene.sdf.objects {
            let eval = evaluate_object(object, warped, scene.sdf.seed, time, config);
            if eval.distance < best.distance {
                best = eval;
            }
        }
        best
    }
}

fn evaluate_node(
    node: &SdfNode,
    point: V3,
    scene_seed: u32,
    time: f32,
    hint: Option<f32>,
    config: RenderConfig,
) -> NodeEval {
    match node {
        SdfNode::Empty => NodeEval {
            distance: f32::INFINITY,
            material: SdfMaterial::default(),
        },
        SdfNode::Primitive { object } => evaluate_object(object, point, scene_seed, time, config),
        SdfNode::Group { children } | SdfNode::Union { children } => {
            let mut best = NodeEval {
                distance: f32::INFINITY,
                material: SdfMaterial::default(),
            };
            for child in children {
                let child_eval =
                    evaluate_node(child, point, scene_seed, time, Some(best.distance), config);
                if child_eval.distance < best.distance {
                    best = child_eval;
                    if let Some(h) = hint
                        && best.distance <= h
                    {
                        break;
                    }
                }
            }
            best
        }
        SdfNode::SmoothUnion { children, k } => {
            let mut acc = NodeEval {
                distance: f32::INFINITY,
                material: SdfMaterial::default(),
            };
            for child in children {
                let next = evaluate_node(child, point, scene_seed, time, hint, config);
                if !acc.distance.is_finite() {
                    acc = next;
                } else {
                    let h = ((*k - (acc.distance - next.distance).abs()).max(0.0) / k.max(1e-6))
                        .powi(2)
                        * 0.25;
                    let d = acc.distance.min(next.distance) - h * *k;
                    let mat = if next.distance < acc.distance {
                        next.material.clone()
                    } else {
                        acc.material.clone()
                    };
                    acc = NodeEval {
                        distance: d,
                        material: mat,
                    };
                }
            }
            acc
        }
        SdfNode::Subtract { base, subtract } => {
            let mut base_eval = evaluate_node(base, point, scene_seed, time, hint, config);
            if subtract.is_empty() {
                return base_eval;
            }
            let mut d_sub = f32::INFINITY;
            for n in subtract {
                let e = evaluate_node(n, point, scene_seed, time, Some(d_sub), config);
                d_sub = d_sub.min(e.distance);
            }
            base_eval.distance = smooth_max(base_eval.distance, -d_sub, 0.0);
            base_eval
        }
        SdfNode::Intersect { children } => {
            if children.is_empty() {
                return NodeEval {
                    distance: f32::INFINITY,
                    material: SdfMaterial::default(),
                };
            }
            let mut acc = evaluate_node(&children[0], point, scene_seed, time, hint, config);
            for child in &children[1..] {
                let e = evaluate_node(child, point, scene_seed, time, hint, config);
                if e.distance > acc.distance {
                    acc.material = e.material.clone();
                }
                acc.distance = acc.distance.max(e.distance);
            }
            acc
        }
        SdfNode::Blend { children, weights } => {
            if children.is_empty() {
                return NodeEval {
                    distance: f32::INFINITY,
                    material: SdfMaterial::default(),
                };
            }
            let mut sum_w = 0.0;
            let mut dist = 0.0;
            let mut nearest = NodeEval {
                distance: f32::INFINITY,
                material: SdfMaterial::default(),
            };
            for (i, child) in children.iter().enumerate() {
                let w = weights.get(i).copied().unwrap_or(1.0).max(0.0001);
                let e = evaluate_node(child, point, scene_seed, time, hint, config);
                dist += e.distance * w;
                sum_w += w;
                if e.distance < nearest.distance {
                    nearest = e;
                }
            }
            nearest.distance = dist / sum_w.max(0.0001);
            nearest
        }
        SdfNode::Transform {
            modifiers,
            child,
            bounds_radius,
        } => {
            let mut p = point;
            let mut scale = 1.0;
            for modifier in modifiers {
                apply_modifier(&mut p, &mut scale, modifier, scene_seed, time);
            }
            if let Some(r) = bounds_radius {
                let bound_d = p.length() - *r;
                if let Some(h) = hint
                    && bound_d > h
                {
                    return NodeEval {
                        distance: bound_d,
                        material: SdfMaterial::default(),
                    };
                }
            }
            let mut eval = evaluate_node(
                child,
                p,
                scene_seed,
                time,
                hint.map(|h| h / scale.max(1e-6)),
                config,
            );
            eval.distance *= scale;
            eval
        }
    }
}

fn evaluate_object(
    object: &SdfObject,
    point: V3,
    scene_seed: u32,
    time: f32,
    config: RenderConfig,
) -> NodeEval {
    let mut p = point;
    let mut distance_scale = 1.0;
    for modifier in &object.modifiers {
        apply_modifier(&mut p, &mut distance_scale, modifier, scene_seed, time);
    }
    if let Some(r) = object.bounds_radius {
        let bound_d = p.length() - r;
        if bound_d > 4.0 {
            return NodeEval {
                distance: bound_d,
                material: object.material.clone(),
            };
        }
    }
    let distance = eval_primitive(&object.primitive, p, scene_seed, time, config) * distance_scale;
    NodeEval {
        distance,
        material: object.material.clone(),
    }
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
            let rp = domain_repeat_grid(to_vec3(*p), *cell);
            *p = from_vec3(rp);
        }
        SdfModifier::RepeatGrid { cell_size } => {
            let rp = domain_repeat_grid(to_vec3(*p), *cell_size);
            *p = from_vec3(rp);
        }
        SdfModifier::RepeatAxis { spacing, axis } => {
            let ax = match axis.as_str() {
                "x" | "X" => Axis::X,
                "y" | "Y" => Axis::Y,
                _ => Axis::Z,
            };
            let rp = domain_repeat_axis(to_vec3(*p), *spacing, ax);
            *p = from_vec3(rp);
        }
        SdfModifier::RepeatPolar { sectors } => {
            let rp = domain_repeat_polar(to_vec3(*p), *sectors);
            *p = from_vec3(rp);
        }
        SdfModifier::RepeatSphere { radius } => {
            let rp = repeat_sphere(to_vec3(*p), *radius);
            *p = from_vec3(rp);
        }
        SdfModifier::MirrorFold => {
            *p = from_vec3(mirror_fold(to_vec3(*p)));
        }
        SdfModifier::KaleidoscopeFold { segments } => {
            *p = from_vec3(kaleidoscope_fold(to_vec3(*p), *segments));
        }
        SdfModifier::FoldSpace => {
            *p = from_vec3(fold_space(to_vec3(*p)));
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

fn eval_primitive(
    primitive: &SdfPrimitive,
    p: V3,
    scene_seed: u32,
    time: f32,
    config: RenderConfig,
) -> f32 {
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
        } => sdf_mandelbulb(p, *power, *iterations, *bailout, config),
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

fn sdf_mandelbulb(p: V3, power: f32, iterations: u32, bailout: f32, config: RenderConfig) -> f32 {
    let mut z = p;
    let mut dr = 1.0;
    let mut r = 0.0;
    let lod_iters = lod_iterations(
        iterations,
        p.length(),
        LodConfig {
            distance_bias: config.distance_bias,
            fractal_iteration_limit: config.fractal_iteration_limit,
            fractal_lod_scale: config.fractal_lod_scale,
            detail_falloff: config.detail_falloff,
        },
    );
    for _ in 0..lod_iters {
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
    let base = 0.5 * r.max(1e-6).ln() * r / dr.max(1e-6);
    let kifs = kifs_fractal(p, lod_iters.min(6), 1.2, bailout.max(2.0));
    base * 0.8 + kifs * 0.2
}

fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    a * h + b * (1.0 - h) - k * h * (1.0 - h)
}

fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
    -smooth_min(-a, -b, k)
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

fn rotate_axis_angle(p: V3, axis: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    p * c + axis.cross(p) * s + axis * axis.dot(p) * (1.0 - c)
}

fn analyze_procedural_audio_opt(scene: &Scene, time_seconds: f32) -> (f32, f32, f32) {
    if let Some(cfg) = &scene.sdf.audio {
        let af = analyze_procedural_audio(cfg, time_seconds);
        let harmonic =
            (af.harmonic_ratios[0] + af.harmonic_ratios[1] + af.harmonic_ratios[2]) / 3.0;
        let energy = (af.bass_energy + af.mid_energy + af.high_energy) / 3.0;
        (energy, af.beat_phase, harmonic)
    } else {
        (0.0, 0.0, 0.0)
    }
}

fn from_vec3(v: Vec3) -> V3 {
    V3::new(v.x, v.y, v.z)
}

fn to_vec3(v: V3) -> Vec3 {
    Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

#[derive(Clone, Copy, Debug)]
struct V2 {
    x: f32,
    y: f32,
}
impl V2 {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub(crate) fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub(crate) fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y))
    }
    pub(crate) fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub(crate) fn max_component(self) -> f32 {
        self.x.max(self.y)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct V3 {
    x: f32,
    y: f32,
    z: f32,
}
impl V3 {
    pub(crate) fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub(crate) fn splat(v: f32) -> Self {
        Self::new(v, v, v)
    }
    pub(crate) fn zero() -> Self {
        Self::splat(0.0)
    }
    pub(crate) fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub(crate) fn normalized(self) -> Self {
        self / self.length().max(1e-6)
    }
    pub(crate) fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub(crate) fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
    pub(crate) fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
    pub(crate) fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y), self.z.max(rhs.z))
    }
    pub(crate) fn max_component(self) -> f32 {
        self.x.max(self.y).max(self.z)
    }
    pub(crate) fn hadamard(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
    pub(crate) fn clamp01(self) -> Self {
        Self::new(
            self.x.clamp(0.0, 1.0),
            self.y.clamp(0.0, 1.0),
            self.z.clamp(0.0, 1.0),
        )
    }
}
impl std::ops::Add for V3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl std::ops::Sub for V3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl std::ops::Mul<f32> for V3 {
    type Output = Self;
    fn mul(self, r: f32) -> Self {
        Self::new(self.x * r, self.y * r, self.z * r)
    }
}
impl std::ops::Div<f32> for V3 {
    type Output = Self;
    fn div(self, r: f32) -> Self {
        Self::new(self.x / r, self.y / r, self.z / r)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RenderConfig, RenderTime, evaluate_material, render_sdf_scene_with_config, smooth_min,
    };
    use aurex_scene::{
        Scene, SdfCamera, SdfLighting, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode,
        SdfObject, SdfPattern, SdfPrimitive, Vec3,
        patterns::{
            PatternBinding, PatternComposeOp, PatternLayer, PatternNetwork, PatternNode,
            PatternParams, PatternPreset,
        },
    };

    fn sample_scene() -> Scene {
        let sphere = SdfNode::Primitive {
            object: SdfObject {
                primitive: SdfPrimitive::Sphere { radius: 1.0 },
                modifiers: vec![],
                material: SdfMaterial {
                    material_type: SdfMaterialType::NeonGrid,
                    base_color: Vec3::new(0.3, 0.95, 1.0),
                    emissive_strength: 0.7,
                    roughness: 0.2,
                    pattern: SdfPattern::Bands,
                    pattern_network: None,
                    parameters: std::collections::BTreeMap::new(),
                },
                bounds_radius: Some(1.2),
            },
        };

        let box_node = SdfNode::Transform {
            modifiers: vec![SdfModifier::Translate {
                offset: Vec3::new(1.1, 0.0, 0.0),
            }],
            child: Box::new(SdfNode::Primitive {
                object: SdfObject {
                    primitive: SdfPrimitive::Box {
                        size: Vec3::new(0.45, 0.45, 0.45),
                    },
                    modifiers: vec![],
                    material: SdfMaterial {
                        material_type: SdfMaterialType::Plasma,
                        ..SdfMaterial::default()
                    },
                    bounds_radius: Some(1.0),
                },
            }),
            bounds_radius: Some(2.0),
        };

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
                    fog_color: Vec3::new(0.08, 0.12, 0.18),
                    fog_density: 0.03,
                    fog_height_falloff: 0.08,
                    volumetric: Default::default(),
                },
                objects: vec![],
                root: SdfNode::SmoothUnion {
                    children: vec![sphere, box_node],
                    k: 0.35,
                },
                timeline: None,
                generator: None,
                fields: vec![aurex_scene::fields::SceneField::Pulse(
                    aurex_scene::fields::PulseField {
                        origin: Vec3::new(0.0, 0.0, 0.0),
                        frequency: 2.0,
                        amplitude: 0.5,
                        falloff: 0.2,
                    },
                )],
                patterns: vec![PatternNetwork {
                    name: Some("sample".into()),
                    preset: Some(PatternPreset::ElectronicCircuit),
                    layers: vec![],
                }],
                harmonics: None,
                rhythm: None,
                audio: Some(aurex_audio::default_demo_audio_config(2027)),
                effect_graph: None,
                automation_tracks: vec![],
                demo_sequence: None,
                temporal_effects: vec![],
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

    #[test]
    fn pattern_network_changes_material_evaluation() {
        let mut m = SdfMaterial {
            material_type: SdfMaterialType::SolidColor,
            ..SdfMaterial::default()
        };
        m.pattern_network = Some(PatternNetwork {
            name: Some("test".into()),
            preset: None,
            layers: vec![PatternLayer {
                node: PatternNode::ConcentricPulsePattern(PatternParams {
                    scale: 2.2,
                    density: 1.1,
                    ..PatternParams::default()
                }),
                op: PatternComposeOp::Blend,
                weight: 1.0,
                binding: PatternBinding::default(),
            }],
        });
        m.parameters.insert("pattern_low".into(), 0.6);
        m.parameters.insert("pattern_beat_phase".into(), 0.2);

        let a = evaluate_material(
            &m,
            [0.1, 0.2, 0.3],
            [0.0, 1.0, 0.0],
            RenderTime { seconds: 0.1 },
            7,
        );
        let b = evaluate_material(
            &m,
            [0.4, 0.2, 0.1],
            [0.0, 1.0, 0.0],
            RenderTime { seconds: 0.1 },
            7,
        );
        assert_ne!(a.color, b.color);
    }

    #[test]
    fn audio_features_influence_render() {
        let mut scene = sample_scene();
        scene.sdf.audio = Some(aurex_audio::default_demo_audio_config(99));
        let a = render_sdf_scene_with_config(
            &scene,
            RenderConfig {
                width: 24,
                height: 14,
                time: RenderTime { seconds: 0.1 },
                ..RenderConfig::default()
            },
        );
        let b = render_sdf_scene_with_config(
            &scene,
            RenderConfig {
                width: 24,
                height: 14,
                time: RenderTime { seconds: 1.1 },
                ..RenderConfig::default()
            },
        );
        assert_ne!(a.pixels, b.pixels);
    }

    #[test]
    fn lighting_toggles_change_output() {
        let base_cfg = RenderConfig {
            width: 32,
            height: 18,
            time: RenderTime { seconds: 1.5 },
            ..RenderConfig::default()
        };
        let with_fx = render_sdf_scene_with_config(&sample_scene(), base_cfg);
        let no_fx = render_sdf_scene_with_config(
            &sample_scene(),
            RenderConfig {
                enable_soft_shadows: false,
                enable_ambient_occlusion: false,
                enable_fog: false,
                enable_scattering: false,
                ..base_cfg
            },
        );
        assert_ne!(with_fx.pixels, no_fx.pixels);
    }

    #[test]
    fn smooth_min_blends_distances() {
        let hard = (-0.2f32).min(0.1);
        let smooth = smooth_min(-0.2, 0.1, 0.3);
        assert!(smooth <= hard + 0.05);
    }
}
