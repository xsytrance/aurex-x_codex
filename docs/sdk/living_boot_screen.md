# Living Boot Screen System (Technical SDK)

The Living Boot Screen is a Boot World behavior layer that adapts presentation from the player's resonance profile.

## Module
`crates/aurex_pulse/src/living_boot.rs`

## Core types
- `LivingBootPresentation`
- `IdleResonanceEventState`
- `LivingBootState`

## Presentation model
`LivingBootPresentation` tracks:
- `dominant_prime`
- `top_three_primes`
- `visual_bias_weights`
- `audio_bias_weights`

Weights are derived from normalized resonance values and intended as non-invasive modulation hooks.

## Idle resonance behavior
`IdleResonanceEventState` tracks:
- `idle_time_seconds`
- `warning_issued`
- `event_count`
- `last_event_time`
- `resonance_event_ready`

Deterministic rule:
- first long-idle threshold => warning only
- subsequent thresholds => resonance event-ready state

## Runtime integration
- Boot World initializes `LivingBootState` from `ResonanceTracker` profile.
- Per update:
  - presentation is refreshed from live resonance values
  - idle timers update from interaction/no-interaction state
- Diagnostics expose Living Boot idle/event state.

## Future compatibility
This is a state framework for future systems:
- personalized boot visuals/audio
- Prime Pulse warnings and world changes
- Boot World evolution hooks
