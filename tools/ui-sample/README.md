# ui-sample

Element 1 of [docs/ux/design-plan.md](../../docs/ux/design-plan.md): one Build-like screen in seven style variants, drawn in real egui, so the maintainer can pick a style before `docs/ux/style-guide.md` is written.

Sample data only, with invented names and plain-shape placeholder icons. No Capcom data. Flat fills and hairlines, one static gradient (variant C), no blur or shadows, nothing animated.

| Variant | Idea |
|---|---|
| A. Modern Dark | Near-monochrome, hairlines, mono labels and numbers, numbered rows, square skill pips, mono headline word |
| B. Slate | Soft slate rows, sentence-case sans labels, blue accent, thin skill bars |
| C. Ember | Warm black, icon tiles, static header gradient, orange accent, round skill dots, large headline word |
| D. Modern Ember | The maintainer's pick (2026-10-28): A as the base with C's orange accent, round skill dots and icon tiles, no gradient |
| D1, D2, D3 | D plus C's header gradient (D1), plus C's tinted rail item (D2), plus both (D3) |

Screenshots are in [docs/ux/samples/](../../docs/ux/samples/).

## Render screenshots (no window, no GPU)

A small CPU rasteriser draws egui's output, so this runs in CI and in the cloud:

```
cd tools/ui-sample
cargo run --release --bin shots -- <out dir> 2     # 2 = pixels per point
```

## Element 2: the app shell

`src/shell.rs` is the D3 app shell (left rail, section divider, status line) laid out from the window size, so it resizes. Content is a labelled placeholder until later elements fill it. Ctrl+1 to Ctrl+6 switch section.

```
cargo run --release --bin shell_shots -- <out dir> 2                  # screenshots, no GPU
cargo run --release --features window --bin shell_window              # real window (the maintainer's PC)
```

Screenshots are in [docs/ux/samples/shell/](../../docs/ux/samples/shell/).

## Element 3: the icon pipeline

`src/icons.rs`: icons come from an `IconSource`, are decoded and downscaled on one background thread (Windows background priority), cached under a 1 MB cap (least recently drawn goes first) and drawn in the 34 pt tile, with the plain glyph while loading and when there is none. A repaint is requested only when an icon arrives. The only source here is synthetic (`SampleSource`); which game files hold the icons is unverified.

```
cargo run --release --bin icons_shots -- <out dir> 2                  # screenshots, no GPU
cargo run --release --features window --bin icons_window              # real window (the maintainer's PC)
```

Screenshots are in [docs/ux/samples/icons/](../../docs/ux/samples/icons/).

## Element 4: the build card

`src/build_card.rs`: the weapon, five armour pieces and the charm as D3 equipment rows inside the shell, each with its icon tile (element 3 cache, plain glyph while loading or when there is none), decoration slot circles (all one size and in fixed columns, the slot level as a number inside, filled when a decoration sits in it, hover says "Decoration slot, level N") and `Owned` or `Missing N`. An empty slot keeps its row shape with a faint word. One quiet line under the title says how many pieces need parts. Sample data only, labelled `SAMPLE DATA, NOT YOUR SAVE`; the real build comes with the optimizer (Phase 4).

```
cargo run --release --bin build_shots -- <out dir> 2                  # screenshots, no GPU
cargo run --release --features window --bin build_window              # real window (the maintainer's PC)
```

The whole UI scales with the window (`shell::auto_scale`): egui's zoom factor follows the window size against the 1000 by 700 design size, never below 1 and capped at 3, so text, icons and spacing grow together. The factor changes only when the window size does, so nothing repaints at idle.

Screenshots are in [docs/ux/samples/build/](../../docs/ux/samples/build/).

## Run it in a real window (the maintainer's PC)

```
cd tools/ui-sample
cargo run --release --features window --bin window       # keys 1 to 4 switch between A to D; or pass a, b, c or d
```

The window repaints only on input. The rail, the Solo and Multiplayer switch, the alternatives row and the main button respond to clicks.

## Notes

- The maintainer approved keeping this tool (2026-10-28).

- Fonts are Inter and JetBrains Mono (OFL), about 1.7 MB as bundled. The real app would subset them; that is a later element.
- It is excluded from the workspace and has its own CI job, so it adds nothing to the app binary or its RAM check.

## Element 5: the skills strip

`src/build_card.rs` (`skills`): the build's skill totals as a side strip level with the weapon label. Each skill is a name, its level in mono at the right, and one round dot per possible level (accent when earned, outlined when not). All dots share one outer size and a fixed pitch, snapped to whole pixels, so they line up down the strip. Hover says `Name, level N of M`. In a window too narrow for the side strip it drops under the charm row; with no skills it shows a faint line. Sample data only, same label as the card.

Screenshots are in [docs/ux/samples/skills/](../../docs/ux/samples/skills/). `build_shots` writes them too (`skills-empty.png` along with the build set).

Long lists (endgame builds): normal skills switch to one-line rows with the dots in a fixed column, and what still does not fit is left out, least important first (must-haves, set and group skills never; optimizer weight when there is one, else lowest level), with a clickable `+N more on Why` line. Set and group skills are pinned at the top with a `SET` or `GROUP` tag and one line per tier (reached tiers green, the next muted with the pieces it still needs, full text in the tooltip); the tier text is invented sample text. `sample_endgame_skills()` is a 22-skill sample.

Set and group skill drawing is automatic (`PinnedStyle::Auto`): full tiers while everything fits, then name plus one summary line, then one row each. In a window at least about 1245 points wide (16:9 at the design height) the strip gets a second column. `sample_shaped_skills` builds the sample shapes used for the research in `docs/research.md` (counts only).

## Element 6: headline damage and toughness

`src/build_card.rs` (`Headline`, `HeadlineStyle`): two blocks in the Build header, each label, `MODELLED` tag, headline word, a 6 pt bar and one detail line. The blocks are one skills column wide, so toughness sits over the skills strip and damage ends where the equipment rows end; both are the same size on the same rows. Three display options for open item A: `Word` (word and bar), `Compare` (adds a tick for current gear and the change against it) and `Number` (adds the key number). Unknown values show `UNKNOWN`, an empty bar and the reason. Sample numbers only; the bar scale (share of the best the unlocked pool allows) and the word thresholds are provisional.

```
cargo run --release --bin headline_shots -- <out dir> 2              # screenshots, no GPU
```

Screenshots are in [docs/ux/samples/headline/](../../docs/ux/samples/headline/).
