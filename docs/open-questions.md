# Open questions

Written only by the coordinator. This file holds research unknowns that the plan still has to answer, plus any question awaiting the maintainer. Every question asked to the maintainer is recorded here; once answered, it moves to [closed-questions.md](closed-questions.md).

Waiting on the maintainer: the four remaining Phase 2 plan decisions below (B to E; A was answered) (asked 2026-10-27, PR #21). Phase 1 is closed; the items below are the research unknowns it left open, with what each still needs and which phase picks it up.

## Open

Numbers are kept stable; answered questions leave a gap (question 8 moved to closed-questions.md).

1. **Save-ID to data-ID mapping.** Do the fixed IDs in the save match the IDs in the game data and in MHDB? Partly answered (step 4): every checked save ID falls in the data ID range, the armor matches the save's own crafted records exactly, and the counts match the maintainer's game. The check by name is not done (needs the text files) and moves to Phase 3. Step 3 found where equipment and item records live (`cEquipParam`, `cItemParam`); what the numbers mean is still unknown. Next: Phase 3 (text files and data-row decoders).
2. **Owned talismans.** Is there an owned-talisman record in the save, and where? Partly answered: step 3 found a `cCharmEquipped` class (talismans in use). Step 4 found one likely talisman entry (confirmed as a count by the maintainer), but its skills are not decoded and how random talismans are stored is unknown. Still open (Phase 3).
3. **Unlocked monsters.** How do kills (`SlayingNum`) and story and mission flags map to "monsters I've unlocked"? Partly answered: step 3 located the records (`cBossReportParam`, 300 entries; `cStoryParam`; `cMissionParam`). The mapping to unlocked monsters is still open (step 4). Which character slot is active is also open.
4. **Game file contents.** Do the game files hold numeric skill parameters, Artian and Gogma Artian bonus tables with pool weights, and motion values? Partly answered (step 5): skills, weapons (14 types), armor, decorations, charms, Artian, items and the monster files are located in the packs and readable, but none is decoded into named fields yet. Gogma Artian tables are not separately identified. Motion values were not found (see question 11). Phase 3 needs a hand-written schema (route B) per data class to answer the rest.
5. **Melding.** How does Wilds melding work, and what does the data offer? Not investigated in Phase 1. Picked up with Phase 7 (farming, decorations and melding), or earlier if the Phase 3 data survey finds melding tables.
6. **Licences, what is left.** The main questions are answered (see closed-questions.md). Still open: Slint's royalty-free terms, the Rust toolchain licence, whether mhdb-wilds-data's output files carry GPL or other terms, MHDB API terms (none published), whether Nexus allows a user-pressed update download, and the licences of the libraries our libraries use.
7. **Training dummy.** What does it display (per-hit numbers, totals, DPS)? Phase 5 or later.
9. **Update validation gate.** How to implement the >95% confidence check (re-scoring old versus new data after skill, set bonus or weight changes)?
10. **Nexus comments and bugs.** Does Nexus's GraphQL v2 expose comments or bug reports, and does Nexus email authors about them? Check the maintainer's author notification settings once the mod page is live (see `docs/research.md`, "Nexus monitoring").
11. **Motion values and base power.** Motion values, and attack and ammo base power (for example bowgun ammo values), were not found in the game files in step 5. The Phase 4 damage formula and any future simulation need them. The Phase 3 and Phase 4 plans must address this early: a cited source, or Phase 5 training-room measurement.

## Waiting on the maintainer (asked 2026-10-27, Phase 2 plan, PR #21)

B. **RAM rule for every PR.** A headless CI check on every PR, plus a PC reading on every PR that changes the app; at most +3 MB private bytes per PR without a reason; at most 140 MB private bytes and 80 MB working set in total. Approve?
C. **Update as a child process.** Run the update as a short-lived child process of the app instead of a thread, so its memory never counts against the thin RAM headroom. Approve?
D. **Needed-files path list in the repo.** Keep a hand-written list of about 80 game file paths (paths only, no Capcom data) in `crates/forgeline-pak/src/paths.rs`. Approve?
E. **Budget changes (decided in step 10).** Split "Idle RAM" into working set (target 80 MB) and private bytes (hard limit 150 MB); make "no stutter" a number (game median and 99th percentile frame time within 1 ms of the control run); add rows for "last view on screen" and "save change to updated view". Agree in principle?
