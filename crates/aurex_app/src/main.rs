use aurex_conductor::ConductorClock;
use aurex_lighting::{LightDescriptor, LightKind};
use aurex_postfx::BloomSettings;
use aurex_render::CameraRig;
use aurex_shape_synth::{PrimitiveType, ShapeDescriptor};

fn main() {
    let mut clock = ConductorClock::default();
    let camera = CameraRig::default();

    let shapes = [
        ShapeDescriptor {
            primitive_type: PrimitiveType::Circle,
            seed: 7,
        },
        ShapeDescriptor {
            primitive_type: PrimitiveType::Ring,
            seed: 11,
        },
        ShapeDescriptor {
            primitive_type: PrimitiveType::Tube,
            seed: 13,
        },
    ];

    let light = LightDescriptor {
        kind: LightKind::Pulse,
        intensity: 0.85,
        color_rgb: [0.2, 0.7, 1.0],
    };

    let bloom = BloomSettings::default();

    for _ in 0..3 {
        clock.advance_frame();
    }

    println!("Aurex runtime scaffold initialized.");
    println!("frame={} tick={}", clock.frame_index.0, clock.sim_tick.0);
    println!("camera_fov={}", camera.fov_degrees);
    println!("shape_count={}", shapes.len());
    println!("light_kind={:?} bloom_intensity={}", light.kind, bloom.intensity);
}
