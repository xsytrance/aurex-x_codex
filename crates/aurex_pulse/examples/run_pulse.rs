use aurex_pulse::{loader::load_pulse_from_path, runner::PulseRunner};
use aurex_render_sdf::RenderConfig;

fn main() {
    let pulse_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/pulses/infinite_circuit_megacity.pulse.json".to_string());
    let pulse = load_pulse_from_path(&pulse_path).expect("pulse should load");
    let mut runner = PulseRunner::load(pulse, Some(std::path::Path::new(&pulse_path)))
        .expect("pulse runner should load");

    runner.initialize();
    runner.update(1.0 / 60.0);
    let frame = runner.render(RenderConfig::default());
    runner.shutdown();

    println!(
        "pulse={} state={:?} frame={}x{} lifecycle_ms(load={:.3}, init={:.3}, update={:.3}, render={:.3}, shutdown={:.3})",
        pulse_path,
        runner.state,
        frame.width,
        frame.height,
        runner.diagnostics.lifecycle.load_ms,
        runner.diagnostics.lifecycle.initialize_ms,
        runner.diagnostics.lifecycle.update_ms,
        runner.diagnostics.lifecycle.render_ms,
        runner.diagnostics.lifecycle.shutdown_ms,
    );

    if let Some(rf) = runner.rhythm_field() {
        println!(
            "rhythm_field tempo={:.1} phase={:.3} strength={:.3} beat={} bar={} phrase={} bass={:.3} harmonic={:.3} flux={:.3} groove=[{:.3},{:.3},{:.3}]",
            rf.tempo,
            rf.beat_phase,
            rf.beat_strength,
            rf.beat_index,
            rf.bar_index,
            rf.phrase_index,
            rf.bass_energy,
            rf.harmonic_energy,
            rf.spectral_flux,
            rf.groove_vector[0],
            rf.groove_vector[1],
            rf.groove_vector[2]
        );
    }

    if let Some(summary) = runner.diagnostics.rhythm_summary {
        println!(
            "rhythm_summary phase={:.3} bar={} bass={:.3}",
            summary.beat_phase, summary.bar_index, summary.bass_energy
        );
    }

    if let Some(prime) = runner.diagnostics.dominant_prime {
        println!(
            "resonance dominant_prime={:?} top_three={:?}",
            prime, runner.diagnostics.top_three_primes
        );
    }

    println!(
        "living_boot idle_time_seconds={:.3} warning_issued={} event_count={} resonance_event_ready={}",
        runner.diagnostics.idle_time_seconds,
        runner.diagnostics.warning_issued,
        runner.diagnostics.resonance_event_count,
        runner.diagnostics.resonance_event_ready
    );

    println!(
        "prime_pulse distance={:.3} layer={} unlocked_layers={} force_field_active={} intensity={:.3} proximity={:.3}",
        runner.diagnostics.prime_pulse_distance,
        runner.diagnostics.prime_pulse_layer,
        runner.diagnostics.prime_pulse_layers_unlocked,
        runner.diagnostics.prime_pulse_force_field_active,
        runner.diagnostics.prime_pulse_intensity,
        runner.diagnostics.prime_pulse_proximity,
    );

    if let Some(diag) = &runner.diagnostics.last_frame {
        println!(
            "renderer stages={:?} total_ms={:.3}",
            diag.stages, diag.total_frame_time_ms
        );
    }
}
