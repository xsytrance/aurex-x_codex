# Aurex X – AI Handoff Document

Project: Aurex X
Type: Procedural fantasy console / demoscene-inspired runtime
Language: Rust
Goal: Build a lightweight procedural game platform with strong audiovisual synthesis.

Aurex X is not a traditional game engine and not a full 3D engine.
It is a constrained runtime designed to generate graphics, audio, and gameplay systems procedurally.

The platform emphasizes:
• procedural visuals
• procedural audio
• small game packages
• ECS-based gameplay
• strong audiovisual synchronization

Games contain almost no assets.

Typical game packages include:
• gameplay logic
• procedural generators
• music patterns
• parameter sets

---

CORE DESIGN PHILOSOPHY

Aurex X is inspired by the demoscene.

Important influences include audiovisual demos where small programs generate complex visuals and music through math and synthesis.

Key constraints:

• no sprite sheets
• no imported meshes
• no traditional texture pipelines
• geometry generated procedurally
• audio generated procedurally
• lighting and post-processing stylized but constrained

The system aims to create visually rich results from minimal data.

---

WORLD MODEL

Aurex uses a 2.5D model:

• gameplay occurs on a primary plane
• camera moves freely in 3D space
• geometry is procedural

Rendering stack:

Procedural Geometry
↓
Material System
↓
Lighting Engine
↓
Particle Engine
↓
Post Processing
↓
Final Frame

---

MAJOR SYSTEMS

Conductor

Central orchestrator responsible for:

• frame scheduling
• system synchronization
• performance budgets
• resonance calculations

---

ECS Runtime

Entity Component System architecture.

Entities contain components such as:

transform
velocity
shape
material
particle_emitter
audio_emitter
collider

Systems process entities deterministically each frame.

---

Shape Synth Unit (SSU)

Responsible for procedural geometry generation.

Supported primitives include:

circle
rectangle
polygon
ring
tube
grid
extrusion
fractal structures

Geometry should be lightweight and instanced efficiently.

---

Material System

Procedural shading styles applied to shapes.

Material types include:

flat
neon
chrome
crystal
noise
plasma
wireframe

Materials react to lighting and post-processing.

---

Particle Engine

Handles large numbers of particles for visual effects.

Examples:

sparks
dust
energy trails
fractals
glow fields

Particles often react to audio events.

---

Lighting Engine

Stylized dynamic lighting system.

Constraints:

maximum lights: ~12

Types:

ambient
point
spot
pulse

Lighting emphasizes artistic style rather than physical realism.

---

Post Processing Pipeline

Built-in post effects include:

bloom
motion trails
fog
color grading
distortion
CRT-style filters

Developers configure parameters rather than writing shaders.

---

Aurex Sound Unit (ASU)

Procedural music and sound synthesis.

Capabilities:

synth voices
drum generation
pattern sequencer
audio events

Audio events drive visuals.

Example events:

beat
kick
snare
drop

Visual systems can respond to these events.

---

RESONANCE SYSTEM

The console tracks player interaction patterns.

Two levels exist:

System Resonance
Profile Resonance

Resonance influences:

• boot visuals
• library environment
• Prime Pulse experience
• audio themes

Resonance aligns with one of the Prime factions.

---

PRIME PULSE EXPERIENCE

Aurex includes a built-in exploration experience called Prime Pulse.

Players explore the interior realm of the Prime Pulse.

This experience demonstrates the capabilities of the Aurex engine.

Movement style:

first-person exploration
smooth gliding movement

Players are known as Resonant Explorers.

---

MANIFEST SYSTEM

Players may temporarily manifest into stylized energy forms.

Manifestation enables limited interactions such as:

energy pulses
geometry manipulation
particle bursts

Different Prime zones demonstrate different interaction styles.

---

PRIME ZONES

Each Prime Maestro faction has a themed region.

Examples:

Electronic Zone
rhythm targeting interactions

Jazz Zone
geometry improvisation

Rock Zone
shockwave mechanics

Classical Zone
harmonic puzzle structures

These zones demonstrate potential gameplay styles for Aurex games.

---

LORE SYSTEM

Story fragments called Echoes are scattered throughout the Prime Pulse.

Echo fragments may appear as:

audio recordings
visual events
symbolic structures
music layers

Players piece together the lore by exploration.

---

THE BASSLINE

Beneath the Prime Pulse lies a subterranean region called the BassLine.

This region represents the foundational rhythm of the universe.

Characteristics include:

massive resonance structures
speaker-like architecture
subharmonic bass environments

Music in the BassLine emphasizes deep bass elements such as:

psytrance bass
funk bass
techno basslines
dub-style pulses

The BassLine may predate the Prime Pulse itself.

---

DEVELOPMENT GOAL

Initial implementation goal:

Create a working renderer prototype capable of:

• window creation
• 3D camera
• procedural shape rendering
• basic lighting
• bloom post-processing

This prototype forms the foundation of the Aurex engine.

Additional information:

---

## AUREX LIBRARY SYSTEM

The Aurex Library is the primary interface where users browse
and launch games.

The library is not a traditional menu.

It is a procedural 3D environment where games appear as
floating worlds or portals.

Library characteristics:

• fully procedural environment
• influenced by Prime faction resonance
• audio-reactive visuals
• dynamic lighting and particle effects

Each game appears as a "world node".

When the player approaches a node they see:

• preview visuals
• music snippet
• game metadata
• trophy progress

Game packages are stored as:

.axg files

The library scans installed packages and generates a visual
representation for each game.

The Prime Pulse experience is also accessible through the
library as the central portal.

---

## RESONANCE SYSTEM

The Resonance Engine tracks player interaction patterns
and aligns the console with Prime factions.

Two resonance layers exist:

System Resonance
Profile Resonance

System resonance influences:

• boot screen theme
• library environment
• Prime Pulse audio themes

Profile resonance influences:

• player titles
• avatar appearance
• trophy progression

Resonance is calculated based on:

• game genres played
• music patterns triggered
• Prime zones explored
• gameplay frequency

Resonance values gradually decay over time so the system
remains dynamic.

---

## STORAGE SYSTEM

All Aurex data lives under a single root directory.

Example structure:

/aurex
├ system
├ library
├ profiles
├ resonance
├ cache
└ devport

Library
stores installed game packages (.axg)

Profiles
stores user profiles (.axp)

Resonance
stores system resonance data

System
stores configuration, trophies, and engine data

Devport
temporary location for uploaded games from creator tools

Cache
temporary runtime-generated assets

The storage system is intentionally simple so the console
can easily be backed up or migrated.

---

## TROPHY SYSTEM

Aurex includes a built-in achievement system similar to
modern consoles.

Trophies are tracked at the system level.

Types of trophies include:

Game trophies
System trophies
Faction trophies
Exploration trophies

Examples:

Pulse Initiate
first game played

Neon Adept
high Electronic resonance

Resonant Explorer
discover multiple Prime zones

Trophies contribute slightly to profile resonance and
player titles.

The trophy system also serves as a long-term progression
system for the Aurex platform.
