# Procedural Pipeline

Geometry is defined as an ordered pipeline:

`base primitive -> modifier[0] -> modifier[1] -> ...`

Supported modifiers:
- `repeat`
- `twist`
- `bend`
- `scale`
- `rotate`
- `translate`
- `noise_displacement`
- `mirror`

## Example
```json
{
  "base": {"type":"sphere","radius":{"kind":"value","value":1.0},"seed":1},
  "modifiers": [
    {"type":"repeat","count":16,"spacing":{"kind":"value","value":2.0}},
    {"type":"twist","angle":{"kind":"expression","expression":"time"}},
    {"type":"noise_displacement","strength":{"kind":"value","value":0.2},"frequency":{"kind":"value","value":1.5},"seed":21}
  ]
}
```

Expected visual effect: repeated tunnel elements with temporal corkscrew motion and seeded surface breakup.
