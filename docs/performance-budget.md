# Performance budget

Status: initial pass, reviewed over time. Agreed as a starting point with Thomas on 2026-10-27. Review it after Phase 1 (UI comparison) and whenever a phase changes what the app does.

Test PC: Intel Core i7-13700K, NVIDIA RTX 4080, 32 GB RAM. All numbers are measured on this PC with Wilds running, median of 3 runs, with our own cross-platform Rust sampler.

| What | Target | Hard limit |
|------|--------|------------|
| Cold start to last build on screen | under 1 s | 2 s |
| Idle RAM | under 80 MB | 150 MB |
| Idle CPU (app open, nothing changing) | about 0% (under 0.5% averaged over 60 s) | 1% |
| Idle GPU | no redraws when nothing changes | under 1% |
| Save change to updated build | under 2 s | 5 s |
| UI freeze during any work | none over 16 ms | 50 ms |
| Background work while Wilds runs | low priority, at most 2 CPU cores | no visible game stutter |
| Extraction (button press) | under 5 min, peak RAM under 1 GB | 10 min, 2 GB |
| Install size | under 50 MB | 100 MB |

## How the app meets it
- The app watches the save folder for file changes instead of checking on a timer, so it does nothing until the game saves.
- It draws only when something changes.
- Heavy jobs pause or slow down if the game's frame rate drops.

## Open items
- **Game stutter measurement:** how to measure the game's frame times without touching the game process. Windows event tracing (as tools like PresentMon use) never touches the game but is Windows-only; the alternative is judging by eye while Thomas plays. Undecided; Thomas asked whether our own tool can do it.

## Minimum spec
To be determined. Candidate approach: measure on lower-end hardware (or constrain CPU and RAM) once the first app exists, then set the minimum spec from what still meets the budget.

## Change log
| Date | Change |
|------|--------|
| 2026-10-27 | Initial pass proposed and accepted by Thomas as a starting point. |
