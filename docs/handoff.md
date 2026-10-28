# Handoff

Current state for whoever coordinates next. Replaced on every update; history is in git. Last updated 2026-10-29.

## Where we are
- Multi-agent workflow experiment ([plan](plans/multi-agent-workflow.md)): steps 1 to 6 done (retro in `research.md`; Claude with Codex review is the standard). Next: split `AGENTS.md` into project rules and a portable `docs/workflow.md` (step 7), then research the starter-kit sync (step 8).
- Forgeline: Phase 2 foundation in progress. Last merged: #65 (update-gate fixes, 2026-10-28 17:02:55 UTC, tested on the maintainer's PC without Wilds; with-Wilds numbers waived). UX elements 1 to 10 merged; element 11 (Customise drawer) is built and looked at, in a draft PR.

## Open PRs
- UX element 11, Customise drawer: draft PR, CI pending. The maintainer approved the look; needs a merge go.
- This workflow docs PR: needs a merge go.

## Waiting on the maintainer
- Merge go for the two PRs above.
- Retire the old coordinator (the retro is done). Daily GitHub issues check at 10:00 Sydney: port to a T3 scheduled task or let it stop.
- Branch deletion (sessions cannot delete branches): `experiment/update-gate-hardening-b`.
- Plan page republish: the old coordinator published the plan as an Artifact via `tools/plan-page`; decide what replaces it here.

## Quirks not in the rules
- This machine's git config has a personal email. Every commit must override it with the VeroxAUS noreply (`GIT_AUTHOR_*` and `GIT_COMMITTER_*` env vars) until the maintainer says otherwise.
- CI runs only on PRs and pushes to `main`; a pushed branch needs a draft PR to get CI.
- Deferred review findings from PR #38: first-bytes container check, the unchanged decision, status-write errors. Known gap: engine-start recovery racing an update's two renames (comment in `forgeline-engine`).
- Only Claude or the maintainer commits; OpenCode drafts text only.
- Permissions: the coordinator thread runs in `auto`. PC test threads and anything risky (real save or install, deleting, installing, pushing) are started with `runtimeMode: approval-required`.
- Model table: Claude Sonnet 5.5 medium by default (Opus high for big or tricky work); Codex `gpt-6.1-terra` medium, high for the extractor, validation gate and optimizer maths; OpenCode a free flash model at low.
- Coordinator checklist: check the commit identity before every commit, take counts from command output, confirm files exist before running tests, check alignment and spacing on zoomed crops before showing UI screenshots.
- PC test rounds run in `E:\mhbis` (repo clone there, dev data under `E:\mhbis\out`, copies of the save in `E:\mhbis\copies`); ask before cleaning it.

## To resume
Read `AGENTS.md`, `docs/plans/multi-agent-workflow.md`, then this file. Report every change to the maintainer before making it.
