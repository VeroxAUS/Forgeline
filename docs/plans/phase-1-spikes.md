# Phase 1: Spikes

Target version: 0.1.0 · Status: draft (awaiting Thomas's review; nothing starts until approved)

Each phase plan is written just before its phase, using what earlier phases found. Thomas reviews and approves it before any work starts. Estimates are in threads and test rounds, not hours; real durations go in the time log (section 10).

## 1. Goal and done
Answer the unknowns that decide the architecture, each with a pass or fail result: can we get the game's type data, can we read the save and map its IDs, what the game files hold, what the licences allow, and which UI toolkit is light enough beside Wilds. Spikes produce findings in `docs/research.md`, not product code. Any spike code is throwaway, lives only on its PR branch (or under `spikes/`, never reused as the foundation without a rewrite in Phase 2), and is never merged to `main` unless Thomas says so.

Done means:
- [ ] Each of the six spikes has a recorded result (pass, partial, fail) in `docs/research.md`, with evidence and a status of verified, reported or open.
- [ ] The type-data route is chosen (A, B, C or D, or MHDB last resort) and the reason is written down.
- [ ] Open questions 1 to 6 and 8 in `open-questions.md` are answered, or re-stated with what is still missing (coordinator moves answers to `closed-questions.md`).
- [ ] The licence review states what a Nexus release may include and whether the project must go open source.
- [ ] The UI toolkit decision (pure Rust, Tauri or Electron) is made from measured numbers with Wilds running, against the performance budget drafted with Thomas.
- [ ] `PLAN.md` is updated with the results, and the Phase 2 plan can be written from them.
- [ ] No save, decrypted dump or extracted Capcom data is in git (checked by the test round).

## 2. Inputs and dependencies
- `docs/PLAN.md` sections 3 (verified findings), 4 (design decisions), 9 (risks); `CLAUDE.md` standing rules; `docs/open-questions.md`; `docs/closed-questions.md`; `docs/research.md` (MHWeditor licence entry and save format facts already known).
- Thomas's PC with Wilds installed, a copy of his save, and the Steam library found through `libraryfolders.vdf`.
- Thomas's yes for each tool in section 5, and a draft performance budget (open question 9) before spike 6 is judged.
- Nothing is assumed about Wilds data beyond what `PLAN.md` marks as verified. Anything else a spike finds is recorded as new evidence; game facts are never filled in from memory.

## 3. Design

### 3.1 Architecture
There is no product architecture in this phase. The spikes are independent except for these orderings:

1. **Spike 1 (type data) runs first.** Spikes 3 and 4 need its result.
2. **Spike 2 (clean-room study)** runs in the cloud and feeds spike 3 with notes only.
3. **Spike 5 (licence review)** runs in the cloud in parallel with everything. Its results can change the route chosen in spike 1 (fallback C) and the plan for open source.
4. **Spike 6 (UI comparison)** is independent of the data spikes, but needs the performance budget and Thomas's PC with Wilds running.

Clean-room rule: the thread that studies MHWeditor (spike 2) writes facts in its own words to `research.md` and never writes code. The thread that writes spike code (spikes 3 and 4) works from `research.md` only and does not open MHWeditor. Threads must be different sessions.

Spike code on the PC is read-only on the game: it opens copies of the save and reads the game files from the install without writing to the install or Steam folders. Heavy work is user-triggered and runs at low priority.

### 3.2 Per-file design
Spikes mostly produce documents. Where code is needed it is throwaway and the file list is indicative; if a spike keeps something, the Phase 2 plan redesigns it properly.

`docs/research.md` (changed by every spike)
- **Purpose:** the single record of results. One section per spike with: question, method, result (pass, partial or fail), evidence (counts, names of classes and fields, never raw dumps), status, and the decision it supports.
- **Notes:** no Capcom data, no hunter IDs, no secrets. Field and class names and counts are fine; full dumps are not.

`spikes/typedump/` (spike 1, throwaway, Rust)
- **Purpose:** try route A: read the game `.exe` on disk and recover the class, field and enum information the save and game files are described by.
- **Public API:** `fn main(exe_path, out_dir)`, writes a summary (counts, a small sample) and the recovered type data to a local, git-ignored folder.
- **Depends on:** nothing in the repo.
- **Notes:** static reading only; never launches the game, never injects or reads game memory. Output goes outside git. Success is judged in section 6.

`spikes/savecheck/` (spikes 2 and 3, throwaway, Rust, written from `research.md` notes only)
- **Purpose:** decrypt a copy of the save using the Steam ID, parse it with the type data from spike 1, and test ID matching and talisman location.
- **Public API:** `fn main(save_copy_path, steam_id, type_data_path)` printing counts and match rates only.
- **Depends on:** `spikes/typedump` output, `research.md` notes.
- **Notes:** takes a copy path and refuses the live `win64_save` folder. Prints no hunter IDs.

`spikes/extract/` (spike 4, throwaway, Rust)
- **Purpose:** locate and read the specific files for skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops and motion values from the install's packs, extracting only those files.
- **Public API:** `fn main(game_dir, out_dir, categories)`.
- **Depends on:** spike 1 result.
- **Notes:** reads pack indexes and individual entries; never copies or unpacks whole packs (the main one is about 41 GB). Reads original packs only; mod markers (`dinput8.dll`, manager files) are noted, not used. Output stays outside git.

`spikes/uibench/` (spike 6, throwaway, Rust plus web code for the HTML/JS candidates)
- **Purpose:** three equivalent minimal apps showing a table of a few hundred rows with a dark theme and a button that triggers a background job: a pure-Rust UI, a Tauri app and an Electron app.
- **Notes:** no network listener, no telemetry. Same content in all three so the numbers are comparable.

### 3.3 Data formats
Results only: markdown in `docs/research.md`. Raw spike output (type data, extracted files, parse dumps) is written to the permanent PC work folder (proposed below), outside any git checkout, and never committed or attached to a PR.

**Proposed permanent PC work folder (for Thomas to approve or change):** `C:\mhbis` (short path to avoid the "path too long" clone problem). Proposed layout: `C:\mhbis\repo` (git checkout), `C:\mhbis\copies` (save copies), `C:\mhbis\out` (spike output, never committed), `C:\mhbis\bench` (UI benchmark results). It is not cleaned without Thomas's say-so. Dev and test builds use a data directory separate from his installed app.

### 3.4 Performance impact
No product impact. For spike 6 the measurements are: cold start to first usable window, idle RAM, idle CPU over a fixed window, and GPU use, each taken with Wilds running and without. Pass thresholds come from the performance budget drafted with Thomas (open question 9). Until that exists, spike 6 records numbers and compares the three options relative to each other. The budget draft is an input Thomas needs to review before the spike is judged.

## 4. Steps
Each step is one thread, one PR and one test round. Spike PRs are doc PRs (results in `research.md`); the PR says whether the code was run against the real files, and spike code stays out of `main` unless approved. Test rounds for PC spikes are queued one at a time.

| # | Step | Thread | Cloud or PC | Output (PR / doc) | Estimate (threads, test rounds) | Depends on |
|---|------|--------|-------------|-------------------|---------------------------------|------------|
| 5 | Licence review: type-data asset licence, c-ola upstream, `mhdb-wilds-data` (GPL-3.0), MHDB data, what Nexus requires of open source, THIRD-PARTY-NOTICES.md decision | Licence thread (cloud) | Cloud | PR: `research.md` section | 1 thread, 1 cloud test round | none, runs in parallel from the start |
| 2 | Clean-room study of the MHWeditor save format, in own words, with credit | Study thread (cloud, never writes code) | Cloud | PR: `research.md` section | 1 thread, 1 cloud test round | none, parallel |
| 1 | Type-data spike: routes A, B, C, D in order, stopping at the first that passes; MHDB last resort | Type-data thread (PC, plus cloud prep for the report) | PC | PR: `research.md` section; spike code on branch only | 1 to 2 threads, 1 to 2 PC test rounds | Thomas's yes on tools; step 5 informs route C |
| 3 | Save-ID to data-ID matching and owned talismans on a save copy | Save thread (PC) | PC | PR: `research.md` section | 1 thread, 1 PC test round | 1, 2 |
| 4 | Extraction test on Thomas's install: skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops, motion values, all weapons | Extraction thread (PC) | PC | PR: `research.md` section (counts, coverage table, what stays hand-written) | 1 to 2 threads, 1 to 2 PC test rounds | 1 |
| 6 | UI comparison: pure-Rust UI vs Tauri vs Electron, with Wilds running | UI bench thread (PC) | PC | PR: `research.md` section with numbers | 1 thread, 1 PC test round | performance budget draft, Thomas's yes on tools |
| 7 | Close-out: update `PLAN.md`, move answered questions (coordinator), record licence and toolkit decisions | Docs thread (cloud) | Cloud | PR: `PLAN.md` | 1 thread, 1 cloud test round | 1 to 6 |

Total estimate: 7 to 9 threads, 7 to 9 test rounds (3 to 5 on the PC). At most 2 or 3 threads run at once; steps 2 and 5 run together, then step 1 starts, then 3, 4 and 6 follow as their inputs arrive. Estimates are guesses and will be compared with the time log. The type-data spike is the largest unknown; if it needs more than 2 threads, stop and report to Thomas rather than continue (Opus on high may then be worth suggesting).

## 5. Tools (need Thomas's yes)
Nothing below is created or run until Thomas says yes to that row. All tools are our own Rust unless stated.

| Tool | Why | Built or third-party | Licence |
|------|-----|----------------------|---------|
| `spikes/typedump` (static `.exe` reader) | Route A for type data (spike 1) | Built, own Rust | ours; any crate used is recorded in THIRD-PARTY-NOTICES.md |
| `spikes/savecheck` (save decrypt, parse, ID match, talisman search) | Spike 3 | Built, own Rust, from `research.md` notes only | ours |
| `spikes/extract` (pack index reader, per-file extraction) | Spike 4 | Built, own Rust | ours |
| `spikes/uibench` (three equivalent minimal apps) | Spike 6 | Built, own Rust and web code | ours; UI toolkits (egui or Slint, Tauri, Electron) and their licences recorded |
| Rust toolchain, `cargo`, GitHub Actions lint and test jobs | Building and CI | Third-party (standard) | MIT/Apache-2.0 (to confirm in step 5) |
| A process-measurement method for spike 6 (Windows Performance Monitor or Task Manager counters, or a small own Rust sampler) | Cold start, RAM, CPU, GPU numbers | Decision for Thomas: built (own sampler) or built-in Windows tools | Windows tool or ours |
| Route C only: a community type dump, downloaded, never run through an injecting tool | Fallback C if A and B fail and the licence allows | Third-party data | Checked in step 5 before use |

Not proposed: any injecting, hooking or memory-reading tool. Those are ruled out by the standing rules.

## 6. Tests
**Automated** (GitHub Actions, fixture saves and fixture data only): spike code is throwaway, so automated tests are limited to lint and a check that no save, dump or extracted-data file is tracked in git (a `.gitignore` and a CI file-pattern check). Spike code that touches the reader or extractor is declared untested against the real files in the PR until the PC round runs.

**Pass or fail criteria per spike** (each recorded in `research.md`):
1. **Type data.** Pass: the recovered type data lets us parse a copy of the save into named classes and fields for the classes we need (the save root, equipment, items, enemy report, story, mission, workshop), with names matching the ones already seen in the format facts, and parse a sample of game data files. Partial: only a subset of classes is recovered (route B can complete it). Fail: route A recovers nothing usable; move to B, then C (only if licence allows), then D, then MHDB. The result states which route was chosen and why.
2. **Clean-room study.** Pass: `research.md` holds the save container layout, the encryption and key derivation, and the structure facts needed by spike 3, in own words with credit, with no copied code or type dumps. Fail: key facts are missing and need another source.
3. **ID matching and talismans.** Pass: at least 99% of the fixed IDs in the save for weapons, armor, decorations and items resolve to the same entries in the game data (checked by names), or a deterministic mapping rule is found. For talismans: pass if owned talismans are found in the save and their skills decoded, fail if no record exists (then the plan states how the app treats talismans). Partial results list which categories match.
4. **Extraction.** Pass per category: skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops and motion values are each found in the files, counts are plausible against the MHDB cross-check (MHDB is only a cross-check), and all weapon types are covered. For each category the result is **found and decoded**, **found but not decoded** or **not in the files**, and the note says what stays hand-written. A category that is not in the files means the corresponding product feature is marked at risk in the plan.
5. **Licences.** Pass: each of the five items has a verdict (may use, may use with conditions, may not use) with the licence text cited, plus a clear statement on whether a Nexus release forces open source. Items that need a lawyer or the author are marked open rather than guessed.
6. **UI comparison.** Pass: a candidate meets the performance budget with Wilds running (cold start, idle RAM, idle CPU, GPU) and does not stutter the game during its background job. Decision rule: choose pure Rust if it meets the budget; otherwise the lightest option that does. Numbers are measured on Thomas's PC with Wilds running, repeated at least 3 times, with the median recorded.

**Test round checklist** (queued, one at a time). Each item says what the app should show for Thomas's current save. No app exists in this phase, so the "app" here is the spike output:
- [ ] Step 1 (PC): the type-data summary lists the classes needed for the save and counts of fields and enums, and says which route succeeded.
- [ ] Step 3 (PC): with your current save copy, the report shows 3 character slots, one marked active, the active slot's equipment box count, and an ID match rate per category with the names of unmatched IDs (names only).
- [ ] Step 3 (PC): the report says whether owned talismans were found and how many, with skill names, and no hunter IDs.
- [ ] Step 4 (PC): the extraction report lists, per category, the number of entries found and its status (found and decoded, found but not decoded, not in files), for all weapon types.
- [ ] Step 6 (PC): the UI benchmark table shows cold start, idle RAM, CPU and GPU for each of the three options, with and without Wilds running, with the median of 3 runs.
- [ ] Every step: no save, dump or extracted Capcom data appears in git or in the PR; the PC thread reports it did not write to the save, the game install or the Steam folders.

## 7. Risks and open questions
- **Route A fails (the hardest spike).** Notice: nothing recovered, or names do not match. Fallback: B, then C, then D, then MHDB labelled older and unsupported. Report to Thomas before spending more than 2 threads.
- **Types recovered but game updates change them.** Notice: spike run on a later build differs. Fallback: record the build ID with each result; the Phase 2 extractor must tolerate change visibly.
- **Fixed IDs do not match game data IDs.** Notice: match rate under 99%. Fallback: a mapping table, name-based matching, or hash and pattern matching (route D); the plan then records the cost.
- **Owned talismans not in the save or not decodable.** Fallback: the user enters talismans by hand, or they are treated as optional; recorded as a product decision for Thomas.
- **The game files do not hold Artian tables, motion values or skill numbers.** Fallback: hand-written tables from cited sources, or dropped features; results feed Phases 4 and 8.
- **Licence blocks route C or forces open source.** Fallback: routes A, B or D; or the project goes open source, which Thomas has already said is acceptable if needed.
- **UI candidates all miss the budget.** Fallback: pure Rust with the lightest toolkit anyway, with the numbers shown to Thomas.
- **Install is modded.** Notice: mod markers in the install folder. Fallback: read original packs only and report the marker.
- **Leaking personal data in reports.** Notice: any hunter ID in output. Fallback: reports print counts and names only; the checklist item above checks it.
- **Questions for Thomas (via the coordinator):** (a) approve the tool list in section 5 row by row; (b) approve or change `C:\mhbis`; (c) when the performance budget will be drafted, since spike 6 depends on it; (d) whether spike code may stay on a PR branch only, or may be kept in `spikes/` on `main`; (e) for spike 6, Windows built-in counters or our own small sampler. New questions go to `docs/open-questions.md` via the coordinator.

## 8. Versioning and changelog
Version 0.1.0 is declared when the phase is closed (step 7). There is no product and no changelog before Phase 2 (changelog starts at 0.2.0 per the standing rules). Spike steps do not bump the version.

## 9. Review log
| Date | Reviewer | Edit or decision |
|------|----------|------------------|
| 2026-10-27 | (draft) | Written by a thread for Thomas's review; not yet reviewed |

## 10. Time log
| Event | Date and time |
|-------|---------------|
| Phase approved by Thomas | |
| Step 5 (licence review) PR merged | |
| Step 2 (clean-room study) PR merged | |
| Step 1 (type data) PR merged | |
| Step 3 (ID matching, talismans) PR merged | |
| Step 4 (extraction) PR merged | |
| Step 6 (UI comparison) PR merged | |
| Step 7 (close-out) PR merged | |
| Phase finished | |

Elapsed from approval to finish: <hours>. Compare with the estimates in section 4.
