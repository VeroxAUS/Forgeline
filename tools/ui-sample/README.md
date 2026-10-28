# ui-sample

Element 1 of [docs/ux/design-plan.md](../../docs/ux/design-plan.md): one Build-like screen in four style variants, drawn in real egui, so the maintainer can pick a style before `docs/ux/style-guide.md` is written.

Sample data only, with invented names and plain-shape placeholder icons. No Capcom data. Flat fills and hairlines, one static gradient (variant C), no blur or shadows, nothing animated.

| Variant | Idea |
|---|---|
| A. Modern Dark | Near-monochrome, hairlines, mono labels and numbers, numbered rows, square skill pips, mono headline word |
| B. Slate | Soft slate rows, sentence-case sans labels, blue accent, thin skill bars |
| C. Ember | Warm black, icon tiles, static header gradient, orange accent, round skill dots, large headline word |
| D. Modern Ember | The maintainer's pick (2026-10-28): A as the base with C's orange accent, round skill dots and icon tiles, no gradient |

Screenshots are in [docs/ux/samples/](../../docs/ux/samples/).

## Render screenshots (no window, no GPU)

A small CPU rasteriser draws egui's output, so this runs in CI and in the cloud:

```
cd tools/ui-sample
cargo run --release --bin shots -- <out dir> 2     # 2 = pixels per point
```

## Run it in a real window (the maintainer's PC)

```
cd tools/ui-sample
cargo run --release --features window --bin window       # keys 1 to 4 switch style; or pass a, b, c or d
```

The window repaints only on input. The rail, the Solo and Multiplayer switch, the alternatives row and the main button respond to clicks.

## Notes

- The maintainer approved keeping this tool (2026-10-28).

- Fonts are Inter and JetBrains Mono (OFL), about 1.7 MB as bundled. The real app would subset them; that is a later element.
- It is excluded from the workspace and has its own CI job, so it adds nothing to the app binary or its RAM check.
