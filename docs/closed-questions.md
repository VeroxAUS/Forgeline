# Closed questions

Questions asked to Thomas and their answers, all answered on 2026-10-27. Open ones are in [open-questions.md](open-questions.md).

## Repo and files
- **Repo name and default branch?** Private repo `VeroxAUS/Forgeline` (renamed from `MonsterHunterProgressionTool` on 2026-10-27), default branch `main`.
- **What goes in the first commit?** PLAN.md, `open-questions.md` (written only by the coordinator, so the questions are backed up and anyone cloning sees them), `closed-questions.md` for answers, and a CLAUDE.md that exactly copies the project instructions. All documentation lives in `docs/`.
- **Should `reference/dumpsave.rs` go in the repo?** No. It is derived from MHWeditor and comes from a local save. Anyone working on the project provides their own local saves.

## Stack
- **Electron or lighter?** Compare against Tauri in Phase 1. Later: a pure-Rust UI is wanted too, measured against the HTML/JS options.
- **Language?** Rust everywhere.
- **Open source?** Unsure. If Nexus cannot be used without open source, the project goes open source. Build our own tools in Rust where we can.

## Phase 1 and the PC
- **PC work folder?** A permanent folder. Ask Thomas before cleaning it; do not assume it is cleaned at the end of Phase 1.
- **May the PC thread run its own commands?** Yes. Anything Thomas could run manually should be automatic. Thomas never talks to threads; the coordinator relays everything.
- **Approved extraction tools?** None in principle. Each new tool, built or third-party, needs Thomas's yes.
- **Training dummy checks?** Wait until Phase 5 or later.
- **Licence review?** Run in the cloud in parallel with the PC spikes, with the results reported in the project chat.

## Workflow
- **Test round timing?** Every PR gets a test round. Later refined: only PRs that change what the app reads or shows run on Thomas's PC; the rest are checked in the cloud and reported the same way. Rounds are queued so Thomas gets one checklist at a time.
- **Who merges?** The coordinator merges after tests pass (Thomas gave the go) and reports each merge in depth.
- **Where do spike results go?** Threads report to the coordinator, who summarises for Thomas. Results are written to `docs/research.md`, not `open-questions.md`.
- **Status posts?** Short status when a phase finishes, when Thomas is needed, when threads start, and on errors.
- **Usage efficiency?** Approved: tests and linters on GitHub Actions; `docs/research.md` as a cache; short thread briefs; never load big dumps (use `jq`/`rg`); at most 2 or 3 threads at once; one PC session per test queue; batched questions; Sonnet 5.5 on medium by default.
- **Plan detail?** Every phase has a detailed plan from the agreed template (`docs/plans/TEMPLATE.md`), with estimates in threads and test rounds, per-file designs, and a time log from approval to finish. Thomas reviews and approves each before work starts.
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
- **Extra tools.** Agreed as a TODO list, built when needed with Thomas's yes each time: save diff, fixture save builder, data diff, build explainer, benchmark harness.
- **Bare window from when?** A bare window from the data viewer in Phase 2, grown each phase; Phase 6 is polish and the equipment display.
- **Type-data fallbacks?** Worked in order A to D (own static dumper, hand-written schema, licence-checked community dump, hash/pattern mapping). If no game data can be read at all, fall back to MHDB data, clearly labelled to the user as older and unsupported.
- **Extractor scope?** Data for every weapon, low and high rank, from Phase 3.
- **Optimizer scope?** Every weapon optimized before 1.0.0, refined one at a time with the light bowgun first.
- **Several test rounds open at once?** No. Queue them so there is one checklist at a time.
