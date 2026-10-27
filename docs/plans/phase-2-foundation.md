# Phase 2: Foundation

Target version: 0.2.0 · Status: draft (waiting for the maintainer's review)

Each phase plan is written just before its phase, using what earlier phases found. The maintainer reviews and approves it before any work starts. Estimates are in threads and test rounds, not hours; real durations go in the time log (section 10).

## How to read this plan

**Run order** (steps are numbered in the order they run; parallel steps share a stage; at most 3 threads at once):

- **Stage 1:** step 1 (workspace, CI, hygiene checks, changelog). Everything else waits for it.
- **Stage 2 (parallel):** step 2 (find the save and the game), step 3 (fixture save builder), step 5 (pack reader and exe scan).
- **Stage 3 (parallel):** step 4 (save reader, needs 3), step 6 (update flow with staging and validation, needs 2 and 5).
- **Stage 4:** step 7 (engine: file watcher, background recompute, cache; needs 2 and 4).
- **Stage 5:** step 8 (the bare egui window built from the debug data viewer; needs 6 and 7).
- **Stage 6:** step 9 (measure on the maintainer's PC with Wilds running; needs 8).
- **Stage 7:** step 10 (re-review the performance budget against real numbers; needs 9).
- **Stage 8:** step 11 (close-out; needs 1 to 10).

**Decisions needed from the maintainer** (details in section 7):

- Approve the four tools in section 5: the fixture save builder, `forgeline-check`, the benchmark harness, and the ported Phase 1 sampler inside it.
- Approve the RAM rule for every PR (section 3.4): a CI check on every PR, plus a PC reading on every PR that changes the app.
- Approve the proposed performance budget changes (step 10, list in section 3.4).
- Approve the update flow running as a short-lived child process of the app, not as a thread (section 3.1).
- Approve keeping a short hand-written list of the game file paths we need in the repo (section 3.3).

**Maintainer's own actions, listed up front (trial arrangement, sessions cannot push tags or delete branches):**

- At close-out: create the `v0.2.0` tag and a GitHub pre-release.
- After each step PR merges: delete its branch if the automatic delete did not (a cloud session gets a 403).
- Keep `spikes-archive` as it is; it is the only branch that stays besides `main`.
- During step 9: run the benchmark as administrator with Wilds at the Scarlet Forest base camp spot.

**Phase 1 retro items folded in:**

- Step 9 closes the Phase 1 step 6 gaps: "last build on screen" time and save-change recompute time, with Wilds running.
- Electron (only 2 valid no-Wilds runs) and Slint (never tested) are **not** re-run. Reasons are in section 2.
- A RAM check on every PR (step 1 for CI, step 9 for the PC reading).
- The budget is re-reviewed against the real Phase 1 numbers (step 10).
- GitHub Actions for the app crates (format, clippy, tests) from the first PR (step 1).

## 1. Goal and done
Build the foundation every later phase stands on: the app finds the maintainer's save and game install by itself, reads the active save slot, watches the save folder without polling, stages and validates an update of the game data, and shows all of it in a bare egui window grown from the debug data viewer. There is no optimizer yet; the window shows what the app knows, not a build.

Done means:

- [ ] On the maintainer's PC the app finds the Steam ID, the active save, the game install and the game build ID with no setup.
- [ ] The app reads the active slot and shows the slot table and counts (equipment box, item types, owned weapons and armor) that match the maintainer's game.
- [ ] The app opens from cached data, shows the last view at once, and refreshes in the background when the save has changed.
- [ ] Changing the save (the game saving) updates the view without a freeze and without polling.
- [ ] The update button stages a new extraction of the needed game files, validates it against the last good data, and swaps it in only if it passes. A failed update keeps the last good data and says so on screen.
- [ ] A game patch (new build ID) shows a visible "data may be out of date" notice.
- [ ] CI (format, clippy, tests, hygiene checks) is green on Linux and Windows for every PR.
- [ ] Cold start to the last view, idle RAM, idle CPU and recompute time are measured on the maintainer's PC with Wilds running and written down against the budget.
- [ ] No save, dump or extracted Capcom data is in git, in a test fixture or in a PR.
- [ ] The performance budget is re-reviewed and the maintainer has decided each proposed change.
- [ ] `CHANGELOG.md` exists and lists 0.2.0.
- [ ] `docs/PLAN.md` is updated.
- [ ] `README.md` text, status badge, version badge (0.2.0) and roadmap are updated at close-out (see the close-out rule in section 4).
- [ ] The banner (`assets/readme/hero.svg` and its alt text) names Phase 3 as the phase in progress.
- [ ] The README carries the first real screenshot of the app window, with no personal data.

## 2. Inputs and dependencies

- **Phase 1 results** in [PLAN.md](../PLAN.md) section 6 and [research.md](../research.md): steps 2 (save container and decryption), 3 (route A names), 4 (save structure), 5 (pack reader and file naming) and 6 (UI comparison).
- **Standing decisions:** egui with the glow backend; route A for names at update time; MPL-2.0 with the Exhibit A notice at the top of every source file; semantic versioning with a changelog from this phase.
- **Performance budget** ([performance-budget.md](../performance-budget.md)) and the standard benchmark spot (Scarlet Forest base camp).
- **RAM headroom is thin.** The empty egui window measured 127 MB private bytes and 73.5 MB working set. That leaves about 23 MB to the 150 MB hard limit and about 6 MB to the 80 MB target. Every step in this phase treats RAM as a feature.
- **Credit and notices:** the pack table key constant comes from ree-pak-rs (MIT). It is credited in `THIRD-PARTY-NOTICES.md` in step 5, together with any crate that enters the product.
- **Clean-room rule still applies.** The save reader (step 4) and fixture builder (step 3) are written from `research.md` only. Nobody opens MHWeditor.
- **Why Electron and Slint are not re-run:**
  - Electron's two valid no-Wilds runs only affect a row that was not decisive. With Wilds running it had all 3 runs, and its private bytes (206 MB) are over the 150 MB hard limit by a wide margin, so another run cannot change the verdict.
  - Slint is only worth testing if egui fails the RAM check in this phase. The trigger is written into the risks (section 7): private bytes above 140 MB with real data in step 9.
  - Slint's licence terms are still open (open question 6), which would also have to be settled before use.
- **Open questions that touch this phase but are not answered here:**
  - Save-ID to data-ID matching by name (question 1), talisman decoding (2) and unlocked-monster mapping (3) are Phase 3. The window shows raw IDs and counts only.
  - Which character slot is active: the notes disagree. [PLAN.md](../PLAN.md) section 3 says one slot has `Active` = 1, but open question 3 lists the active slot as still open. Step 4 reads the `Active` field and step 9 confirms it on the maintainer's save; if it does not hold, the window shows all slots and says the active one is unknown.
  - Update confidence above 95% (question 9). Step 6 builds the structural half of the gate; the scoring half needs the optimizer.
  - Motion values (question 11): this phase only makes sure the pack reader can serve the deep search in Phase 3 (see [PLAN.md](../PLAN.md) section 6, Phases 3 and 5).

## 3. Design

### 3.1 Architecture

A Cargo workspace of small crates. The UI is a thin shell; nothing heavy lives in it.

| Crate | Holds | Step |
|-------|-------|------|
| `forgeline-core` | paths and discovery (Steam, library, game, save), the data-directory layout, shared types, errors, the view summary | 1, 2 |
| `forgeline-save` | save container decryption, inflate, class-stream walker, typed readers for the classes we need | 4 |
| `forgeline-pak` | pack index and single-entry reader, path hashing, exe string scan (route A), type-name table | 5 |
| `forgeline-update` | the update flow: extract needed files, stage, validate, swap | 6 |
| `forgeline-engine` | file watcher, background worker, cache, the single `Snapshot` the UI reads | 7 |
| `forgeline-app` | the egui window (debug data viewer) | 8 |
| `tools/forgeline-check` | developer command-line report (counts and names only) | 2 |
| `tools/fixture-builder` | builds small synthetic fixture saves | 3 |
| `tools/bench` | benchmark harness, ported from the Phase 1 sampler | 9 |

What runs where:

- **UI thread:** draws from the latest `Snapshot` only. It never reads a file, never parses, never waits. It asks for a repaint only when the engine reports a change or the user interacts, so idle CPU and GPU stay at zero.
- **Engine thread (one, lowest priority):** waits on the file watcher. When the save changes it waits for the write to settle, reads the save into memory, parses it, builds a new `Snapshot` and hands it over by swapping a pointer. A second change during a recompute marks the work dirty and runs once more after, so changes are never lost and never pile up.
- **Update process:** pressing the button starts `forgeline.exe --update` as a **short-lived child process** at below-normal priority. It exits when done, so everything it used (pack indexes of 349,348 entries, file buffers) goes back to the system and never counts against the app's RAM headroom. It writes only to the staging folder in the app's data directory and reports progress through a small status file. It is not a thread because the extraction peak (target under 1 GB) would otherwise stay in the app's private bytes.
- **Order of work at start:** read the view cache, draw it (this is the "last build on screen" moment), then check in the background whether the save changed since the cache was written, and recompute only if it did.

Step by step:

- **Step 1:** first; the others build on its workspace and checks.
- **Step 2:** no special ordering after step 1. Discovery is pure functions over injected paths so tests use fixture folders.
- **Step 3:** no special ordering after step 1. It writes the container format from `research.md` notes, so step 4 can read what it writes.
- **Step 4:** needs step 3's fixtures and step 1. It takes the type-name table as input; step 5 produces that table, and until then tests use a small fixture table.
- **Step 5:** no special ordering after step 1. Its output (the type-name table and the pack reader) is used by steps 4 on the real files and by step 6.
- **Step 6:** needs step 2 (where the game is) and step 5 (how to read it).
- **Step 7:** needs step 2 and step 4.
- **Step 8:** needs step 6 (update status) and step 7 (the `Snapshot`).
- **Step 9:** needs the working window from step 8.
- **Step 10:** needs step 9's numbers.
- **Step 11:** last; needs steps 1 to 10.

PC safety (all steps): the app never writes to the save, the game install or the Steam folders. It opens the save file with read-and-share access (other programs, including the game, may keep writing), reads the whole file (about 200 KB) into memory, checks size and modified time were unchanged across the read, and retries on a mismatch. All parsing is on that in-memory copy, never on a handle to the live file. It never injects, hooks or reads the game's memory.

### 3.2 Per-file design

Files are listed by step. Every source file starts with the MPL-2.0 Exhibit A notice (checked by step 1's hygiene test).

#### `Cargo.toml`, `rust-toolchain.toml`, `.gitignore` (step 1)
- **Purpose:** workspace definition, pinned toolchain, and ignore rules for saves, dumps, packs and extracted data.
- **Inputs and outputs:** in, nothing. Out, a workspace that builds on Linux (all crates except the window) and Windows (everything).
- **Depends on:** nothing.
- **Notes:** release profile tuned for size and start-up (`opt-level = "s"` or `3` decided by measurement in step 9, `lto`, `panic = "abort"`, `strip`). Version `0.2.0-dev` until close-out.

#### `.github/workflows/ci.yml` (step 1)
- **Purpose:** format check, clippy with warnings denied, tests, and the RAM check, on every PR.
- **Inputs and outputs:** in, the PR. Out, a pass or fail per job. Jobs run on `ubuntu-latest` and `windows-latest`.
- **Depends on:** the workspace and `crates/forgeline-core/tests/repo_hygiene.rs`.
- **Notes:** the RAM job builds the release binary on Windows and runs `forgeline --headless-check <fixture>`, which loads the fixture data, builds a `Snapshot` and prints peak private bytes (no window, because hosted runners have no GPU). It fails the job above the limit in section 3.4. This checks the data and engine side only; the GL window cost is read on the PC (step 9).

#### `crates/forgeline-core/tests/repo_hygiene.rs` (step 1)
- **Purpose:** keep personal data and Capcom data out of git, and keep source files uniform.
- **Inputs and outputs:** in, the list of tracked files. Out, a test failure naming the offending file.
- **Depends on:** nothing.
- **Notes:** fails on tracked files matching save, pak, dump or extracted-data patterns (`*.bin` outside the fixtures folder, `*.pak`, `*.user.3`, `*.msg.*`, `*dump*`), on a source file without the MPL-2.0 notice, and on any commit in the PR range whose author or committer email is not the VeroxAUS noreply, Claude's noreply or GitHub's. Fixtures are allowed only under `fixtures/` and only when listed in `fixtures/README.md` as synthetic.

#### `CHANGELOG.md` (steps 1, 11)
- **Purpose:** the changelog, from this phase on.
- **Inputs and outputs:** in, one line per merged step. Out, an `Unreleased` section that becomes `0.2.0` at close-out.
- **Depends on:** nothing.
- **Notes:** keep-a-changelog style headings (Added, Changed, Fixed).

#### `THIRD-PARTY-NOTICES.md` (steps 1, 5)
- **Purpose:** licence record for every crate and credit that enters the repo.
- **Inputs and outputs:** in, each new dependency. Out, a table row with name, version, licence and use.
- **Depends on:** nothing.
- **Notes:** step 5 adds the ree-pak-rs credit for the table key constant. A GPL crate is refused.

#### `crates/forgeline-core/src/discovery.rs` (step 2)
- **Purpose:** find the Steam folder, the library holding Wilds, the install, the build ID, the Steam ID folder and the save files.
- **Inputs and outputs:** `discover(env: &DiscoveryEnv) -> Result<Discovered, DiscoveryError>`. `DiscoveryEnv` carries the Steam root (from the Windows registry, or injected in tests) and a filesystem trait. `Discovered` holds the library path, install path, build ID (from the Steam app manifest for app 2246340), Steam account ID, 64-bit Steam ID, the save folder and the save slot files found.
- **Depends on:** `vdf.rs` below.
- **Notes:** the install is found through `libraryfolders.vdf`, never hardcoded. If several account folders exist, only the one with `2246340\remote\win64_save` counts; if more than one does, the result lists them and the window asks. Mod markers (`dinput8.dll`, `reframework`, `pak_mods`) are reported as facts and never read from. Windows registry access sits behind `cfg(windows)`.

#### `crates/forgeline-core/src/vdf.rs` (step 2)
- **Purpose:** small reader for Valve's text format (`libraryfolders.vdf`, `appmanifest_*.acf`).
- **Inputs and outputs:** in, text. Out, a nested map. Errors on malformed input, never panics.
- **Depends on:** nothing.
- **Notes:** hand-written; the format is simple and a crate would add RAM and a licence entry.

#### `crates/forgeline-core/src/datadir.rs` (steps 1, 2)
- **Purpose:** the app's own data directory layout (section 3.3) and the rule that dev builds use a different one.
- **Inputs and outputs:** `DataDir::open(override: Option<PathBuf>) -> DataDir`. Default is `%LOCALAPPDATA%\Forgeline`; `--data-dir` or `FORGELINE_DATA_DIR` overrides it.
- **Depends on:** nothing.
- **Notes:** refuses any path inside the Steam folders, the game install or `win64_save`. The maintainer's installed copy and its cache are reserved; dev and test builds always pass an override.

#### `tools/forgeline-check/src/main.rs` (step 2, grown in steps 4, 5, 6)
- **Purpose:** a developer report for PC test rounds: prints what the foundation sees, as counts and names only.
- **Inputs and outputs:** subcommands `discover`, `save`, `pak`, `update`. Out, plain text. Never prints hunter IDs, hunter names or file contents.
- **Depends on:** the crates it reports on.
- **Notes:** it is the thing the PC thread runs in steps 2 to 6, before the window exists. It copies the save to its own work folder and reads only the copy; it refuses the live `win64_save` folder as an output location.

#### `tools/fixture-builder/src/main.rs` (step 3)
- **Purpose:** build small synthetic saves in the real container format (encrypted, deflated, class stream) with known contents.
- **Inputs and outputs:** in, a Steam ID and a recipe (slots, equipment, items, flags). Out, a `data001Slot.bin`-shaped file and an expected-values JSON next to it.
- **Depends on:** `research.md` notes for the container; the type-name fixture table.
- **Notes:** contents are invented and say so (fixed fake Steam ID, no hunter IDs, round numbers). Nothing is derived from the maintainer's save. Files are a few KB. Output goes to `fixtures/saves/`.

#### `crates/forgeline-save/src/container.rs` (step 4)
- **Purpose:** decrypt with the Steam ID and inflate.
- **Inputs and outputs:** `open(bytes: &[u8], steam_id: u64) -> Result<Vec<u8>, SaveError>` returns the class stream.
- **Depends on:** `research.md` notes (Phase 1 step 2 and its follow-up).
- **Notes:** a failed inflate or a stream walk that does not end exactly at the end is the integrity check (CityHash is not needed). Large buffers are dropped as soon as the walk is done.

#### `crates/forgeline-save/src/stream.rs` (step 4)
- **Purpose:** walk the class stream, including the two format corrections from Phase 1 step 3 (16-byte struct alignment, packed scalar and struct arrays).
- **Inputs and outputs:** in, the stream and a `TypeNames` table. Out, a lightweight tree that visits only the classes and fields we ask for and skips the rest without allocating.
- **Depends on:** `container.rs`.
- **Notes:** the full stream is 16.5 MB inflated and has 187 class kinds; we must not build a tree of all of it. Skipping uses the size prefixes.

#### `crates/forgeline-save/src/model.rs` (step 4)
- **Purpose:** typed summary of the active slot: slot table, equipment box (category, Artian fields, decoration slots), item box and pouch counts, created and checked flags, and the story, mission and enemy-report records as raw counts.
- **Inputs and outputs:** `read_save(bytes, steam_id, names) -> Result<SaveSummary, SaveError>`. `SaveSummary` holds data the window needs and nothing else.
- **Depends on:** `stream.rs`.
- **Notes:** reads the `Active` field to pick the slot (see section 2). Hunter name and IDs are not copied into the summary. Enum fields are read as numbers.

#### `crates/forgeline-pak/src/index.rs` (step 5)
- **Purpose:** read pack headers and entry tables, handle the encrypted table and the chunk table, and resolve "which pack wins" across patches.
- **Inputs and outputs:** `PackSet::open(install: &Path) -> Result<PackSet>`; `PackSet::entry(path_hashes) -> Option<EntryRef>`.
- **Depends on:** nothing in the repo.
- **Notes:** reads only the original `re_*.pak` files, never `pak_mods`. Index memory is about 17 MB for 349,348 entries and lives only inside the update process. Format facts and the table key are credited to ree-pak-rs (MIT).

#### `crates/forgeline-pak/src/read.rs` (step 5)
- **Purpose:** read one entry: stored, deflate or zstd, including chunked entries.
- **Inputs and outputs:** `PackSet::read(entry) -> Result<Vec<u8>>`.
- **Depends on:** `index.rs`.
- **Notes:** never reads more than the entry; never writes. Size limit per entry to stop a corrupt table from asking for gigabytes.

#### `crates/forgeline-pak/src/paths.rs` (step 5)
- **Purpose:** path hashing (two murmur3 hashes, seed `0xFFFFFFFF`, UTF-16, lower and upper case) and the hand-written list of game files we need.
- **Inputs and outputs:** `hash_pair(path) -> (u32, u32)`; `NEEDED: &[NeededFile]` with the path (version suffix found by trying 0 to 70) and a category tag.
- **Depends on:** nothing.
- **Notes:** the list holds paths only (about 80 under `GameDesign/Common/...`, taken from the Phase 1 step 5 categories), no Capcom data. Whether a path list in the repo is acceptable is a decision in the box at the top.

#### `crates/forgeline-pak/src/exescan.rs` (step 5)
- **Purpose:** route A: scan the game `.exe` on disk for strings, rebuild class and field names, and write the type-name table for `app.savedata` classes starting with `c`.
- **Inputs and outputs:** `scan(exe: &Path, wanted: &SaveHashes) -> Result<TypeNames>`. Input includes the hashes found in a save copy (so only names that matter are rebuilt).
- **Depends on:** nothing in the repo.
- **Notes:** the restricted rebuild must run in seconds, not the minutes of the spike. Records the exe size, modified time and the build ID with the table. Reads the `.exe` with a streaming reader, never loads 546 MB at once.

#### `crates/forgeline-update/src/lib.rs` (step 6)
- **Purpose:** the update flow, as a function the `--update` process calls.
- **Inputs and outputs:** `run_update(env, datadir, progress) -> UpdateOutcome` where the outcome is `Replaced`, `Unchanged` or `Rejected(Vec<Finding>)` or `Failed(Error)`.
- **Depends on:** `forgeline-pak`, `forgeline-core`, `validate.rs`.
- **Notes:** writes to `staging/` only. Order: discover, scan exe, extract the needed files, write the manifest, validate, then swap. Any error leaves `current/` untouched. Never fetches from the network.

#### `crates/forgeline-update/src/validate.rs` (step 6)
- **Purpose:** the structural half of the validation gate.
- **Inputs and outputs:** `validate(staged: &Manifest, last_good: Option<&Manifest>) -> Vec<Finding>`; any `Block` finding stops the swap.
- **Depends on:** `manifest.rs`.
- **Notes:** blocks if a file present in the last good data is missing, if a file's category count shrinks, if a file does not open as the expected container, if the build ID is older than the last good one, or if the type-name table lost any class. Warns (does not block) on a new build ID or a large growth. The scoring half (false upgrades, builds scoring worse) needs the optimizer and arrives in Phase 4; until then the app says "structure checked, builds not re-scored".

#### `crates/forgeline-update/src/manifest.rs` (step 6)
- **Purpose:** the record of an extraction.
- **Inputs and outputs:** serde JSON: schema version, game build ID, exe size and modified time, tool version, per file path, size, hash and category.
- **Depends on:** nothing.
- **Notes:** the same manifest is used for the patch warning: current build ID versus the manifest's.

#### `crates/forgeline-engine/src/lib.rs` (step 7)
- **Purpose:** owns the worker thread and the `Snapshot`.
- **Inputs and outputs:** `Engine::start(datadir, notify: impl Fn()) -> Engine`; `engine.snapshot() -> Arc<Snapshot>`. `Snapshot` holds discovery results, save summary or the reason it is missing, data status (current manifest, patch warning, last update outcome) and a version counter.
- **Depends on:** `forgeline-core`, `forgeline-save`, `watch.rs`, `cache.rs`.
- **Notes:** the thread runs at the lowest priority. Parse failures keep the previous good summary and set a visible error, never silent wrong data.

#### `crates/forgeline-engine/src/watch.rs` (step 7)
- **Purpose:** watch the save folder with the operating system's change notifications (no timer).
- **Inputs and outputs:** a callback on "`data001Slot.bin` changed".
- **Depends on:** the `notify` crate (licence checked in step 1 and recorded).
- **Notes:** waits about 500 ms of quiet after the last event so a half-written file is not read; the size and modified-time check in section 3.1 is the second guard. Watches the one folder, not recursively. If the folder does not exist yet it watches the parent for its creation.

#### `crates/forgeline-engine/src/cache.rs` (step 7)
- **Purpose:** the view cache, so the window shows the last view at once.
- **Inputs and outputs:** `load() -> Option<Snapshot>`, `store(&Snapshot)`. One small JSON file.
- **Depends on:** `forgeline-core`.
- **Notes:** holds the derived summary only (counts, slot table, build ID), never the raw save, and no hunter name or ID. Stamped with the save's size and modified time and the schema version; a mismatch means "show it, mark it as refreshing, recompute". A corrupt cache is ignored and rebuilt.

#### `crates/forgeline-app/src/main.rs` (step 8)
- **Purpose:** program entry: window, the `--update` child mode, the `--headless-check` mode used by CI.
- **Inputs and outputs:** command-line flags (`--data-dir`, `--update`, `--headless-check <fixture>`); otherwise opens the window.
- **Depends on:** `forgeline-engine`, `forgeline-update`, `view.rs`.
- **Notes:** dark theme set once at start. No network code is linked in.

#### `crates/forgeline-app/src/view.rs` (step 8)
- **Purpose:** the debug data viewer, drawn from a `Snapshot`.
- **Inputs and outputs:** `fn show(ui, &Snapshot, &mut ViewState)`.
- **Depends on:** `forgeline-engine`.
- **Notes:** panels: where things were found (Steam ID folder, library, install, build ID, mod markers); save (slot table with the active slot marked, counts); game data (extraction time, build ID, status, patch notice, last update result with findings); an Update button with progress. Text is large and high-contrast for a second monitor. It never lays out more than it shows (tables are virtualised), and it never repaints without a reason.

#### `tools/bench/src/main.rs` (step 9)
- **Purpose:** repeatable numbers: cold start to the last view, idle RAM (working set and private bytes), idle CPU over 60 s, recompute time after a save change, and the game's frame times.
- **Inputs and outputs:** in, the app path, a data directory and a scenario. Out, a results file in `E:\mhbis\bench` with median of 3 runs.
- **Depends on:** the app's trace output (below).
- **Notes:** ported from the archived Phase 1 `spikes/sampler`, now our own product-side tool. The Windows frame-time module needs administrator rights and never touches the game process. The app prints two timestamps when `FORGELINE_TRACE=1` is set: "first view drawn" and "snapshot ready after save change". Tests copy a fixture or a save copy over a copy of the save in a scratch folder, never the live one.

#### `docs/ram-log.md` (steps 1, 9, every later PR)
- **Purpose:** the running RAM record (section 3.4).
- **Inputs and outputs:** one line per PR that changes the app: date, PR, private bytes, working set, change from the previous line.
- **Depends on:** nothing.
- **Notes:** the coordinator adds the line with the test round result.

### 3.3 Data formats

App data directory (`%LOCALAPPDATA%\Forgeline`, overridden for dev builds):

```
data/
  current/            last good extraction
    manifest.json     schema version, build ID, exe size and time, files with hashes
    typenames.json    type-name table (route A)
    raw/              only the files in the needed list, as read from the packs
  staging/            a new extraction while it runs; deleted after a swap or a failure
  update-status.json  progress and the last outcome, written by the --update process
view-cache.json       derived summary for instant start (no raw save, no hunter name or ID)
settings.json         window size and the user's chosen options (nothing else)
```

- **Swap:** staging is renamed over `current/` only after validation passes; the old `current/` is kept as `previous/` until the next successful update, so a bad swap can be rolled back by hand.
- **Schema versions:** `manifest.json` and `view-cache.json` carry a `schema` number. An unknown number means "ignore the cache and recompute", never a crash.
- **Nothing here is committed or shipped.** `raw/` holds Capcom data and stays in the user's own data directory. The repo `.gitignore` and the hygiene test keep it out of git; the Nexus package excludes it by construction.
- **Repo fixtures:** `fixtures/saves/` (synthetic saves and their expected values), `fixtures/packs/` (a tiny synthetic pack and exe), `fixtures/steam/` (invented `libraryfolders.vdf`, app manifest and folder tree). All invented and listed in `fixtures/README.md`.
- **The needed-files list** is in `crates/forgeline-pak/src/paths.rs`. A new file is added by hand when a phase needs it.

### 3.4 Performance impact

Budget rows from [performance-budget.md](../performance-budget.md), what this phase expects, and where it is checked:

| Row | Target / hard limit | Phase 1 measured (empty egui window) | Phase 2 expectation | Checked in |
|-----|---------------------|--------------------------------------|---------------------|------------|
| Cold start to last view | under 1 s / 2 s | 200 ms to first frame | stays well under 1 s: read one small JSON, draw | step 9 |
| Idle RAM | under 80 MB / 150 MB | 73.5 MB working set, 127 MB private | at most a few MB more; data held is a summary of a few KB | every PR, step 9 |
| Idle CPU / GPU | 0% / no redraws | 0.00% / 0.00% | unchanged; the watcher is event driven | step 9 |
| Save change to updated view | under 2 s / 5 s | not measured | read, inflate, walk and summarise 16.5 MB: expected well under 1 s on the test PC | step 9 |
| UI freeze during work | none over 16 ms / 50 ms | none seen | all work off the UI thread | step 9 |
| Background work | low priority, at most 2 cores | job on 1 core, no stutter | engine is one thread; update is one process at below-normal priority | step 9 |
| Extraction | under 5 min, under 1 GB / 10 min, 2 GB | not measured | pack index about 17 MB plus single files; mostly disk time | step 9 |
| Install size | under 50 MB / 100 MB | not measured | one binary, no bundled data | step 9 |

**RAM rule for every PR (proposed, needs approval):**

- **CI, every PR:** the Windows job runs the headless check and fails above 20 MB private bytes for the data and engine side with fixture data. This catches data structures that grow, and runs without a GPU.
- **PC reading, every PR that changes the app:** the test round reads private bytes and working set in the real window, with Wilds running, and adds a line to `docs/ram-log.md`.
- **Limits on the PC reading:**
  - a PR may raise private bytes by at most 3 MB without a written reason
  - the total must stay at or below 140 MB private (10 MB under the hard limit) and 80 MB working set
  - over either limit, the PR does not merge until the RAM is recovered or the maintainer accepts the cost
- **Docs-only PRs** skip both checks.
- **Known gap:** the headless check does not include the graphics driver's share, which is most of the 127 MB. That is why the PC reading exists.

**Proposed changes to the budget (decided in step 10, from step 9's numbers):**

- Define "Idle RAM" as two rows: working set (target 80 MB) and private bytes (hard limit 150 MB), because Phase 1 showed they differ by 50 MB and the old row did not say which was meant.
- Make "no visible game stutter" a number: the game's median and 99th percentile frame time within 1 ms of the control run.
- Add rows for "last view on screen" (cold start with real data) and "save change to updated view", now that they can be measured.
- Reconsider the 80 MB working-set target if step 9 shows it cannot be met with real data; the maintainer decides between loosening it and cutting features.

## 4. Steps
Each step is one thread, one PR and one test round. Steps 2 to 8 change what the app reads or shows, so their rounds run on the maintainer's PC (steps 2 to 6 through `forgeline-check`, 7 and 8 through the app). Steps 1, 10 and 11 are cloud-only. Every PR from step 2 on says whether it is untested against the real files.

The last step of every phase is the close-out. Besides updating `PLAN.md`, it updates `README.md`:
- the README text (status paragraph, roadmap list, repository layout, versioning note) matches the phase result
- the status badge and version badge match the phase's target version
- the roadmap shows this phase's progress
- the banner (`assets/readme/hero.svg` and the alt text in `README.md`) names the phase that is now in progress
- the placeholder hero tiles are replaced with a real screenshot of the app window (the first one exists from this phase)

| # | Stage | Step | Thread | Cloud or PC | Output (PR / doc) | Estimate (threads, test rounds) | Depends on |
|---|-------|------|--------|-------------|-------------------|---------------------------------|------------|
| 1 | 1 | Workspace, CI (format, clippy, tests, RAM check on Linux and Windows), hygiene test, `CHANGELOG.md`, `.gitignore`, MPL notice, `ram-log.md` | Foundation thread | Cloud | PR: workspace and CI | 1 thread, 1 cloud round; about 1 h | none |
| 2 | 2 | Discovery: Steam, library, install, build ID, Steam ID, save files; data directory; `forgeline-check discover` | Discovery thread | Cloud code, PC check | PR: `forgeline-core`, `forgeline-check` | 1 thread, 1 cloud round, 1 PC round; about 1.5 h | 1 |
| 3 | 2 | Fixture save builder and first fixtures | Fixture thread (separate from the reader thread) | Cloud | PR: `tools/fixture-builder`, `fixtures/` | 1 thread, 1 cloud round; about 1 h | 1 |
| 4 | 3 | Save reader: decrypt, inflate, walk, typed summary, active slot; `forgeline-check save` | Reader thread (reads `research.md` only) | Cloud code, PC check | PR: `forgeline-save` | 1 to 2 threads, 1 cloud round, 1 PC round; about 2 h | 1, 3 |
| 5 | 2 | Pack reader and exe scan (route A), needed-files list, ree-pak-rs credit; `forgeline-check pak` | Pack thread | Cloud code, PC check | PR: `forgeline-pak`, notices | 1 to 2 threads, 1 cloud round, 1 to 2 PC rounds; about 3 h | 1 |
| 6 | 3 | Update flow: stage, validate (structural gate), swap, keep last good; `--update` mode; `forgeline-check update` | Update thread | Cloud code, PC check | PR: `forgeline-update` | 1 thread, 1 cloud round, 1 PC round; about 2 h | 2, 5 |
| 7 | 4 | Engine: save-folder watcher, background recompute at low priority, view cache, `Snapshot` | Engine thread | Cloud code, PC check | PR: `forgeline-engine` | 1 thread, 1 cloud round, 1 PC round; about 1.5 h | 2, 4 |
| 8 | 5 | The bare egui window from the debug data viewer, update button and notices | Window thread | Cloud code, PC check | PR: `forgeline-app` | 1 to 2 threads, 1 cloud round, 1 PC round; about 2 h | 6, 7 |
| 9 | 6 | Measure with Wilds running: benchmark harness, cold start to last view, recompute time, idle RAM, CPU and GPU, game frame times; fill `ram-log.md`; confirm the active-slot field | Benchmark thread | PC | PR: `tools/bench`, `research.md` section | 1 thread, 1 to 2 PC rounds; about 2 h | 8 |
| 10 | 7 | Re-review `performance-budget.md` against the Phase 1 and step 9 numbers | Docs thread | Cloud | PR: budget changes | 1 thread, 1 cloud round; about 0.5 h | 9 |
| 11 | 8 | Close-out: `PLAN.md`, `README.md` (text, badges, roadmap, banner phase label, first screenshot), `CHANGELOG.md` 0.2.0, version bump, plan time log | Docs thread | Cloud | PR: docs and version | 1 thread, 1 cloud round; about 0.75 h | 1 to 10 |

Total estimate: 11 to 14 threads and 14 to 16 test rounds, 8 to 9 of them on the PC. At most 3 threads run at once. Estimates are guesses and will be compared with the time log.

Hours (rough wall-clock from thread start to merge, each including its test round and the wait for the maintainer to pass it):

- **Per step:** the figures in the table above, 17 h added up (1 + 1.5 + 1 + 2 + 3 + 2 + 1.5 + 2 + 2 + 0.5 + 0.75 = 17.25).
- **Total with parallel stages:** about 13 h from approval to the last merge. Stage 1 is 1 h, stage 2 is 3 h (steps 2, 3 and 5 overlap, step 5 is the longest), stage 3 is 2 h, stage 4 is 1.5 h, stage 5 is 2 h, stage 6 is 2 h, stage 7 is 0.5 h and stage 8 is 0.75 h.
- **Plausible range:** 10 to 18 h. It assumes the maintainer passes each test round within about 15 minutes; a slower reply adds to the total.
- **Calibration:** Phase 1 took 2 h 06 min for 7 steps (about 18 min a step), but those were docs and throwaway spikes with several threads in parallel. Phase 2 steps are real code with tests, CI and PC rounds, so each is estimated at 3 to 6 times a Phase 1 step.
- **Least certain:** step 5 (3 h) and step 9 (2 h, a second round if RAM is over a limit).

Where the estimate is most likely wrong:
- Step 5 is the largest unknown: the restricted exe rebuild must run in seconds, and the pack table handling for the two newest patches uses the chunk table. If it needs more than 2 threads, stop and report to the maintainer (Opus on high may then be worth suggesting).
- Step 9 may take a second round if the first reading is over a RAM limit and a fix is needed.

## 5. Tools (need the maintainer's yes)
All are our own Rust. Each needs a yes before it is created or run. Nothing here installs outside normal user folders.

| Tool | Step | Why (plain words) | Built or third-party | Licence |
|------|------|-------------------|----------------------|---------|
| `tools/fixture-builder` | 3 | Problem: tests must pass without Wilds and must never use your real save, because it holds hunter IDs. This tool builds tiny invented saves in the real format with known contents. Without it we either commit a real save (forbidden) or cannot test the reader. It touches nothing on your PC; it only writes files into the repo checkout. | Built, from `research.md` notes only | ours (MPL-2.0) |
| `tools/forgeline-check` | 2 | Problem: until the window exists (step 8) there is no way to see what the reader and extractor do on your real files, and every PC round would need a custom program. This is one small report tool for all of them. It prints counts and names only. On your PC it copies your save into its own work folder and reads the copy, reads the game install read-only, and writes only to `E:\mhbis`. | Built | ours |
| `tools/bench` | 9 | Problem: the budget is judged on numbers from your PC with Wilds running, not impressions. This measures start-up, RAM, CPU, recompute time and the game's frame times the same way every time. Without it the RAM rule cannot be checked and the Phase 1 gaps stay open. It is the Phase 1 sampler, ported from `spikes-archive`, with the Windows-only frame-time module (needs administrator rights while running; it reads Windows present events and never touches the game process). On your PC it reads process and event figures and writes to `E:\mhbis\bench`. | Built (ported) | ours; `ferrisetw` (MIT OR Apache-2.0), `sysinfo` (MIT) and `windows` crates recorded in `THIRD-PARTY-NOTICES.md` |
| Rust crates for the product | 1, 2, 5, 7, 8 | Problem: some jobs (window, file watching, inflate and zstd, JSON) are not worth hand-writing and a bug there costs more than a dependency. Each crate's licence and RAM cost is checked when it is added and recorded in `THIRD-PARTY-NOTICES.md`; GPL crates are refused. Candidates: `eframe`/`egui`, `notify`, `flate2` or `miniz_oxide`, `zstd`, `serde`, `serde_json`, `winreg`. | Third-party | MIT/Apache-2.0 expected; confirmed per crate |
| GitHub Actions jobs (format, clippy, tests, RAM check) | 1 | Problem: nothing is checked unless something runs it on every PR. Without it, a PR that breaks the Linux build or adds RAM goes unnoticed until your PC. It runs on GitHub's machines, never on yours. | Third-party service (standard) | GitHub terms |

Not proposed: any injecting, hooking or memory-reading tool, and any tool that writes to the save, the game install or the Steam folders.

## 6. Tests

### 6.1 Automated
GitHub Actions on Ubuntu and Windows, fixtures only. Nothing touches the real save, the real install or the MHDB API.

- `repo_hygiene.rs` (step 1): no save, pack, dump or extracted-data file tracked; every source file has the MPL-2.0 notice; every commit in the PR range uses an allowed identity.
- `vdf.rs` (step 2): parses a fixture `libraryfolders.vdf` and `appmanifest_2246340.acf`; rejects malformed input without panicking.
- `discovery.rs` (step 2): over a fixture Steam tree with two libraries and two account folders, finds the install, the build ID, the one Wilds account and its save files; handles a missing library, no Wilds folder and several accounts.
- `datadir.rs` (steps 1, 2): refuses paths inside a fake Steam folder and a fake install; honours the override.
- fixture builder (step 3): builds a save whose expected values file matches what the builder was told to put in; output is deterministic.
- `container.rs`, `stream.rs`, `model.rs` (step 4): read every fixture save and match its expected values; a flipped byte fails the integrity check; truncated input errors cleanly; the active slot is picked by the `Active` field; hunter name and ID are absent from the summary.
- `index.rs`, `read.rs`, `paths.rs` (step 5): read a tiny synthetic pack with an encrypted table, a chunked entry, a deflate entry and a zstd entry; later packs override earlier ones; known hash vectors match.
- `exescan.rs` (step 5): finds the expected names in a synthetic exe and records its size and time.
- `validate.rs`, `lib.rs` of `forgeline-update` (step 6): a good staging swaps in; a missing file, a shrunk category, a file that does not open or an older build blocks; a failure partway leaves `current/` unchanged; the old data is kept as `previous/`.
- `watch.rs` (step 7): a write to a temp file triggers one callback after the quiet period; several quick writes trigger one; no events means no wake-ups.
- `engine` (step 7): a changed fixture save produces a new `Snapshot`; a corrupt save keeps the old summary and sets an error; a change during a recompute runs once more.
- `cache.rs` (step 7): round trip; a corrupt or old-schema file is ignored.
- `view.rs` (step 8): the view function draws every `Snapshot` state (empty, loading, ok, error, patch notice, update rejected) without panicking, using egui's headless context.
- RAM check (steps 1, 8): headless check on Windows stays under the limit in section 3.4.
- A smoke test that starts the binary with `--headless-check` against a fixture and exits cleanly.

### 6.2 Pass or fail criteria

**Step 1, workspace and CI**
- Pass: CI is green on Ubuntu and Windows for format, clippy (warnings denied), tests and the RAM check.
- Pass: the hygiene test fails on a deliberately added `.bin` and on a missing licence notice, and passes on the clean tree.
- Pass: `CHANGELOG.md`, `THIRD-PARTY-NOTICES.md` and `docs/ram-log.md` exist.

**Step 2, discovery**
- Pass: on the maintainer's PC the report names the one Steam account folder with Wilds saves, the library holding the game, the install and the build ID, with no configuration.
- Pass: it lists the mod markers as facts and opens none of those files.
- Pass: it never writes inside the Steam folders, the install or `win64_save`.

**Step 3, fixture builder**
- Pass: every fixture is a few KB, invented, deterministic, and listed in `fixtures/README.md`.
- Pass: the builder runs offline and writes only into the repo checkout.

**Step 4, save reader**
- Pass: every fixture reads back with the expected values.
- Pass: on a copy of the maintainer's save, the report shows 3 slots with the active one marked, and counts that the maintainer confirms match the game.
- Pass: reading the real save takes under 1 s and its peak memory is reported.
- Partial: the container reads but the `Active` field does not pick a slot; the window will show all slots (section 2).
- Fail: decryption or the walk errors on the real save; fix before step 7.

**Step 5, pack reader and exe scan**
- Pass: on the real install the pack index builds in under 10 s and the needed-files list resolves every path (a missing one is named).
- Pass: the type-name table covers the classes the save needs, rebuilt in under 30 s.
- Pass: only original `re_*.pak` files are opened; peak memory is reported.
- Fail: a needed file does not resolve, or the table rebuild takes minutes; fix before step 6.

**Step 6, update flow**
- Pass: running the update twice on the real install gives `Replaced` and then `Unchanged`.
- Pass: a deliberately broken staging (the test removes a file) is rejected, with a named finding, and `current/` is unchanged.
- Pass: the update takes under 5 minutes and under 1 GB peak, at below-normal priority, and the game shows no stutter by eye.
- Pass: nothing is written outside the app's data directory.

**Step 7, engine**
- Pass: the cache shows within 100 ms of start on the PC.
- Pass: a save by the game updates the snapshot with no polling (the watcher wakes the thread; no timer).
- Pass: recompute after a save change is under 2 s (target) on the PC with Wilds running.
- Pass: a half-written or locked save never produces a wrong view; the old summary stays and the error is visible.

**Step 8, window**
- Pass: the window opens on the maintainer's PC, dark theme, readable at a glance on a second monitor.
- Pass: it shows the paths, build ID, slot table, counts, data status and the update button, and every number traces to a source in the report.
- Pass: the update button runs the update, shows progress, shows a rejection or failure visibly, and keeps the last good data.
- Pass: a game patch (build ID differs from the manifest) shows the "data may be out of date" notice and does not recommend pressing update (no confidence measure exists yet).
- Pass: hunter name and ID are not shown.

**Step 9, measurement**
- Pass: the numbers are median of 3 runs at the standard benchmark spot with Wilds running, written to `research.md`.
- Pass: cold start to last view under 2 s (target under 1 s); idle CPU and GPU about 0; save change to updated view under 5 s (target under 2 s).
- Pass: the game's median and 99th percentile frame times stay within 1 ms of the control run during a recompute and during an update.
- Pass: private bytes at most 140 MB and working set at most 80 MB, or the overage is reported to the maintainer with options.
- Fail: any hard limit broken; fix in a follow-up PR before step 10, and consider the Slint test.

**Step 10, budget review**
- Pass: each proposed change in section 3.4 has the maintainer's decision recorded in the change log of `performance-budget.md`.

**Step 11, close-out**
- Pass: `PLAN.md` reflects the results, lists the Phase 3 inputs and the motion-value plan.
- Pass: `README.md` text, badges, roadmap and screenshot match the phase result; the screenshot hides any personal data.
- Pass: the banner and its alt text name Phase 3 as in progress.
- Pass: `CHANGELOG.md` lists 0.2.0; the plan's time log is filled in.
- Pass: every answered question is flagged for the coordinator to move.

### 6.3 Test round checklist
Queued, one at a time. Steps 2 to 6 use `forgeline-check`; the window exists from step 8. Expected values are from the maintainer's save as last checked (Phase 1 step 4: 3 character slots, 1 in use, about 14 light bowguns and 7 armor pieces created, one talisman, about 221 item types in the box). Counts may have changed since; the round says so if they differ.

**Step 1**
- [ ] CI is green on Ubuntu and Windows.
- [ ] The hygiene test fails when a `.bin` file is added (checked in the cloud).

**Step 2**
- [ ] `forgeline-check discover` names one Steam account folder, one library, the Wilds install and the build ID (25921695 when last seen; a newer number only means the game was patched).
- [ ] The report lists the mod markers found in the install (`dinput8.dll`, `reframework`, `pak_mods`) and says it did not read them.
- [ ] No file in the Steam folders, the install or `win64_save` has a new modified time afterwards.

**Step 4**
- [ ] With your save, the report shows 3 slots, 1 in use and marked active, about 14 light bowguns and 7 armor pieces created, one talisman candidate, about 221 item types.
- [ ] The report prints no hunter name or ID.
- [ ] The read takes under 1 second.

**Step 5**
- [ ] The pack report shows 32 packs opened (the original `re_*.pak` only), about 349,000 files, and every needed path resolved.
- [ ] The type-name table report shows the classes the save needs (about 185 of 187) and finishes in under 30 seconds.
- [ ] `pak_mods` was not opened.

**Step 6**
- [ ] The first update says `Replaced`, the second says `Unchanged`, and each finishes in under 5 minutes.
- [ ] The broken-staging test says `Rejected` with the missing file's name and `current` is unchanged.
- [ ] Wilds showed no stutter by eye during the update.
- [ ] Your save and the game folders have no new modified times.

**Step 7**
- [ ] The engine report shows the snapshot appearing within 100 ms from the cache.
- [ ] After the game saves, the snapshot changes within 2 seconds, with no polling.

**Step 8**
- [ ] The window opens, dark, readable on the second monitor.
- [ ] It shows the Steam account folder, the install, the build ID, the active slot and the same counts as step 4.
- [ ] Pressing Update shows progress and ends with `Replaced` or `Unchanged`; the window stays responsive.
- [ ] Hunter name and ID appear nowhere.

**Step 9**
- [ ] The benchmark table shows cold start to last view, private bytes, working set, idle CPU and GPU, and save-change recompute time, median of 3 runs, with Wilds running at the Scarlet Forest base camp.
- [ ] The frame-time report (run as administrator) shows the game's median, 99th percentile and worst frame time during idle, a recompute and an update.
- [ ] `docs/ram-log.md` has the first real reading.
- [ ] The report says whether the `Active` field picked your active slot.

**Every PC step**
- [ ] No save, dump or extracted Capcom data appears in git or in the PR.
- [ ] The PC thread reports it did not write to the save, the game install or the Steam folders, and that it stopped the dev build when done.

Steps 1, 3, 10 and 11 are cloud-only and checked in the cloud.

## 7. Risks and open questions

- **Private bytes pass 140 MB with real data (step 9).** Notice: the RAM rule or the step 9 reading. Fallbacks, in order: trim features of the window; try a lighter egui build (fewer features, smaller fonts, no image loaders); check whether the allocator holds on to freed buffers; test Slint as the Phase 1 gap and settle its licence terms first (open question 6); finally ask the maintainer whether to loosen the budget.
- **Parse peaks stay in the app's memory after a recompute.** Notice: private bytes grow after the first save change. Fallback: large buffers are freed back to Windows immediately in most cases; if not, run the parse in a short-lived worker process like the update.
- **Pack handling for the newest patches (chunk table) breaks on a future patch.** Notice: step 5 report, or an update finding. Fallback: the update is rejected visibly and the last good data stays.
- **The restricted exe rebuild is still slow or misses classes.** Notice: step 5 timing or the class count below 185. Fallback: widen the hash set from the save copy, then fall back to route B for the missing classes.
- **A needed file moves or is renamed in a patch.** Notice: the validation gate names it. Fallback: update the needed-files list by hand; the app keeps the last good data.
- **The `Active` field does not pick the right slot.** Notice: step 9. Fallback: show all slots and let the user pick, saved in `settings.json`.
- **Reading the save while the game writes.** Notice: size or modified time changes during a read, or inflate fails. Fallback: retry after the quiet period; keep the previous summary and show the error.
- **The watcher misses events (network drive, antivirus, cloud sync).** Notice: the save changed but the view did not. Fallback: a "Refresh" button, and a single check of the save's modified time on window focus (not a timer).
- **Windows hosted runners cannot show a window.** The CI RAM check is headless by design, so the GL cost is only seen on the PC. Fallback: the PC reading on every app-changing PR.
- **Dependency licence or size surprise.** Notice: step 1 and each later crate add. Fallback: hand-write the small ones (`vdf.rs` already is).
- **Personal data leaks into a fixture, cache or screenshot.** Notice: the hygiene test, the review of every PR, and the checklist. Fallback: the PR does not merge; if it was pushed, the coordinator tells the maintainer at once.
- **Questions for the maintainer** are the decisions listed in the box at the top. New questions go to `docs/open-questions.md` via the coordinator.

## 8. Versioning and changelog
- The workspace version is `0.2.0-dev` from step 1 and becomes `0.2.0` at close-out (step 11).
- Steps do not bump the version. A fix after 0.2.0 bumps `0.2.1`.
- `CHANGELOG.md` gets one line per merged step under `Unreleased`, in Added, Changed or Fixed:
  - Step 1: Added the workspace, CI and hygiene checks.
  - Step 2: Added automatic discovery of the Steam account, library, game install, build ID and save files.
  - Step 3: Added the fixture save builder and synthetic fixtures.
  - Step 4: Added the save reader and the active slot summary.
  - Step 5: Added the pack reader, the exe scan and the type-name table.
  - Step 6: Added the update flow with staging, structural validation and keep-last-good.
  - Step 7: Added the save watcher, background recompute and the view cache.
  - Step 8: Added the first window (the debug data viewer).
  - Step 9: Added the benchmark harness and the first RAM log.
  - Step 10: Changed the performance budget.
  - Step 11: Released 0.2.0.
- The release and the `v0.2.0` tag are the maintainer's action (box at the top).

## 9. Review log
| Date | Reviewer | Edit or decision |
|------|----------|------------------|
| 2026-10-27 | The maintainer | Asked for the Phase 1 retro items to be folded in (steps 1, 9, 10, RAM rule, CI), and for releases, tags and branch deletions to be listed up front. |
| 2026-10-27 | The maintainer | Asked for an hours estimate in every plan. Added per step and in total (about 13 h, range 10 to 18 h), calibrated on Phase 1's 2 h 06 min. |
| 2026-10-27 | The maintainer | The README banner still said Phase 1: this PR sets it to Phase 2, and the close-out step and the template now check the README text, badges, roadmap and banner every phase. |
| 2026-10-27 | The maintainer | Motion values: all four routes approved (effective raw meanwhile, deep game-file search early in Phase 3 for all 14 weapons, training-dummy measurement in Phase 5, credited community data labelled unverified as a fallback). Recorded in `PLAN.md` and `closed-questions.md`. |

## 10. Time log
| Event | Date and time |
|-------|---------------|
| Phase approved by the maintainer | |
| Step 1 PR merged | |
| Step 2 PR merged | |
| Step 3 PR merged | |
| Step 4 PR merged | |
| Step 5 PR merged | |
| Step 6 PR merged | |
| Step 7 PR merged | |
| Step 8 PR merged | |
| Step 9 PR merged | |
| Step 10 PR merged | |
| Step 11 PR merged | |
| Phase finished | |

Elapsed from approval to finish: `<hours>`. Compare with the estimates in section 4.
