# UX design plan

Status: draft for the maintainer's approval · Replaces the mock-up approach of Phase 2 step 7

This plan restarts the UX from first principles. The mock-up round was hard to iterate because many choices were locked before anyone had seen them working. From here, one real egui element is added at a time, each reviewed from a screenshot of the real build. Nothing is built until the maintainer approves this plan. The brief and style guide from the earlier step 7 branch predate this restart and are source material only.

## 1. Goal and done

Give Forgeline a UI that feels invisible: open it, trust the build at a glance, know what to get next, and never wait or read walls of text.

Done means:

- [ ] Every element in section 6 is built in the real egui app and reviewed by the maintainer from a screenshot.
- [ ] Each element has passed the look check and the performance check (cold start, idle CPU, RAM against the budget) on the maintainer's PC with Wilds running.
- [ ] The style rules in `docs/ux/style-guide.md` exist, written after the sample screen (element 1) and not before.
- [ ] The two open design items in section 7 are decided.

## 2. Principles

1. **Invisible first.** The app opens on the last build from cache with no step before it. Computing never blocks, never pops up and never moves what the user is looking at.
2. **One primary action per view.** Everything else is quiet.
3. **Icons and small visuals do the heavy lifting.** Text is short and plain. Where trust needs reasoning, one sentence sits under the recommendation, with details one click away.
4. **Meaning first, numbers second.** Headlines say what the numbers mean. Raw stats live in details.
5. **Honest by construction.** Modelled values carry a "modelled" label with the assumptions one click away. Unknown values say "unknown" and why. Nothing is shown from memory or guessed.
6. **Dark and quiet.** Dark theme only, readable on a second monitor, hairlines over boxes, no boxes inside boxes. One accent for the current selection and next action; status colours are separate.
7. **Cheap to draw.** No per-frame blur, shadows or glows; one static gradient is allowed. Nothing animates while idle. Repaint only on change or input.
8. **Customisable.** Overlay size, scale and hotkey are the user's to set. A minimum readable size exists (12 px text, window from 1000 by 700, to be tested).
9. **Not set in stone.** Every feature is reviewed when added and may be changed or removed.

## 3. Information architecture

Left rail, narrow, icon plus short label. The planner sections come first; a line separates the housekeeping sections below them.

| Section | Holds |
|---|---|
| **Build** (home) | Current best build as an equipment set, skills, headline damage and toughness, must-have chips, Solo or Multiplayer switch, alternatives toggle row, monster selector (manual for now) |
| **Shopping** | Missing items, three views: by monster, by piece, by upgrade path |
| **Why** | Reasoning sentence, skill value, compare two builds, what changed |
| *(line)* | |
| **Settings** | Customise drawer content (weights, assumptions, saved presets), overlay size, scale and hotkey |
| **Data** | Update, version, validation result, out-of-date notice |
| **About** | Version, credits, licence |

The Customise drawer opens from the Build screen; must-have chips stay on the Build screen so changing them is quick. A thin status line is always visible (data age, save state, errors). Errors are plain and say what to do, and are never modal.

First run is one screen: find save, find game, extract data, with a progress line, then Build.

### Desktop and overlay

The overlay is on by default as a window; live data stays off by default (CLAUDE.md). It shares components with the desktop window in a compact form. One customisable hotkey (F8 default) shows or hides it. Default under 320 px wide and a short column; the user can resize it and it scales with screen resolution. Position and colours stay provisional until seen in game.

| Mode | Overlay shows |
|---|---|
| Save only (default) | Top few shopping items, which monster to hunt next, save-based info |
| Live data on (needs forgeline-live) | Shopping parts for the monsters on the map, target hints (parts to break, things to watch), build swap hints per monster matchup |

Desktop only shows no live content. Before and after a hunt, decisions happen on the desktop window.

### Data these screens depend on (all unverified until confirmed)

- "Best place to farm" (event or optional mission) and part-break hints need drop, part-break and mission data. Whether the game files hold it is open question 12. Until confirmed, the shopping row shows item, icon, owned and missing counts, monster and the piece it unlocks.
- Automatic monster switching needs either the save recording a started quest (not found in step 4) or reading memory through forgeline-live. Until then the selector is manual.
- Icons come from the user's own install at runtime. Whether the needed icon files can be read is unverified; each icon element falls back to a plain glyph.
- Playstyle tips need skill data from the game files.

Before the optimizer exists (Phase 4), build, why and tips elements draw from clearly labelled sample data, never shown as the user's real result.

## 4. How elements get added

For each element: build it in egui in the real app; screenshot it on the maintainer's PC; the maintainer reviews the look and says keep, change or drop; run the look check and the performance check; then merge. The maintainer talks to the coordinator only, so screenshots and verdicts go through it. Changes to earlier elements are expected.

## 5. Tools

None new. Sample data uses the existing fixture approach.

## 6. Elements in order

Estimates are thread hours (build plus one revision), not review time. They are estimates, not measurements.

| # | Element | Hours | Notes |
|---|---|---|---|
| 1 | **Sample screen in real egui**: one Build-like screen in 2 or 3 style variants, to settle style before rules are written | 4 | Output: the first `style-guide.md` |
| 2 | **App shell**: left rail, section divider, status line | 3 | |
| 3 | **Icon pipeline**: read icons from the install at runtime, cache, plain-glyph fallback | 5 | Needs a check that icon files are readable; never shipped or committed |
| 4 | **Build card**: weapon and armour slots with icons | 5 | Sample data until Phase 4 |
| 5 | **Skills strip**: skills with levels | 3 | |
| 6 | **Headline damage and toughness** | 4 | Open item A (section 7) |
| 7 | **Must-have chips** on Build, with non-blocking recompute indicator | 4 | |
| 8 | **Solo or Multiplayer switch and alternatives row** | 3 | Safe is an alternative, not a preset |
| 9 | **Monster selector** (manual) | 3 | Auto-switch later, see section 3 |
| 10 | **Shopping list**, three views | 6 | Farm place only if data confirmed |
| 11 | **Customise drawer**: weight sliders with baseline and reset, assumption sliders, saved presets | 8 | |
| 12 | **Why: reasoning and skill value** | 5 | |
| 13 | **Why: compare two builds** | 4 | |
| 14 | **Why: what changed** | 3 | |
| 15 | **Data screen** | 3 | Update, validation, out-of-date notice |
| 16 | **Settings and About** | 2 | |
| 17 | **Overlay shell**: transparent, click-through, always on top, hotkey, size, scale | 6 | Borderless windowed only; provisional position |
| 18 | **Overlay content**, save-only mode | 4 | Live mode comes with forgeline-live |
| 19 | **Playstyle tips from skills** | 4 | Needs skill data |
| 20 | **Minimum size and scaling test** | 3 | Fixes the minimum readable size |
| | **Total** | **82** | |

Elements 4, 5, 6, 12 to 14 and 19 get real data in later phases; this plan only covers their UI.

## 7. Open design items

- **A. Damage and toughness presentation.** Words alone for toughness (Solid, Tankier, Much tankier) may be too vague. When element 6 is built, try 2 or 3 options and let the maintainer choose, for example: a bar with a plain-word label; a bar with the key number on hover; a bar compared against the current gear. Raw numbers stay in details either way.
- **B. Overlay look** (position, colours): provisional until seen in game.

## 8. Risks

- RAM headroom is thin (about 23 MB private). Icons and images are the biggest risk; element 3 sets a cache cap and checks it against the budget before anything else uses icons.
- Sample data could be mistaken for real results. Every sample is labelled.
- Elements built before the optimizer may need rework when real data arrives. Accepted; reviews happen again then.

## 9. Versioning and changelog

UI elements land in minor versions with the phase that supplies their data. Each CHANGELOG.md entry ends with "Reviewed and tested by VeroxAUS".

## 10. Review log

- 2026-10-28: questionnaire answered; follow-up on six reopened items answered. Question and answer log kept by the coordinator in open-questions.md and closed-questions.md.
- 2026-10-28: element 1 look check passed at 07:39 UTC. Style locked in: **D3**, which is A (Modern Dark: near-monochrome, hairline rows, mono labels and numbers, numbered rows) with C's orange accent, round skill dots, icon tiles, the static header gradient and the tinted selected rail item. Screenshot: `docs/ux/samples/d3-modern-ember-both.png`. `docs/ux/style-guide.md` is written from it in a fresh thread. `tools/ui-sample` is approved to keep.

## 11. Time log

| Element | Thread | Result |
|---|---|---|
| 1. Sample screen | 2026-10-28, started 07:16 UTC, look check passed 07:39 UTC, merged 07:43:49 UTC (about 25 min of thread time, against the 4 h estimate; cloud, CI green on Ubuntu and Windows) | Built as `tools/ui-sample` (PR #47): variants A to C, then D and its three variants. The maintainer locked in **D3** (see the review log). Reviewed from screenshots in the cloud; not run on the PC. |
