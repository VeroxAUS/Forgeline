# RAM log

One line per pull request that changes the app, with the private bytes and working set read on the maintainer's PC in the real window with Wilds running. Each line is generated from that test round's results file in `bench/results/` by `tools/bench` (step 10), not typed. The rule for what a line may show is in the [Phase 2 plan](plans/phase-2-foundation.md) (section 3.4): at most +3 MB private per PR without a written reason, and never above 140 MB private or 80 MB working set.

Phase 1 reference (empty egui window, glow backend): 127 MB private bytes, 73.5 MB working set.

| Date | PR | Private bytes | Working set | Change from previous |
|------|----|---------------|-------------|----------------------|
| (first reading comes in step 11) | | | | |
