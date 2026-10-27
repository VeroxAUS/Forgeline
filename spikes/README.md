# Spikes archive

Throwaway code from Phase 1 (spikes), kept for reference. It is **not** product code and is never merged to `main`. Phase 2 rewrites anything it keeps. Results and findings are in [docs/research.md](../docs/research.md); each folder below says which step and PR it came from.

| Folder | Step | Source PR (closed, never merged) | What it does |
|--------|------|----------------------------------|--------------|
| `step3-typedump/typedump` | 3, type data (route A) | [#5](https://github.com/VeroxAUS/Forgeline/pull/5) | Reads the game `.exe` from disk to recover class and field names, and decrypts and walks a save copy. |
| `step4-savecheck/savecheck` | 4, ID matching and talismans | [#11](https://github.com/VeroxAUS/Forgeline/pull/11) | Parses a save copy and checks IDs and the talisman candidate. Refuses the live save folder; prints counts only. |
| `step5-extract/extract` | 5, extraction test | [#9](https://github.com/VeroxAUS/Forgeline/pull/9) | Pack reader and path-discovery crawl over the install; reads index entries, never whole packs. |
| `step6-uibench/uibench` | 6, UI comparison | [#16](https://github.com/VeroxAUS/Forgeline/pull/16) | Three equivalent minimal apps: egui, Tauri, Electron. |
| `step6-uibench/sampler` | 6, UI comparison | [#16](https://github.com/VeroxAUS/Forgeline/pull/16) | Process sampler plus the Windows-only frame-time module (ETW present events). |

## Notes

- Each crate is its own Cargo project (build from inside its folder). Nothing here is in a workspace, CI or the release package.
- Most of it ran on the maintainer's PC against a save copy and the real install. No saves, dumps or extracted Capcom data are in this branch.
- `step5-extract/extract/src/pak.rs` holds the 129-byte pak table modulus constant copied from [eigeen/ree-pak-rs](https://github.com/eigeen/ree-pak-rs) (MIT), with credit; it is listed in `THIRD-PARTY-NOTICES.md` on `main`.
- Tauri's generated `gen/schemas` folder was left out; `cargo build` recreates it.
- Libraries used (licences were checked in step 1; record them in `THIRD-PARTY-NOTICES.md` if any reaches the product): `aes`, `num-bigint`, `sysinfo`, `ferrisetw`, `windows`, `eframe`/`egui`, `tauri`, Electron.
- Licence of the code here: MPL-2.0 like the rest of the repo (the files predate the per-file notice, which is not added to throwaway code).
