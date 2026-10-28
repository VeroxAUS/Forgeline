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

## Element 7: must-have chips

`src/build_card.rs` (`MustHave`, `chips`, `popup`): the skills the user insists on, as one row of chips above the equipment on the Build screen. Every chip is the same size (116 by 24 points, four fit across the equipment column at base and at 16:9) on a fixed grid snapped to whole pixels, each a name and the lowest level wanted. A name too long for a chip is cut with an ellipsis (about 14 characters, "Convert Thund…") and the tooltip has the full name. When the row is full the last cell becomes `+N more`, which always stands for at least two chips. `+ Add` on the label row opens the list of skills not yet chosen; a click on a chip opens its level cells and Remove; `+N more` lists the rest on compact rows. Escape or a click elsewhere closes a popup. A chip the unlocked gear cannot reach gets an amber outline, a line under the title says which, and the tooltip gives the best level. Any change keeps the last build on screen, shows `UPDATING` on the label row and the status line, and ends after a pretend 0.9 s; a repaint is asked for only while that is pending. Must-haves are never cut from the skills strip. Sample data only; the skill list and the out-of-reach levels are invented. The long names in the worst-case shots come from a community wiki, not the game files (unverified).

```
cargo run --release --bin chips_shots -- <out dir> 2                  # screenshots, no GPU
```

Screenshots are in [docs/ux/samples/chips/](../../docs/ux/samples/chips/).

## Element 9: the monster selector

`src/build_card/monster.rs`: the Monster cell opens a search panel (recent picks, monsters, Guardian and Arch-Tempered on a 28 point row grid, a Tempered switch that filters the list to monsters with a Tempered version). The roster is the research list in docs/research.md (38 entries, names not yet checked against the game files); weak elements are unknown and have five reserved cells per row for the food plan's matchup. The pick names the build in the breadcrumb.

```
cargo run --release --bin monster_shots -- <out dir> 2                # screenshots, no GPU
```

Screenshots are in [docs/ux/samples/monster/](../../docs/ux/samples/monster/).

## Element 10: the Shopping list

`src/build_card/shopping.rs`: the Shopping section, three views (`Monster`, `Piece`, `Upgrade`) on one 36 point row grid, driven by the save's counts so there is nothing to tick off, with an optional piece filter. A farm place column exists but draws only when some row has a confirmed farm place (open question 12); the sample has none. It also owns the `Missing N` word on the Build rows: hover lists the missing items (item, count, monster), click opens Shopping by piece filtered to that piece. Item names are real Wilds materials from a public page (docs/research.md); which monster drops what in the sample is illustrative.

```
cargo run --release --bin shopping_shots -- <out dir> 2               # screenshots, no GPU
```

Screenshots are in [docs/ux/samples/shopping/](../../docs/ux/samples/shopping/).

## Element 11: the Customise drawer

`src/build_card/customise.rs`: a 340 point drawer over the right of the Build screen, opened by the sliders icon on the Must have label row, left of `+ Add` (centred on the same line as that text) (closed by its cross, Escape or the same button). Top to bottom: saved presets (click to load, cross to delete, a name field and `Save`, which becomes `Replace` for a name that exists; capped at 8), the two assumption sliders (uptime, weak-spot rate; steps of 5 percent) and the comfort and defence weight sliders (0 to 10). Every slider row is the same size on one grid: name, value and `RESET` on the first line, the bar under it with a tick marking its baseline (weights) or default (assumptions); Reset shows only once the bar has moved, `RESET ALL` in the header sets every bar back. A moved bar or a loaded preset keeps the last build on screen and starts the non-blocking recompute. Long names get a plain ellipsis and the tooltip has the full name and the baseline.

Everything is sample data and says so (`SAMPLE DATA, NOT RESEARCHED`): comfort baselines do not exist until the Phase 4 research round, the assumption defaults are invented and the skill names are illustrative. Sliders have no keyboard control yet.

```
cargo run --release --bin customise_shots -- <out dir> 2              # screenshots, no GPU
```

Screenshots are in [docs/ux/samples/customise/](../../docs/ux/samples/customise/).
