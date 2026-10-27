# Phase 1: Spikes

Target version: 0.1.0 · Status: closed at step 7 (approved by the maintainer 2026-10-27 13:05 UTC; results in [PLAN.md](../PLAN.md) section 6)

Each phase plan is written just before its phase, using what earlier phases found. The maintainer reviews and approves it before any work starts. Estimates are in threads and test rounds, not hours; real durations go in the time log (section 10).

## How to read this plan

**Run order** (steps are numbered in the order they run; parallel steps share a stage):

- **Stage 1** (cloud, parallel, start straight away): step 1 licence review, step 2 clean-room study of the MHWeditor save format.
- **Stage 2** (PC): step 3 type-data spike.
- **Stage 3** (PC): step 4 ID matching and talismans, step 5 extraction test, step 6 UI comparison. Steps 4 and 5 need step 3. Step 6 needs only the performance budget and can start earlier if the maintainer prefers.
- **Stage 4** (cloud): step 7 close-out.

**Decisions made by the maintainer (2026-10-27):**

- All tools in section 5 are approved.
- Step 6 measurement is our own cross-platform Rust sampler, not Windows built-ins.
- PC work folder is `E:\mhbis`.
- Spike code stays on separate PR branches and is never merged to `main`; at the end of Phase 1 the branches are collected into a `spikes-archive` branch, kept for reference (an approved exception to the main-only rule).
- The performance budget is in [performance-budget.md](../performance-budget.md) (initial pass, reviewed over time).

- Game frame times are measured by our own Windows-only module of the sampler (`spikes/sampler`), reading Windows present events (ETW, PresentMon-style) without touching the game process. It needs admin rights while running and is a test-PC tool, never shipped.
- Minimum spec: limit cores and RAM on the maintainer's PC, with Wilds' official minimum spec (cited from Capcom's page) as the baseline.

**Still open:**

- The minimum spec number itself (decided near release).

## 1. Goal and done
Answer the unknowns that decide the architecture, each with a pass or fail result: can we get the game's type data, can we read the save and map its IDs, what the game files hold, what the licences allow, and which UI toolkit is light enough beside Wilds.

Spikes produce findings in `docs/research.md`, not product code. Spike code is throwaway: it lives only on its own PR branch and is never merged to `main`. When Phase 1 is done the branches are collected into a `spikes-archive` branch kept for reference. Phase 2 rewrites anything it keeps.

Done means:

- [x] Each of the seven steps has a recorded result in `docs/research.md` (pass, partial or fail), with evidence and a status of verified, reported or open.
- [x] The type-data route is chosen (A, B, C or D, or the MHDB last resort), with the reason written down.
- [x] Open questions 1 to 6 and 8 are answered, or re-stated with what is still missing (the coordinator moves answers to `closed-questions.md`).
- [x] The licence review states what a Nexus release may include and whether the project must go open source.
- [x] The UI toolkit (pure Rust, Tauri or Electron) is chosen from numbers measured with Wilds running, against the performance budget drafted with the maintainer.
- [x] `docs/PLAN.md` is updated with the results, so the Phase 2 plan can be written from them.
- [x] `README.md` is updated at close-out (status and version badges, roadmap progress).
- [x] No save, decrypted dump or extracted Capcom data is in git.

Close-out notes: steps 4 and 5 ended partial (by-name ID matching, talisman skills, data-file decoding and motion values are carried to Phases 3 and 4 in `open-questions.md`); route A is chosen for the save and route B schemas are needed for game data; egui is the UI; MPL-2.0 is the licence.

## 2. Inputs and dependencies

- `docs/PLAN.md` sections 3 (verified findings), 4 (design decisions) and 9 (risks).
- `CLAUDE.md` standing rules, `docs/open-questions.md`, `docs/closed-questions.md`, and `docs/research.md` (the MHWeditor licence entry and the save format facts already known).
- The maintainer's PC with Wilds installed, a copy of the maintainer's save, and the Steam library found through `libraryfolders.vdf`.
- The maintainer's yes for each tool in section 5.
- A draft performance budget (open question 9) before step 6 is judged.
- Nothing is assumed about Wilds data beyond what `PLAN.md` marks as verified. Anything else a step finds is recorded as new evidence; game facts are never filled in from memory.

## 3. Design

### 3.1 Architecture
There is no product architecture in this phase. Each step is listed with how it relates to the others.

- **Step 1, licence review (cloud, Stage 1):** no ordering needed to start. Its verdicts can change the route chosen in step 3 (fallback C) and the open-source question.
- **Step 2, clean-room study (cloud, Stage 1):** no ordering needed to start. Its notes in `research.md` are the only input to the code in step 4.
- **Step 3, type data (PC, Stage 2):** starts after the maintainer approves its tool. Steps 4 and 5 wait for its result.
- **Step 4, ID matching and talismans (PC, Stage 3):** needs step 3's type data and step 2's notes.
- **Step 5, extraction test (PC, Stage 3):** needs step 3's result. Independent of step 4, so the two can run in the same stage.
- **Step 6, UI comparison (PC, Stage 3):** independent of the data steps. Needs [performance-budget.md](../performance-budget.md) and the maintainer's PC with Wilds running.
- **Step 7, close-out (cloud, Stage 4):** needs steps 1 to 6.

Clean-room rule: the thread for step 2 studies MHWeditor and writes facts in its own words, and never writes code. The thread for step 4 writes code from `research.md` only and does not open MHWeditor. They must be different sessions.

PC safety: spike code reads copies of the save and reads the game files from the install. It never writes to the save, the install or the Steam folders, never launches the game for its own purposes, and never injects, hooks or reads game memory. Heavy work is started by hand and runs at low priority.

### 3.2 Per-file design
Spikes mostly produce documents. Code is throwaway and the file list is indicative; Phase 2 redesigns anything it keeps.

#### `docs/research.md` (steps 1 to 7)

- **Purpose:** the single record of results, one section per step.
- **Inputs and outputs:** in, each step's findings. Out, per step: question, method, result (pass, partial or fail), evidence (counts, class and field names, never raw dumps), status (verified, reported or open), and the decision it supports.
- **Depends on:** nothing.
- **Notes:** no Capcom data, no hunter IDs, no secrets.

#### `spikes/typedump/` (step 3, throwaway Rust)

- **Purpose:** try route A: read the game `.exe` on disk and recover the class, field and enum information that the save and game files are described by.
- **Inputs and outputs:** in, the path to the `.exe` and an output folder. Out, a short summary (counts and a small sample) plus the recovered type data in a local, git-ignored folder.
- **Depends on:** nothing in the repo.
- **Notes:** static reading only. Output stays outside git. Success is judged in section 6.2.

#### `spikes/savecheck/` (step 4, throwaway Rust, written from `research.md` notes only)

- **Purpose:** decrypt a copy of the save with the Steam ID, parse it with step 3's type data, and test ID matching and talisman location.
- **Inputs and outputs:** in, a save copy path, the Steam ID and the type data path. Out, counts and match rates only.
- **Depends on:** `spikes/typedump` output, step 2's notes in `research.md`.
- **Notes:** takes a copy and refuses the live `win64_save` folder. Prints no hunter IDs.

#### `spikes/extract/` (step 5, throwaway Rust)

- **Purpose:** find and read only the files for skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops and motion values.
- **Inputs and outputs:** in, the game folder, an output folder and the categories to read. Out, per category a count and a status, plus the extracted files in a local, git-ignored folder.
- **Depends on:** step 3's result.
- **Notes:** reads pack indexes and single entries, never whole packs (the main one is about 41 GB). Reads original packs only. Mod markers such as `dinput8.dll` are reported, not used.

#### `spikes/sampler/` (step 6, throwaway Rust)

- **Purpose:** cross-platform sampler for cold start, RAM, CPU and GPU of a process, so the three UI test apps are measured the same way.
- **Inputs and outputs:** in, a process to launch or attach to, and a sampling window. Out, a results file in `E:\mhbis\bench` with the median of 3 runs, including game frame-time statistics from the frame-time module.
- **Depends on:** `spikes/uibench`.
- **Notes:** the cross-platform core reads process figures only. A Windows-only frame-time module listens to Windows present (ETW) events to measure the game's frame times and stutter; it never touches the game process, needs admin rights while running, runs on the test PC only and is never shipped. Any ETW crate's licence is checked in step 1 first.

#### `spikes/uibench/` (step 6, throwaway Rust plus web code)

- **Purpose:** three equivalent minimal apps for the comparison: a pure-Rust UI, a Tauri app and an Electron app.
- **Inputs and outputs:** in, none. Out, a window with a dark-themed table of a few hundred rows and a button that starts a background job; the numbers come from the measurement in section 3.4.
- **Depends on:** nothing in the repo.
- **Notes:** same content in all three so the numbers compare. No network listener, no telemetry.

### 3.3 Data formats
Results only: markdown in `docs/research.md`. Raw output (type data, extracted files, parse dumps) goes to the permanent PC work folder, outside any git checkout, and is never committed or attached to a PR.

**Proposed permanent PC work folder (the maintainer to approve or change):** `E:\mhbis`, a short path to avoid the "path too long" clone problem.

- `E:\mhbis\repo`: the git checkout.
- `E:\mhbis\copies`: save copies.
- `E:\mhbis\out`: spike output, never committed.
- `E:\mhbis\bench`: UI benchmark results.

It is not cleaned without the maintainer's say-so. Dev and test builds use a data directory separate from the maintainer's installed app.

### 3.4 Performance impact
No product impact. Step 6 measures, with Wilds running and without:

- cold start to first usable window
- idle RAM
- idle CPU over a fixed window
- GPU use
- whether the background job stutters the game

Pass thresholds come from [performance-budget.md](../performance-budget.md) (initial pass, test PC 13700K / RTX 4080 / 32 GB). Measured with our own cross-platform Rust sampler, median of 3 runs. Game frame times come from the Windows-only frame-time module of `spikes/sampler` (ETW present events, never touching the game process).

## 4. Steps
Each step is one thread, one PR and one test round. Spike PRs are doc PRs (results in `research.md`); each PR says whether its code ran against the real files, and spike code stays out of `main` unless approved. PC test rounds are queued one at a time.

| # | Stage | Step | Thread | Cloud or PC | Output (PR / doc) | Estimate (threads, test rounds) | Depends on |
|---|-------|------|--------|-------------|-------------------|---------------------------------|------------|
| 1 | 1 | Licence review: type-data asset licence, c-ola upstream, `mhdb-wilds-data` (GPL-3.0), MHDB data, what Nexus requires of open source, THIRD-PARTY-NOTICES.md decision | Licence thread | Cloud | PR: `research.md` section | 1 thread, 1 cloud round | none |
| 2 | 1 | Clean-room study of the MHWeditor save format, in own words, with credit | Study thread (never writes code) | Cloud | PR: `research.md` section | 1 thread, 1 cloud round | none |
| 3 | 2 | Type-data spike: routes A, B, C, D in order, stopping at the first that passes; MHDB as last resort | Type-data thread | PC (cloud prep for the report) | PR: `research.md` section; code on branch only | 1 to 2 threads, 1 to 2 PC rounds | The maintainer's yes on its tool; step 1 informs route C |
| 4 | 3 | Save-ID to data-ID matching and owned talismans on a save copy | Save thread | PC | PR: `research.md` section | 1 thread, 1 PC round | 2, 3 |
| 5 | 3 | Extraction test on the maintainer's install, all weapons | Extraction thread | PC | PR: `research.md` section (counts, coverage table, what stays hand-written) | 1 to 2 threads, 1 to 2 PC rounds | 3 |
| 6 | 3 | UI comparison: pure Rust vs Tauri vs Electron, with Wilds running, plus the frame-time module | UI bench thread (plus a second thread for the frame-time module) | PC | PR: `research.md` section with numbers | 2 to 3 threads, 1 to 2 PC rounds | performance budget (drafted), our sampler tool |
| 7 | 4 | Close-out: update `PLAN.md`, record licence and toolkit decisions, update `README.md` (status and version badges, roadmap progress; no screenshot yet, there is no window) | Docs thread | Cloud | PR: `PLAN.md` | 1 thread, 1 cloud round | 1 to 6 |

Total estimate: 8 to 11 threads and 7 to 10 test rounds, 3 to 6 of them on the PC. At most 2 or 3 threads run at once. Estimates are guesses and will be compared with the time log. Step 3 is the largest unknown; if it needs more than 2 threads, stop and report to the maintainer rather than continue (Opus on high may then be worth suggesting).

## 5. Tools (need the maintainer's yes)
All rows approved by the maintainer on 2026-10-27. All tools are our own Rust unless stated.

| Tool | Step | Why (plain words) | Built or third-party | Licence |
|------|------|-------------------|----------------------|---------|
| `spikes/typedump` | 3 | Problem: the save and game files are described by type data we do not have and cannot copy from MHWeditor. This tool reads the game's `.exe` from disk to rebuild it ourselves. Without it we fall back to a hand-written schema, a community dump or guessing. On your PC it only reads the `.exe`; it writes to `E:\mhbis\out`. | Built | ours; any crate used goes in THIRD-PARTY-NOTICES.md |
| `spikes/savecheck` | 4 | Problem: we do not know whether the IDs in the save match the game data, or where owned talismans are. This tool decrypts a copy of your save and checks both. Without it, the optimizer could name the wrong gear. On your PC it reads a save copy in `E:\mhbis\copies` only, never the live save. | Built, from `research.md` notes only | ours |
| `spikes/extract` | 5 | Problem: we need to know which stats, tables and drops the game files really hold. This tool pulls only those files out of the packs. Without it, we cannot tell what must be hand-written or dropped. On your PC it reads the install and Steam library read-only and writes to `E:\mhbis\out`; it never unpacks whole packs. | Built | ours |
| `spikes/uibench` | 6 | Problem: the app must stay light beside Wilds, and we do not know which UI toolkit manages that. These three tiny apps give comparable start-up, RAM, CPU and GPU numbers. Without it, the choice is a guess. On your PC they run while Wilds runs and are stopped afterwards; they use a separate data directory. | Built | ours; UI toolkits (egui or Slint, Tauri, Electron) and their licences recorded |
| Rust toolchain, `cargo`, GitHub Actions lint and test jobs | all code | Problem: nothing can be built or checked without them. They install on your PC under normal user folders and run on GitHub for tests. | Third-party, standard | MIT/Apache-2.0 (to confirm in step 1) |
| `spikes/sampler` (cross-platform process sampler) | 6 | Problem: we need repeatable numbers, not impressions, and your preference is our own tools that work on every OS. This samples the three test apps' start-up time, RAM, CPU and GPU. Without it we depend on Windows-only counters. Its frame-time module reads Windows present events (needs admin while running, test PC only, never shipped) so we can see game stutter without touching the game. On your PC it only reads process and event figures and writes to `E:\mhbis\bench`. | Built | ours; any ETW crate used is licence-checked in step 1 and recorded in THIRD-PARTY-NOTICES.md |
| Route C only: a community type dump (downloaded data, never run through an injecting tool) | 3 | Problem: if routes A and B fail, a licensed community dump can stand in. Without it we use hash and pattern mapping (route D), which is slower and less exact. Used only if step 1 clears the licence; it is a download, not something that touches the game. | Third-party data | Checked in step 1 before use |

Not proposed: any injecting, hooking or memory-reading tool. The standing rules rule them out.

## 6. Tests

### 6.1 Automated
GitHub Actions, fixture saves and fixture data only. Spike code is throwaway, so automated tests are minimal:

- Lint on any Rust spike code.
- A check that no save, dump or extracted-data file is tracked in git (`.gitignore` plus a file-pattern check in CI).
- Spike code that touches the reader or extractor is declared untested against the real files in its PR until the PC round runs.

### 6.2 Pass or fail criteria
Each result is recorded in `docs/research.md`.

**Step 1, licence review**

- Pass: each of the five items has a verdict (may use, may use with conditions, may not use) with the licence text cited.
- Pass: a clear statement on whether a Nexus release forces open source.
- Items needing the author or a lawyer are marked open, not guessed.

**Step 2, clean-room study**

- Pass: `research.md` holds the save container layout, the encryption and key derivation, and the structure facts step 4 needs.
- Pass: it is in our own words with credit, with no copied code or type dumps.
- Fail: key facts are missing and need another source.

**Step 3, type data**

- Pass: the recovered type data lets us parse a save copy into named classes and fields for the classes we need (save root, equipment, items, enemy report, story, mission, workshop), with names matching the format facts already known.
- Pass: it can also parse a sample of game data files.
- Partial: only some classes are recovered (route B can complete the rest).
- Fail: route A recovers nothing usable; move to B, then C (only if the licence allows), then D, then MHDB.
- The result states which route was chosen and why.

**Step 4, ID matching and talismans**

- Pass: at least 99% of the save's fixed IDs for weapons, armor, decorations and items resolve to the same entries in the game data (checked by name), or a deterministic mapping rule is found.
- Talismans: pass if owned talismans are found in the save and their skills decoded; fail if no record exists (the plan then states how the app treats talismans).
- Partial results list which categories match.

**Step 5, extraction test**

- Each of these categories gets one status: skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops and motion values.
- Statuses: found and decoded, found but not decoded, or not in the files.
- Counts are plausible against the MHDB cross-check (a cross-check only).
- All weapon types are covered.
- The report says what stays hand-written, and marks product features at risk where a category is not in the files.

**Step 6, UI comparison**

- Pass: a candidate meets the performance budget with Wilds running (cold start, idle RAM, idle CPU, GPU) and does not stutter the game during its background job.
- Decision rule: pure Rust if it meets the budget; otherwise the lightest option that does.
- Numbers come from the maintainer's PC with Wilds running, at least 3 runs each, median recorded, taken with our own sampler.
- Pass: the frame-time module reports the game's frame-time statistics (median, 99th percentile, worst frame) with and without each candidate's background job.
- Pass: the module never opens or touches the game process.

**Step 7, close-out**

- Pass: `PLAN.md` reflects every result and lists the Phase 2 inputs.
- Pass: `README.md` status and version badges and roadmap progress match the phase result; the placeholder tiles stay labelled as placeholders because there is no window yet.
- Pass: every answered open question is flagged for the coordinator to move.

### 6.3 Test round checklist
Queued, one at a time. There is no app in this phase, so each item says what the spike report should show for the maintainer's current save.

**Step 3**

- [ ] The type-data summary lists the classes needed for the save, with counts of fields and enums.
- [ ] The summary says which route (A, B, C or D) succeeded.

**Step 4**

- [ ] With your save copy, the report shows 3 character slots, one marked active, and the active slot's equipment box count.
- [ ] The report gives an ID match rate per category, with names of unmatched IDs (names only).
- [ ] The report says whether owned talismans were found and how many, with skill names and no hunter IDs.

**Step 5**

- [ ] The extraction report lists, per category, the number of entries found and its status.
- [ ] The report covers every weapon type.

**Step 6**

- [ ] The frame-time report shows the game's median, 99th percentile and worst frame time with each candidate running (run as admin).
- [ ] The benchmark table shows cold start, idle RAM, CPU and GPU for each of the three options, with and without Wilds running, median of 3 runs.

**Every PC step**

- [ ] No save, dump or extracted Capcom data appears in git or in the PR.
- [ ] The PC thread reports it did not write to the save, the game install or the Steam folders.

Steps 1, 2 and 7 are cloud-only and checked in the cloud: the PR contains only docs and the `research.md` or `PLAN.md` change, with citations.

## 7. Risks and open questions

- **Route A fails (the hardest step).** Notice: nothing recovered, or names do not match. Fallback: B, then C, then D, then MHDB labelled older and unsupported. Report to the maintainer before spending more than 2 threads.
- **Types recovered, but game updates change them.** Notice: a later build differs. Fallback: record the build ID with each result; the Phase 2 extractor must tolerate change visibly.
- **Fixed IDs do not match game data IDs.** Notice: match rate under 99%. Fallback: a mapping table, name-based matching or route D; the plan records the cost.
- **Owned talismans not in the save or not decodable.** Fallback: the user enters talismans by hand, or they are optional; a product decision for the maintainer.
- **The game files lack Artian tables, motion values or skill numbers.** Fallback: hand-written tables from cited sources, or dropped features; the results feed Phases 4 and 8.
- **ETW access or an ETW crate's licence blocks the frame-time module.** Notice: step 1 licence verdict, or no events with admin rights. Fallback: judge stutter by eye while the maintainer plays, and record that.
- **Licence blocks route C or forces open source.** Fallback: routes A, B or D, or the project goes open source, which the maintainer has said is acceptable if needed.
- **All UI candidates miss the budget.** Fallback: pure Rust with the lightest toolkit, and the numbers shown to the maintainer.
- **Install is modded.** Notice: mod markers in the install folder. Fallback: read original packs only and report the markers.
- **Personal data leaks into reports.** Notice: any hunter ID in output. Fallback: reports print counts and names only; the checklist checks it.
- **Questions for the maintainer** are the open items in the box at the top. New questions go to `docs/open-questions.md` via the coordinator.

## 8. Versioning and changelog
Version 0.1.0 is declared when the phase closes (step 7). There is no product and no changelog before Phase 2 (the changelog starts at 0.2.0 per the standing rules). Steps do not bump the version.

## 9. Review log
| Date | Reviewer | Edit or decision |
|------|----------|------------------|
| 2026-10-27 13:05 UTC | The maintainer | Approved Phase 1. |
| 2026-10-27 | The maintainer | Answers: all tools approved, own sampler, `E:\mhbis`, spikes-archive branch, performance budget added. |
| 2026-10-27 | The maintainer | Readability review: steps renumbered in run order, every step in Architecture, lists instead of paragraphs, fixed per-file blocks, plain-words tool reasons, "how to read" box. Plan writing rules added to the template. |

## 10. Time log
| Event | Date and time |
|-------|---------------|
| Phase approved by the maintainer | 2026-10-27 13:05 UTC |
| Step 1 (licence review) PR #2 merged | 2026-10-27 13:08 UTC (docs only; merged after cloud check, no separate pass from the maintainer) |
| Step 2 (clean-room study) PR #3 merged | 2026-10-27 13:09 UTC (docs only; merged after cloud check, no separate pass from the maintainer) |
| Step 2 addendum: key-setup re-study PR #7 merged | 2026-10-27 13:32 UTC (added after step 3 found a gap in the decryption notes; docs only, cloud check) |
| Step 3 PR #6 merged | 2026-10-27 13:42 UTC (the maintainer passed 13:40 UTC; save half only, data-file sample moved to step 5) |
| Step 4 (ID matching, talismans) PR #10 merged | 2026-10-27 14:06 UTC (the maintainer passed 14:05 UTC; partial: by-name check moves to Phase 3, talisman skills undecoded) |
| Step 5 (extraction test) PR #8 merged | 2026-10-27 14:00 UTC (the maintainer passed 13:58 UTC; partial: located but not decoded, route B schemas needed in Phase 3; motion values not found) |
| Plan page builder (`tools/plan-page`, not a numbered step) PR #12 merged | 2026-10-27 14:20 UTC (the maintainer passed 14:20 UTC: looks fine, keep the How-to-read box; docs tooling, cloud check) |
| Step 6 (UI comparison) PR #15 merged | 2026-10-27 14:58:32 UTC (the maintainer passed; pass: egui chosen; Scarlet Forest base camp is the standard benchmark spot; docs only, merge commit 25c7114) |
| Licence PR #14 merged (not a numbered step) | 2026-10-27 15:02:04 UTC (the maintainer passed 15:01 UTC; pass: MPL-2.0, no DCO, Ko-fi on GitHub only; docs and LICENSE only, merge commit 054d586) |
| Step 7 (close-out) PR opened | 2026-10-27 (docs only; also adds `CONTRIBUTING.md`, `SECURITY.md`, issue templates and `CODE_OF_CONDUCT.md`, approved by the maintainer; merge time and pass to be logged by the coordinator) |
| Step 7 (close-out) PR #18 merged | 2026-10-27 15:11:17 UTC (the maintainer passed; pass; modulus restored with credit; tag v0.1.0; docs only, squash commit 9a7bd65) |
| Phase finished | |

Elapsed from approval to finish: `<hours>`. Compare with the estimates in section 4.
