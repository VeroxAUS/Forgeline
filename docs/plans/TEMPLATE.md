# Phase N: <name>

Target version: 0.N.0 · Status: draft / approved / in progress / done

Each phase plan is written just before its phase, using what earlier phases found. Thomas reviews and approves it before any work starts. Estimates are in threads and test rounds, not hours; real durations go in the time log (section 10).

## 1. Goal and done
What this phase delivers, in two or three sentences.

Done means:
- [ ] <checklist the test round can verify>

## 2. Inputs and dependencies
Earlier phases, `research.md` findings, closed questions this phase relies on.

## 3. Design

### 3.1 Architecture
Crates and modules, data flow, and what runs where (UI thread versus background work at low priority).

### 3.2 Per-file design
Repeat for each new or changed file.

`crates/<crate>/src/<file>.rs`
- **Purpose:** one line
- **Public API:** function and struct signatures
- **Depends on:** other files
- **Notes:** algorithms, error handling, performance concerns

### 3.3 Data formats
Files on disk, cache layout, schema versions.

### 3.4 Performance impact
Expected cold start, RAM and CPU, checked against the performance budget.

## 4. Steps
Each step is one thread, one PR and one test round.

| # | Step | Thread | Cloud or PC | Output (PR / doc) | Estimate (threads, test rounds) | Depends on |
|---|------|--------|-------------|-------------------|---------------------------------|------------|

## 5. Tools (need Thomas's yes)
| Tool | Why | Built or third-party | Licence |
|------|-----|----------------------|---------|

## 6. Tests
**Automated** (GitHub Actions, fixture saves and fixture data only): unit and fixture tests per file.

**Test round checklist** (queued, one at a time). Each item says what the app should show for Thomas's current save:
- [ ] With your save, the app should show <X>

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
