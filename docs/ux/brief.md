# UX brief (Phase 2, step 7)

Draft for the maintainer to react to. All sample content in the mock-ups is invented.

## The user
A Monster Hunter Wilds player who wants the best build they can make right now, without reading spreadsheets. They play with the game on the main monitor and Forgeline on a second monitor, or glance at the overlay mid-hunt. They know the game's words (skills, decorations, sets) and do not want them explained.

## Key moments
1. **Just finished a hunt, what now?** Open the app. The last build is on screen at once, marked "refreshing" while the new save is read, then updated in place.
2. **Second monitor during a quest.** The best build and the shopping list are readable from a normal seat distance. Nothing animates and nothing needs a click.
3. **Mid-hunt, missing a part.** The overlay shows which parts are still needed (for example "2x plate") and where or how to get them. No alt-tab.
4. **Why this build?** One click from the best build shows the alternatives (safe, defensive, monster-specific, general hunting, Gogma-specific) and the assumptions behind the numbers.
5. **Game just patched.** A visible banner says data may be out of date, and offers Update only when confidence is high. The last good data stays in use.
6. **Something failed.** An update was rejected or a save could not be read. The screen says so in words and shows what is still being used. Never silent wrong data.

## Screens
| Screen | Purpose |
| --- | --- |
| Best build | The build to craft now: equipment, key skills, defence, effective raw. |
| Why this build | Alternatives side by side, the assumptions (uptime, weak-spot rate, weights), and a flag when the best build changes within a plausible range. |
| Shopping list | Missing materials grouped by where to get them, tick-off not needed. |
| Data and update | Game build, data age, last update result, the Update button, patch warning. |
| Live-data warning | Shown once when live data is switched on; "Keep it off" is the default. |
| Overlay | Small in-game panel, two modes (save-only, live data). |

## Information and visuals (maintainer feedback, 2026-10-28)
- A bare name is not enough. Every item (weapon, armour piece, talisman, part, decoration) opens a detail view: what it is, rarity, defence, skills, slots, what it is made from, what is missing and where to get it.
- Every item has an icon, so the screen is not only text. Skills show level pips and a short effect on hover; resistances and decoration slots are drawn, not listed.
- "Why this build" shows: a bar of where the number comes from, all five builds in one table with differences, a sensitivity chart (the uptime at which the best build changes), the assumptions, and what was considered and left out.
- Shopping list is driven entirely by the save. No manual tick-off.

## Icons and images
- **Decision (maintainer, 2026-10-28):** icons and images are read at runtime from the user's own game install, like the other game data. They are never committed, never shipped and never in the Nexus package. Only the files needed are extracted (never whole packs), into the local cache.
- **Own glyphs as fallback:** the mock-ups use our own simple glyphs (slot shapes, part, gem, coin). They stay in the app for any item whose icon cannot be read, so a missing icon never leaves a hole.
- **Unverified:** whether item icons can be found and decoded from the game files, in which format, and what they cost in disk, RAM and start-up. To be checked in a spike before the equipment display (Phase 6). The performance budget applies: icons load lazily, only for visible rows, from a small cache, and never block the first frame.

## Glance rules
- The answer comes first: the build and its headline number sit top left, above everything else.
- Modelled numbers are labelled "modelled" or "estimated". Measured numbers are never mixed with them.
- Every screen shows its data state in one line: save time, game build, and "refreshing" or "up to date".
- No more than three colours carry meaning at once: accent (action and selected), warning, error.
- Text readable at about 60 cm on a 24 inch second monitor: body 14 pt at 100% scaling, key numbers 22 pt or larger.
- No scrolling needed for the best build and the top of the shopping list at 1920x1080.

## In-game shopping-list overlay
- **Purpose:** mid-hunt, see which parts are still needed (for example "2x plate") and where or how to get them, without alt-tabbing.
- **How it works:** our own transparent, click-through, always-on-top window placed over the game. It never hooks, injects into or draws inside Wilds. Borderless windowed only; exclusive fullscreen hides it.
- **Position:** top right under the game's own mini-map area, 24 px from the edge; movable in settings. (Proposal.)
- **Hotkey:** F8 to show or hide. Remappable. Off at start unless the user turned it on. (Proposal.)
- **Size:** about 320 x 360 px at 1080p, scaling with the display. Background about 80% opaque so text reads over any scene. (Proposal.)
- **Shows:** the next 6 to 8 missing parts with counts, and where to get them; the target build name; in live mode, the current monster and broken parts.
- **Never shows:** hunter name or ID, anything from other players, anything that needs a click, notifications that pop up on their own.
- **Modes:** save-only (default, works with nothing else running), and live data (only if forgeline-live is on). If live data fails the overlay falls back to save data and says so in one small line.
- Built in Phase 6 from the same data as the main window.

## Open for the maintainer
- Overlay position, hotkey and size above are proposals; react in the mock-up.
- Whether the shopping list needs tick-off (saved by hand) or only mirrors the save.
