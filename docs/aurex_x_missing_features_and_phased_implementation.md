# Aurex-X Missing Features Review and Phased Implementation Plan

**Purpose:** This document compares the uploaded older Aurex-X docs against the later design/runtime discussions and identifies the major features and systems that are missing or under-specified. It then proposes a phased technical implementation plan intended for Codex/agents to execute in a disciplined order.

**Audience:** Codex, Cursor, engineering agents, and future maintainers.

**Status context:** The uploaded docs represent an older architecture snapshot. They already cover core vision, pipeline structure, Pulse runtime concepts, timelines, sequencer, RhythmField, builder APIs, and AI authoring interfaces. They do **not** fully capture many later discussions around runtime stabilization, real renderer handoff, safe/legacy geometry modes, user-facing authoring UX, optional LLM connectivity, or MIDI-driven Pulse generation.

---

# 1. What the older docs already cover well

The uploaded docs already establish a strong foundation in these areas:

- procedural-first fantasy console vision
- demoscene-inspired runtime philosophy
- ECS/runtime contracts
- Conductor/boot/runtime architecture
- render pipeline stage order
- Pulse runtime and PulseBuilder concepts
- Timeline / PulseSequence / transition engine concepts
- procedural world generation and GeneratorStack ideas
- deterministic music sequencer and RhythmField concepts
- AI handoff and authoring surfaces for generated pulses

This means the project does **not** need another top-level vision rewrite. The gap is not ideology. The gap is the set of **later-discussed systems, stabilization rules, and product-facing workflows** that are not yet reflected in the docs.

---

# 2. Missing or under-documented features we discussed later

Below is the feature gap list relative to the older docs.

## 2.1 Real runtime stabilization and debug architecture

The older docs describe the intended pipeline, but they do not document the practical runtime stabilization systems discussed later:

- explicit boot → procedural handoff state machine
- persistent render mode latching (`Boot`, `Procedural`)
- procedural warmup frames before first real scene content
- delayed audio unlock after handoff
- procedural setup stage diagnostics
- SDF stage diagnostics (`sdf_stage_begin/end`)
- diagnostic environment switches for runtime isolation
- flat / safe / legacy GeometrySdf modes
- crash/freeze isolation strategy and fallback behavior

These are critical because the current engine has already shown that architecture-level intent is not enough; the runtime needs explicit stabilization and observability controls.

## 2.2 GeometrySdf mode architecture

The older docs describe a raymarch renderer and SDF composition, but they do **not** describe a multi-mode operational model for GeometrySdf:

- `flat` mode: no raymarch, solid framebuffer, purely diagnostic
- `safe` mode: minimal bounded raymarch scene (sphere/plane only)
- `legacy` mode: full recursive/dynamic scene path for later A/B comparison

This mode separation is now essential for practical development.

## 2.3 Runtime isolation / debug flags contract

The older docs do not define the runtime debug/isolation contract used during stabilization. Missing items include env-driven controls such as:

- `AUREX_STOP_AFTER_PROCEDURAL_STAGE`
- `AUREX_STOP_AFTER_SDF_STAGE`
- `AUREX_BYPASS_PROCEDURAL_SETUP`
- `AUREX_DIAGNOSTIC_GPU_TRIANGLE`
- `AUREX_FORCE_FLAT_RENDER`
- `AUREX_DISABLE_AUDIO`
- `AUREX_DISABLE_GPU_ERROR_SCOPES`
- `AUREX_LOG_ONLY_PROCEDURAL_TRANSITION`
- `AUREX_SKIP_ROOT_TREE_BUILD`
- `AUREX_SKIP_PROCEDURAL_CAMERA`
- `AUREX_GEOMETRY_SDF_MODE`

These should be documented as an official engineering-only contract.

## 2.4 Procedural renderer handoff discipline

The older docs imply a clean Pulse runtime, but later work showed the need for stricter rules:

- boot renderer must remain isolated until handoff
- procedural mode must activate only once
- no fallback to boot/demo visuals after entering procedural mode
- first procedural present should be explicitly observable
- scene setup should not mutate render mode mid-frame

These state-machine rules are missing from the earlier docs.

## 2.5 Pulse/runtime observability and diagnostics schema

The earlier docs mention diagnostics broadly, but they do not define a structured diagnostics schema for runtime debugging. Missing concepts include:

- render mode diagnostics
- handoff readiness and warmup counters
- active scene vs rendered scene distinction
- procedural frame counters
- stage duration/failure markers
- bounded warning logs (`geometry_sdf_warning`, `raymarch_abort`, etc.)
- operational confidence states (`safe`, `fallback`, `legacy`, `flat`)

## 2.6 Optional LLM connectivity as a first-class but non-mandatory subsystem

We discussed a major design decision later:

> Aurex should be able to connect to an LLM of the player’s choice (online or offline), **but this must remain optional**.

The older docs do not capture this.

The later architecture direction was:

- Aurex must remain fully self-contained and viable without an LLM
- LLM integration should be a bonus layer for:
  - assisted Pulse generation
  - assisted world authoring
  - character/dialogue generation
  - in-world responsive systems
  - creative copilot functions
- the runtime should support both:
  - local/offline model endpoints
  - networked/online providers

This is one of the most important missing product/architecture decisions in the docs.

## 2.7 Pulse generation interface for end users (beginner/pro hybrid)

The older docs contain AI-facing PulseBuilder and pulse generation interface material, but not the full product-facing concept discussed later:

- a web or app-based Pulse Builder UI
- beginner mode with guided fields and defaults
- pro mode exposing lower-level controls
- hybrid mode where an LLM fills missing fields/context for the user
- preview window and safe iterative generation flow

We discussed the design direction:

- **Option A** style beginner experience now
- but implemented in a way that preserves a future **Option B / pro mode**
- LLM can guide the user and infer missing values

That needs to be captured explicitly.

## 2.8 MIDI → automatic Pulse generation

This is a major missing feature.

The older docs cover the sequencer and music authoring systems, but not the newer concept:

> Aurex should accept a MIDI file, analyze it, and automatically generate a Pulse/demo from it.

We explicitly chose:

- **Option B:** MIDI should generate a full structured Pulse/demo automatically

not just direct real-time visual modulation.

Important implications:

- MIDI ingestion layer
- MIDI feature extraction
- section/phrase analysis
- automatic scene generation
- Pulse config emission
- editable output file (`.pulse`, JSON, or similar)

This is a major missing feature and should become its own subsystem.

## 2.9 Pulse package/output format for generated experiences

The docs mention `.axg`, Pulse definitions, and JSON graph examples, but the newer discussions imply the need for a more explicit generated Pulse artifact:

- generated Pulse file format
- deterministic seeds + authored scene plans
- reusable / editable generated configs
- ability to emit Pulse configs from AI/MIDI tools

This should be formalized.

## 2.10 Authoring pipeline for the Aurielle intro / living boot sequence

The older docs mention living boot screen concepts, cinematic systems, and the Prime Pulse boot identity, but they do not yet capture the specific later direction around:

- cinematic scripted intro construction
- Aurielle intro / conductor-style boot ritual
- giant `AUREX-X` logo climax
- phased reveal design
- safe runtime path for intro orchestration

This doesn’t need to be overfit to one intro, but the doc set should define how “boot ritual pulses” are authored safely.

## 2.11 Formal recovery / rollback / compare workflow

Given how much time was spent stabilizing runtime behavior, the docs need a development operations section for:

- preserving safe checkpoints
- compare-vs-rollback methodology
- branch naming for stabilization work
- what counts as “last known good state”
- how to re-apply useful patches from unstable branches

The earlier docs do not include this operational discipline.

---

# 3. High-level feature gap summary

The missing features fall into four groups:

## 3.1 Stability / engineering gap

- handoff state machine
- SDF mode architecture
- runtime debug toggles
- diagnostics schema
- recovery workflow

## 3.2 Product / authoring gap

- beginner/pro Pulse Builder UX
- preview loop
- editable generated Pulse files

## 3.3 Intelligence / external systems gap

- optional LLM connectivity
- local/remote provider abstraction
- agent-facing authoring contracts beyond static docs

## 3.4 Music-driven generation gap

- MIDI ingestion
- MIDI analysis
- MIDI → automatic Pulse generation

---

# 4. Recommended phased implementation plan

This is the plan Codex should follow. It is intentionally sequenced so that foundation and stability come before user-facing features.

---

# Phase 0 — Documentation and contract consolidation

## Goals

- make the current state of the engine legible
- define missing contracts before new systems branch out

## Deliverables

1. New engineering docs
   - `docs/sdk/runtime_debug_contract.md`
   - `docs/sdk/geometry_sdf_modes.md`
   - `docs/sdk/runtime_handoff_state_machine.md`
   - `docs/sdk/recovery_and_rollback_workflow.md`

2. Update existing docs
   - `docs/sdk/engine_architecture.md`
   - `docs/sdk/pulse_runtime_system.md`
   - `docs/sdk/renderer_stages.md`
   - `docs/sdk_ai/pulse_generation_interface.md`

3. Formal runtime contracts
   - environment variable matrix
   - mode/state diagram
   - diagnostics schema

## Notes for Codex

This phase is doc-first and should not change behavior unless documentation reveals a missing constant/enum that should be codified.

---

# Phase 1 — Runtime stabilization floor

## Goals

- ensure the engine can always produce a procedural frame safely
- establish an always-works renderer path

## Required implementation

1. Finalize GeometrySdf mode contract
   - `flat`
   - `safe`
   - `legacy`
   - default should remain `safe`

2. Ensure `safe` mode is independent of the dynamic scene tree
   - no recursion
   - no modifiers/CSG
   - no legacy traversal
   - render sphere + plane only

3. Complete handoff state-machine contract
   - boot mode latched
   - one-shot transition
   - warmup persistence
   - first procedural present event

4. Add runtime confidence state
   - `BOOT`
   - `PROCEDURAL_SAFE`
   - `PROCEDURAL_LEGACY`
   - `FALLBACK`

5. Add low-noise diagnostics
   - bounded logs
   - per-transition logs only
   - optional verbose mode

## Exit criteria

- `aurielle_intro` survives handoff
- safe mode continues rendering frames
- app remains responsive on Windows
- no crash in default configuration

---

# Phase 2 — Safe procedural rendering expansion

## Goals

- make safe mode visually useful while keeping it deterministic and bounded

## Required implementation

1. Expand safe GeometrySdf in controlled steps
   - sphere
   - plane
   - box / ring
   - simple normals
   - simple directional/ambient lighting
   - bounded shadows (optional)

2. Add camera stability rules
   - clamped FOV
   - fixed near/far assumptions
   - safe camera presets for intros/worlds

3. Add scene composition presets for safe mode
   - megacity-lite
   - aurielle-lite
   - ambient-lite
   - jazz-lite

These should not yet use arbitrary scene trees. They should be generated from constrained templates.

## Exit criteria

- different pulses visibly diverge in safe mode
- visuals are no longer just a flat placeholder
- no freezes/crashes in safe mode

---

# Phase 3 — Legacy GeometrySdf rehabilitation

## Goals

- make the old rich SDF path optionally usable again
- reintroduce complexity in a controlled way

## Required implementation

1. Legacy path isolation
   - keep legacy explicitly opt-in
   - compare image signatures vs safe mode where possible

2. Reintroduce features one by one
   - transforms
   - multiple primitives
   - scene-tree traversal
   - simple CSG
   - modifiers
   - warps
   - shadows
   - temporal feedback interactions

3. Add watchdogs everywhere
   - traversal depth
   - march limits
   - node count limits
   - operation count limits

4. Add stage-level benchmark telemetry

## Exit criteria

- legacy path stable for at least one known scene
- no infinite traversal/raymarch
- clear parity/benefit over safe mode

---

# Phase 4 — Pulse file and generation contracts

## Goals

- define a durable artifact for generated experiences
- make generated pulses editable and reusable

## Required implementation

1. Define Pulse package/config format
   - recommended: JSON or TOML-backed authoring format first
   - optional binary/package wrapper later

2. Minimum Pulse fields
   - metadata
   - seed
   - scenes
   - timeline events
   - music/sequencer references
   - visual theme
   - camera/style presets
   - optional LLM generation provenance

3. Add deterministic serialization tests

4. Add versioned schema doc
   - `docs/sdk/pulse_file_format.md`

## Exit criteria

- generated pulses can be saved and reloaded
- Pulse Builder and future MIDI generator both target the same schema

---

# Phase 5 — Beginner/pro Pulse Builder UX contract

## Goals

- define and implement the user-facing Pulse Builder structure discussed later

## Product direction

We explicitly discussed a hybrid strategy:

- beginner mode like “Option A”
- architecture that supports “Option B” / pro mode later
- optional LLM help to fill gaps

## Required implementation

1. Builder schema split
   - `PulseIntent` (high-level user intent)
   - `PulseConfig` (resolved full config)

2. Beginner mode input surface
   - name
   - theme
   - mood
   - energy
   - density
   - camera style
   - audio reactivity
   - duration

3. Pro mode input surface
   - expose advanced scene/timeline controls
   - expose generator controls
   - expose camera and transition tuning

4. Resolver layer
   - maps beginner intent to full PulseConfig
   - can be optionally LLM-assisted

5. Preview architecture
   - preview uses safe renderer path first
   - legacy preview optional later

## Deliverables

- `docs/sdk/pulse_builder_user_interface.md`
- `docs/sdk_ai/pulse_intent_resolution.md`

## Exit criteria

- a user can author a Pulse without touching low-level scene graph details
- the same builder can evolve toward pro mode without breaking file formats

---

# Phase 6 — Optional LLM provider layer

## Goals

- add optional intelligence without making Aurex dependent on it

## Non-negotiable rule

Aurex must remain fully functional with **no LLM connected**.

## Required implementation

1. Provider abstraction
   - local/offline endpoint
   - remote API endpoint
   - null provider (default)

2. LLM use cases
   - pulse intent resolution
   - scene naming/themes
   - world text / codex entries
   - character/system flavor text
   - guided Pulse Builder assistance

3. Provider safety contract
   - timeouts
   - deterministic fallback behavior
   - no runtime-critical dependency

4. Docs
   - `docs/sdk/llm_provider_interface.md`
   - `docs/sdk_ai/llm_augmented_pulse_authoring.md`

## Exit criteria

- LLM is an enhancement layer only
- offline/local model support is possible
- Aurex remains self-contained forever

---

# Phase 7 — MIDI → automatic Pulse generation

This is the major feature you explicitly chose.

## Product decision already made

We chose:

**MIDI should generate a full structured Pulse/demo automatically**.

Not just direct visual response.

## Architecture

```text
MIDI file
↓
MIDI parser
↓
temporal / structural analysis
↓
feature extraction
↓
Pulse blueprint generation
↓
Pulse config file
↓
editable Pulse
```

## Required subsystem split

1. `aurex_midi`
   - parser wrapper around `midly` or equivalent
   - file loading
   - track extraction
   - tempo map extraction

2. `aurex_music_analysis`
   - beat grid
   - bar/phrase segmentation
   - note density analysis
   - velocity/energy analysis
   - register/instrument/channel analysis
   - section boundary detection

3. `aurex_pulse_generation`
   - map MIDI features → scene/timeline plan
   - emit Pulse config

## Initial mapping rules

Examples:

- intro / sparse section → low geometry, wider camera, fewer particles
- build section → increasing particle density and glow
- drop → stronger camera motion, more geometry, brighter lighting
- breakdown → fog, reduced density, slower transitions
- high note density → more particle activity
- low register energy → more structural weight / ground emphasis

## MVP output

CLI command:

```text
aurex pulse generate song.mid
```

Output:

```text
song.pulse.json
```

## Next-level expansion

- DAW export workflow
- multi-track role assignment
- user override pass after generation
- live MIDI visualization mode later (separate feature)

## Exit criteria

- a MIDI file can generate an editable Pulse config
- generated pulse runs through the same Pulse runtime path as hand-authored pulses

---

# Phase 8 — Boot ritual / intro authoring system

## Goals

- formalize the kind of intro experience discussed around Aurielle and the giant AUREX-X logo

## Required implementation

1. Boot ritual pulse type
   - strongly scripted timeline
   - privileged camera/transition presets
   - title/logo reveal support

2. Boot-safe rendering requirement
   - defaults to safe GeometrySdf mode unless user explicitly opts in otherwise

3. Intro authoring schema
   - reveal phases
   - logo timing
   - scene overlays
   - audio cue plan

4. Diagnostics
   - explicit boot/intro state machine logs

## Exit criteria

- intros no longer require ad hoc engine hacking
- Aurielle intro can be authored as data, not hardcoded behavior

---

# Phase 9 — Development operations and confidence workflow

## Goals

- avoid repeating the current stabilization pain

## Required implementation

1. known-good runtime matrix
   - safe/default modes
   - expected behavior by mode

2. compare workflow
   - current head vs known-good commit
   - branch naming conventions

3. rollback docs
   - how to checkpoint before experimental renderer work

4. CI recommendations
   - tests for handoff progression
   - safe-mode smoke tests
   - SDF stage stop tests
   - generated Pulse schema validation

---

# 5. Priority order for Codex

If Codex must choose what to work on first, the order should be:

1. **Phase 1** — runtime stabilization floor
2. **Phase 2** — safe procedural rendering expansion
3. **Phase 4** — Pulse file/schema
4. **Phase 5** — beginner/pro Pulse Builder contract
5. **Phase 7** — MIDI → Pulse generation
6. **Phase 6** — optional LLM provider layer
7. **Phase 8** — boot ritual / intro authoring system
8. **Phase 3** — legacy GeometrySdf rehabilitation (only after safe mode is useful)
9. **Phase 9** — ops/CI/documented compare workflow

Why this order:

- stability before creativity
- file/schema before generation tools
- generation tools before UX polish
- optional intelligence only after deterministic core is stable

---

# 6. Recommended immediate next action

For the very next concrete implementation step, Codex should:

1. verify that `safe` mode really uses no legacy scene-tree traversal
2. make safe mode visually distinct but still trivially bounded
3. get `aurielle_intro` and `megacity` both rendering stably in safe mode
4. only then move to Pulse file schema and MIDI generation

---

# 7. Final note

The old docs are not wrong. They are simply missing the later engineering reality:

- the runtime needs a stable operating floor
- the renderer needs operational modes
- generation must target a concrete Pulse artifact
- LLM support must be optional
- MIDI → Pulse generation is a major product opportunity and should be treated as a first-class subsystem, not a side experiment

