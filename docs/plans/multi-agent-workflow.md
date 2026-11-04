# Multi-agent workflow plan (experiment): Claude, Codex, OpenCode in T3 Code

Status: **draft, awaiting the maintainer's approval. Nothing is pushed.** Written 2026-10-29. Not a phase plan; it changes how work is run, not the product.

## 1. Goal and done

Run the existing workflow (one coordinator, one thread per PR, maintainer talks only to the coordinator) inside T3 Code with three agents:

| Agent | Role | Models (from T3 `orchestrator_capabilities`, 2026-10-29) |
|---|---|---|
| **Claude** | Brains and coordinator. Plans, designs, writes the serious code, relays to the maintainer, does merge duties. Default Sonnet 5.5 medium; Opus high only per the existing model rule. | `claude-sonnet-5-5`, `claude-opus-5-5` |
| **Codex** | Second opinion. Reviews Claude's diffs (risky PRs only), checks fine detail, takes independent parallel tasks. Read-only reviewer by default. | `gpt-6.x` family (pick at brief time; suggest a mid model, `high` only for reader/extractor/optimizer maths) |
| **OpenCode** | Cheap and free. Brainstorming, scaffolding, docs, scripts, exploring ideas, drafting commit messages. Nothing serious, nothing that decides behaviour. | free models: DeepSeek V4.1 Flash or GLM-5.3 Flash first choice |

Done = the experiment ran for a few PRs, the maintainer could still use the same habits (one checklist, one voice, same test rounds), and the maintainer decides keep, adjust or drop.

## 2. Decisions already made (2026-10-29)

1. `AGENTS.md` is the single source of the standing rules. `CLAUDE.md` becomes a short file: `@AGENTS.md` import plus a Claude-only coordinator section. The old rule "CLAUDE.md is an exact copy" is replaced by "AGENTS.md is the source; CLAUDE.md imports it".
2. Codex reviews **risky PRs only**: save reader, extractor, pack reader, update validation gate, optimizer maths, anything touching save or game files, anything that changes performance-sensitive paths. UI-only and docs PRs skip it.
3. Threads run in **local T3 worktrees, no push**. Each branches from a freshly fetched `origin/main`. Commits stay local. Push and PR only on the maintainer's explicit go, per PR.
4. This machine is the test PC. PC test rounds work as before (one PC thread per test queue, permanent work folder, resolved after each round), but the thread is a local T3 thread here instead of a Remote Control thread.
5. The daily 10:00 Sydney GitHub issues check stays in the old coordinator for now. T3 has a scheduled-task tool, so it can be ported if the experiment is kept; decide at the retro.
6. OpenCode **drafts** commit messages; Claude (or the maintainer) runs `git commit`, so the identity and trailer rules live in one place.
7. During the overlap the Claude coordinator thread writes the shared docs; the old coordinator no longer does.
8. The old coordinator is retired after the retro (step 6).
9. The workflow must work in any future project, so it is kept apart from Forgeline's own rules: `AGENTS.md` holds project rules, `docs/workflow.md` holds the portable workflow, `docs/handoff.md` holds the current state and is replaced on each update, never appended.

## 3. Design

### 3.1 Files

- `AGENTS.md` (repo root): the current CLAUDE.md text, moved verbatim, with these edits only:
  - the Docs paragraph's "CLAUDE.md is an exact copy" sentence replaced (decision 1);
  - Workflow paragraph additions for the three agents (section 3.2 to 3.4 below, short form);
  - "Tools" paragraph: a line recording that Codex and OpenCode are approved dev tooling for this experiment (the rule says each new third-party tool needs the maintainer's yes; this plan is that yes once approved). They are never part of the shipped app.
  - "Model and effort" paragraph: add the Codex and OpenCode defaults from section 1.
- `CLAUDE.md`: `@AGENTS.md` plus a section for things only Claude does (coordinator duties, how to call `delegate_task`, the briefing template below).
- `opencode.json` (repo root, only if needed): OpenCode reads `AGENTS.md` natively, so likely **not needed**. Verify in step 2 before creating anything.
- Codex reads `AGENTS.md` natively. No extra file.
- Nothing else new. No Claude-specific rules duplicated into AGENTS.md; per-agent differences stay in section 3.2 to 3.4 of AGENTS.md as three short bullets each.

Why an import and not a plain pointer: Claude Code only loads another file when told to via `@path`. A prose "see AGENTS.md" would silently drop the rules.

### 3.2 Coordinator (Claude) duties, unchanged

Maintainer talks only to Claude. Claude: starts threads, writes short briefs (plan step plus files that matter), relays results, merge duties (PR comment "Tested and approved by the maintainer:", CHANGELOG line "Reviewed and tested by VeroxAUS", time log, closed questions), owns `open-questions.md`. One thread per PR, resolved when merged. At most 2 or 3 threads at once, and that count now includes Codex and OpenCode threads.

### 3.3 Codex duties

- **Review**: Claude starts a new `delegate_task` per review (never a follow-up message to an old one), brief = branch, base, the diff scope, the risky-area checklist from AGENTS.md, and "read-only; report findings, do not edit". Codex returns a ranked list. Claude fixes small findings itself; disputes go to the maintainer only if they change behaviour.
- **Independent task**: only when the task has a clear spec in `docs/plans/` and touches files no other running thread touches (separate worktree, enforced by T3 `isolation: worktree`). Good fits: test harness work, fixture building, tight algorithm implementations from a written spec. A task like this still gets its own PR and its own test round.
- Codex must follow the same read-only-on-game, no-personal-data, no-Capcom-data rules (they are in AGENTS.md, so it sees them).
- Codex never reviews its own work; a Codex-written change is reviewed by Claude.

### 3.4 OpenCode duties

Allowed: brainstorming notes, scaffolding (crate skeletons, CI snippets), docs drafts, throwaway scripts, exploring ideas, drafting commit messages and PR descriptions, mechanical renames.
Not allowed: reader, extractor, validation gate, optimizer, anything in `docs/PLAN.md` decisions, anything resolving an open question, merge duties, running `git commit` or `git push`.
Its output is always treated as a draft: Claude reads it before it is used. It gets `approval-required` runtime mode so it asks before writing. Free models can be rate-limited or weaker; if one fails twice, switch to another free model; never fall back to a paid model or another provider without the maintainer's say.

### 3.5 Briefing template (all agents)

```
Read AGENTS.md first. Task: <plan step>. Base: origin/main (fetched). Files that matter: <list>.
Out of scope: <list>. Output: <diff in your worktree | findings list | draft text>.
Do not push, tag or open PRs. Report in under 200 words.
```

### 3.6 Flow of one PR

1. Claude fetches origin, starts a Claude thread in a local worktree off `origin/main`.
2. Claude implements. OpenCode may draft docs/scaffolding in parallel (separate worktree).
3. Risky PR: Claude starts a Codex review thread on the branch. Fixes applied. Codex re-check is a new review thread if the fixes were big.
4. Claude checks the identity rule (`git log --format='%an %ae %cn %ce'`) and the personal-data / Capcom-data scan.
5. Claude shows the maintainer: diff summary, review result, test-round checklist. Maintainer's go is needed to push or open a PR.
6. Test round and merge duties exactly as today.

## 4. Steps (estimates in threads)

| # | Step | Agent | Notes |
|---|---|---|---|
| 1 | Maintainer approves this plan and the section 2 decisions | Claude, maintainer | 0 threads |
| 2 | Verify OpenCode and Codex both load `AGENTS.md` (a throwaway read-only thread each: "list the first rule about the game saves") | OpenCode, Codex | 2 small threads, nothing written |
| 3 | Create `AGENTS.md`, slim `CLAUDE.md`, update the Docs paragraph | Claude | 1 thread, local commit only; OpenCode drafts the commit message |
| 4 | Dry-run review: Codex reviews an already-merged small risky PR's diff (pick one from git log) to see signal-to-noise and usage | Codex | 1 thread, read-only |
| 5 | First live PR through the full flow (suggest a small, low-risk item) | all three | 1 to 3 threads |
| 6 | Retro: what cost how much, what was noise, keep/adjust/drop | Claude | write to `docs/research.md` (workflow notes) |
| 7 | Split `AGENTS.md` into project rules and a portable `docs/workflow.md` (roles, delegation, review, handoff, merge duties); the starter kit packages `workflow.md`, a blank `handoff.md` and a skeleton `AGENTS.md` | Claude | 1 thread, diff shown first |
| 8 | Research how the starter-kit sync (private `VeroxAUS/verox-template` plus the `new-verox-project` skill) works in T3 Code; port it if the experiment is kept. Not started; needs research on how skills and private repo writes work here | Claude | 1 research thread |

No push in any step. Steps 3 onward produce local commits the maintainer can inspect.

## 5. Risks

- **Rule drift**: three agents reading rules differently. Mitigation: one file, short per-agent bullets, step 2 proves each loads it.
- **Usage**: Codex reviews add cost. Mitigation: risky-only, one review per PR, the 2 to 3 thread cap.
- **Free-model quality**: OpenCode drafts can be wrong. Mitigation: never trusted unread; kept off anything that decides behaviour.
- **Identity leak**: another tool writes a commit with the wrong author. Mitigation: only Claude/maintainer commit; the pre-push log check stays.
- **Instruction conflicts**: AGENTS.md says "cloud threads" and "coordinator"; in T3 the work is local. Mitigation: AGENTS.md gets a short "while this experiment runs" note rather than rewriting the rules; reverting is deleting that note.
- **Reversibility**: everything here is additive docs. Dropping the experiment = restore the old CLAUDE.md from git (`git show main:CLAUDE.md`).

## 6. Open items for the maintainer

- Approve or edit sections 2 to 4.
- Which Codex model/effort to default to (suggest decide at step 4 after seeing cost).
- Whether the experiment's retro lives in `docs/research.md` or its own file.

## 7. Review log / time log

(empty until approved)
