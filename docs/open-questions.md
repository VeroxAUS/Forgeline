# Open questions

Written only by the coordinator. This file holds research unknowns that the plan still has to answer, plus any question awaiting the maintainer. Every question asked to the maintainer is recorded here; once answered, it moves to [closed-questions.md](closed-questions.md).

Items 12 to 16 are waiting on the maintainer. The rest are research unknowns that the Phase 1 spikes are working on.

## Open

1. **Save-ID to data-ID mapping.** Do the fixed IDs in the save match the IDs in the game data and in MHDB? Partly answered (step 4): every checked save ID falls in the data ID range, the armor matches the save's own crafted records exactly, and the counts match the maintainer's game. The check by name is not done (needs the text files) and moves to Phase 3. Step 3 found where equipment and item records live (`cEquipParam`, `cItemParam`); what the numbers mean is still unknown.
2. **Owned talismans.** Is there an owned-talisman record in the save, and where? Partly answered: step 3 found a `cCharmEquipped` class (talismans in use). Step 4 found one likely talisman entry (confirmed as a count by the maintainer), but its skills are not decoded and how random talismans are stored is unknown. Still open.
3. **Unlocked monsters.** How do kills (`SlayingNum`) and story and mission flags map to "monsters I've unlocked"? Partly answered: step 3 located the records (`cBossReportParam`, 300 entries; `cStoryParam`; `cMissionParam`). The mapping to unlocked monsters is still open (step 4). Which character slot is active is also open.
4. **Game file contents.** Do the game files hold numeric skill parameters, Artian and Gogma Artian bonus tables with pool weights, and motion values? How are they stored and named? Open (step 5; the pack reader is under way).
5. **Melding.** How does Wilds melding work, and what does the data offer?
6. **Licences, what is left.** The main questions are answered (see closed-questions.md). Still open: Slint's royalty-free terms, the Rust toolchain licence, whether mhdb-wilds-data's output files carry GPL or other terms, MHDB API terms (none published), whether Nexus allows a user-pressed update download, and the licences of the libraries our libraries use.
7. **Training dummy.** What does it display (per-hit numbers, totals, DPS)? Phase 5 or later.
8. **UI toolkit.** Does pure-Rust UI pass the Phase 1 comparison against Tauri and Electron (measured with Wilds running)? Step 6.
9. **Update validation gate.** How to implement the >95% confidence check (re-scoring old versus new data after skill, set bonus or weight changes)?
10. **Nexus comments and bugs.** Does Nexus's GraphQL v2 expose comments or bug reports, and does Nexus email authors about them? Check the maintainer's author notification settings once the mod page is live (see `docs/research.md`, "Nexus monitoring").
11. **Motion values and base power.** Motion values, and attack and ammo base power (for example bowgun ammo values), were not found in the game files in step 5. The Phase 4 damage formula and any future simulation need them. The Phase 3 and Phase 4 plans must address this early: a cited source, or Phase 5 training-room measurement.

### Asked to the maintainer (2026-10-27, licence and donations; see [research.md](research.md), "Licence options and donations")

12. **Source-available licence.** Is a source-available licence (not OSI open source) acceptable for the "ask permission to republish" rule? Recommended: PolyForm Shield 1.0.0. The open-source alternative is MPL-2.0, which cannot give that rule.
13. **Competing forks.** Is it fine to bar all competing forks, including free ones that add features and share them as their own?
14. **Fork names.** Must forks rename, or is "based on Forgeline" enough?
15. **Contributions.** Should contributions be licensed to the maintainer so the licence can change later, or stay under the contributor's own copyright?
16. **Tip link.** A voluntary tip link with no perks: yes or no? If yes, GitHub only, or also on Nexus?
