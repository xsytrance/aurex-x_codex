# Resonance Tracker System (Technical SDK)

`ResonanceTracker` models player affinity across 12 Prime factions and is designed for deterministic runtime updates.

## Module
`crates/aurex_pulse/src/resonance.rs`

## Core types
- `PrimeFaction`
- `ResonanceValue`
- `ResonanceProfile`
- `ResonanceTracker`

## Prime factions
- Pop (`Aurielle`)
- Rock (`Lord Riffion`)
- Hip-Hop / Rap (`MC Baraka`)
- Electronic (`DJinn`)
- R&B / Soul / Funk (`Velouria Groove`)
- Classical (`Octavius Audius Rudwig`)
- Jazz (`Blue Rondo`)
- Country / Folk (`Dust Strummer`)
- Reggae / Caribbean / Afrobeat (`Oba Fyah Irie`)
- World / Traditional (`Terra Sonora`)
- Ambient / Experimental (`Aetherion`)
- Play / Toy / Cartoon (`Kazoom`)

## Runtime behavior
- pulse affinity contributes incremental resonance over time (`update_from_pulse`)
- district visits raise activity and visit counters (`record_district_visit`)
- pulse launches increase pulse count and resonance (`record_pulse_launch`)

## Integration points
- `PulseRunner` owns optional `resonance_tracker`
- `PulseDefinition.metadata.prime_affinity` maps to `PrimeFaction`
- Boot World district enter/portal launches can feed tracker events
- pulse diagnostics expose:
  - `dominant_prime`
  - `top_three_primes`

## Determinism
For fixed pulse definitions + fixed dt progression, resonance updates are deterministic.

## Future systems
Resonance data is intended to drive:
- Living Boot Screen
- Pulse recommendations
- Boot World evolution
- Prime megastructure unlock logic
- player identity/profile systems
