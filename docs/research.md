# Research log

Spike and research results go here. Check this file before researching anything, so nothing is looked up twice.

Each entry: date, question, finding, source (cited), and status (**verified** = seen ourselves, **reported** = from a source, unchecked, **open**). Never fill gaps from memory.

## MHWeditor licence (2026-10-27)

Question: may we reuse anything from MHWeditor (https://github.com/this-nerd/MHWeditor)?

Finding (**verified** by reading the repository on 2026-10-27):
- No LICENSE file.
- None of its five Rust package files (Cargo.toml) has a licence field.
- The README has no licence section, only a disclaimer: it modifies proprietary game data, is used at your own risk, and the author is not responsible for banned accounts or lost saves.

Reading: with no licence, normal copyright applies. We have no permission to copy, modify or redistribute it. It is treated as all rights reserved.

Consequences:
- No MHWeditor code, type data files (`rszmhwilds.json`, `enumsmhwilds.json`) or close rewrites enter this repo.
- One thread may study it to learn facts about the save format and write them here in its own words, with credit. A separate thread writes our reader from those notes only, without opening MHWeditor.

Not yet checked (**open**):
- Whether the type data assets carry a different licence from the code.
- Its code appears to derive partly from another author's tools (possibly c-ola's). The upstream licence has not been checked.

## Save format facts already known

Findings from the design conversation (2026-10-27), source: a spike using MHWeditor on a copy of a save. Full detail is in [PLAN.md](PLAN.md); keep any further notes here in own words.
- **Verified:** the save is encrypted with the Steam ID. The 64-bit Steam ID is the folder number under `Steam\userdata` plus `0x0110000100000000`.
