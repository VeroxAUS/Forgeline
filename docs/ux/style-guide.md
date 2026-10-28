# Style guide (Phase 2, step 7)

Dark theme only. Every later window follows this guide; Phase 6 polish builds on it. Values are proposals until the maintainer reacts to the mock-ups.

## Colour tokens
| Token | Hex | Use |
| --- | --- | --- |
| `bg` | `#0f1216` | window background |
| `surface` | `#171b21` | panels, table rows |
| `surface-2` | `#1f242c` | raised parts, hover, table header |
| `line` | `#2c333d` | borders, dividers |
| `text` | `#e6e9ee` | body text (contrast about 14:1 on `bg`) |
| `text-dim` | `#a3acb9` | labels, secondary (about 7.5:1) |
| `accent` | `#e0a63a` | primary action, selected tab, key numbers |
| `ok` | `#58b97c` | up to date, makeable |
| `warn` | `#e5873a` | out of date, assumption flag |
| `error` | `#e06c6c` | failed or rejected |
| `overlay-bg` | `#0f1216` at 80% | in-game overlay panel |

Colour never carries meaning alone: warn and error also have an icon and a word.

## Type (100% scaling, second-monitor distance)
| Role | Size | Weight |
| --- | --- | --- |
| Key number | 26 pt | bold |
| Screen title | 18 pt | bold |
| Body, tables | 14 pt | regular |
| Label, hint | 12 pt | regular, `text-dim` |
| Overlay body | 15 pt | regular (it sits over a busy scene) |

One sans-serif font for all text, one monospace for numbers in tables so columns align. A font ships with the app only if its licence is recorded in `THIRD-PARTY-NOTICES.md`; otherwise the system font is used.

## Spacing and layout
- 4 px base unit; padding 8, 12, 16, 24 px.
- Window layout: left rail of screens (160 px), top status line (data state), content area. Content never needs horizontal scrolling at 1280 px wide.
- Panels have 8 px corner radius and a 1 px `line` border; no shadows, no gradients, no animation beyond a 100 ms fade (cost of redraws matters).
- Tables: 36 px rows, right-aligned numbers, sticky header, no zebra stripes.
- The status line is always visible: save time, game build, refresh state, and a warning or error chip when needed.

## Widget list (step 10 sample view must contain these)
- Table of invented gear (sortable header, 36 px rows).
- Stat tiles: label above, key number below (for example effective raw, defence).
- Tooltip on a hover over a skill name (skill level and effect text).
- Comparison: two builds side by side with differences highlighted in `accent`.
- Primary button (`accent`), secondary button (`surface-2`), the default choice in a dialog is the highlighted one.
- Item card (icon box, name, one-line facts, status) and detail panel.
- Meter: skill pips, resistance bar, decoration slots, stacked contribution bar.
- Small line chart with direct labels (sensitivity).
- Banner (warn or error) with a short sentence and one action.
- Status chip (ok, warn, error) with icon and word.

## Icons
- 44 px icon box on cards and table rows, 20 px in the overlay. Rarity tints the glyph (low to high: `text-dim`, blue, purple, `accent`); a small rarity badge sits at the corner.
- Game icons come from the user's install at runtime (never committed). Our own glyphs (1.6 px line, rounded) are the fallback and the placeholder in mock-ups.
- Meters (pips, bars, slots, stacked bars) use `accent` for the main value, `ok` and `warn` for good and weak, never colour alone.
- Charts are drawn with the same tokens, labelled directly, and carry an assumption caption when the number is modelled.

## Rules for the overlay
- Background `overlay-bg`, text `text`, no border glow, corners 8 px.
- Rows 28 px, never more than 8 rows; counts bold in `accent`.
- Fully click-through; nothing hoverable.

## Rules for later phases
- New screens reuse these tokens; a new colour needs a change here first.
- Phase 6 may change shapes and spacing, not the meaning of the colours.
