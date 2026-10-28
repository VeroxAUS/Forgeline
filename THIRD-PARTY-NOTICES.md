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

## tools/fixture-builder (test tool, not shipped in the app)

| Crate | Version | Licence | Use |
|---|---|---|---|
| aes | 0.8.4 | MIT OR Apache-2.0 | AES-128 block cipher, used in OFB mode to write the synthetic fixture saves |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | raw deflate compression for the fixture saves |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | dependency of miniz_oxide |
| cipher, crypto-common, inout | 0.4.4, 0.1.7, 0.1.4 | MIT OR Apache-2.0 | dependencies of aes |
| generic-array | 0.14.7 | MIT | dependency of cipher |
| typenum, version_check | 1.20.1, 0.9.5 | MIT OR Apache-2.0 | dependencies of generic-array |
| cpufeatures, libc, cfg-if | 0.2.17, 0.2.191, 1.0.5 | MIT OR Apache-2.0 | dependencies of aes |

Licences read from each crate's `Cargo.toml` (`cargo metadata`). No GPL crate is used. The container format the builder writes comes from our own notes in `docs/research.md`; no code from MHWeditor or any other unlicensed project is used.

## spikes-archive branch (reference only, not on `main`, not shipped)

Throwaway Phase 1 spike code. Its use of eigeen/ree-pak-rs is the same as in the `forgeline-pak` section below.

## crates/forgeline-save (the save reader, linked into the app)

| Crate | Licence | Use |
|---|---|---|
| aes, miniz_oxide | MIT OR Apache-2.0, MIT OR Zlib OR Apache-2.0 | AES-128 (OFB mode) to decrypt the save container, raw deflate to inflate it |

Their own dependencies are the ones listed for `tools/fixture-builder` above (same versions, same lockfile). The container and class-stream readers are written from our own notes in `docs/research.md`; no code from MHWeditor or any other unlicensed project is used.

## crates/forgeline-pak (the pack reader, linked into the app and the update process)

| Crate | Version | Licence | Use |
|---|---|---|---|
| num-bigint | 0.4.8 | MIT OR Apache-2.0 | the big-number power that turns the table key in a pack into the XOR key |
| num-integer, num-traits, autocfg | 0.1.47, 0.2.19, 1.5.1 | MIT OR Apache-2.0 | dependencies of num-bigint |
| zstd | 0.13.3 | MIT | unpacks zstd entries and chunks |
| zstd-safe, zstd-sys | 7.3.0, 2.1.2 | BSD-3-Clause | dependencies of zstd; zstd-sys builds the bundled zstd C library, used under its BSD-3-Clause licence (it is dual-licensed with GPL-2.0; the GPL option is not used) |
| cc, shlex, find-msvc-tools, jobserver, pkg-config, getrandom, r-efi | 1.10.0, 2.0.1, 0.1.15, 0.1.35, 0.3.34, 0.4.3, 6.0.0 | MIT OR Apache-2.0 (r-efi also lists LGPL-2.1-or-later as an option; the MIT/Apache option is used) | build-time dependencies of zstd-sys (compiling the C library) |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | raw deflate for deflate entries (already listed above) |

Licences read from each crate's `Cargo.toml` (`cargo metadata`). No GPL crate is used.

### eigeen/ree-pak-rs (MIT)

The pack format facts (header, encrypted entry table, chunk table) and the 129-byte table modulus constant in `crates/forgeline-pak/src/index.rs` come from [eigeen/ree-pak-rs](https://github.com/eigeen/ree-pak-rs). The reader code is our own, written from those facts. Licence, copied in full as the licence requires:

> MIT License
>
> Copyright (c) 2024 Eigeen
>
> Permission is hereby granted, free of charge, to any person obtaining a copy
> of this software and associated documentation files (the "Software"), to deal
> in the Software without restriction, including without limitation the rights
> to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
> copies of the Software, and to permit persons to whom the Software is
> furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
> FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
> AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
> LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
> SOFTWARE.

## crates/forgeline-update (the update flow, linked into the app's `--update` mode)

No third-party crate beyond those already listed for `forgeline-pak`, `forgeline-save` and `forgeline-core` (serde and serde_json). The file hash used to notice changes (FNV-1a, 64 bit) is a few lines of our own code from the published algorithm.

## crates/forgeline-engine (compiled into the app)

| Crate | Version | Licence | Use |
|---|---|---|---|
| notify | 8.2.1 | CC0-1.0 | operating-system file change notifications for the save watcher (default features off) |
| notify-types | 2.2.0 | MIT OR Apache-2.0 | dependency of notify |
| inotify, inotify-sys | 0.11.5, 0.1.8 | ISC | Linux backend of notify (CI and development only; not used on Windows) |
| kqueue, kqueue-sys | 1.2.1, 1.1.2 | MIT | macOS and BSD backend of notify (never built for Windows) |
| mio | 1.2.4 | MIT | event loop used by notify |
| walkdir, same-file, winapi-util | 2.5.0, 1.0.6, 0.1.11 | Unlicense OR MIT (walkdir, same-file, winapi-util) | dependencies of notify |
| bitflags, libc, log | 2.13.2, 0.2.191, 0.4.34 | MIT OR Apache-2.0 | dependencies of notify |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | second version, used by notify and mio on Windows |

Licences read from each crate's `Cargo.toml` (`cargo metadata`). CC0-1.0 is a public-domain dedication and compatible with MPL-2.0. No GPL crate is used.
