# Prime Pulse Megastructure System (Technical SDK)

Prime Pulse is the central Boot World megastructure representing the universal metronome.

## Module
`crates/aurex_pulse/src/prime_pulse.rs`

## Core types
- `PrimePulseState`
- `PrimePulseLayer`

## Layered resonance gates
Each layer has deterministic requirements:
- `required_prime_count`
- `required_resonance`

Default configuration:
- layer 1: 3 primes >= 0.25
- layer 2: 6 primes >= 0.35
- layer 3: 9 primes >= 0.50
- core: 12 primes >= 0.60

## Runtime behavior
Per update:
1. measure player distance to Prime Pulse
2. evaluate layer unlock conditions from `ResonanceProfile`
3. update active layer / unlocked layer count
4. expose force-field and modulation diagnostics

No collision/physics or world mutation is performed yet.

## Exposed modulation hooks
- `prime_pulse_intensity`
- `prime_pulse_proximity`

These are intended for future audio/visual systems.

## Diagnostics
Pulse diagnostics expose:
- `prime_pulse_distance`
- `prime_pulse_layer`
- `prime_pulse_layers_unlocked`
- `prime_pulse_force_field_active`
- `prime_pulse_intensity`
- `prime_pulse_proximity`
