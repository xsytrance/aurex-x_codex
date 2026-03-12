use aurex_render_sdf::{
    RenderConfig, RenderTime, render_sdf_scene_with_config, render_sdf_scene_with_diagnostics,
};
use aurex_scene::load_scene_from_json_path;

fn write_ppm(path: &str, width: u32, height: u32, pixels: &[aurex_render_sdf::Rgba8]) {
    let mut out = format!("P3\n{} {}\n255\n", width, height);
    for px in pixels {
        out.push_str(&format!("{} {} {}\n", px.r, px.g, px.b));
    }
    std::fs::write(path, out).expect("ppm should write");
}

fn main() {
    let scenes = [
        "examples/neon_tunnel.json",
        "examples/fractal_temple.json",
        "examples/particle_storm.json",
        "examples/aurora_flight.json",
        "examples/psy_tunnel.json",
        "examples/fractal_temple_world.json",
        "examples/circuit_world.json",
        "examples/circuit_rhythm_world.json",
        "examples/psytrance_dimension.json",
        "examples/prime_pulse_music.json",
        "examples/harmonic_tunnel.json",
        "examples/singing_fractal_temple.json",
        "examples/prime_pulse_concert.json",
        "examples/rhythm_tunnel.json",
        "examples/prime_pulse_rhythm.json",
        "examples/choir_fractal_growth.json",
        "examples/djinn_circuit_cathedral.json",
        "examples/prime_pulse_sanctum.json",
        "examples/psyspiral_tunnel.json",
        "examples/opera_resonance_hall.json",
        "examples/prime_pulse_flythrough.json",
        "examples/psytrance_tunnel_run.json",
        "examples/fractal_cathedral_orbit.json",
        "examples/circuit_cathedral_cinematic.json",
        "examples/psytrance_effect_graph.json",
        "examples/pattern_automation_demo.json",
        "examples/harmonic_temple_growth.json",
        "examples/prime_pulse_demo_sequence.json",
        "examples/psytrance_transition_demo.json",
        "examples/fractal_cathedral_morph.json",
        "examples/prime_pulse_awaken_demo.json",
        "examples/aurex_showcase_demo.json",
        "examples/infinite_circuit_city.json",
        "examples/infinite_fractal_temple.json",
        "examples/prime_pulse_cathedral.json",
        "examples/psytrance_infinite_tunnel.json",
        "examples/neon_motion_trails.json",
        "examples/rhythm_echo_city.json",
        "examples/synesthesia_tunnel.json",
        "examples/prime_pulse_temporal_wave.json",
        "examples/circuit_megacity_stress.json",
        "examples/pattern_storm_stress.json",
        "examples/prime_pulse_performance_test.json",
        "examples/showcase_demo_stress.json",
    ];

    for scene_path in scenes {
        let scene = load_scene_from_json_path(scene_path).expect("scene should load");
        let is_stress = scene_path.contains("_stress") || scene_path.contains("performance_test");
        let cfg = RenderConfig {
            width: if is_stress { 200 } else { 320 },
            height: if is_stress { 112 } else { 180 },
            time: RenderTime { seconds: 2.5 },
            output_diagnostics: std::env::var("AUREX_DIAGNOSTICS").is_ok(),
            ..RenderConfig::default()
        };
        let (frame, diag) = if cfg.output_diagnostics {
            render_sdf_scene_with_diagnostics(&scene, cfg)
        } else {
            (
                render_sdf_scene_with_config(&scene, cfg),
                Default::default(),
            )
        };

        let output_name = scene_path
            .rsplit('/')
            .next()
            .unwrap_or("scene.json")
            .replace(".json", ".ppm");

        write_ppm(&output_name, frame.width, frame.height, &frame.pixels);
        let bloom_avg = frame
            .bloom_prepass
            .as_ref()
            .map(|b| b.iter().copied().sum::<f32>() / b.len() as f32)
            .unwrap_or(0.0);
        if cfg.output_diagnostics {
            println!(
                "rendered {} -> {} (avg bloom {:.3}) steps:{} rays:{} cache[p:{}/{} f:{}/{} eg:{}] temporal[size:{} depth:{}] total_ms:{:.3}",
                scene_path,
                output_name,
                bloom_avg,
                diag.stats.raymarch_steps_total,
                diag.stats.rays_traced,
                diag.stats.cache.pattern_hits,
                diag.stats.cache.pattern_misses,
                diag.stats.cache.field_hits,
                diag.stats.cache.field_misses,
                diag.stats.cache.effect_graph_evals,
                diag.stats.temporal_buffer_size,
                diag.stats.temporal_history_depth,
                diag.total_frame_time_ms
            );
            println!(
                "stage_ms_pct: ScenePreprocess={:.3}ms/{:.1}% EffectGraphEvaluation={:.3}ms/{:.1}% GeometrySdf={:.3}ms/{:.1}% MaterialPattern={:.3}ms/{:.1}% LightingAtmosphere={:.3}ms/{:.1}% PostProcessing={:.3}ms/{:.1}% TemporalFeedback={:.3}ms/{:.1}%",
                *diag
                    .stage_durations_ms
                    .get("ScenePreprocess")
                    .unwrap_or(&0.0),
                *diag
                    .stage_percentages
                    .get("ScenePreprocess")
                    .unwrap_or(&0.0),
                *diag
                    .stage_durations_ms
                    .get("EffectGraphEvaluation")
                    .unwrap_or(&0.0),
                *diag
                    .stage_percentages
                    .get("EffectGraphEvaluation")
                    .unwrap_or(&0.0),
                *diag.stage_durations_ms.get("GeometrySdf").unwrap_or(&0.0),
                *diag.stage_percentages.get("GeometrySdf").unwrap_or(&0.0),
                *diag
                    .stage_durations_ms
                    .get("MaterialPattern")
                    .unwrap_or(&0.0),
                *diag
                    .stage_percentages
                    .get("MaterialPattern")
                    .unwrap_or(&0.0),
                *diag
                    .stage_durations_ms
                    .get("LightingAtmosphere")
                    .unwrap_or(&0.0),
                *diag
                    .stage_percentages
                    .get("LightingAtmosphere")
                    .unwrap_or(&0.0),
                *diag
                    .stage_durations_ms
                    .get("PostProcessing")
                    .unwrap_or(&0.0),
                *diag.stage_percentages.get("PostProcessing").unwrap_or(&0.0),
                *diag
                    .stage_durations_ms
                    .get("TemporalFeedback")
                    .unwrap_or(&0.0),
                *diag
                    .stage_percentages
                    .get("TemporalFeedback")
                    .unwrap_or(&0.0)
            );
        } else {
            println!(
                "rendered {} -> {} (avg bloom {:.3})",
                scene_path, output_name, bloom_avg
            );
        }
    }
}
