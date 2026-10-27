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

## Phase 1 step 1: licence review (2026-10-27)

Method: read each project's GitHub page and crates.io entry on 2026-10-27 (page snapshots, so a file not shown in a listing could still exist). Status is **verified** where we read the page ourselves, **open** where it needs the author or a lawyer. This is not legal advice.

### Verdicts

| # | Item | Verdict | Evidence |
|---|------|---------|----------|
| 1 | MHWeditor type data (`rszmhwilds.json`, `enumsmhwilds.json`) | **May not use** | No LICENSE file, no licence label, no credits section; README only says "This tool modifies proprietary game data. Use at your own risk." Its page says "forked from kvasszn/ree-save-editor". The data files have no separate licence that we could find. All rights reserved. |
| 2 | Upstream of MHWeditor (the "c-ola" guess in the earlier entry is **not confirmed**) | **May not use** (code and data) | The page names its upstream as [kvasszn/ree-save-editor](https://github.com/kvasszn/ree-save-editor), not c-ola. That repo also shows no LICENSE file and no licence label. Its README says it uses texture/file-reading code from [wwylele/mhrice](https://github.com/wwylele/mhrice) (dual Apache-2.0 / MIT) and "Uses praydog's emulation dumper for the rsz files." The dumper's project, [praydog/REFramework](https://github.com/praydog/REFramework), is MIT, but its licence page says nothing about the dumped type data, and the JSON's own origin is not stated. **Open:** whether the type dump can be reused needs the authors (kvasszn, praydog). Until then it stays out. |
| 3 | [LartTyler/mhdb-wilds-data](https://github.com/LartTyler/mhdb-wilds-data) | **Code: may not use in the repo for now (GPL-3.0). Data: open** | Page lists "GPL-3.0 license" with a LICENSE file. README says the repo is the MHDB Wilds Project's "primary data source" and that merged data files are in `/output/merged`. The page does not say whether those output files are covered by the GPL, and the files are converted Capcom game data (Capcom's copyright, not the toolkit author's to license). Its tools README credits REMSG_Converter (dtlnor), ree-pak-gui (eigeen) and MHWS-Editor (Synthlight) with no licence stated on that page. **Open:** ask the author whether the output data has its own terms. |
| 4 | MHDB data / API (`wilds.mhdb.io`) | **May use as cross-check only, nothing shipped** | Docs page, read in full, has no licence, terms or attribution statement; the end of the page only links to mhdb-wilds-data without giving its licence. Page says it uses "data from the game files". Consistent with existing rules: never auto-fetched, never shipped, labelled last resort. **Open:** no terms are published, so ask the MHDB maintainers before any bundled or last-resort use. |
| 5 | Does a Nexus release force open source? | **No. Nexus does not; the GPL could** | See below. |

### Does a Nexus release force open source?

- **Our network use:** the app's update button reads the user's local game files and makes no network call. Only the MHDB last-resort fallback and route C (community dump) would connect, so the Nexus internet rule below applies only if one of those is used.
- **Nexus itself: no.** [File Submission Guidelines](https://help.nexusmods.com/article/28-file-submission-guidelines) do not mention open source, GPL or source code in general. Reading: nothing in them requires a closed or open licence for ordinary uploads.
- **Nexus conditions that matter to us:**
  - Executables that connect to the internet are "prohibited unless that connection is crucial to the mod or utility", and tools that need internet access "must be discussed with staff" with "the source code" provided. Our update button is user-triggered; auto-update does not qualify. **Open:** whether a user-pressed update fetch is allowed; it is a question for Nexus staff, and route C (downloading a community dump) falls under it.
  - Copyrighted material, including game files, needs the owner's permission; crediting is "no substitute". This matches our rule that no extracted Capcom data ships.
  - Files are virus-scanned; AI-generated code must carry the "AI-Generated Content" tag. **Open:** whether this project's code needs that tag (likely yes, since Claude writes it).
- **GPL-3.0 could force it.** If a GPL-3.0 component (for example mhdb-wilds-data code) is copied into or linked with what we distribute, the GPL requires the whole combined work to be offered under GPL-3.0 with source. If no GPL code is distributed, nothing forces open source.
- **Conclusion:** with our own code, MIT/Apache crates only, and no MHWeditor, kvasszn or mhdb-wilds-data code or data, a closed Nexus release is not forced open by any licence found. Open source becomes forced only if we choose Slint under its GPL option or take GPL code.

### Crates and tools named in the plan

| Item | Licence (crates.io / repo, 2026-10-27) | Verdict |
|------|----------------------------------------|---------|
| [ferrisetw](https://crates.io/crates/ferrisetw) 1.2.0 (ETW events for the frame-time module) | MIT OR Apache-2.0 | May use; record in THIRD-PARTY-NOTICES.md. Spike tool only, never shipped. |
| [egui](https://crates.io/crates/egui) 0.37.0 | MIT OR Apache-2.0 | May use. (Only egui itself checked; eframe and its dependencies not.) |
| [Slint](https://crates.io/crates/slint) 1.18.1 | GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0 | May use **with conditions**: the GPL option forces open source; the royalty-free option has its own terms (attribution, and we did not read its text). **Open:** read the royalty-free terms before choosing Slint for a closed release. |
| [Tauri](https://crates.io/crates/tauri) 2.12.5 | Apache-2.0 OR MIT | May use. Bundles a system WebView; not checked. |
| [Electron](https://github.com/electron/electron/blob/main/LICENSE) | MIT ("Copyright (c) Electron contributors / GitHub Inc."; wording is the standard MIT text) | May use. It bundles Chromium with its own notices; not checked. |
| Rust toolchain, cargo | Not fetched this round | **Open** (plan says MIT/Apache-2.0, "to confirm"). Not confirmed here. |

Every dependency of any shipped crate still needs its own check at Phase 2 (for example `cargo-deny` style listing in CI); only top-level crates were read here.

### Decisions this supports
- `THIRD-PARTY-NOTICES.md` is created at Phase 2, when the first third-party crate enters the repo. No file is needed in Phase 1, which merges no code.
- Route C (community dump) is **not cleared**: the only dump we found (MHWeditor's) has no licence. Treat C as blocked unless an author grants permission.
- Choosing a UI toolkit does not need a licence change for egui or Tauri.

### Still open (for the author, Nexus staff or a lawyer)
- Reuse terms of the type dump behind MHWeditor (kvasszn, praydog).
- Whether mhdb-wilds-data's output files carry the GPL or other terms.
- MHDB API terms (none published on the docs page).
- Whether Nexus allows a user-pressed update download, and the AI-generated tag.
- Slint royalty-free terms.
- Licence of Rust toolchain.
