# Open questions

Written only by the coordinator. Every question asked to Thomas is recorded here; once answered, it moves to [closed-questions.md](closed-questions.md).

## Open

1. **Save-ID to data-ID mapping.** Do the fixed IDs in the save match the IDs in the game data and in MHDB?
2. **Owned talismans.** Is there an owned-talisman record in the save, and where?
3. **Unlocked monsters.** How do kills (`SlayingNum`) and story and mission flags map to "monsters I've unlocked"?
4. **Game file contents.** Do the game files hold numeric skill parameters, Artian and Gogma Artian bonus tables with pool weights, and motion values? How are they stored and named?
5. **Melding.** How does Wilds melding work, and what does the data offer?
6. **Licences.** What MHWeditor's upstream (possibly c-ola) allows; the licence of its type data assets; whether GPL code forces the Nexus release to be open source.
7. **Training dummy.** What does it display (per-hit numbers, totals, DPS)? Phase 5 or later.
8. **UI toolkit.** Does pure-Rust UI pass the Phase 1 comparison against Tauri and Electron (measured with Wilds running)?
9. **Performance budget.** Target numbers for cold start, idle RAM, idle CPU and recompute time, to be drafted together with Thomas.
10. **Update validation gate.** How to implement the >95% confidence check (re-scoring old versus new data after skill, set bonus or weight changes)?
