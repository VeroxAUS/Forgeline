# Fixtures

Small invented files that tests read instead of the maintainer's real save or the real game install. Everything here is synthetic: it was built by `tools/fixture-builder` from a fake Steam account (number 4242, Steam ID `76561197960269970`) and invented round numbers. There is no real hunter name or ID and no Capcom data in them, and nothing was derived from the maintainer's save.

The repo hygiene test fails on any file under `fixtures/` whose name is not listed in this file as synthetic, and on any save, pack or dump anywhere else.

## `fixtures/saves/`

Regenerate with `cargo run -p fixture-builder` (offline, deterministic; a test fails if the committed files differ from what the builder makes now). Each fixture is a `.bin` in the real container format (header, block encryption keyed by the Steam ID, raw-deflate wrapper, class stream, tail checksum) with an `.expected.json` next to it that holds what a correct reader must find. The key for both is the fake Steam ID above.

| File | Contents |
|------|----------|
| `one-slot.bin` | synthetic: slot 1 holds data and is the active one; slots 2 and 3 are empty (flag `Active` 0, sentinel stamp -1.0) |
| `one-slot.expected.json` | synthetic: expected counts for `one-slot.bin` and the strings that must never appear in a summary |
| `two-slots.bin` | synthetic: slots 1 and 3 hold data and slot 3 is active, so "first slot with data" is the wrong answer |
| `two-slots.expected.json` | synthetic: expected counts for `two-slots.bin` and the strings that must never appear in a summary |

Why the files are about 99 KB and not a few KB: the container stores the data in blocks of at least 16 KiB, and the game's block-size sequence gives the first block six units (96 KiB). The padded tail of a block is real format behaviour, so it is kept.

What is real and what is invented:
- **Real, from the Phase 1 notes:** the container steps, the class stream rules (4-byte alignment, size-prefixed scalars, 16-byte structs aligned to 16, packed scalar arrays, class arrays that follow the array header directly, with no marker and no hashes, as seen in the real save), murmur3 name hashes, and the class names `app.savedata.cUserSaveData`, `cUserSaveParam`, `cEquipParam`, `cEquipWork`, `cItemParam`, `cItemWork`, `cBossReportParam`. Confirmed on the maintainer's PC (2026-10-28, by name hash): the root field `_Data` and the slot field `Active`.
- **Invented (marked with a star here because the real save may differ):** `_LastSaveStamp*`, `_BasicData*` with `_CharName*`, `_HunterId*`, `_PlayTime*`, `_Created*` and `_Anchor*` (a 16-byte struct, to exercise alignment), `Category_Gender*` and `EquipIdFixed*` (the real names are unknown; the reader takes the first two fields of an equipment entry by position), `cBasicParam*`, `cFlagParam*`, `cEnemyReportParam*`, the number of fields per class. The block headers hold only the first 8 bytes of the key/IV proof (the rest is zero) and a zero per-block checksum, because the reader ignores both.
- Item, equipment and boss numbers are made up (for example a charm entry with category 21, id 57, mirrors the shape seen in the real save but means nothing).

Hunter names and IDs in the fixtures (`Fixture Hunter`, `FIXT-0001-NOPE` and so on) are there so tests can check they never reach a summary.

## `fixtures/steam/`, `fixtures/packs/`

Not added yet; later steps add small invented Steam trees and a synthetic pack as needed.
