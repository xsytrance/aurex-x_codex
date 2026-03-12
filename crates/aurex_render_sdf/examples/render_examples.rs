use aurex_render_sdf::{RenderConfig, RenderTime, render_sdf_scene_with_config};
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
    ];

    for scene_path in scenes {
        let scene = load_scene_from_json_path(scene_path).expect("scene should load");
        let frame = render_sdf_scene_with_config(
            &scene,
            RenderConfig {
                width: 320,
                height: 180,
                time: RenderTime { seconds: 2.5 },
                ..RenderConfig::default()
            },
        );

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
        println!(
            "rendered {} -> {} (avg bloom {:.3})",
            scene_path, output_name, bloom_avg
        );
    }
}
