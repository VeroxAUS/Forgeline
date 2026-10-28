# Changelog

All notable changes to Forgeline. Versions follow [semantic versioning](https://semver.org/): 1.0.0 is the full release, a major addition bumps the minor number, a bug fix bumps the patch number. The changelog starts with Phase 2.

## Unreleased (0.2.0)

### Added
- Cargo workspace, pinned toolchain, GitHub Actions for format, clippy and tests on Linux and Windows, and a repo hygiene test that keeps saves, dumps, packs and extracted data (and any other commit identity than the allowed ones) out of git. Reviewed and tested by VeroxAUS
- A headless RAM check in CI (`forgeline --headless-check`) that fails above 20 MB private bytes and writes a machine-readable results file (`bench/results/`), plus `docs/ram-log.md`. Reviewed and tested by VeroxAUS
- Automatic discovery of the Steam account, library, Wilds install, build ID and save files (read-only, via `libraryfolders.vdf` and the app manifest), the data-directory rules that refuse Steam, the install and the save folder, and `forgeline-check discover` for PC test rounds. Reviewed and tested by VeroxAUS
