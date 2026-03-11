use aurex_render_sdf::{RenderConfig, render_sdf_scene_with_config};
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
    ];

    for scene_path in scenes {
        let scene = load_scene_from_json_path(scene_path).expect("scene should load");
        let frame = render_sdf_scene_with_config(
            &scene,
            RenderConfig {
                width: 320,
                height: 180,
                ..RenderConfig::default()
            },
        );

        let output_name = scene_path
            .rsplit('/')
            .next()
            .unwrap_or("scene.json")
            .replace(".json", ".ppm");

        write_ppm(&output_name, frame.width, frame.height, &frame.pixels);
        println!("rendered {} -> {}", scene_path, output_name);
    }
}
