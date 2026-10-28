# Style guide: D3 "Modern Ember"

Status: locked at the element 1 look check, 2026-10-28 07:39 UTC · Part of `docs/ux/design-plan.md`

D3 is near-monochrome dark with hairline rows, mono labels and numbers, and one orange accent. Reference: `docs/ux/samples/d3-modern-ember-both.png`. The values below come from `tools/ui-sample/src/lib.rs` (the D3 `Style`); that file is the code reference, and if the two ever differ, change both in the same PR. All sizes are in egui points at 1.0 scale on a 1000 by 700 window.

## 1. Colours

| Token | RGB | Use |
|---|---|---|
| `bg` | 10, 10, 11 | Window background |
| `fg` | 236, 236, 238 | Primary text |
| `dim` | 140, 140, 148 | Secondary text, mono labels |
| `faint` | 82, 82, 90 | Row numbers, empty slots, version tag |
| `line` | 34, 34, 38 | Hairlines, bar tracks, empty dots |
| `accent` | 255, 138, 61 | Current selection and next action only |
| `accent_ink` | 26, 12, 2 | Text on an accent fill |
| `ok` | 76, 195, 138 | Owned, saved, synced |
| `warn` | 229, 176, 74 | Missing, sample or out-of-date notices |
| rail tint | 44, 30, 21 | Selected rail item fill |
| tile fill | 40, 29, 22 | Icon tile background |
| gradient top | 38, 24, 16 | Header gradient start (fades to `bg`) |

Rules:
- The accent marks one thing: where the user is, or what to do next. It is never decoration and never used for status.
- Status colours (`ok`, `warn`) are separate from the accent. Never use red as an alarm unless something is broken.
- Controls and rows have no fill; structure comes from hairlines.

## 2. Type

Inter for text, JetBrains Mono for labels and numbers (both OFL, listed in `THIRD-PARTY-NOTICES.md`).

| Role | Face | Size |
|---|---|---|
| Wordmark | Inter Medium | 16 |
| Screen title | Inter Medium | 20 |
| Headline word (damage, toughness) | JetBrains Mono Medium, caps | 20 |
| Row title, rail item, notice title | Inter Medium | 13.5 (rail 13) |
| Body, row detail, status line | Inter | 11.5 to 13 |
| Buttons, tabs, chips | Inter Medium (chips Inter) | 12 to 12.5 |
| Section label | JetBrains Mono, caps | 10.5 |
| Row numbers | JetBrains Mono | 11 |

Minimum readable text is 12 px at the target size (design plan principle 8); the 10.5 and 11 labels are mono caps and numerals only, never sentences. Numbers are always mono.

## 3. Layout

- **Window:** 1000 by 700. Left rail 176 wide, 1 px hairline on its right edge. Status line along the bottom, 24 high.
- **Content margin:** 28 from the rail; content column ends 28 from the right edge.
- **Rhythm:** 4 px base. Common steps are 4, 8, 12, 24 and 28.
- **Rows:** 48 high with 6 px of gap, a hairline between rows (not above the first). No boxes inside boxes.
- **Hairlines:** 1 px, `line`. Dividers and row separators only.
- **Radius:** 2 for frames, buttons, chips and tiles; bars use half their height; dots are circles.

## 4. Components

**Rail.** Wordmark with a 2.5 px accent dot. Items are 34 high (13 Inter Medium, 16 px icon at left). The selected item has the rail-tint fill and a 2 px accent bar on its left edge. Planner sections first, then a hairline, then Settings, Data and About.

**Header.** Mono caps breadcrumb (`BUILD · LIGHT BOWGUN`), title, and the controls on the right: monster selector and Solo/Multiplayer switch, both 30 high. The selected half of the switch is a filled accent block with `accent_ink` text. A `warn` dot plus mono caps notice appears when data is sample or stale. The header sits on the one static gradient.

**Tabs (alternatives).** Inter Medium 12.5, 28 high. The active tab has a 2 px accent underline; inactive tabs are `dim`. A hairline runs under the row.

**Equipment row.** Row number (mono, `faint`), 34 px icon tile, name in Inter Medium with skills in `dim` underneath, then slot circles, then a status word on the right: `Owned` in `ok`, `Missing N` in `warn`.

**Icon tile.** 34 by 34, tile fill, 2 px radius, glyph inset 7 px in the accent. A real icon is drawn inset 3 px, untinted. The plain glyph shows while an icon loads and when there is none, so the tile never changes size or shifts. Icons are cached at 64 px on the longest edge under a 1 MB cap (`tools/ui-sample/src/icons.rs`).

**Slot circles.** 3.5 px outlined circles in `faint` (decoration slots).

**Skill dots.** Round, one per level up to the skill's maximum: filled accent for earned levels, outlined `faint` for the rest. Skill name sits above the dots, the level number on the right in mono `dim`. Filled and outlined dots share one outer diameter (8 px) on a 13 px pitch, snapped to whole pixels, so dots line up across skills. In the Build screen the strip is a 232 px side column (one skill per row, 38 px pitch) level with the weapon label; if the window is too narrow for that it drops under the card with columns that follow the width. Hover says `Name, level N of M`. Long lists change density instead of scrolling: normal skills are two-line blocks while they fit, then one-line rows (22 px pitch, dots in a fixed column). If they still do not fit, the least important are left out and a mono `+N more on Why` line (clickable, opens Why) says where the rest are. Must-haves and set and group skills are never left out; the rest go by the optimizer's weight when there is one, otherwise lowest level first.

**Set and group skills.** Pinned at the top of the strip, above a hairline, in two-line blocks: name in Inter Medium, a small outlined mono tag (`SET` or `GROUP`, `dim` on a `faint` outline) at the right, and progress underneath as plain words (`3 of 4 pieces`, `ok` once a bonus is active, `dim` before). A numbered-circle variant (one circle per piece threshold, slot-circle size) exists in `tools/ui-sample` but reads like decoration slots, so words are the default.

**Headline block.** Mono caps label left, optional `MODELLED` tag right (mono caps, `warn`). Below it the headline word, a 6 px flat bar (`line` track, accent fill, rounded ends) and a 1.5 px `fg` tick for the current gear. Delta text in mono `ok`, right-aligned (`+12% vs your gear`).

**Chips (must-haves).** 26 high, hairline outline, 12 Inter, with a small close mark. The add chip is a 26 by 26 square with a plus.

**Primary button.** Accent fill, `accent_ink` text, Inter Medium 12.5, radius 2. One per view.

**Next-action bar.** Full-width hairline frame, 56 high: cart icon in the accent, a title (`Next: …`), one `dim` line, and the primary button on the right.

**Status line.** Left: `ok` dot and save state. Middle: data age. Right: version tag in mono `faint`. Always visible; errors appear here in plain words, never as a modal.

## 5. Performance limits

These apply to every element (design plan principle 7, `docs/performance-budget.md`).

- No per-frame blur, shadows or glows.
- One static gradient only: the header, drawn as a single two-colour mesh. No other gradients.
- Nothing animates while idle. Repaint only on change or input.
- Icons and tiles are flat shapes or cached textures, never redrawn effects.
- A new element that costs noticeable frame time, RAM or start-up time does not ship (CLAUDE.md).

## 6. Wording

- Short and plain. Lead with the decision, then at most one line of reasoning (`Highest damage that keeps both must-haves, using only parts you can get now.`).
- Meaning first, numbers second. Raw stats live in details, one click away.
- Never compare tiny numbers in the headline; if the difference is small, say so in words.
- Modelled values carry a `MODELLED` label with the assumptions one click away. Unknown values say `unknown` and why. Nothing is filled in from memory.
- Errors say what happened and what to do, in one line, and are never modal.
- Sample data is always labelled (`SAMPLE DATA, NOT YOUR SAVE`).
- Section labels are mono caps; everything else is sentence case.

## 7. Open items

Damage and toughness display (design plan section 7, A) and the overlay look are not covered here; add them when decided. This guide is not set in stone: a later element may change a rule, and the change updates this file.
