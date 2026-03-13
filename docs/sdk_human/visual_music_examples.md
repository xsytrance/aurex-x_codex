# Visual Music Examples

Aurex ships three visual-music showcase pulses:
- **Electronic Megacity**
- **Jazz Atmosphere**
- **Ambient Dreamscape**

They are built with `PulseBuilder`, which lets developers (and future tools) define pulse intent with simple hints, then map them to the existing world + rhythm systems.

Each pulse follows the same deterministic flow:
1. Builder config defines world identity and style hints.
2. Base generator parameters are created.
3. RhythmField signals modulate those parameters.
4. Renderer executes the unchanged pipeline.

Same pulse type + same seed gives the same output.

## Time-based phases
Some showcase pulses also use a phase sequence so scenes evolve in clear chapters while staying deterministic.
