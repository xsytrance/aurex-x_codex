# What Is a Pulse?

A Pulse is a playable or viewable Aurex-X experience package.

A Pulse can be:
- a game
- a procedural world
- a visual music journey
- a cinematic demo
- an ambient environment

Each Pulse describes metadata plus which scene and systems to run. The Pulse runtime then loads it, updates it over time, renders frames, and shuts it down cleanly.

Multiple pulses can be chained in a **PulseGraph** for larger journeys.

Example PulseGraph flow:
- AmbientIntroPulse
- PsytranceTunnelPulse
- InfiniteCircuitCityPulse
- FractalFinalePulse

Transitions can be:
- manual (player trigger)
- timeline based (after a duration)
- musical cue based (RhythmField events)
- generator/scene event based

Boot World is a special pulse that acts as a hub. Its districts (Electronic, Jazz, Rock, Ambient) contain portals. Walking near a portal can trigger a graph transition into the linked pulse.

Pulses also contribute to a **Resonance Tracker** profile that measures your affinity with Prime factions over time. District exploration and portal launches in Boot World can increase those affinities.

This profile is planned to power Living Boot Screen personalization and pulse recommendations.

The Living Boot Screen reacts to your top resonance primes and idle behavior:
- first long idle period gives a warning
- later long idle periods prepare resonance events

These events are currently state-only (no major world mutation yet), so behavior stays predictable while future systems are added.

At the center of Boot World is the Prime Pulse megastructure. It has layered resonance gates:
- as your resonance profile grows, more layers unlock
- until then, a force-field state remains active

Current implementation exposes progression and proximity state for future world changes and Pulse Navigator-style features.
