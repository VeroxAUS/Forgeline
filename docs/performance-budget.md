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
- None at the moment.

## Game stutter measurement
Decided 2026-10-27: our own Windows-only frame-time module in the sampler (`spikes/sampler`, Phase 1 step 6) reads Windows present events (ETW, as PresentMon does) and never touches the game process. It needs admin rights while running and is a test-PC tool, never shipped. If ETW access or a crate's licence blocks it, stutter is judged by eye while Thomas plays.

## Minimum spec
To be determined, near release. Approach agreed with Thomas: limit cores and RAM on the test PC and see what still meets the budget, with Wilds' official minimum spec as the baseline, quoted from Capcom's official page (not from memory).

## Change log
| Date | Change |
|------|--------|
| 2026-10-27 | Initial pass proposed and accepted by Thomas as a starting point. |
| 2026-10-27 | Frame-time module and minimum-spec approach agreed. |
