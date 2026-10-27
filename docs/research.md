# Research log

Spike and research results go here. Check this file before researching anything, so nothing is looked up twice.

Each entry: date, question, finding, source (cited), and status (**verified** = seen ourselves, **reported** = from a source, unchecked, **open**). Never fill gaps from memory.

## MHWeditor licence (2026-10-27)

Question: may we reuse anything from MHWeditor (https://github.com/this-nerd/MHWeditor)?

Finding (**verified** by reading the repository on 2026-10-27):
- No LICENSE file.
- None of its five Rust package files (Cargo.toml) has a licence field.
- The README has no licence section, only a disclaimer: it modifies proprietary game data, is used at your own risk, and the author is not responsible for banned accounts or lost saves.

Reading: with no licence, normal copyright applies. We have no permission to copy, modify or redistribute it. It is treated as all rights reserved.

Consequences:
- No MHWeditor code, type data files (`rszmhwilds.json`, `enumsmhwilds.json`) or close rewrites enter this repo.
- One thread may study it to learn facts about the save format and write them here in its own words, with credit. A separate thread writes our reader from those notes only, without opening MHWeditor.

Not yet checked (**open**):
- Whether the type data assets carry a different licence from the code.
- Its code appears to derive partly from another author's tools (possibly c-ola's). The upstream licence has not been checked.

## Save format facts already known

Findings from the design conversation (2026-10-27), source: a spike using MHWeditor on a copy of a save. Full detail is in [PLAN.md](PLAN.md); keep any further notes here in own words.
- **Verified:** the save is encrypted with the Steam ID. The 64-bit Steam ID is the folder number under `Steam\userdata` plus `0x0110000100000000`.

## Phase 1 step 1: licence review (2026-10-27)

Method: read each project's GitHub page and crates.io entry on 2026-10-27 (page snapshots, so a file not shown in a listing could still exist). Status is **verified** where we read the page ourselves, **open** where it needs the author or a lawyer. This is not legal advice.

### Verdicts

| # | Item | Verdict | Evidence |
|---|------|---------|----------|
| 1 | MHWeditor type data (`rszmhwilds.json`, `enumsmhwilds.json`) | **May not use** | No LICENSE file, no licence label, no credits section; README only says "This tool modifies proprietary game data. Use at your own risk." Its page says "forked from kvasszn/ree-save-editor". The data files have no separate licence that we could find. All rights reserved. |
| 2 | Upstream of MHWeditor (the "c-ola" guess in the earlier entry is **not confirmed**) | **May not use** (code and data) | The page names its upstream as [kvasszn/ree-save-editor](https://github.com/kvasszn/ree-save-editor), not c-ola. That repo also shows no LICENSE file and no licence label. Its README says it uses texture/file-reading code from [wwylele/mhrice](https://github.com/wwylele/mhrice) (dual Apache-2.0 / MIT) and "Uses praydog's emulation dumper for the rsz files." The dumper's project, [praydog/REFramework](https://github.com/praydog/REFramework), is MIT, but its licence page says nothing about the dumped type data, and the JSON's own origin is not stated. **Open:** whether the type dump can be reused needs the authors (kvasszn, praydog). Until then it stays out. |
| 3 | [LartTyler/mhdb-wilds-data](https://github.com/LartTyler/mhdb-wilds-data) | **Code: may not use in the repo for now (GPL-3.0). Data: open** | Page lists "GPL-3.0 license" with a LICENSE file. README says the repo is the MHDB Wilds Project's "primary data source" and that merged data files are in `/output/merged`. The page does not say whether those output files are covered by the GPL, and the files are converted Capcom game data (Capcom's copyright, not the toolkit author's to license). Its tools README credits REMSG_Converter (dtlnor), ree-pak-gui (eigeen) and MHWS-Editor (Synthlight) with no licence stated on that page. **Open:** ask the author whether the output data has its own terms. |
| 4 | MHDB data / API (`wilds.mhdb.io`) | **May use as cross-check only, nothing shipped** | Docs page, read in full, has no licence, terms or attribution statement; the end of the page only links to mhdb-wilds-data without giving its licence. Page says it uses "data from the game files". Consistent with existing rules: never auto-fetched, never shipped, labelled last resort. **Open:** no terms are published, so ask the MHDB maintainers before any bundled or last-resort use. |
| 5 | Does a Nexus release force open source? | **No. Nexus does not; the GPL could** | See below. |

### Does a Nexus release force open source?

- **Our network use:** the app's update button reads the user's local game files and makes no network call. Only the MHDB last-resort fallback and route C (community dump) would connect, so the Nexus internet rule below applies only if one of those is used.
- **Nexus itself: no.** [File Submission Guidelines](https://help.nexusmods.com/article/28-file-submission-guidelines) do not mention open source, GPL or source code in general. Reading: nothing in them requires a closed or open licence for ordinary uploads.
- **Nexus conditions that matter to us:**
  - Executables that connect to the internet are "prohibited unless that connection is crucial to the mod or utility", and tools that need internet access "must be discussed with staff" with "the source code" provided. Our update button is user-triggered; auto-update does not qualify. **Open:** whether a user-pressed update fetch is allowed; it is a question for Nexus staff, and route C (downloading a community dump) falls under it.
  - Copyrighted material, including game files, needs the owner's permission; crediting is "no substitute". This matches our rule that no extracted Capcom data ships.
  - Files are virus-scanned; AI-generated code must carry the "AI-Generated Content" tag. **Open:** whether this project's code needs that tag (likely yes, since Claude writes it).
- **GPL-3.0 could force it.** If a GPL-3.0 component (for example mhdb-wilds-data code) is copied into or linked with what we distribute, the GPL requires the whole combined work to be offered under GPL-3.0 with source. If no GPL code is distributed, nothing forces open source.
- **Conclusion:** with our own code, MIT/Apache crates only, and no MHWeditor, kvasszn or mhdb-wilds-data code or data, a closed Nexus release is not forced open by any licence found. Open source becomes forced only if we choose Slint under its GPL option or take GPL code.

### Crates and tools named in the plan

| Item | Licence (crates.io / repo, 2026-10-27) | Verdict |
|------|----------------------------------------|---------|
| [ferrisetw](https://crates.io/crates/ferrisetw) 1.2.0 (ETW events for the frame-time module) | MIT OR Apache-2.0 | May use; record in THIRD-PARTY-NOTICES.md. Spike tool only, never shipped. |
| [egui](https://crates.io/crates/egui) 0.37.0 | MIT OR Apache-2.0 | May use. (Only egui itself checked; eframe and its dependencies not.) |
| [Slint](https://crates.io/crates/slint) 1.18.1 | GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0 | May use **with conditions**: the GPL option forces open source; the royalty-free option has its own terms (attribution, and we did not read its text). **Open:** read the royalty-free terms before choosing Slint for a closed release. |
| [Tauri](https://crates.io/crates/tauri) 2.12.5 | Apache-2.0 OR MIT | May use. Bundles a system WebView; not checked. |
| [Electron](https://github.com/electron/electron/blob/main/LICENSE) | MIT ("Copyright (c) Electron contributors / GitHub Inc."; wording is the standard MIT text) | May use. It bundles Chromium with its own notices; not checked. |
| Rust toolchain, cargo | Not fetched this round | **Open** (plan says MIT/Apache-2.0, "to confirm"). Not confirmed here. |

Every dependency of any shipped crate still needs its own check at Phase 2 (for example `cargo-deny` style listing in CI); only top-level crates were read here.

### Decisions this supports
- `THIRD-PARTY-NOTICES.md` is created at Phase 2, when the first third-party crate enters the repo. No file is needed in Phase 1, which merges no code.
- Route C (community dump) is **not cleared**: the only dump we found (MHWeditor's) has no licence. Treat C as blocked unless an author grants permission.
- Choosing a UI toolkit does not need a licence change for egui or Tauri.

### Still open (for the author, Nexus staff or a lawyer)
- Reuse terms of the type dump behind MHWeditor (kvasszn, praydog).
- Whether mhdb-wilds-data's output files carry the GPL or other terms.
- MHDB API terms (none published on the docs page).
- Whether Nexus allows a user-pressed update download, and the AI-generated tag.
- Slint royalty-free terms.
- Licence of Rust toolchain.

## Step 2: save format, clean-room study of MHWeditor (2026-10-27)

Question: what does the Wilds save file look like on disk, how is it decrypted, and how are its contents described, so that a separate thread can write our own reader from these notes without opening MHWeditor?

Method: read the source of MHWeditor (https://github.com/this-nerd/MHWeditor, no licence, see the licence entry above) on 2026-10-27, mainly its save module (container, crypto, types), its type map and its dashboard code. Facts are restated in my own words. No code, no type data files and no real save data were copied or opened. The README says the project is a fork of kvasszn/ree-save-editor and the code looks derived from earlier REE tools; credit goes to those authors for the reverse engineering. Status: **reported** for everything below (read from their source, not yet reproduced by us). Spike 4 turns it into **verified** or not.

### 1. File layout, outermost first
- The file ends with a 4-byte little-endian checksum: murmur3 (32-bit, x86 variant) of every byte before it, seed `0xFFFFFFFF`. MHWeditor only warns on a mismatch.
- Header, all little-endian:
  - bytes 0 to 3: the ASCII magic `DSSS`.
  - bytes 4 to 7: version, must be 2.
  - bytes 8 to 11: flags.
- Flag bits: `0x1` Blowfish, `0x2` has an ID, `0x4` Citrus (a different cipher, not used by Wilds), `0x8` Deflate, `0x10` Mandarin (called Lime in another game).
- Wilds uses Deflate plus Mandarin only (flags `0x18`). Blowfish and ID fields are therefore absent; the header is then 12 bytes, and the encrypted section starts at the next 16-byte boundary (offset 0x10). **Open:** confirm the real flag value on our own save.
- Layers, outer to inner: Mandarin encryption, then a Deflate wrapper, then the class stream.

### 2. Mandarin decryption (the Steam ID layer)
- Key: the full 64-bit Steam ID, used as is for Wilds (no transformation). It is the folder number under `userdata` plus `0x0110000100000000`. The tool treats the key as the bitwise NOT of that number wherever it is mixed in.
- The file tail is, in order: the 0x80-byte RSA-style blob, then the decrypted length as a 64-bit number, then the 4-byte file checksum. So the decrypted length sits 12 bytes from the end. The encrypted blocks run from the aligned start up to the blob. Our reader can ignore the blob when decrypting.
- The encrypted data is cut into blocks. The block sizes come from a pseudo-random sequence:
  - generator: SplitMix64 (a public algorithm; add `0x9E3779B97F4A7C15`, then the usual two multiply and xor-shift steps with `0xBF58476D1CE4E5B9` and `0x94D049BB133111EB`, then a final xor-shift of 31).
  - starting state for the size sequence: a per-game constant. For Wilds it is `0x7A36955255266CED`. A second per-game constant, `0xBFACF76C3F96`, seeds a sequence that is only used on the encrypt side to build the RSA-style blob.
  - for each potential block, size = (state & 7) + 1, in units of 0x4000 bytes (so 16 KiB to 128 KiB), then the state advances once. The number of potential blocks is the decrypted length divided by 0x4000, rounded up.
  - real blocks are consumed in order until the remaining decrypted length fits in the current block; the last block is shorter in plain data but still stored at full block size.
- After the size sequence, the state is advanced by adding the NOT of the key (wrapping). From there every block does the same, in order:
  1. Draw 16 values from the generator. For each, the low byte is the next AES key byte and the second byte (bits 8 to 15) is the matching IV byte.
  2. The block on disk is a 0x210-byte header followed by the data (block size bytes). XOR the 0x210 header with 0x210 further generator outputs (low byte each). This keeps the generator in step even if you ignore the header.
  3. Decrypt the data with AES-128 in OFB mode using that key and IV. (OFB is symmetrical, so this is the same call as encrypting.)
  4. The header holds a key/IV proof (4 pairs of 64-byte numbers, 0x200 bytes), then a 64-bit checksum at offset 0x200, then 8 bytes of unknown purpose at 0x208. The checksum is CityHash64 of the decrypted data of the block (only the real data length for the last block). MHWeditor treats a failed proof as a log line only and a failed checksum as a hard error. So for our reader the checksum is the useful "is the Steam ID right" test, and the proof (an ElGamal-style check) can be skipped.
- Wrong Steam ID: the keystream is wrong, the checksum of the first block fails straight away. MHWeditor also has a brute-force mode that tests ID guesses on the first 8 header bytes only; we do not need it because we know the ID from the folder name.
- Cost: one pass of AES-OFB and CityHash over a few MB. Light enough to run on every save change. **Open:** save size not measured.

### 3. Deflate wrapper
- After decryption, at the 8-byte aligned position, comes a small header: a 64-bit value (compressed size plus 0x10), a 32-bit value that is always 1 in their writer, a 32-bit compressed size, then a 64-bit decompressed size.
- The data that follows is a raw deflate stream (no zlib header) of the stated compressed size. MHWeditor warns but continues if the output length differs from the stated size.
- Output is the class stream. Offsets inside it are measured as "stream position", with the header bytes before the payload kept in place in their buffer. If our reader works with the payload alone, field alignment (below) must be counted from the same origin as the game does. **Open:** alignment is relative to the start of the whole file buffer in MHWeditor; check on a real save whether it matches relative to the payload (matters because the payload starts at an offset which is a multiple of 8, so likely equivalent).

### 4. The class stream (self-describing, no type data needed to walk it)
Everything is little-endian and aligned to 4 bytes unless stated.
- The stream is a flat list of top-level entries until the end: a 32-bit hash, then one class. MHWeditor stops 7 bytes before the end of the buffer.
- Class: a 32-bit field count, a 32-bit class hash, then that many fields.
- Field: a 32-bit field-name hash, a 32-bit type code, then the value, then padding to a 4-byte boundary.
- Type codes: -1 array, 1 enum, 2 bool, 3 s8, 4 u8, 5 s16, 6 u16, 7 s32, 8 u32, 9 s64, 10 u64, 11 f32, 12 f64, 13 char8, 14 char16, 15 string, 16 struct (raw bytes), 17 class (a nested class). 0 means unknown and is an error.
- Value encoding:
  - Scalars and enums: align to 4, a 32-bit size in bytes (1, 2, 4 or 8), then the value aligned to its own size. Enum widths come from that size field, so enums can be read without type data.
  - Struct: same size prefix, then that many raw bytes. Known uses: 16-byte vectors, 4-byte colours, and a 16-byte pair of 64-bit numbers they call "mandrake" (value and multiplier, likely an obfuscated number; **open**).
  - String: align to 4, a 32-bit length in characters, then that many UTF-16 code units.
  - Class: just a nested class (count, hash, fields).
  - Array: align to 4, then member type code (32-bit), member size (32-bit), element count (32-bit), array kind (0 = values, 1 = classes). For class arrays an optional marker `0xFFEEFFEE` may follow, then one 32-bit class hash per element; if the marker is absent the 4 bytes are not consumed. Then the elements: values use the sized encoding (strings use the string encoding); classes are full class entries. Aligned to 4 at the end.
- Consequence: a reader can walk the whole save, and decode all numbers, with no type data at all. Type data is only needed to turn hashes into names and to know which fields mean what.

### 5. Hashes and type data
- Both field names and class names are hashed the same way: murmur3 (32-bit x86 variant), seed `0xFFFFFFFF`, over the plain name bytes (UTF-8, no terminator). Class names are full dotted names such as `app.something.cSomething`.
- The type data MHWeditor loads is a JSON file keyed by class-name hash. Each class has a CRC, a name, and a map of field-name hash to field info: name, original type name, a simple type label, array flag, size, alignment and a native flag. This is the same RSZ type dump format used by other REE tools; it is their data asset, not ours (do not use it, see licence entry). A separate enum file maps enum names to values.
- They also keep a map from murmur3 hash to string for every string found in the game executable. This supports our fallback A and D: hashes can be turned back into names by hashing every string found in the exe and matching. **Open:** whether the executable holds all class and field names; that is spike 3.
- Fields not found in the type data are still readable from the stream, as shown in section 4.

### 6. Where things live (only what they use; names come from their code and are **reported**)
- The top-level class of interest has a `_Data` array. One entry per character slot. Each slot has a `_BasicData` field (name, ids, play time and similar; hunter IDs live in here, so never print it).
- Inside a slot, fields they touch: `_Item` with `_BoxItem` and `_PouchItem` (item box and pouch), `_Quest` (mission completion), flag lists called `_ItemFoundFlag` and `_ItemCheckedFlag` with a `_Value`. Mission IDs are typed `app.MissionIDList.ID`.
- Some fields are bitsets with no type attached in the dump; MHWeditor maintains a hand-written file that tells it how to read them.
- The slot selection in the file name (`data001Slot.bin`) is outside this format. How "active slot" is chosen is not covered here. **Open.**

### 7. Writing (for reference only; we never write saves)
They re-encrypt by reversing every step above. We are read-only on the game, so none of this is needed.

### Gaps for spike 4
- **Not found in their code:** equipment, armour pieces, decorations, owned charms/talismans, weapon lists, monster kill counts and story flags. Their dashboard only deals with item box, pouch and missions. Locating these (open questions: save-ID to data-ID mapping, owned talismans, kills to unlocked monsters) needs our own look at a save copy with the class walker from section 4.
- The exact slot count, active slot marker and the top-level hashes need a real save.
- Encoded numbers ("mandrake") might guard some counters; unknown which.
- Block sizes, header length and the flag value above should be checked against a real save, since their code handles several games and I only followed the Wilds path.
- CityHash64 version: they use a Rust crate; the checksum must match the game's variant. If our crate gives a different value on a known-good block, try the other CityHash revisions (v1.0 or v1.1, or Google FarmHash `Hash64`) before suspecting the key.
- Licence of the type data asset and of the code they may derive from: still open (see licence entry).

## Step 3: type-data spike, route A (2026-10-28)

Question: can the game `.exe` on disk give us the names behind the save's hashes (route A)?

Method: our own throwaway Rust tool (`spikes/typedump`, branch `spike/step3-typedump`, never merged) read `MonsterHunterWilds.exe` from the Steam library found through `libraryfolders.vdf` (read-only, one pass, about 2 s), took every printable-ASCII string of 3 to 200 characters and hashed it with murmur3 (seed `0xFFFFFFFF`, as in step 2). The install is modded (`dinput8.dll`, `reframework`, `pak_mods`, Fluffy Mod Manager marker); the `.exe` is the one on disk and was not launched. Results stay in `E:\mhbis\out` (not in git). Code ran against the real exe and a copy of the real save.

Results:
- **Verified:** 2,252,128 distinct strings; the hashes of `_Data`, `_BasicData`, `_BoxItem`, `_PouchItem` and `_ItemFoundFlag` (field names from step 2) all appear in the exe's strings. So field names can be recovered by hashing exe strings (about 67,700 strings start with `_` and a capital).
- **Verified:** full dotted class names (for example `app.MissionIDList.ID`) are **not** stored as plain strings. Only namespace pieces (198 `app.` strings, e.g. `app.savedata`) and about 17,500 short class-like names (`cXxx`) exist. Class hashes would have to be rebuilt by combining namespace and short name and hashing; that is untested because the next point blocks it.
- **Blocked:** the save copy could not be decrypted from the step 2 notes. With flags `0x18` and layout confirmed (encrypted area is exactly two blocks of 6 x 0x4000 bytes plus two 0x210 headers, if block sizes use the generator output rather than the raw state), no key derivation I tried produced the expected deflate wrapper (about 1.5 million combinations of generator start, how the Steam ID is mixed in, skipped outputs, key/IV byte lanes and AES mode OFB/CTR/CFB). So no save hash was available to match, and the step 3 pass criterion (parse a save into named classes) is **not yet met**.

Status of route A: **partial** (names for fields recoverable, class names unconfirmed, no save parse). Not a fail of the exe approach itself; the blocker is step 2's crypto notes.

Needs from the coordinator: a clean-room study thread to re-read MHWeditor's key setup (how the Steam ID enters the generator and how the key and IV bytes are drawn) and write the answer in own words, or a licensed alternative. Then step 3 can finish (class-name reconstruction plus parse) in about one more thread.
