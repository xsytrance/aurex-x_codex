# Aurex X System Architecture

Aurex is orchestrated by a central system called the Conductor.

The Conductor manages:

• frame scheduling
• subsystem synchronization
• performance budgets
• resonance tracking

Core runtime systems:

Conductor
├ ECS Runtime
├ Shape Synth Unit (SSU)
├ Material System
├ Particle Engine
├ Lighting Engine
├ Post Processing Pipeline
├ Aurex Sound Unit (ASU)
├ Resonance Engine
├ Library System
└ Trophy System

The ECS Runtime drives gameplay logic.

Entities consist of components such as:

transform
velocity
shape
material
particle emitter
audio emitter
collider

Systems operate on these components deterministically.

Rendering Pipeline:

Procedural Geometry
↓
Material Shading
↓
Dynamic Lighting
↓
Particles
↓
Post Processing
↓
Final Frame

All rendering is procedural and asset-light.

Aurex uses a 2.5D world model:

• gameplay occurs on a primary plane
• camera moves in full 3D space

This allows cinematic visual effects while keeping gameplay simple.
