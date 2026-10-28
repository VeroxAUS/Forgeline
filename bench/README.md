# Benchmark results

`bench/results/` holds one JSON file per measurement, written by `tools/bench` and the CI headless check (`forgeline --headless-check`). The schema is `forgeline_core::results::BenchResult`; the repo hygiene test rejects any committed file that does not fit it, or that holds a path.

- CI readings (`"ci": true`) are uploaded as a build artifact and not committed.
- Readings from the maintainer's PC are committed with the test round PR.
- Full-resolution frame-time traces stay local (`E:\mhbis\bench`) and are never committed.
- Published numbers (README, the future developer site, `docs/ram-log.md`) are generated from these files, never typed by hand.
