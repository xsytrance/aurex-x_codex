mod boot_runtime;
mod runtime_flags;

use std::io::{self, BufRead};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use aurex_pulse::{
    PulseRuntime,
    runner::PulseRunner,
    schema::{Interactivity, PulseDefinition, PulseKind, PulseMetadata, PulseSceneSource},
};
use aurex_render::run_real_renderer_event_loop_with_frame_hook;
use aurex_render_sdf::{GeometrySdfMode, RenderConfig};
use boot_runtime::{BootRuntime, BootScreenMode};
use runtime_flags::RuntimeDebugFlags;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeMode {
    Boot,
    MidiDemo { midi_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeOptions {
    mode: RuntimeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputCommand {
    Exit,
    LaunchDemoPulse,
}

fn parse_runtime_options(args: impl IntoIterator<Item = String>) -> Result<RuntimeOptions, String> {
    let positional: Vec<String> = args.into_iter().collect();

    if positional.is_empty() {
        return Ok(RuntimeOptions {
            mode: RuntimeMode::Boot,
        });
    }

    match positional[0].as_str() {
        "boot" => {
            if positional.len() > 1 {
                return Err("Too many positional arguments for boot".to_string());
            }
            Ok(RuntimeOptions {
                mode: RuntimeMode::Boot,
            })
        }
        "midi_demo" => {
            let midi_path = positional
                .get(1)
                .ok_or_else(|| {
                    "Missing MIDI path. Usage: cargo run -p aurex_app -- midi_demo <path.mid>"
                        .to_string()
                })?
                .to_string();
            if positional.len() > 2 {
                return Err("Too many positional arguments for midi_demo".to_string());
            }
            Ok(RuntimeOptions {
                mode: RuntimeMode::MidiDemo { midi_path },
            })
        }
        other => Err(format!(
            "Unknown mode '{other}'. Use: boot (default) or midi_demo <path.mid>"
        )),
    }
}

fn spawn_input_listener() -> mpsc::Receiver<InputCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("exit")
                || trimmed.eq_ignore_ascii_case("esc")
                || trimmed == "\u{1b}"
            {
                let _ = tx.send(InputCommand::Exit);
            } else if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("enter") {
                let _ = tx.send(InputCommand::LaunchDemoPulse);
            }
        }
    });
    rx
}

fn main() {
    let debug_flags = RuntimeDebugFlags::from_env();
    println!("runtime_debug_flags={}", debug_flags.summary());

    let options = match parse_runtime_options(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    match options.mode {
        RuntimeMode::Boot => run_boot_mode(),
        RuntimeMode::MidiDemo { midi_path } => run_midi_demo(&midi_path),
    }
}

fn run_boot_mode() {
    println!("Boot mode started. Type 'exit' (or 'esc') + Enter to quit.");
    println!("Library screen controls: empty line/'enter' triggers demo placeholder.");

    let mut boot_runtime = BootRuntime::new(2026);
    let input_rx = spawn_input_listener();
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_hook = Arc::clone(&should_exit);

    let definition = PulseDefinition {
        metadata: PulseMetadata {
            title: "Aurex-X Boot Runtime".to_string(),
            author: "Aurex".to_string(),
            description: "Deterministic cinematic boot and library screen".to_string(),
            tags: vec!["boot".to_string(), "library".to_string()],
            seed: boot_runtime.scene.sdf.seed,
            pulse_kind: PulseKind::VisualMusic,
            duration_hint: Some(30.0),
            interactivity: Interactivity::Hybrid,
            prime_affinity: None,
        },
        pulse_kind: PulseKind::VisualMusic,
        scene: PulseSceneSource::Inline(boot_runtime.scene.clone()),
        audio: None,
        timeline: None,
        generators: vec![],
        music: None,
        boot_world: None,
        parameters: Default::default(),
    };

    let mut runner = match PulseRunner::load(definition, None) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("boot_runtime=error detail:{err}");
            return;
        }
    };
    runner.initialize();

    let mut last_time = 0.0_f32;
    if let Err(err) = run_real_renderer_event_loop_with_frame_hook(move |t| {
        let dt = (t - last_time).max(0.0);
        last_time = t;

        while let Ok(cmd) = input_rx.try_recv() {
            match cmd {
                InputCommand::Exit => should_exit_hook.store(true, Ordering::Relaxed),
                InputCommand::LaunchDemoPulse => {
                    if boot_runtime.screen_mode() == BootScreenMode::Library {
                        println!("[ Launch Demo Pulse ] placeholder activated.");
                    }
                }
            }
        }

        if should_exit_hook.load(Ordering::Relaxed) {
            std::process::exit(0);
        }

        boot_runtime.update(dt);
        runner.scene = boot_runtime.scene.clone();
        runner.update(dt);
        let _frame = runner.render(RenderConfig {
            geometry_mode: GeometrySdfMode::Legacy,
            ..RenderConfig::default()
        });

        if boot_runtime.screen_mode() == BootScreenMode::Library {
            println!("AUREX-X\n[ Launch Demo Pulse ]\n[ Settings ]\nExit");
            should_exit_hook.store(true, Ordering::Relaxed);
        }
    }) {
        eprintln!("boot_runtime=error detail:{err}");
    }
}

fn run_midi_demo(midi_path: &str) {
    match PulseRuntime::load_runtime_from_midi_file(std::path::Path::new(midi_path)) {
        Ok(mut runtime) => {
            runtime.print_debug();
            let scene = runtime.generate_scene();
            let mut runner = match PulseRunner::load(
                PulseDefinition {
                    metadata: PulseMetadata {
                        title: "MIDI Demo Runtime".to_string(),
                        author: "Aurex".to_string(),
                        description: "Music-driven procedural runtime scene".to_string(),
                        tags: vec!["midi_demo".to_string()],
                        seed: scene.sdf.seed,
                        pulse_kind: PulseKind::VisualMusic,
                        duration_hint: None,
                        interactivity: Interactivity::Passive,
                        prime_affinity: None,
                    },
                    pulse_kind: PulseKind::VisualMusic,
                    scene: PulseSceneSource::Inline(scene),
                    audio: None,
                    timeline: None,
                    generators: vec![],
                    music: None,
                    boot_world: None,
                    parameters: Default::default(),
                },
                None,
            ) {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("midi_demo_runtime=error detail:{err}");
                    return;
                }
            };

            runner.initialize();
            let mut last_time = 0.0_f32;
            if let Err(err) = run_real_renderer_event_loop_with_frame_hook(move |t| {
                let delta = (t - last_time).max(0.0);
                last_time = t;
                runner.update(delta);
                let _beat = runtime.update_scene_for_frame(&mut runner.scene, delta);
                let _frame = runner.render(RenderConfig {
                    geometry_mode: GeometrySdfMode::Legacy,
                    ..RenderConfig::default()
                });
            }) {
                eprintln!("midi_demo_runtime=error detail:{err}");
            }
        }
        Err(err) => eprintln!("midi_demo_runtime=error detail:{err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeMode, parse_runtime_options};

    #[test]
    fn runtime_defaults_to_boot_mode() {
        let options =
            parse_runtime_options(Vec::<String>::new()).expect("boot default should parse");
        assert_eq!(options.mode, RuntimeMode::Boot);
    }

    #[test]
    fn runtime_supports_boot_mode() {
        let options =
            parse_runtime_options(vec!["boot".to_string()]).expect("boot mode should parse");
        assert_eq!(options.mode, RuntimeMode::Boot);
    }
}

#[test]
fn runtime_supports_midi_demo_mode() {
    let options = parse_runtime_options(vec!["midi_demo".to_string(), "example.mid".to_string()])
        .expect("midi_demo mode should parse");
    assert_eq!(
        options.mode,
        RuntimeMode::MidiDemo {
            midi_path: "example.mid".to_string()
        }
    );
}
