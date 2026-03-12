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
