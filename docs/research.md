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

### Follow-up (2026-10-27): precise Mandarin recipe, second read of MHWeditor
Why: the step 3 PC spike found a valid flags value and block layout but no valid deflate wrapper after ~1.5M key/IV variants. I re-read their decrypt path line by line (src/save/mod.rs, src/save/crypto/mandarin.rs, the SplitMix helper and game table). Status **reported**; the test vector at the end is computed by me from their published constants and gives a cheap check before any AES work. Credit as above.

**Corrections to section 2 and 3 above (these are probably what the spike missed):**
1. **The generator feeds its output back as its state.** Their helper adds the golden constant to the state, mixes the new value and *returns the mix*; every call site then assigns the returned value *back into the state*. So the chain is `x(n+1) = mix(x(n) + 0x9E3779B97F4A7C15)`, not the textbook SplitMix64 where the additive counter keeps running and only the output is mixed. `mix(z)`: `z = (z ^ z>>30) * 0xBF58476D1CE4E5B9; z = (z ^ z>>27) * 0x94D049BB133111EB; return z ^ z>>31`, all wrapping 64-bit.
2. **The size loop advances over every potential block, not only real ones.** So the state entering the key/IV phase is `x(N)`, with `N = ceil(decrypted_len / 0x4000)`.
3. **The deflate wrapper starts at the very first decrypted byte** (not at some later aligned offset).

**Recipe, in order (all integers little-endian):**
1. Read the file. Check magic `DSSS`, version 2, flags `0x18`. Header is 12 bytes; the encrypted region starts at file offset **0x10** (next 16-byte boundary) and runs to `file_len - 4` (the last 4 bytes are the murmur3 checksum and are not part of it). The encrypted region therefore still contains the tail (0x80-byte blob, then the 8-byte length).
2. `decrypted_len` = u64 at `file_len - 12`.
3. `key` = the full 64-bit Steam ID (the 0x0110000100000000-based one) with **no transform** for Wilds. Only its bitwise NOT, `nk = !key`, is used.
4. `N = (decrypted_len >> 14) + (decrypted_len & 0x3FFF != 0)`. Start `x = 0x7A36955255266CED` (used raw, the first size comes from the seed itself). For `i` in `0..N`: `size[i] = (x & 7) + 1`; then `x = step(x)` (x becomes the mixed output). After the loop `x = x(N)`. For this seed the sizes begin `6, 6, 2, 2, 5, 1, 1, 5` (checked by me).
5. Real block count: loop `i` from 0; count the block; if `remaining <= size[i]*0x4000` stop, else `remaining -= size[i]*0x4000` (start `remaining = decrypted_len`). Do not touch `x` here.
6. `x = x.wrapping_add(nk)`. (`rands` and the 0xBFACF76C3F96 seed are encrypt-side only; ignore both.)
7. For each real block in order (block `i` sits on disk at the running sum of `0x210 + size*0x4000` of the earlier blocks, starting at file offset 0x10; on disk it is `0x210` header bytes, then `size*0x4000` data bytes, back to back):
   a. Do 16 calls `x = step(x)`. For call `j`: `aes_key[j] = low byte of x`, `iv[j] = bits 8..15 of x` (both from the same output, key and IV drawn together, interleaved by j, not 16 key bytes then 16 IV bytes).
   b. Do 0x210 more calls `x = step(x)`; byte `k` of the header is XORed with the low byte of call `k`. (Only needed to keep `x` in step; the header content is not otherwise needed.)
   c. Decrypt the **whole** `size*0x4000` data bytes (even the padded tail of the last block) with AES-128 in **OFB** mode (standard full-block feedback: keystream block 1 = AES(iv), block 2 = AES(block 1), ...), using `aes_key` and `iv`. The cipher is created fresh per block; no counter, no carry-over between blocks.
   d. Keep the first `min(size*0x4000, remaining)` bytes; append to the output; `remaining -= size*0x4000` (their code lets it wrap on the last block; ignore).
8. Block validation (hard error in their code): header offset 0x200 holds a u64, compare with CityHash64 of the kept bytes of that block (after the header XOR of step 7b). They use the Rust crate `cityhasher` 0.1.0, function `hash::<u64>` (plain data bytes, no seed). Which CityHash revision that crate implements is not recorded; treat a mismatch with a correct key as an implementation-variant problem. The ElGamal proof in the same header is skipped by them on failure (log only); we skip it entirely.
9. Output length is `decrypted_len`. **Deflate wrapper at output byte 0**: u64 (compressed size + 0x10), u32 (value 1), u32 compressed size, u64 decompressed size, then a **raw** deflate stream (no zlib header, no gzip header) of the stated compressed size starting at output byte `0x18`. Only the stated compressed-size bytes are inflated.
10. The class stream (section 4) begins at the first inflated byte. In their buffer it sits at file position 0x10, so "alignment relative to the whole buffer" and "relative to the stream start" are identical (0x10 is a multiple of 16). This closes the open note in section 3.
11. File checksum: murmur3 x86 32-bit (the Rust `murmur3` crate's `murmur3_32`, i.e. standard MurmurHash3_x86_32) over all bytes except the last 4, seed `0xFFFFFFFF`, compared to the final u32. Mismatch is only a warning.

**Cheap oracle for the generator before any AES (my own derivation from their key-search routine):** their brute-force step proves a candidate ID using only the first 8 bytes of block 1's header. Recipe: take `x(N)`, add `!id`, do 16 `step` calls (the key/IV draws, discarded), then 8 more calls; the low bytes of those 8 outputs are the header mask. File bytes `0x10..0x18` XOR that mask must equal the constant **`bb39c81702d5495b`** (first 8 bytes of the little-endian 64-byte form of R^20 mod P, where P and R are the 32-byte ElGamal constants in their crypto util; I computed it, they hold it as expected value). If this matches, the Steam ID, the feedback chain and `N` are right and failures after that can only be AES/OFB details. If it does not, check the ID (full 64-bit value, not the 32-bit account number), then `N`, then the feedback in point 1. Cost: a few dozen hash steps.

**Things I confirmed unchanged:** block sizes `size = (x & 7) + 1` in 0x4000 units; header 0x210; flags; blob 0x80 then length then checksum at the tail; murmur3 seed; type codes of the class stream.
**Still not verified on a real save:** all of the above is reported, not reproduced. The CityHash revision is unknown. The ID passed to the cipher: their reader takes it from the caller and, for the brute force, tries the full 64-bit range starting at 0x0110000100000000.

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

Question: can the game `.exe` on disk give us the names behind the save's hashes (route A), well enough to parse a save into named classes?

**Result: route A passes for the save. The "sample of game data files" half of the criterion is not done (it needs step 5's pack reader).** Chosen route: **A**. B, C and D were not needed for the save. Status of everything below: **verified** on the maintainer's PC (real exe, a copy of the real save).

Method: our own throwaway Rust tool (`spikes/typedump`, PR #5, never merged; ran against the real files). It reads `MonsterHunterWilds.exe` on disk (read-only, about 2 s for 546 MB, never launched), hashes every printable-ASCII string of 3 to 200 characters with murmur3 (seed `0xFFFFFFFF`) and matches the hashes found in the save. The install is modded (`dinput8.dll`, `reframework`, `pak_mods`, a Fluffy Mod Manager marker); the `.exe` on disk was used as is. Outputs stay in `E:\mhbis\out` (not in git).

### Save decryption (confirms the step 2 follow-up recipe)
- The recipe in the step 2 follow-up works as written: the oracle value matched on the first try, then the whole save decrypted and inflated (header size equals stream size; about 16.5 MB inflated from a 198 KB file). Flags `0x18` confirmed, two real blocks of 6 x 0x4000 bytes. My first attempt (from the first version of the notes) failed only because of the generator feedback chain and step count the follow-up corrected.
- CityHash was not needed: a successful inflate plus a clean full-stream walk is the check. CityHash revision still unknown.

### Corrections to the class stream format (step 2 section 4), found by walking the real save
1. **16-byte structs are 16-aligned:** after the size prefix (16), the 16 bytes start on a 16-byte boundary (relative to the stream start, also a 16-byte boundary in the file). Without this the walk desyncs in the first slot.
2. **Arrays of scalars and structs are packed:** after the array header (member type, member size, count, kind) the elements are `count x member size` raw bytes with no per-element size prefix (arrays of 16-byte structs are 16-aligned once, at the start). Arrays of strings and of classes are as in section 4.
3. Everything else in section 4 held: field and class layout, hashes, the `0xFFEEFFEE` optional marker, top-level entries starting with a 32-bit hash.

### Correction from the reader's first PC round (2026-10-28, step 4 of Phase 2)
The structure words printed from the real decrypted stream (16,458,692 bytes; no text printed) start: `dbe3f199` (entry hash), `3` (field count), `827fccdc` (class hash of `app.savedata.cUserSaveData`), `695a3627` (hash of `_Data`), `-1` (array), `0x11` (member type: class), `8` (member size), `3` (count), `1` (kind: classes), then immediately `0x76` (118 fields) and `d32f821c` (class hash of the first slot). The first slot's first field is `69dbae94` = `Active` (type 4, size 1), and the second field is a string.
- **A class array with `kind` 1 has no marker and no per-element hashes in this save.** The elements follow the array header directly. The `0xFFEEFFEE` marker followed by hashes may still occur; the reader takes the hashes only when the marker is there. My first reader assumed one hash per element even without the marker and lost sync at the second field.
- The class array's member size field holds `8`, not 0.
- The reference walker of the Phase 1 spike also accepts scalar types with any value size (aligned only when the size is 1, 2, 4 or 8) and arrays of kind 0 whose members are strings, classes or arrays; the reader now does the same.
- Confirmed by name hash: `Active` (field of the slot) and `_Data` (field of the root) are real names, not invented.

### Equipment box entries (Phase 2 step 4, checked against the maintainer's game screenshots, 2026-10-28)
`_Equip._EquipBox` is an array of 2,400 `cEquipWork`; 59 are in use in the maintainer's save. The first field of an unused entry is 141. Field order and types (names unknown, all hashes): `u8`, `s32`, `s32`, `s32`, `u32`, `u64` (zero in all 59), `u8`, `s16[4]`, `s16[3]`, `s32`, `u8`, `u8`. The array holds equipped pieces too, not only spare ones.
- **First field (category):** 13 weapon (16 entries), 0 and 1 armour (25 and 15 entries), 21 talisman (1), 37 mantle (2).
- **Weapon:** second field = weapon type (0 to 13 in the save's order, which matches the game's weapon menu order 0, 3, 1, 2, 4 to 10, 13, 12, 11 in the box; 13 is the light bowgun, confirmed). Third field = the weapon within its type, 0 for the 13 other base weapons. The four `s16[4]` values hold bowgun customisation (bowguns only) and the `s16[3]` array holds decoration ids.
- **Armour:** category 0 is the "A" look (green dot in the game's box) and category 1 the "B" look (orange diamond): same stats, other appearance. Second field = series, third field = piece number 0 to 4. The starter series 2, 3 and 4 exist in both looks (15 + 15 entries); every other piece is look A only. The maintainer's waist screenshot (9 pieces: 6 rarity-1 starters in three A/B pairs, then rarity 2, 3 and 4) matches piece number 3 exactly, and 40 armour entries across the five piece numbers (7, 8, 8, 9, 8) match the maintainer's count of 40 from screenshots (an earlier count of 48 was a miscount).
- **Talisman and mantle:** category 21 (second field 57 here) and 37 (second field 0 and 10, two mantles, one equipped, both rarity 4).
- **Light bowgun count (screenshots, unverified):** the maintainer sees 4 light bowguns in the Customize Bowgun view (Hope Cannon I, Dosha Faithbreaker I, Hope Rifle III, Dosha Faithbreaker II) but 3 in the equipment box (rarity 3, 2 and 4), which matches the 3 type-13 box entries (third field 2, 16 and 17). The maintainer reports that Customize Bowgun lists every bowgun ever crafted, even ones no longer owned, so Hope Cannon I is most likely upgraded away and the reader's 3 owned light bowguns is right. Not verified in the save: no crafted-history list was identified, and a craft/upgrade save diff would settle it. Whether a base weapon counts as always available is a product decision for the maintainer and is not made here. The meaning of the third field for weapons is not confirmed. Meanings of the other numeric fields (third field of armour 30, 70 and 150) are not known.
- **Craft test (PC, 2026-10-28 05:16 UTC, Wilds running, reader at b56dccb):** crafting one Hunter's Rifle I took the light bowgun count from 3 to 4 and the equipment entries in use from 59 to 60. The new entry has category 13, type 13, third field 41 and the first small array `[16, 4]`. The weapon flag bitsets changed, and the item box changed by the craft costs (including an item id 809 that was not there before). So a crafted weapon adds a box entry, and the third field is a per-weapon id (41 for this one). Which weapon id 41 is, what `[16, 4]` means, and what the flag bitsets record are not verified. This also fits the maintainer's view that Customize Bowgun lists every crafted bowgun while the box holds only owned ones, but it does not prove it.
- Other arrays in the same class (counts only): 140 equipment sets, 500 palico entries, 1,000 Artian parts, a list of 29 item ids with counts (probably decorations), 2,400-bit and smaller bit lists.


With those fixes the whole 16.5 MB stream walks to the exact end with no error: **187 distinct classes**.

### Names recovered from the exe
- Exe scan: 2,252,128 distinct strings. About 67,700 are field-like (`_Xxx`) and about 17,500 are short class-like names (`cXxx`). Full dotted class names are **not** stored as strings, only namespace pieces (6,648 `app.`/`via.` strings), so class names are rebuilt as namespace + "." + short name and hashed.
- **Field names: 1,215 of 1,221 distinct (class, field) pairs resolved (99.5%)**, from exe strings alone.
- **Class names: 185 of 187 resolved** (183 as `app.savedata.cXxx`, one `app.cEmProp`, plus the top-level `via.storage.saveService.SaveFileDetail`). Unresolved: one class with 2 fields repeated 14,346 times, and one other. Caveat: murmur3 is 32 bits and about 5.5 billion candidate names were tried, so chance collisions exist (some hashes also match odd `via.*` names). Rule used: accept only `app.<namespace>.c<Upper...>`; 2 hashes still had two plausible names and need a field-name check before use.
- The classes step 3 asked for are all present and named (entries counted in the save copy):

| Need | Class | Entries | Fields |
|------|-------|---------|--------|
| save root | `app.savedata.cUserSaveData` | 1 | 3 |
| character slot | `app.savedata.cUserSaveParam` | 3 | 118 |
| equipment | `app.savedata.cEquipParam` | 3 | 58 |
| items | `app.savedata.cItemParam` | 3 | 16 |
| enemy report | `app.savedata.cBossReportParam` | 300 | 9 |
| story | `app.savedata.cStoryParam` | 3 | 8 |
| mission | `app.savedata.cMissionParam` | present | not recorded |
| workshop | `app.savedata.cLargeWorkshopParam` | 3 | 5 |

  Also present and likely useful later: `cEquipWork` (7,245 entries), `cEquipMyset`, `cItemWork` (10,254), `cCharmEquipped`, `cArtianPartsWork`, `cArtianCreateCountParam`, `cWeaponFlagParam`, `cArmorFlagParam`, `cOtEquipWork`. Three slots are visible (`cUserSaveParam` x3). Which slot is active and what the ids mean is step 4.
- Enums: the stream stores enum values as numbers with a width. Enum type and value names were not examined (they would need other string tables or the game data files).

### Not done / open
- **Sample of game data files not parsed** (second half of the step 3 pass criterion). It needs the pack reader of step 5; carry it there.
- A class name is only "plausible" until its fields match the exe field names for that class (not checked per class).
- Only names are recovered; field meaning and which enum a number belongs to are not. A small hand-written schema (route B) will probably be needed for meanings, built on these names.
- Unchanged from step 2: CityHash revision, the 16-byte "mandrake" structs, bitset fields.
- Route C was skipped (licence not cleared); route D (hashing exe strings) turned out to be the same method as route A here.

### Decision this supports
Route A is enough to name the save. Phase 2's reader can use the same method at update time (scan the user's exe; nothing shipped). The exe scan takes about 2 s. The class-name rebuild took minutes in the throwaway tool and must be restricted (namespace `app.savedata`, names starting with `c`) to run quickly.

## Nexus monitoring

Question: can a scheduled cloud session read new comments (Posts tab) and bug reports (Bugs tab) on our Nexus mod page? Researched 2026-10-27, read-only, no account made, nothing posted.

### Findings
- **Official API does not expose comments or bug reports (as far as found).** The public REST v1 API covers mods, files, games and a user's tracked/endorsed mods ([API announcement](https://www.nexusmods.com/news/13921), 2019; docs at https://api-docs.nexusmods.com/). A Nexus forum suggestion dated May 2026 asks for "basic read-only access to recent comments, open bug reports, status, and timestamps" to be added ([thread](https://forums.nexusmods.com/topic/13537574-comments-and-bug-reports-being-available-in-the-api)), which implies it is not there yet. **Unverified:** I could not read the GraphQL v2 schema (https://graphql.nexusmods.com/ refused my fetcher) or the current api-docs page (JS-rendered), so a comments query in v2 is not fully ruled out.
- **Auth:** a Personal API key (account settings, API Access) works for REST; SSO needs approval from Nexus. Personal keys are tolerated for private/testing use only ([API Acceptable Use Policy](https://help.nexusmods.com/article/114-api-acceptable-use-policy)). Requests must carry honest `Application-Name` / `Application-Version` headers. Storing other users' keys, or using them without the user starting the action, is prohibited.
- **Rate limits:** 20,000 requests per 24 h, then 500 per hour; remaining count is in response headers ([rate limiting](https://help.nexusmods.com/article/105-api-acceptable-use-policy)). Far more than a daily check needs.
- **Web pages, logged out:** my fetcher got the mod page shell with "JavaScript is required" and a log-in prompt, no comments, no Cloudflare-style challenge. The mod listing was also empty without JavaScript. So plain HTTP gives no comments; a real browser is needed. I did not test a Wilds page that is live (the one I tried was hidden by its author). The cloud container's network policy blocks `nexusmods.com` outright (proxy 403 on CONNECT), so a scheduled cloud session cannot reach it unless that host is allowed in the environment's network settings.
- **robots.txt** (https://www.nexusmods.com/robots.txt): `Allow: /` for general crawlers, `Disallow: /ajax/` for everyone, and ClaudeBot, GPTBot, CCBot and similar AI crawlers are disallowed from the whole site; content signal `ai-train=no, use=reference`. A Claude-run reader of the web pages is therefore against the site's stated wishes.
- **Terms:** the Acceptable Use Policy bans en-masse scraping for rehosting and says Nexus may act on any use it deems detrimental. I did not locate a general Terms of Service clause on automated reading of pages; treat unverified.
- **Third-party tools exist** that read comments/bugs by driving a headless browser (for example [nexusmods-mcp-server](https://github.com/UberMorgott/nexusmods-mcp-server)); they are unofficial and break when the site changes. Not used or run by us.
- **Notifications:** Nexus turned off comment notifications for *tracked* mods ([Notifications update](https://www.nexusmods.com/news/14927)). I found nothing confirming email or RSS for comments or bug reports on a mod you author. **Open:** check the author's own Notification settings once the page exists.

### Options, safest first
1. **The maintainer checks the Nexus site / the maintainer's own notification settings and forwards anything.** Zero risk, no secrets. Ask the maintainer to look at the author notification settings once the page is live.
2. **Official API, if comments/bugs appear in it (or v2 has them):** a scheduled session with a Personal API key stored as an environment secret, one call a day, honest headers. Needs the key from the maintainer and the host allowed in the cloud network policy.
3. **Reading the web pages with a headless browser:** works technically, but fragile, against robots.txt for AI crawlers, and needs a browser plus network access. Not recommended.

### Open
- Does GraphQL v2 expose comments or bugs? Does Nexus send authors an email for new comments or bug reports?

## Step 4: save structure, ID matching and talismans (2026-10-28)

Question: where do the owned gear, items and talismans sit in the save, and do the save's fixed IDs match the game data files?

**Result: partial.** The save's structure is mapped (**verified** on a copy of the maintainer's save from 2026-10-20, on the maintainer's PC). ID matching is only shown **by range and by internal consistency**, not by name: names live in text files that nothing has read yet. Talismans: one candidate location found, **not confirmed**. Code: throwaway `spikes/savecheck` on branch `spike/step4-savecheck` (never merged; ran against the real save copy; the live save folder is refused by the code). Output below is structure and counts only; no hunter data was printed or saved to git.

### Slots and equipment box (verified)
- The save root (`app.savedata.cUserSaveData`) holds `_Data`, an array of **3** character slots (`cUserSaveParam`, 118 fields each). Field names resolve from exe strings (all but 7 of the slot's fields).
- Only slot 0 has data. Slots 1 and 2 carry a sentinel (a float field equal to -1.0) and empty content. A one-byte field is 1 in slot 0 and 0 in the others, so it is a **candidate** for the "active" marker; with only one used slot it cannot be told apart from "has data". **Open:** how the game marks the active slot when two slots are used; the app should fall back to "the slot that has data, else the most recently written".
- Active slot's equipment box: `_Equip._EquipBox` is an array of **2,400** `cEquipWork` entries; **59** are in use in this save. Other arrays: item box `_Item._BoxItem` 900 entries (221 in use), pouch 25, `_ArtianPartsBox` 1,000, `_EquipMySet` 140, `_OtEquipBox` 500 (palico gear).
- Entry layout (`cEquipWork`, field meanings inferred from values, not verified): one byte category (141 means empty), a 32-bit id, a 32-bit second field, 4 and 3 small 16-bit slot arrays (decoration and augment slots, -1 means empty), and a few more small numbers.

### ID checks (partial)
| Category | What was compared | Result | Strength |
|----------|-------------------|--------|----------|
| Armor | Save armor ids per slot (2, 3, 4, 21, 25, 33, 34) against the set bits of the save's own "armor created" flags | exact match for helmet (4 of 4), body (5 of 5), female helmet (3 of 3) | Internal consistency only: the box id is the bit index. |
| Armor | Same ids against `ArmorSeriesData.user.3` ids (1 to 194) | 7 of 7 inside the range | Weak (range check). |
| Weapons | Light bowgun ids in the box (0 to 13) against the light bowgun data file | 14 of 14 present in a column of the data rows | Weak: a small dense id range. |
| Items | Item box ids (221 distinct, highest 805) against the item data file | 221 of 221 present in the id column (809 row-aligned values, 784 distinct, highest 838) | Weak: the column covers about 93% of the id space, so a random id would match most of the time. |
| Decorations | | **not located** in the save | A decoration is probably an item-box entry or one of the equipment slot ids; not checked. |
| By name | | **not done** | Needs the text (name) files from the packs. |

Match rate against the 99% criterion therefore cannot be called: every id checked exists in the data, but the check cannot catch a wrong mapping. To close it: read the name text files, then compare names for the ids in the box (about 80 ids on this save). The save-id to data-id question stays **open**, with the working hypothesis "save id = the id column of the matching data row" (true for every category tested, no counter-example).

### Bitsets (verified)
Weapon created/checked flags exist per weapon type (14 types, 128 bits each), armor created/checked flags per gender and slot (256 bits), `_ItemFoundFlag` and `_ItemCheckedFlag` (2,016 bits; 264 set here). The weapon type order in the save is: Long sword, Short sword, Twin sword, Tachi, Hammer, Whistle, Lance, Gun lance, Slash axe, Charge axe, Rod, Bow, Heavy bowgun, Light bowgun (field names from the exe). In the box, category 13 lines up with the light bowgun (13 entries, ids 0 to 12 plus 13 three times with different second values). Created bits for the light bowgun are 0, 1, 2, 16, 17.

### Talismans (candidate, not confirmed)
- Charm data is in the game files (`Charm.user.3`: 167 rows, 48 bytes each, plus `AmuletData` 187 rows and random talisman tables, see step 5).
- In the save, the equipment box has a category 21 entry (one, id 57) and a category 37 pair (ids 0 and 10) that are neither weapons nor armor. Category 21 id 57 is inside the charm id range, so it is a **candidate** owned charm; its skills are not decoded. Random (mystery) talismans, if they exist in this game, would need their skill rolls stored per entry; the entries here carry no field that looks like that, so **how a random talisman is stored is open**. The fallback in the plan stays valid (user enters talismans by hand) until a save with several talismans is checked.

### Not done / open
- Names (needed for the name-based ID check).
- Which field marks the active slot when more than one slot is used.
- Decoration ownership; talisman decoding; whether this older save (2026-10-20) represents the current one.
- Kills and story flags to unlocked monsters (`_EnemyReport` has 300 `cBossReportParam`; not examined), mission completion.
- Unresolved field names stay as hashes: 7 of 118 slot fields and all 12 `cEquipWork` fields (so the entry layout above is inferred from values alone).

### Test round: checked against the maintainer's real game (2026-10-27)
The maintainer checked the reader's counts against the maintainer's real game, as of the 2026-10-20 save copy (passed 14:05 UTC, 2026-10-27):

| Item | Reader found | The maintainer |
|------|--------------|--------|
| Character slots in use | 1 of 3 (slot 1) | right |
| Light bowguns crafted | about 14 | right |
| Armor pieces crafted | 7 | right |
| Item types in the item box | about 221 | hard to tell, seems correct (approximate) |
| Talismans owned | 1 | right |
| Nothing from the save in git | confirmed by the thread | right |

This confirms the box counts and the ID ranges against the maintainer's game. It does not close the by-name check, which moves to Phase 3 (needs the text files). The one talisman entry is confirmed as a count only; its skills are still undecoded.

### Decision this supports
The reader can already list gear, items and crafted flags by id from the save. The final ID mapping needs a text-file reader (names) and a data-row decoder; both belong to Phase 3.

## Step 5: extraction test, pack reader and data files (2026-10-28)

Question: can we read the game's own packs, find the files for skills, weapons, armor, decorations, charms, Artian tables, hitzones, drops and motion values, and say which of them are decoded?

**Result: partial.** Pack reading and file naming work (**verified** on the maintainer's PC, real install, original packs only). Most categories have located data files; none is decoded into named fields yet (that needs a schema for the data files, see "Not done"). Motion values are not located. Code: throwaway `spikes/extract` on branch `spike/step5-extract` (never merged; ran against the real files). Nothing extracted is in git; outputs are in `E:\mhbis\out` only.

### Pack reader (verified)
- Install found through Steam's `libraryfolders.vdf` (library on `E:`), build ID 25921695. Only the original `re_*.pak` files are read: 32 packs (base, 15 patches, `sub_000` and its patches, the DLC pack). The mod pak in `pak_mods` is never opened. Mod markers present: `dinput8.dll`, `reframework`, `pak_mods`, a Fluffy Mod Manager marker file.
- Format facts used (credit: [eigeen/ree-pak-rs](https://github.com/eigeen/ree-pak-rs), MIT licence, read for format facts; our reader is our own code, but it holds the one 129-byte table key constant from that project, so credit it in THIRD-PARTY-NOTICES.md if this ever leaves the spike branch):
  - Header 16 bytes: `KPKA`, version 4.1, feature flags, file count, hash. All 32 paks are version 4.1.
  - Feature `0x08`: the entry table (48 bytes per file: two path hashes, offset, compressed size, uncompressed size, attributes, checksum) is encrypted. A 128-byte key follows the table; it is raised to the power 65537 modulo a fixed 1024-bit number, and the first 32 bytes of the result drive a byte-wise XOR over the table. It worked first time (entry counts and offsets came out sane).
  - Feature `0x20`: a chunk table follows the key. It is used by the two newest patches (014 and 015). Files whose attribute bit 24 is set store a chunk index instead of an offset; each chunk is stored or zstd-compressed.
  - Compression in the low 4 bits of attributes: 0 none, 1 deflate, 2 zstd.
- The 32 tables hold 349,348 distinct files after later patches override earlier ones. Reading an entry reads only that entry. The full index (tables only, no file data) builds in a few seconds.

### Naming files without a file list (verified)
- Entries are keyed by two hashes of the path (murmur3, seed `0xFFFFFFFF`, over the UTF-16 path, once lower-cased and once upper-cased). There are no names in the packs.
- No third-party file list was used. Path strings were found in the exe (only 9,399 strings contain a slash) and then inside the game's own scene, prefab and user data files: each path string found in a file is hashed with the `natives/stm/` prefix and tried with every file-version suffix from 0 to 70 until both hashes match an entry.
- Starting from one exe string (`Boot/Master.scn`, which resolves to `natives/stm/Boot/Master.scn.21`), one crawl over files up to 8 MB named **113,869 of 349,348 files**, including 48,646 `.user.3` data files. File contents were only held in memory. The rest (textures, meshes, motions, sound) are not referenced from those files by path and stay unnamed; they are not needed.

### Categories (status: located and sized, not decoded)
Counts are RSZ instances measured in the files; they include wrapper instances and are not item counts. Each file is a `USR` container with an `RSZ` object block whose instances carry a type ID. The type IDs are not hashes of any string in the exe (tested against the 2.25 M exe strings and our rebuilt class names), so they cannot be named by route A.

| Category | Status | Files (under `natives/stm/GameDesign/`) | Evidence |
|----------|--------|-----------------------------------------|----------|
| Skills | found, not decoded | `Common/Equip/SkillData.user.3`, `SkillCommonData.user.3`, `HunterActiveSkillData.user.3` | 443 and 218 row-type instances. Per-level numbers not confirmed to be in these files. |
| Weapons | found, not decoded | `Common/Weapon/<Type>.user.3` for 14 types, plus `<Type>Tree`, `<Type>Recipe`, `Outer<Type>Data`, `RodInsectData`, bowgun customisation | Names: Bow, ChargeAxe, GunLance, Hammer, HeavyBowgun, Lance, LightBowgun, LongSword, Tachi, Rod, ShortSword, SlashAxe, TwinSword, Whistle. The mapping to in-game weapon names (for example Tachi to Long Sword) is **inferred**, not verified. Light bowgun file: 81 weapon rows, 2,108 instances. |
| Armor | found, not decoded | `Common/Equip/ArmorData`, `ArmorSeriesData`, `ArmorRecipeData`, `ArmorUpgradeData`, `ArmorUpgradeRecipeData`, `ArmorSpUpgradeCostData`, `OuterArmorData` | `ArmorData`: 719 piece rows (9,349 instances); 195 series rows. |
| Decorations | found, not decoded | `Common/Equip/AccessoryData`, `AccessoryJudgeData`, `AccessoryRankJudgeData` | 361 rows. |
| Charms | found, not decoded | `Common/Equip/Charm`, `AmuletData`, `AmuletRecipeData`, `RandomAmulet*Table`, `Maka*Table` | 168 charm rows, 187 amulet rows, 181 recipe rows. Random talisman lot tables present. |
| Artian | found, not decoded | `Common/Equip/ArtianGrindingData`, `ArtianSkillGroupData`, `Em0078_ArtianDatas`, `Facility/ArtianBonusData`, `ArtianBonusCategoryData` | 49, 296, 4, 53 and 6 instances. Gogma Artian tables not separately identified (**open**). |
| Items | found, not decoded | `Common/Item/itemData`, `ItemRecipe`, `FixItems` | 782 item rows. |
| Hitzones | located, **unconfirmed** | `Common/Enemy/EM<id>_<variant>_0.user.3`, 88 files (one per monster form) | A monster file holds 29 repeated groups of 4 related rows (262 to 292 instances), consistent with per-part damage data but not decoded, so not confirmed to be hitzones. |
| Drops | partly located | `Common/Enemy/EnemyReport*RewardData`, `EnemyRewardAddData`, `EnemyReportBossMaterialDispData` | Report-style reward data (716 instances in the part reward file). `EnemyRewardDataSetting.user.3` (18.7 KB) has no RSZ block in its first 400 bytes, so it is a different layout, not read. Real per-monster drop tables not confirmed. |
| Motion values | **not located** | | Only action shell parameters and collider data were found under `Player/ActionData/Wp*`; no table that looks like motion values. The open question stays open. |

Rows are readable as plain numbers: the 228-byte `ArmorSpUpgradeCostData` shows four rows of small integer triples directly in the data section, so decoding needs a layout per class, not decompression or decryption.

### Not done / open
- **No category is decoded into named fields.** Next needed: a hand-written schema (route B) for the classes we need, checked against a cited source (MHDB as cross-check only). Field names for the data files cannot come from the exe by hashing (type IDs are not string hashes); how the exe stores its RSZ type table is **open**.
- The 67% of files with no name (mostly art, sound, motion) were not examined.
- `EquipDataSetting.user.3` and `EnemyRewardDataSetting.user.3` have no RSZ block near the start; **open**.
- Names (item, skill, monster) are in separate text message files, not looked for yet.
- Plausibility against MHDB counts is not done (needs decoding first). The weapon rows per type and the 719 armor pieces look plausible only by size.
- The survey ran on the patched install (build 25921695); a new game patch will change counts.

### Decision this supports
All needed categories except motion values (and real drop tables) exist in the game files and can be read from the user's own install at update time, without a third-party file list. Phase 3 must plan a hand-written schema per data class; motion values and drop tables need a separate investigation or a cited source.

## Licence options and donations (2026-10-27)

Read from public pages on 2026-10-27. Not legal advice. Full write-up with a comparison table: https://claude.ai/artifact/ShBiYjTMa8jjQU4VNrrSK1

- **Goal:** others may modify parts, fork for themselves and contribute; nobody may republish the whole app or a rewrite as their own without permission.
- **OSI open source cannot do that.** MIT, Apache-2.0, MPL-2.0, GPL and AGPL all allow republishing the whole codebase. Only source-available licences can restrict it.
- **Best off-the-shelf match: PolyForm Shield 1.0.0.** Any purpose is permitted except providing a product that competes with the software; the text says goods "compete even when provided free of charge". Personal forks and patches are allowed. PolyForm Noncommercial still allows free noncommercial redistribution, so it does not fit. Commons Clause only blocks selling. CC BY-NC-SA is not meant for software. A custom "ask permission" licence fits exactly but is untested.
- **Nexus Mods:** the [File Submission Guidelines](https://help.nexusmods.com/article/28-file-submission-guidelines) do not require open source. AI-written code needs the AI-Generated Content tag. Per the [donation guidelines](https://www.nexusmods.com/rust/news/14698), links to Patreon or Ko-fi are fine unless the author monetises modding (files, updates or support for donations, early access, stretch goals, donor requests); then the links must go.
- **Dependencies:** egui, Tauri, Electron, pulldown-cmark and its dependencies are MIT or Apache-2.0 (see above); fine under any licence if notices are kept. Slint's GPL option and any GPL code would force open source.
- **AI authorship:** a US Copyright Office report (Jan 2025, read via a [law-firm summary](https://datamatters.sidley.com/2025/02/07/u-s-copyright-office-issues-report-on-artificial-intelligence-and-copyrightability/)) says purely prompt-generated material is not copyrightable, while human selection, arrangement and modification can be. A licence cannot protect uncopyrightable parts.
- **Donations:** a voluntary tip link with no perks looks lowest risk. Paywalls, early access and perks conflict with Nexus rules. Capcom's fan-content guidelines (Nov 2025, press report only) bar commercial use of fan content generally and say nothing found about tools or donations. Anthropic's consumer terms assign Outputs to the user "if any" and show no restriction found on tips.
- **Not verified:** Capcom's own page, Nexus staff view on PolyForm Shield, full Patreon, Ko-fi and Anthropic commercial terms, the Copyright Office report text.

## Step 6: UI toolkit comparison beside Wilds (2026-10-28)

Question: which UI toolkit (pure Rust, Tauri or Electron) is light enough to run beside Wilds, measured against [performance-budget.md](performance-budget.md)?

**Result: pass for pure Rust.** The pure-Rust candidate (egui with the glow backend) meets the budget on every measured row; Tauri and Electron start fast and cost nothing at idle, but both exceed the RAM hard limit. None of the three measurably disturbed the game's frame times. Decision rule from the plan ("pure Rust if it meets the budget") picks **pure Rust**. Status: **verified** on the maintainer's PC, with Wilds running. Code: throwaway `spikes/uibench` (three apps), `spikes/sampler` (process sampler and the Windows frame-time module) on branch `spike/step6-uibench`, never merged.

### Method
- Three equivalent minimal apps, dark theme, a table of 300 rows, and a background job (one thread of pure CPU work for 5 s, started by the sampler's timer, lowest thread priority where the toolkit allows). egui 0.37 (glow backend, `eframe`), Tauri 2 (system WebView2, static page), Electron 33.2.1 (job in a worker thread; Node cannot lower one thread's priority, so the Electron job runs at normal priority, which makes its job-window result slightly harsher than the others).
- Our own sampler (`sysinfo` for the whole process tree: working set, private bytes, CPU time; Windows performance counters for GPU use per process). Cold start is spawn to the app's own "ready" line, printed after its first frame (the real app's "last build on screen" is not built yet, so this is an upper bound on the window cost only). Idle window 60 s after a 3 s settle, then the 5 s job, then 5 s after. 3 runs per app, median reported. Each app was killed after its run.
- Game frame times: our own Windows-only module (`frametime`, in `spikes/sampler`). It listens to the Windows DXGI present events (ETW, the source PresentMon uses; needs administrator rights while running) and records the gap between one process's presents, matched by process id. It never opens or reads the game process; the events carry only a process id and a timestamp. It ran as administrator, started by the maintainer. 9 runs of ~115 s plus a 90 s control with the game alone. The clock is the trace's QPC clock.
- **Standard benchmark spot** (use it for every later round): Scarlet Forest main base camp, standing outside the tent, camera still, solo / offline session. The game ran at about 85 to 90 frames per second (median frame time 11 ms) and was not limited by our test.
- Baseline without Wilds ran earlier the same day (3 runs per app; one Electron run lost its ready signal, so Electron has 2 valid no-Wilds runs).

### Results (median of 3 runs, with Wilds running)
| Measure | Budget (target / hard limit) | egui (pure Rust) | Tauri | Electron |
|---------|------------------------------|------------------|-------|----------|
| Cold start to first window | under 1 s / 2 s | 200 ms | 423 ms | 453 ms |
| Idle RAM, working set | under 80 MB / 150 MB | 73.5 MB | 350 MB | 302 MB |
| Idle RAM, private bytes | (same row) | 127 MB | 167 MB | 206 MB |
| Idle CPU (60 s) | under 0.5% / 1% | 0.00% | 0.00% | 0.00% |
| Idle GPU | under 1% | 0.00% | 0.00% | 0.00% |
| Job, CPU of one core | at most 2 cores | 98% | 98% | 97% |
| Game median / p99 frame time, idle window | no visible stutter | 11.13 / 12.53 ms | 11.16 / 12.64 ms | 11.15 / 12.52 ms |
| Same, during the job | | 11.07 / 12.63 ms | 11.21 / 12.45 ms | 11.23 / 12.65 ms |
| Same, after the job | | 11.16 / 12.63 ms | 11.17 / 12.61 ms | 11.14 / 12.97 ms |

- Control (game alone, 90 s): median 11.04 ms, p99 12.91 ms, worst 90.4 ms (one hitch of the game's own, 8,078 frames).
- Per-run job minus idle change in the game's median frame time: egui +0.06, -0.06, +0.13 ms; Tauri +0.05, +0.05, -0.03 ms; Electron -0.07, +0.08, -0.14 ms. All within the run-to-run noise (the game's median moved by about 1 ms between the first and last runs, whatever was running beside it). The worst frame during the job window (13 to 24 ms) stayed within about 1 ms of the worst frame in the idle windows (21 to 24 ms).
- Both web-based options keep their shared WebView or Chromium pages in the "working set"; the private-bytes figure is the fairer one, and it is still over the hard limit for both (167 MB and 206 MB against 150 MB). egui is under 150 MB private and under 80 MB working set.

### Verdict against the budget
- **egui (pure Rust): meets every measured row**, with the caveat that this is an empty window with 300 rows. Its private figure (127 MB) leaves about 23 MB of headroom to the hard limit before we add data; the working-set row (73.5 MB) leaves 6 MB to the 80 MB target. Watch RAM in Phase 2.
- **Tauri and Electron fail the RAM hard limit** at idle, before any app data. They pass the speed, CPU, GPU and stutter rows.
- No candidate stuttered the game. This test is one thread of CPU work; a real recompute that uses more cores, or memory-heavy extraction, must be re-measured in Phase 2 and Phase 3.

### Not done / open
- Slint was not tested (licence conditions in the step 1 entry still open).
- No "last build on screen" time (needs the real app); no save-change-to-build time; no window hidden/minimised test; GPU rows are zero because nothing was redrawn while idle, which is the desired result but means they say nothing about a busy UI.
- Electron job priority could not be lowered (see method), and Electron has only 2 valid no-Wilds runs.
- DXGI present events reflect the game's present calls, not what reaches the screen (frame generation or overlays could change that).
- The ETW crate used is `ferrisetw` 1.2.0 (MIT OR Apache-2.0, step 1); `sysinfo`, `windows`, `windows-sys`, `eframe`, `tauri` and `electron` were used as listed in step 1. Record them in THIRD-PARTY-NOTICES.md if any reaches the product.

### Decision this supports
Build the Phase 2 interface in pure Rust (egui with glow is the measured baseline). Re-run `spikes/sampler` against the Phase 2 window to check RAM against the 80 MB target as soon as it holds real data.

### Decision (2026-10-27)
The maintainer chose **MPL-2.0**, forks allowed with "based on Forgeline" credit, no competing-fork ban, no contributor paperwork, and a Ko-fi tip jar on GitHub only (none on Nexus). Compatibility with our dependencies: MIT, Apache-2.0, Unlicense and MIT OR Apache-2.0 crates can be combined with MPL-2.0 code; only the notices must be kept (`THIRD-PARTY-NOTICES.md`). GPL code (including Slint's GPL option and mhdb-wilds-data's toolkit) is not covered by that and stays out. MPL-2.0 asks for a short notice at the top of each source file (its Exhibit A); add it when code files appear.

## Overlay target detection: how HunterPie reads Wilds, and read-only options (2026-10-27)

Question: can an in-game shopping-list overlay learn the current target monster automatically, and how would a read-only approach look? Research only; nothing built, run or downloaded. Status: **reported** unless noted. Readable page: the Artifact report published with this entry (link in the PR).

### HunterPie (https://github.com/HunterPie/HunterPie)
- Its own [overview](https://docs.hunterpie.com/posts/hunterpie-an-in-depth-overview/) describes two methods: memory scanning (per-game map files of addresses and offsets, pointer paths from static memory, background thread at a user-set interval, a circuit-breaker for failing scans) and injection of a native library for data that is not in memory (remote thread plus `LoadLibraryW`, a local TCP link, function detours). The article uses injection for Rise damage numbers.
- **Not confirmed:** whether the Wilds integration injects; whether it uses `ReadProcessMemory` by name; patch turnaround; Wilds window-mode needs. The Wilds source was not readable (fetch blocked by robots rules). The World FAQ says the overlay needs DirectX 12 or borderless windowed.
- Wilds support: "partial" in [v2.12.0](https://docs.hunterpie.com/posts/update-v2.12.0) (monster, abnormalities, clock, damage meter widgets).
- Licence: v2 Apache-2.0 (repo label); archived v1 (Haato3o/HunterPie) MIT. Permissive, so we may learn from it with credit; copying code needs a THIRD-PARTY-NOTICES.md entry. Offset tables are not to be used without checking their licence.

### Anti-cheat and Capcom
- Denuvo Anti-Tamper from launch ([Deltia's Gaming, Feb 2025](https://deltiasgaming.com/?p=155498)); a July 2025 [Automaton report](https://automaton-media.com/en/news/monster-hunter-wilds-players-think-anti-cheat-system-is-the-cause-of-repeated-crashes/) adds a Capcom proprietary anti-cheat and community reports (player claims about per-frame checks are unconfirmed). No Enigma mention found.
- Capcom, [April 2025](https://www.gfinityesports.com/article/capcom-issues-warning-over-modding-in-monster-hunter-wilds): action against mods that alter game data in online quests; suspensions or permanent bans possible. Nothing about external or read-only tools. [MeinMMO, March 2025](https://mein-mmo.de/monster-hunter-wilds-mods-permanenter-account-bann/): reported bans were for a mod unlocking paid DLC; no official mod statement.
- No report of a ban for a read-only reader or overlay found in Wilds, Rise or World. This is absence of evidence only.

### Nexus
- [File Submission Guidelines](https://help.nexusmods.com/article/28-file-submission-guidelines) have no rule on reading game memory. Relevant: nothing disruptive to persistent multiplayer, truthful claims, no internet-connected executables unless essential. HunterPie is listed for Wilds on Nexus (seen in search results only; page not readable).

### Save file
- Step 4 above found `cMissionParam`, `cStoryParam`, `cBossReportParam`; nobody has looked for an accepted, tracked or in-progress quest. **Open.** Capcom's April 2025 advice says autosave only runs after the results screen, which suggests accepting a quest does not write the save (inference). Test: decode a save copy before and after accepting a quest (needs the save-diff tool the maintainer has not yet approved).

### Screen reading
- [windows-capture](https://docs.rs/windows-capture) (MIT) wraps Windows Graphics Capture and DXGI duplication; no benchmark numbers. `Windows.Media.Ocr.OcrEngine` is Windows 10+, documented for UWP; desktop use unconfirmed; needs an installed OCR language. CPU cost **unmeasured**. HUD position, language, window mode (exclusive fullscreen) and the capture border are unknowns.

### Recommendation and safeguards
- Ladder by risk: save test, then screen-reading prototype, then external read-only memory reading only if both fail.
- Memory safeguards if ever approved: opt-in with a warning; `PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ` only; `ReadProcessMemory` only (no injection, hooks, debugger attach or thread suspension); own separate overlay window, no renderer hooks; enabled only for a game build with a known offset map; validate every read against known monster ids with a circuit breaker; slow, only during a quest, low priority; offset map is local data. **Unsolved:** how we would find addresses without debugger-style tools.

### Architecture note: one app, two halves (2026-10-27, maintainer's proposal, my assessment)
- Isolation helps against **breakage** (a patch breaks only the reader; the desktop app and save-based overlay keep working). It narrows **ban exposure** to users who opt in, but does not reduce the risk for them: detection sees the machine, not our module layout.
- Strongest form: a separate small executable the overlay starts only when live reading is enabled, so the main download holds no code that opens the game process. A new executable is a new tool and needs the maintainer's yes; Nexus's stance on an optional reader file is unchecked.
- Boundary: a "target provider" interface returning game build, quest-active flag, monster id or none, status (off, unsupported build, ok, failed) and timestamp. The reader knows nothing about saves or shopping lists. Transport: overlay launches the reader as a child process and reads an anonymous pipe, so there is no listener (fits the project rule). Screen reading and memory reading are interchangeable providers; a "none" provider (top shopping-list items) is the fallback.
- **Decision (2026-10-27):** the maintainer adopted the two-halves split with live reading (`forgeline-live`) opt-in and strictly read-only, and approved changing the no-memory-reading rule for it.
