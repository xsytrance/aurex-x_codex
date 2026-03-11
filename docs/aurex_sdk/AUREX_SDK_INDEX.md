# Aurex SDK Index

Aurex is a real-time audiovisual demo engine organized as modular Rust crates for app lifecycle, rendering, audio, ECS, lighting, and post-processing.

This SDK section defines the **data contract layer** for AI-assisted content generation, starting with the Scene IR specification and node parameter manifests.

## How scenes are described

Scenes are represented as JSON documents matching the Scene IR schema documented in [scene_ir.md](./scene_ir.md). A scene describes:

- scene metadata (`name`)
- geometry/control nodes (`nodes`)
- optional camera path (`camera`)
- optional audio-reactive bindings (`sync`)

## How LLMs should generate content

When generating Aurex scene content, LLMs should:

1. Emit valid Scene IR JSON first.
2. Use supported node names and parameter ranges from [geometry_nodes.md](./geometry_nodes.md).
3. Prefer machine-readable validation against [`spec/geometry_nodes.json`](./spec/geometry_nodes.json).
4. Keep references stable (`node.id`) so tooling and future runtime systems can bind updates safely.

## Specification files

- Human-readable Scene IR: [scene_ir.md](./scene_ir.md)
- Human-readable node catalog: [geometry_nodes.md](./geometry_nodes.md)
- Human-readable audio notes: [audio_system.md](./audio_system.md)
- Machine-readable node schema: [`spec/geometry_nodes.json`](./spec/geometry_nodes.json)
