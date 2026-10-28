# plan-page

Turns a plan markdown file (`docs/plans/*.md`, `docs/performance-budget.md`) into one self-contained dark-themed HTML page, so every published plan page looks the same. Styling is baked in (`src/style.css`); the page has no scripts, no external fonts and makes no network requests.

## Use

```
cargo run --release --manifest-path tools/plan-page/Cargo.toml -- docs/plans/phase-1-spikes.md -o page.html
```

- Default output is the fragment the Artifact tool expects (`<title>`, `<style>`, `<main>`); it adds its own document skeleton. Publish it with the Artifact tool, using the same URL to update an existing plan page.
- `--standalone` writes a complete HTML document for opening straight in a browser.
- `--title <text>` overrides the title (default: the first `# ` heading).
- No `-o` prints to stdout.

## What it does

- Tables are wrapped in a scrolling box; task lists (`- [ ]`) show checkboxes.
- Raw HTML and `<...>` in the markdown are escaped and shown as text.
- Headings get ids (`3.1 Architecture` becomes `#31-architecture`).

## Rules for plan markdown

- Put a blank line between a bold label and the list under it, or the list collapses into a paragraph.
- Wrap placeholders like `<hours>` in backticks.

## Checks

```
cargo test --manifest-path tools/plan-page/Cargo.toml
cargo clippy --manifest-path tools/plan-page/Cargo.toml --all-targets -- -D warnings
```

Dependency licences are in the root `THIRD-PARTY-NOTICES.md`.
