# Contributing to Forgeline

Thanks for helping. Forgeline is a Windows and Steam build planner for Monster Hunter Wilds, written in Rust. It is pre-alpha: there is no app yet, so right now the most useful contributions are research, review and ideas. Please read [CLAUDE.md](CLAUDE.md) (standing rules) and [docs/PLAN.md](docs/PLAN.md) first.

## Sending changes

1. Open an issue first for anything bigger than a typo or small fix, so we can agree on the approach.
2. Fork the repo, make a branch, and open a pull request against `main`. Keep each PR to one change.
3. Tests and linters run on GitHub Actions and must pass. Tests use small fixture saves and fixture data only; they must pass on a machine without Wilds and never touch a real save, a real install or the live MHDB API.
4. Say in the PR whether you ran it against real game files. If you did not, say so.

## Licence

Forgeline's code is under [MPL-2.0](LICENSE). Contributions come in under the same licence. There is no DCO sign-off, no CLA and no copyright assignment. New source files carry the short MPL-2.0 notice from the licence's Exhibit A.

Forks and republishing are welcome. Please credit **"based on Forgeline"**, keep the copyright notices and licence, and do not present a fork as the official Forgeline.

Third-party code needs its licence recorded in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) before it enters the repo. Code from projects with no licence is not used, and GPL code stays out for now.

## Rules that never bend

- **Read-only on the game.** Never write to, move or delete Wilds save files, the game install or the Steam folders. Analyse copies of the save.
- **No injection.** Never inject into, hook or read the memory of the running game.
- **No game packs.** Never copy or unpack whole game packs; read only the files needed.
- **No personal or Capcom data in git.** Never commit saves, decrypted save dumps (they hold hunter IDs), extracted Capcom data, tokens, keys or personal details. Bring your own local save.
- **Local only.** No network listener, no telemetry, no accounts.
- **Light and fast.** The app runs beside the game; see the [performance budget](docs/performance-budget.md).
- **Honest data.** Game facts come from extracted data or a cited source. If something is unverified, say so; do not fill it in from memory.

## AI-assisted development

Much of this project is written with AI assistance (Claude) and directed and reviewed by the maintainer. AI-assisted contributions are fine under the same rules: you are responsible for what you submit, game facts must still be verified, and nothing generated may include copied third-party code or data.

## Conduct

Be kind. See the [Code of Conduct](CODE_OF_CONDUCT.md).
