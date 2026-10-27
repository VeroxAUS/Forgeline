# Closed questions

Questions asked to the maintainer and their answers, all answered on 2026-10-27. Open ones are in [open-questions.md](open-questions.md).

## Repo and files
- **Repo name and default branch?** Public repo `VeroxAUS/Forgeline` (renamed from `MonsterHunterProgressionTool` on 2026-10-27), default branch `main`.
- **What goes in the first commit?** PLAN.md, `open-questions.md` (written only by the coordinator, so the questions are backed up and anyone cloning sees them), `closed-questions.md` for answers, and a CLAUDE.md that exactly copies the project instructions. All documentation lives in `docs/`.
- **Should `reference/dumpsave.rs` go in the repo?** No. It is derived from MHWeditor and comes from a local save. Anyone working on the project provides their own local saves.

## Stack
- **Electron or lighter?** Compare against Tauri in Phase 1. Later: a pure-Rust UI is wanted too, measured against the HTML/JS options.
- **Language?** Rust everywhere.
- **Open source?** Unsure. If Nexus cannot be used without open source, the project goes open source. Build our own tools in Rust where we can.

## Phase 1 and the PC
- **PC work folder?** A permanent folder. Ask the maintainer before cleaning it; do not assume it is cleaned at the end of Phase 1.
- **May the PC thread run its own commands?** Yes. Anything the maintainer could run manually should be automatic. The maintainer never talks to threads; the coordinator relays everything.
- **Approved extraction tools?** None in principle. Each new tool, built or third-party, needs the maintainer's yes.
- **Training dummy checks?** Wait until Phase 5 or later.
- **Licence review?** Run in the cloud in parallel with the PC spikes, with the results reported in the project chat.

- **Phase 1 step 5 result?** Passed 2026-10-27 as partial: pack reading and naming work, data categories are located but not decoded (route B schemas needed in Phase 3), motion values not found. The maintainer noted missing motion values matter little now but must be kept in mind for later simulation ideas (tracked as open question 11).

## Workflow
- **Test round timing?** Every PR gets a test round. Later refined: only PRs that change what the app reads or shows run on the maintainer's PC; the rest are checked in the cloud and reported the same way. Rounds are queued so the maintainer gets one checklist at a time.
- **Who merges?** The coordinator merges after tests pass (the maintainer gave the go) and reports each merge in depth.
- **Where do spike results go?** Threads report to the coordinator, who summarises for the maintainer. Results are written to `docs/research.md`, not `open-questions.md`.
- **Status posts?** Short status when a phase finishes, when the maintainer is needed, when threads start, and on errors.
- **Usage efficiency?** Approved: tests and linters on GitHub Actions; `docs/research.md` as a cache; short thread briefs; never load big dumps (use `jq`/`rg`); at most 2 or 3 threads at once; one PC session per test queue; batched questions; Sonnet 5.5 on medium by default.
- **Plan detail?** Every phase has a detailed plan from the agreed template (`docs/plans/TEMPLATE.md`), with estimates in threads and test rounds, per-file designs, and a time log from approval to finish. The maintainer reviews and approves each before work starts.
- **Versioning?** Semantic versioning. 1.0.0 is the full release; a major addition bumps x.1.x; a bug fix bumps x.x.1.

## Product
- **High rank test weapon and low rank?** No high rank weapon in the save; low rank builds are included.
- **Comfort-skill must-haves?** From in-depth research rounds, with a baseline for every skill. Users can override them.
- **Which gear is "available"?** Any gear from monsters the user has unlocked. Missing materials go on the shopping list.
- **Several accounts and character slots?** The app auto-picks the active slot.
- **After a game patch?** Warn that data may be out of date. Suggest pressing update only when confidence is above 95% that nothing will break. Confidence covers nonexistent items and false upgrades, and also skill, set bonus or weight changes: the model re-scores builds with old and new data and flags worse ones. How to implement it is still open.
- **MHWeditor.** Scrapped. Learn from how it works, but all code is our own Rust. Nothing from it enters the repo (see `research.md`).
- **UI.** Pure-Rust UI wanted, compared against HTML/JS (Tauri, Electron) on how they run.
- **Next weapon after the light bowgun?** Decide later.
- **Comfort research round placement?** Phase 4, next to the optimizer.
- **Suggested additions.** Agreed: fixture saves (Phase 2), a data viewer for debugging, changelog and versioning from Phase 2. A performance budget will be drafted together later.
- **Extra tools.** Agreed as a TODO list, built when needed with the maintainer's yes each time: save diff, fixture save builder, data diff, build explainer, benchmark harness.
- **Bare window from when?** A bare window from the data viewer in Phase 2, grown each phase; Phase 6 is polish and the equipment display.
- **Type-data fallbacks?** Worked in order A to D (own static dumper, hand-written schema, licence-checked community dump, hash/pattern mapping). If no game data can be read at all, fall back to MHDB data, clearly labelled to the user as older and unsupported.
- **Extractor scope?** Data for every weapon, low and high rank, from Phase 3.
- **Optimizer scope?** Every weapon optimized before 1.0.0, refined one at a time with the light bowgun first.
- **Several test rounds open at once?** No. Queue them so there is one checklist at a time.

## Added later on 2026-10-27 (after the setup batch)
All answered 2026-10-27, in the order they were asked.

### Plan format and readability
- **Why were Phase 1 steps out of order, and why were spikes 3 and 4 missing from the architecture section?** The maintainer flagged both, plus walls of text in the done list, per-file designs, tool reasons, tests and checklists. Approved seven rules, now in `docs/plans/TEMPLATE.md`: number steps in run order with the same numbers everywhere (parallel steps share a stage); every step appears in every section that covers steps; no packed paragraphs (done list, pass/fail, tests and checklists are one item per line); per-file designs use one fixed block per file; each tool's "why" says the problem, what happens without it and what it touches; a "how to read this plan" box at the top; check the page against the list before publishing. Phase 1 was rewritten from it.
- **Estimates and time log?** Estimates in threads and test rounds. A time log records the time from the maintainer's approval to the finish of each phase and step.
- **Publish every plan as an artifact too?** Yes. Every plan is pushed to the repo and published as an artifact that the maintainer can open, and the same artifact is updated when the plan changes.

### MHWeditor and the licence review
- **Is the clean-room approach for MHWeditor OK?** Yes. It has no licence at all, so nothing from it enters the repo. One thread studies it and writes facts in its own words to `docs/research.md` with credit. A separate thread writes our Rust code from those notes only. All implementation is our own.
- **Nexus internet use.** Accepted. Nexus restricts mods that connect to the internet. The MHDB last-resort fallback is the only part that would, so before release it is cleared with Nexus or dropped.
- **Nexus AI-generated content tag.** Yes, use it.

### Phase 1 approval
- **Approve the tools in the Phase 1 plan?** All approved. For the measuring method the maintainer wanted our own cross-platform tool instead of Windows' built-in counters (see the performance budget section).
- **PC work folder?** `E:\mhbis`. Ask before cleaning it.
- **Where does spike code live?** On separate PR branches, never merged to `main`. At the end of Phase 1 they go to a `spikes-archive` branch that is kept for reference and never merged. `main` as the only long-lived branch is a fluid rule: other exceptions are decided per feature.
- **Phase 1 approved?** Yes, 13:05 UTC. Stage 1 (steps 1 and 2) then started.
- **Step 3 (type data) test round?** Pass. Route A worked. Stage 3 (steps 4 to 6) started on the PC at the maintainer's go.

### Performance budget
- **Are the budget numbers right?** Yes, as an initial pass. They live in `docs/performance-budget.md` so they can be reviewed and adjusted. Measured on the maintainer's PC with Wilds running, median of 3 runs.
- **Should our tool measure game frame times?** Yes, built by us as a Windows-only module (event tracing, no contact with the game process) of our own cross-platform measuring tool. It needs admin rights, is for the test PC only and is never shipped. It is part of step 6. If a licence or Windows access blocks it, stutter is judged by eye while the maintainer plays.
- **Test PC and minimum spec?** Test PC: RTX 4080, i7-13700K, 32 GB RAM. There is no weaker machine. The minimum spec is worked out near release by limiting cores and RAM on the test PC, with Wilds' official minimum spec (quoted from Capcom, not from memory) as the baseline.

### README, name and repo
- **Add a README?** Yes. Drafted, then merged.
- **Install the `beautify-github-readme` skill and run it on the README?** Yes. Saved as a skill, then run on the README.
- **Skill setup choices.** "Go with yours": refresh the whole README (not images only), pure SVG banner (no AI-generated art, no Capcom art), still banner (no animation).
- **Banner wording.** Tagline "The best build you can make right now", second line "From your first hunt to endgame". The longer low-rank-to-high-rank line was cut as redundant. Phone layout does not matter.
- **Badges?** Yes, only things that are true today: pre-alpha, Windows and Steam, Rust, unreleased, licence not decided, local only. More (tests, downloads) come when they exist.
- **AI disclaimer on the GitHub page too?** Yes, one short line in the README's Disclaimer section, matching the Nexus page.
- **README update in every phase close-out?** Yes, added to the plan template and to Phase 1's close-out: refresh badges and roadmap, and from Phase 2 swap the placeholder tiles for a real screenshot.
- **Project name?** Forgeline (picked from a list of themed names). On Nexus: "Forgeline: MH Wilds Build Planner".
- **Repo rename, About text and topics?** The maintainer renamed the repo to `Forgeline` and set the About description. The project instructions and CLAUDE.md were updated to match, and the maintainer removed the old repo from the project and renamed the project.
- **Delete merged branches automatically?** Yes. The maintainer enabled "Automatically delete head branches". Sessions get a 403 when deleting branches themselves.

### Public repo, email and history
- **Is the repo public?** Yes. The instructions and docs were changed from private to public, with a new rule to check every commit for personal and Capcom data before pushing. A check of all 32 commits found no Steam IDs, save data, hunter IDs, character names, Windows user paths, tokens or game files.
- **Which email for commits?** The maintainer's GitHub noreply address, `VeroxAUS <3145728+VeroxAUS@users.noreply.github.com>`, for all commits and set in CLAUDE.md.
- **Rewrite past commits to remove the two personal email addresses?** Yes. History was rewritten and force-pushed; a search found neither address.
- **Hidden copies of closed PRs (#1 to #4, #6, #7)?** Option 1: The maintainer asks GitHub Support to purge cached views and the `refs/pull/*` refs. (Option 2, recreating the repo, was not chosen.) Support only accepts the request from the repo owner, so the ticket is the maintainer's to file.

### Monitoring after release
- **Daily GitHub issues check?** Yes, once a day at 10:00 Sydney time (follows daylight saving). It summarises new or updated issues for the maintainer, and never replies to, labels or closes anything. First run the next morning.
- **Nexus comments and bug reports?** Researched (see `docs/research.md`, "Nexus monitoring"). Not automatable for now: the API has no comments or bugs endpoint, and reading the pages needs a real browser and goes against Nexus's rules for AI crawlers. The maintainer checks the author notification settings once the mod page is live and forwards anything. If Nexus adds comments to its API, it can be automated with an API key from the maintainer.

### Research unknowns answered (moved from open-questions.md)
- **Performance budget: target numbers for cold start, idle RAM, idle CPU and recompute time?** Drafted with the maintainer and approved as an initial pass. See `docs/performance-budget.md` and the performance budget section above.
- **Licences: what MHWeditor and its type data allow, and whether GPL code forces the Nexus release to be open source?** Answered by Phase 1 step 1 (`docs/research.md`, "Phase 1 step 1: licence review"). MHWeditor has no licence, so nothing from it can be used; its type dump comes from other unlicensed work, so community-dump fallback C is not cleared. mhdb-wilds-data code is GPL, so it stays out. Nexus does not force open source as long as we use no GPL code, and we use none. The licence part that remains open (Slint terms, Rust toolchain, mhdb-wilds-data output files, MHDB terms, Nexus rules on an update download) stays in `open-questions.md`.
- **Type data: can we recover the names behind the save's hashes?** Yes, with route A (our own scan of the game's .exe on disk). Step 3 named 185 of 187 classes and 1,215 of 1,221 class and field pairs (99.5%) on the maintainer's real save copy. See `docs/research.md`, "Step 3: type-data spike, route A". Reading a sample of game data files moved to step 5.

### Step 4 test round (2026-10-27, as of the 2026-10-20 save copy)
- **Does the reader's picture of your save match your game?** Yes. The maintainer confirmed: only 1 of 3 character slots in use, about 14 light bowguns crafted, 7 armor pieces crafted, one talisman owned, and nothing from the save in git. About 221 item types in the item box: "hard to tell, seems correct" (approximate). Details in `docs/research.md`, "Step 4".
- **Rewrite git history again to remove the maintainer's name?** No. The name stays in past commits (file contents, two messages, two author display names) and old branches; it was removed from the current files and open PRs only. No history rewrite.

### Plan page builder (2026-10-27)
- **Is a small Rust tool OK to build plan pages, so every published plan page looks the same?** Yes (tool approved, `tools/plan-page`, PR #12).
- **What is the block at the start of the plan?** It is the "How to read this plan" box (run order, decisions made, what is still open), required by plan-writing rule 6. The maintainer said it is fine and to keep it as it is; PR #12 merged on that pass.

### Licence and donations (2026-10-27)
- **Barring all competing forks?** No, the maintainer finds it too harsh. This rules out PolyForm Shield as recommended; MPL-2.0 is being considered instead.
- **Must forks rename?** No. "Based on Forgeline" is fine.
- **Tip link?** Yes: a voluntary Ko-fi tip jar with no perks, linked from GitHub only and nothing on the Nexus page, to stay clear of Nexus's donation rules.

### Step 6 UI comparison (2026-10-27)
- **Does the step 6 result pass?** Yes, the maintainer passed it (PR #15, merged 2026-10-27 14:58:32 UTC). Pure Rust (egui) is chosen: it meets the budget with Wilds running; Tauri and Electron fail the RAM hard limit. The standard benchmark spot for every later measurement is Scarlet Forest base camp.

### Licence choice (2026-10-27)
- **Which licence?** MPL-2.0 (full text in `LICENSE`), chosen over PolyForm Shield, GPL/AGPL and a custom licence. It lets anyone fork and republish with credit; changed MPL files stay open. It does not give an "ask permission" rule. Dependencies checked: MIT, Apache-2.0 and Unlicense crates are compatible; see `docs/research.md`.
- **Contributors?** No extra process: no DCO sign-off, no CLA, no copyright assignment. Contributions come in under MPL-2.0. Changing the licence later would need the contributors' agreement.
- **Does the licence PR pass?** Yes, the maintainer passed it (PR #14, merged 2026-10-27 15:02:04 UTC): MPL-2.0, no DCO, Ko-fi tip jar on GitHub only (link in the README and `.github/FUNDING.yml`).

### Phase 1 close-out (2026-10-27)
- **Does pure-Rust UI pass the Phase 1 comparison against Tauri and Electron (old open question 8)?** Yes. Answered by step 6 and passed by the maintainer (PR #15): egui is the UI toolkit. See the step 6 entry above and `docs/research.md`. Moved here from `open-questions.md` at close-out.
- **Spike code from the never-merged PRs?** Collected into the `spikes-archive` branch (`spikes/`), one folder per step, with a README; PRs #5, #9, #11 and #16 closed with a pointer. The 129-byte pak table constant from ree-pak-rs (MIT) was redacted from the archive copy.
- **Does the close-out PR pass, and can the pak table constant go back in the archive?** Yes, the maintainer passed it (PR #18, merged 2026-10-27 15:11:17 UTC): pass; modulus restored with credit; tag v0.1.0. The constant is back in `spikes-archive` with a credit to ree-pak-rs (MIT) and an entry in `THIRD-PARTY-NOTICES.md`. The maintainer also approved adding `CONTRIBUTING.md`, `SECURITY.md`, issue templates and a code of conduct.

### Motion values approach (2026-10-27)
- **How do we get motion values and base power (open question 11)?** The maintainer approved all four routes: compare builds by effective raw meanwhile (Phase 4); a deep game-file search early in Phase 3 covering every weapon's motion values and attack and ammo base power; training-dummy measurement in Phase 5; credited, licence-checked community data labelled unverified as a fallback. The maintainer pointed out that motion values matter for all 14 weapons, so the "bowgun ammo matters more" argument does not narrow the search. Written into `docs/PLAN.md` (Phases 3, 4 and 5). Question 11 itself stays open in `open-questions.md` until the values are found.

### Hours estimates in plans (2026-10-27)
- **Should plans include an hours estimate?** Yes, the maintainer said so ("Yeah probably"). Every plan's estimates section now gives rough wall-clock hours per step and in total next to threads and test rounds (`docs/plans/TEMPLATE.md`, rule 11). The Phase 2 plan estimates about 13 h (range 10 to 18 h), calibrated on Phase 1's 2 h 06 min for 7 steps.

### Phase 2 tools (2026-10-27)
- **Approve the Phase 2 tools?** Yes, the maintainer approved all three: the fixture save builder (`tools/fixture-builder`), `forgeline-check` (counts and names only) and the benchmark harness (`tools/bench`, the Phase 1 sampler ported from `spikes-archive`). Recorded as decision A of the Phase 2 plan (PR #21).

