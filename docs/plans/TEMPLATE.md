# Phase N: <name>

Target version: 0.N.0 · Status: draft / approved / in progress / done

Each phase plan is written just before its phase, using what earlier phases found. Thomas reviews and approves it before any work starts. Estimates are in threads and test rounds, not hours; real durations go in the time log (section 10).

## How to read this plan
A short box at the top, filled in last:
- **Run order:** the stages and which steps run in each (parallel steps share a stage), for example "Stage 1: steps 1 and 2 in parallel. Stage 2: step 3."
- **Decisions needed from Thomas:** a list, one item per line.

## 1. Goal and done
What this phase delivers, in two or three sentences.

Done means:
- [ ] <checklist the test round can verify>
- [ ] `README.md` is updated at close-out (see the close-out rule in section 4)

## 2. Inputs and dependencies
Earlier phases, `research.md` findings, closed questions this phase relies on.

## 3. Design

### 3.1 Architecture
Crates and modules, data flow, and what runs where (UI thread versus background work at low priority).

Every step (by its number) appears here, even if it only says "no special ordering".

### 3.2 Per-file design
One fixed block per new or changed file, with the file name as a heading and these labelled lines underneath:

#### `crates/<crate>/src/<file>.rs` (step N)
- **Purpose:** one line
- **Inputs and outputs:** what goes in, what comes out (signatures where code)
- **Depends on:** other files
- **Notes:** algorithms, error handling, performance concerns

### 3.3 Data formats
Files on disk, cache layout, schema versions.

### 3.4 Performance impact
Expected cold start, RAM and CPU, checked against the performance budget.

## 4. Steps
Each step is one thread, one PR and one test round.

The last step of every phase is the close-out. Besides updating `PLAN.md`, it updates `README.md`:
- the status badge and version badge match the phase's target version
- the roadmap shows this phase's progress
- placeholder visuals (such as the hero gear tiles) are replaced with a real screenshot of the app showing a build, once a window exists (from Phase 2); until then they stay labelled as placeholders

| # | Step | Thread | Cloud or PC | Output (PR / doc) | Estimate (threads, test rounds) | Depends on |
|---|------|--------|-------------|-------------------|---------------------------------|------------|

## 5. Tools (need Thomas's yes)
Why is plain words: the problem the tool solves, what happens without it, and what it touches on Thomas's PC.

| Tool | Why | Built or third-party | Licence |
|------|-----|----------------------|---------|

## 6. Tests

### 6.1 Automated
GitHub Actions, fixture saves and fixture data only. One bullet per file or behaviour.

### 6.2 Pass or fail criteria
One sub-list per step or feature, one criterion per line.

### 6.3 Test round checklist
Queued, one at a time, grouped by step. Each item says what the app should show for Thomas's current save:
- [ ] Step N: with your save, the app should show <X>

## 7. Risks and open questions
Each risk: how we would notice it, and the fallback. New questions go to `docs/open-questions.md` via the coordinator.

## 8. Versioning and changelog
Version bump per step (0.N.x for additions, x.x.1 for fixes) and the changelog lines.

## 9. Review log
| Date | Reviewer | Edit or decision |
|------|----------|------------------|

## 10. Time log
| Event | Date and time |
|-------|---------------|
| Phase approved by Thomas | |
| Step 1 PR merged | |
| Step 2 PR merged | |
| Phase finished | |

Elapsed from approval to finish: <hours>. Compare with the estimates in section 4.

## Plan writing rules (check before publishing)
1. Number steps in the order they run and use the same numbers in every section. Parallel steps share a stage.
2. Every step appears in every section that covers steps, even if only as "no special ordering".
3. No packed paragraphs: the done checklist, pass/fail criteria, tests and test round checklist are lists, one item per line.
4. Per-file designs use one fixed block per file (heading, Purpose, Inputs and outputs, Depends on, Notes).
5. Each tool's "Why" says what problem it solves, what happens without it, and what it touches on Thomas's PC.
6. A "How to read this plan" box sits at the top with the run order and the decisions needed.
7. The final step is a close-out that includes the `README.md` update (section 4), and the done checklist lists it.
8. Check the rendered page against this list before publishing the PR and the Artifact. Republish the same Artifact link when the plan changes.
9. Publish plan pages only via `tools/plan-page` (see its README), never with a hand-built page, so every republish looks identical.
