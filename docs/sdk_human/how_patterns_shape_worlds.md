# How Patterns Shape Worlds (Human Guide)

Pattern networks give Aurex-X visual personality without texture files.

## What they do
- add carved lines, glyphs, spirals, veins, rails, and panel motifs
- react to music (kick/hat/melody energy)
- react to rhythm structure (beat/measure/phrase)
- make worlds feel authored and faction-like

## Built-in style bundles
- **ElectronicCircuit**: motherboard traces + panel lines
- **PsySpiral**: tunnel spirals + flowing wave warps
- **PrimePulseTemple**: ceremonial concentric/glyph motifs
- **OperaCathedral**: carved resonance ornament
- and more (`JazzLoungeGlow`, `ReggaeSunwave`, `ClassicalOrnament`, `HipHopSignal`)

## Practical workflow
1. Add a scene-level pattern preset in `sdf.patterns`.
2. Let generator materials inherit it automatically.
3. Override specific materials with custom `pattern_network` if needed.
4. Bind reactivity to `High` for crisp flicker, `Low` for heavy pulses, `Phrase` for large shifts.

## Visual intent tips
- Keep `scale` moderate first (1.5–4.0).
- Use `Mask`/`Warp` to avoid noisy clutter.
- Pair rhythm-space with pattern reactivity for cinematic motion.
