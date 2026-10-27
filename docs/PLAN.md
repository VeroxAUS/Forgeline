# MH Wilds BiS app: project plan and handover

Written from the design conversation with Thomas on 2026-10-27 and updated with every decision made that day. Nothing has been built yet. Everything below is either marked **verified** (we saw it ourselves), **reported** (from a web source, unchecked) or **open**. Standing rules are in [CLAUDE.md](../CLAUDE.md); this file is the context behind them. Answered questions are in [closed-questions.md](closed-questions.md), unanswered ones in [open-questions.md](open-questions.md), findings in [research.md](research.md).

## 1. Goal

A Windows and Steam desktop app, shared publicly on Nexus Mods, that reads the user's Monster Hunter Wilds save automatically and always shows the **best build they can make right now**: weapon, armor, decorations, talisman, upgrades. It covers low rank and high rank, updates as they progress and never shows an end-game build they can't make yet ("wait until you get there" is explicitly not acceptable). It also gives an in-depth shopping and farming list for the chosen build.

The prototype is the **light bowgun**. The engine is generic and the extractor and data model cover every weapon type from Phase 3. Every weapon is optimized before the 1.0.0 full release, refined one at a time with the light bowgun first, so that anybody can use it.

## 2. What was asked for

- Pick a weapon and get the **optimal** gear path, not just "upgrade to the next version": weapons, armor, decorations, talismans.
- **Available gear is anything from monsters the user has unlocked.** Materials the user is missing go on the shopping list. Artian and Gogma Artian weapons are handled as their own track.
- When Artian or Gogma Artian is the best available weapon, work out the best rolls and reinforcement to aim for, and give optimal farming routes.
- Pick skills and the app finds armor, decorations and talisman that reach them, while also taking community-agreed builds into account.
- Playstyle presets as a toggle (solo, multiplayer, safe). Comfort skills (Earplugs, Tremor Resistance) and defensive options are part of the optimizer, not just damage.
- A per-monster mode plus a general hunting build. Weapon element and bowgun ammo type (normal, pierce, spread) follow the monster's weaknesses.
- Alternatives next to the best build: safe, defensive, monster-specific, general hunting, Gogma-specific.
- Shopping list that updates as the save changes, per monster part ("1/2 parts from this monster"), best place to farm, decoration farming and what to meld for. In depth, within what the data allows.
- Data updates only when the user presses a button (so a bad update can't break builds). Dark theme only, readable on a second monitor, an equipment-style display with hover details for items and upgrade levels. An in-game overlay is a maybe, deferred.
- Releasable on Nexus, usable by anyone.

## 3. Verified findings

### Save file (verified)
- Wilds saves live at `Steam\userdata\<steam account id>\2246340\remote\win64_save\`. The main file is `data001Slot.bin` (about 198 KB); `data00-1.bin` sits next to it. A machine may have several Steam account folders; only one holds Wilds saves.
- The save is encrypted with the Steam ID. The 64-bit Steam ID is the account folder number plus `0x0110000100000000`.
- The open-source MHWeditor (Rust, library crate `ree-lib`) was used in a spike to decrypt and parse a **copy** of a save. It needs its own type data (RSZ class and enum dumps), and field names are hashed with murmur3 (seed `0xffffffff`). **MHWeditor has no licence**, so none of its code or type data enters this repo (see section 4 and [research.md](research.md)). The facts below are format facts we observed.
- The decrypted save has one top-level class `cUserSaveData` with `_Data`, an array of 3 character slots. One slot has `Active` = 1 (the app auto-picks it). Per-slot fields seen:
  - `_Item`: `_BoxItem` (900 entries of `ItemIdFixed`, `Num`), `_PouchItem`, `_PouchShell`, `_ItemFoundFlag` and `_ItemCheckedFlag` (bitsets, max 2000 items).
  - `_Equip`: `_EquipBox` (2400 entries with `Category_Gender`, `BonusByCreating`, `BonusByGrinding`, `GrindingNum`, `BowgunCustomizeId[4]`, `EquipmentAccessoryIdArray[3]` and free values), `_WeaponFlagParam` (per weapon type "created" and "checked" bitsets), armor created/checked bitsets per slot.
  - `_Story` (flags, `ProgressId`), `_Mission` (clear, active and failed flag bitsets), `_EnemyReport` (`_Boss`: `FixedId`, `SlayingNum`, `CaptureNum`, size records; also `_Zako`, `_Animal`, `_AnimalFishing`), `_Collection`, `_ItemRecipe`, `_LargeWorkshop`, `_QuestRecord`, `_Player`, and many more.
- Item and equipment IDs in the save are "fixed IDs". **Whether they match any data source's IDs is not yet verified.** Owned talismans were not located yet.
- Reading is safe; the MHWeditor disclaimer concerns editing. Our app is read-only and only ever analyses copies of the save.

### Game install (verified)
- The game is found through Steam's `libraryfolders.vdf` (several library drives are possible), never hardcoded. The main pack `re_chunk_000.pak` is about 41 GB with patch packs `patch_001` upward. Extract only the files needed; never unpack whole packs.
- An install may be modded (mod manager marker files, `dinput8.dll`). Extraction must read the original packs only.

### MHDB API (verified for what we queried)
Community API at `https://wilds.mhdb.io/en/...`, docs at https://docs.wilds.mhdb.io, source toolkit https://github.com/LartTyler/mhdb-wilds-data (GPL-3.0).
- **Present:** weapons (slots, crafting and upgrade materials, tree position, specials), armor and armor sets (skills, slots, set and group bonus skills), decorations, charms, items, skills, monsters with hitzones per part (slash, blunt, pierce, element, stun) and rewards with drop chance, condition and rank. Gogmazios is present.
- **Missing or empty:** motion values (empty `[]`), Artian reinforcement and rolls (Artian Blaster I and II are stub entries with no bonus data), Gogma Artian, random talismans (charms are all `random: false`).
- Skill effects are **prose only**, with wording that varies between ranks, so they can't be parsed reliably.
- The data version stamp is **2026-04-15**, older than the game build at the time. It is a cross-check and a labelled last resort only.
- No licence stated for the data. The toolkit code is GPL-3.0.

### Other sources (reported)
- [kassent/mhwilds_data](https://github.com/kassent/mhwilds_data): last pushed Aug 2026, only enemies, items, missions and rewards, no licence. Not a full source.
- [mhdb-wilds-data](https://github.com/LartTyler/mhdb-wilds-data): last pushed 2026-07-30. Converts the game's `.user.3` and `.msg.23` files to JSON.
- A dtlnor datamine of motion values exists for launch and patch 1.0.1 (via a r/MonsterHunterMeta post); stale, unopened.
- No complete fresher ready-made source was found. The game files are the freshest.

### Artian and Gogma Artian (reported, guides only)
- Artian weapons: four part types (Blade, Device, Disc, Tube). Up to five upgrades, each giving one of four random bonuses (attack, affinity, sharpness, element). Guides give no pool sizes or odds.
- Gogma Artian (Title Update 4, Dec 2025): converts a Rarity 8 Artian weapon using Tarred Devices from Gogmazios; random bonus skill and group skill, a larger bonus pool; guides disagree on reroll details. Sources: https://www.icy-veins.com/monster-hunter-wilds/gogma-artian-weapons, https://mobalytics.gg/mhw/guides/gogma-artian-weapons-explained
- The save stores each weapon's actual Artian fields (`BonusByCreating`, `BonusByGrinding`, `GrindingNum`), so owned weapons' rolls can be read. How those numbers map to bonus values is **open**.

## 4. Design decisions

- **Rust everywhere**, with our own tools. Every new tool needs Thomas's yes before it is created or run. If a Nexus release requires open source, the project goes open source.
- **MHWeditor is dropped as a dependency.** It has no licence (no LICENSE file, no Cargo licence fields, only a README disclaimer), so it is treated as all rights reserved. We learn facts from it, not code: one thread studies it and writes the facts in its own words to `research.md` with credit; a separate thread writes our reader from those notes only. Its type data licence and a possible upstream (c-ola) are still unchecked. Third-party code in the repo needs its licence recorded in `THIRD-PARTY-NOTICES.md`; GPL code stays out until the licence review says what a Nexus release must be.
- **Type data (needed to read the save and game files), fallbacks in order:**
  - A: our own static dumper that reads the game's `.exe` on disk, run on the user's PC, nothing shipped.
  - B: a hand-written schema for only the classes we need.
  - C: a licence-checked community dump downloaded on update, never shipped; we never run injecting tools ourselves.
  - D: map fields by hashes and value patterns.
  - Last resort if no game data can be read: MHDB data, clearly labelled to the user as older and unsupported.
  This spike runs first in Phase 1.
- **UI:** pure Rust if the Phase 1 comparison allows. Phase 1 measures pure-Rust UI (egui, Slint or similar) against Tauri and Electron (cold start, idle RAM and CPU) with Wilds running. Dark theme only.
- **Data from the game files.** Our own extractor reads the user's own install when they press the update button. We never ship Capcom data. The update never auto-fetches.
- **Update validation gate.** A new extraction is staged next to the last good data and validated before it replaces it. It is blocked if items the build uses disappear, if an upgrade is not a real increase, or if builds score worse after skill, set bonus or weight changes (the model re-scores old versus new data and flags worse ones). If extraction fails, a format changed or validation fails, the app keeps the last good data and says so visibly. How to implement the confidence measure is open.
- **Patch warning.** When the game build changes, the app warns that data may be out of date. It suggests pressing update only when confidence is above 95% that nothing will break or get worse. Game build ID and file structure hashes are stored with each extraction as inputs.
- **Optimizer:** constraints first, then a score. Must-haves (Earplugs, Tremor Resistance, and similar) are hard constraints. Damage is calculated from raw, affinity, element, sharpness or ammo and the skills' real effects. Comfort and defence skills use adjustable weights set by the playstyle preset. A solver searches armor, decorations and talisman. Low rank builds are included.
- **Comfort skills:** a Phase 4 research round sets a baseline for every skill. Users can override any of them.
- **Uptime:** an explicit adjustable input (attack time, weak-spot rate) with defaults per weapon type and skill level. A **robustness check** ranks builds across a plausible range and flags when the winner depends on the assumption. Modelled numbers are never presented as measured.
- **Community builds** are added by hand with credit (no scraping) and used as sanity checks and to calibrate the comfort weights (find weights that make trusted builds score near the top). A mismatch means either the model has a gap or the guide leaves damage unused; investigate, don't copy.
- **Progression:** gear from unlocked monsters, with missing materials on the shopping list; recompute whenever the save changes.
- **Artian and Gogma Artian:** read actual rolls from the save, rank them against normal weapons, say keep or reroll, show the best target roll; odds only if the game files hold the pools. Do not build around the "craft another weapon first to reroll" trick (one guide only).
- **Out of scope unless Thomas reopens it:** a rotation-based DPS simulator and a statistical fight model.
- **Weapons:** the extractor and data model cover all weapons from Phase 3. Every weapon is optimized before 1.0.0, light bowgun first; each weapon is labelled "tuned" or "generic model" in the app. Which weapon follows the light bowgun is decided later.
- **Saves:** the app auto-picks the active slot and the Steam ID comes from the folder name.
- **Distribution:** Nexus. Local only (no network listener, telemetry or accounts). Windows and Steam only.
- **Versioning:** semantic versioning. 1.0.0 is the full release; a major addition bumps x.1.x; a bug fix bumps x.x.1. Changelog from Phase 2.

## 5. Performance and safety

- Runs alongside the game: low idle CPU, memory and GPU, no polling when nothing changed. Heavy work (extraction, optimizer search) runs only on user trigger, off the UI thread at low priority.
- Snappy: opens fast and shows the last build from cached data immediately; recompute never blocks the UI. Cold start and recompute time are checked on the developer's PC with Wilds running before a feature ships.
- A **performance budget** (cold start, idle RAM and CPU, recompute time) is to be drafted together with Thomas; the Phase 1 UI comparison measures against it.
- Read-only on the game. Never write to the save, the install or the Steam folders. Never inject into, hook or read the memory of the running game.
- No secrets or personal data in git: saves, decrypted dumps (they hold hunter IDs) and extracted Capcom data stay out of the repo and the Nexus package. Contributors provide their own local saves.

## 6. Phases

Each phase has a detailed plan in `docs/plans/phase-N-name.md`, written just before the phase from the template in [plans/TEMPLATE.md](plans/TEMPLATE.md), reviewed and approved by Thomas before any work. When a phase finishes, this file is updated.

1. **Spikes**, each with a pass or fail result:
   - **Type data first**, with fallbacks A to D named in the plan.
   - Save-to-data ID matching, and owned talismans, on a copy of the save.
   - Licence review (cloud, in parallel): MHWeditor and its possible upstream, mhdb-wilds-data (GPL-3.0), MHDB data.
   - Extraction test on the developer's install: skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops, motion values. Then decide what stays hand-written.
   - UI comparison: pure-Rust UI versus Tauri versus Electron, measured with Wilds running.
2. **Foundation:** finds the save and game install automatically, our save reader, a file watcher, the extractor and the update button. Fixture saves for tests, a debug data viewer shown in a bare window (grown each phase), changelog and versioning.
3. **Data and progression:** data model for every weapon, low and high rank; unlock logic (kills and story flags to unlocked monsters).
4. **Light bowgun optimizer:** damage formula, must-haves, presets, per-monster mode, alternative builds, uptime inputs and robustness check. Includes the comfort-skill research round.
5. **Calibration:** comfort weights from reference builds, plus training-room checks (single shot, one skill at a time, affinity over 100 or more hits, ammo types) and finding out what the training dummy displays. Tests compare **ratios** because the dummy's hitzones are unknown.
6. **Interface and shopping list:** polish of the bare window, dark theme, equipment-style display with hover details, shopping list.
7. **Farming routes, decorations and melding.**
8. **Artian and Gogma Artian planner.**

After that: the other weapons one at a time (each optimized and labelled tuned or generic before 1.0.0), then Nexus packaging.

### TODO tools
Built when needed, each needing Thomas's yes at the time: save diff (compare two save copies to map fields to unlocks), fixture save builder, data diff (compare two extractions, feeds the validation gate), build explainer (why the optimizer picked a build), benchmark harness (start-up, RAM, recompute time).

## 7. Workflow

- Thomas talks only to the coordinator; threads report to it and it summarises. It posts short status when a phase finishes, when Thomas is needed, when it starts a thread, and on errors.
- Feature and fix threads work in the cloud on their own branch and open a PR. `main` is the only long-lived branch; branches are deleted after merge. Cleanup and docs-only PRs may go straight to `main`.
- Tests and linters run on GitHub Actions using fixture saves and fixture data; they pass on a machine without Wilds and never touch a real save, install or the live MHDB API. A PR touching the reader or extractor says it is untested against the real files.
- Every PR gets a test round. Only PRs that change what the app reads or shows run on the developer's PC; the rest are checked in the cloud and reported the same way. Rounds are queued so there is one checklist at a time, each item saying what the app should show for the current save. The coordinator merges once tests pass and reports each merge in detail.
- The developer's PC is only for what only it can do: running the reader and extractor against the real save and install, building, packaging and installing. One PC thread per queue, in a permanent work folder that is cleaned only when Thomas says so. Installing the app needs Thomas's OK.
- Usage: check `research.md` before researching; short thread briefs; search big files with `jq`/`rg` and never load them whole; at most 2 or 3 threads at once, resolved when their work merges; batch questions; a fresh thread per feature. Default model Sonnet 5.5 on medium.

## 8. Open questions

The full list is in [open-questions.md](open-questions.md). In short:
- Do the save's fixed IDs match the data's IDs? Is there an owned-talisman record?
- How do kills and story and mission flags map to "monsters I've unlocked"?
- Do the game files contain numeric skill parameters, Artian and Gogma Artian bonus tables with pool weights, and motion values?
- How does Wilds melding work, and what does the data offer?
- Licences: MHWeditor's upstream and type data assets; whether GPL code forces an open-source Nexus release.
- How the update validation gate measures confidence.
- What the training dummy shows (Phase 5 or later).

## 9. Risks

- Game updates can break the extractor or the save reader. The app must fail visibly and keep the last good data. A read-only reader cannot damage a save.
- Getting the game's type data ourselves may be the hardest spike; if it fails, the reader and extractor stall. Hence it runs first, with fallbacks A to D.
- Extracted numbers still need interpreting; that is reverse-engineering work whatever the source.
- The model can't be checked against the real game beyond hands-on tests. The last part of accuracy costs more at each step and needs real hunt data.
- GPL code may force an open-source release.
- Covering all 14 weapons is a long project; the prototype is one.
- Building and running our own extraction tools on a PC needs Thomas's yes for each tool.
- Usage cost: mitigated by the rules in section 7.

## 10. Practical notes

- The developer's terminal is PowerShell: no `&&`, and Windows-style paths.
- Long paths can break clones ("path too long"); work in a short permanent path.
- Decrypted dumps and named JSON dumps of a save hold hunter IDs and must never be committed.
