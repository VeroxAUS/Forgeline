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

## spikes-archive branch (reference only, not on `main`, not shipped)

| Source | Licence | Use |
|---|---|---|
| [eigeen/ree-pak-rs](https://github.com/eigeen/ree-pak-rs) | MIT | The 129-byte pak table modulus constant in `spikes/step5-extract/extract/src/pak.rs` on the `spikes-archive` branch, and the pak format facts behind our own reader. Credit retained. If this constant enters the product, copy the MIT notice here in full. |
