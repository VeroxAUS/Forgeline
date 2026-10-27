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
