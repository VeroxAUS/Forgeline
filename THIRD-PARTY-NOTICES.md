# Third-party notices

Third-party code used by Forgeline, with its licence. Add a row before any third-party code enters the repo.

## tools/plan-page (build-time docs tool, not shipped in the app)

| Crate | Version | Licence | Use |
|---|---|---|---|
| pulldown-cmark | 0.13.5 | MIT | Markdown parser (CommonMark, tables, task lists) |
| pulldown-cmark-escape | 0.11.0 | MIT | HTML escaping, dependency of pulldown-cmark |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | dependency of pulldown-cmark |
| memchr | 2.8.3 | Unlicense OR MIT | dependency of pulldown-cmark |
| unicase | 2.10.0 | MIT OR Apache-2.0 | dependency of pulldown-cmark |

Licences read from each crate's `Cargo.toml` in the crates.io registry.

## Workspace crates (`crates/`, compiled into the app)

| Crate | Version | Licence | Use |
|---|---|---|---|
| serde | 1.0.229 | MIT OR Apache-2.0 | results files (JSON) |
| serde_core, serde_derive | 1.0.229 | MIT OR Apache-2.0 | dependencies of serde |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | reads and writes the results files |
| itoa, zmij | 1.0.18, 1.0.23 | MIT OR Apache-2.0, MIT | number formatting, dependencies of serde_json |
| memchr | 2.8.3 | Unlicense OR MIT | dependency of serde_json |
| proc-macro2, quote, syn | 1.0.107, 1.0.47, 3.0.9 | MIT OR Apache-2.0 | build-time only (derive macros) |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | build-time only (derive macros) |
| windows-sys | 0.59.0 | MIT OR Apache-2.0 | reads this process's memory use and the machine's RAM on Windows |
| windows-targets and the `windows_*` import crates | 0.52.6 | MIT OR Apache-2.0 | dependencies of windows-sys |

Licences read from each crate's `Cargo.toml` (`cargo metadata`). No GPL crate is used.

## spikes-archive branch (reference only, not on `main`, not shipped)

| Source | Licence | Use |
|---|---|---|
| [eigeen/ree-pak-rs](https://github.com/eigeen/ree-pak-rs) | MIT | The 129-byte pak table modulus constant in `spikes/step5-extract/extract/src/pak.rs` on the `spikes-archive` branch, and the pak format facts behind our own reader. Credit retained. If this constant enters the product, copy the MIT notice here in full. |
