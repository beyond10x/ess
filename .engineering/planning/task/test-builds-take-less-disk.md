---
format: aep.planning-md/3
id: task:test-builds-take-less-disk
kind: task
status: draft
title: Test builds take less disk
summary: Keep DWARF out of every test executable; measure du of target/debug/deps before and after
relations:
- serves: vision:O2
revision: 2
---
## Task

Cut the disk a test build takes. One gate run of five crates (`ess-domain`, `ess-gen`, `ess-synth`,
`ess-conformance`, `ess-diff`) after a `cargo clean` wrote 322 test executables totalling 23453 MB into
`target/debug/deps`, against 1278 MB of rlibs (du, tree `ess-0680-diff-scalar`, 2026-10-22 02:13). Builds on
this machine share one disk and have been paused six times in one night for it.

## Where the bytes are

`readelf -SW` on one executable, `ess-conformance`'s `view_paging` test, 153141264 bytes, built with the
current `[profile.dev] debug = "line-tables-only"`:

| section | bytes |
|---|---|
| `.debug_str` | 35209367 |
| `.strtab` | 29603865 |
| `.text` | 29476245 |
| `.debug_line` | 21018079 |
| `.debug_info` | 14251361 |
| `.debug_ranges` | 5748816 |
| `.symtab` | 3786288 |

About 76 MB of the 153 MB is DWARF and 33 MB is the symbol and string tables. Every test executable links
its own copy of every dependency's DWARF.

## Candidates, in order

1. `split-debuginfo = "unpacked"` on `[profile.dev]`: DWARF stays in one `.dwo` per compilation unit
   instead of being linked into each executable. Backtraces still resolve.
2. `[profile.dev.package."*"] debug = false`: no DWARF for dependencies; workspace crates keep line tables.
3. Merging a crate's integration test files into one binary per crate. A larger change: it moves every
   `tests/*.rs` file under one harness, and `ci_lanes.rs` and the nextest archives name binaries. Not in
   this pass.

## Acceptance

- `du -sm target/debug/deps` (plus any `.dwo` it writes elsewhere) for the same gate run, before and after,
  on a cleaned tree, recorded as evidence.
- A panic backtrace in a test still names file and line.
- CI's archive lanes (`test-archive-workspace`, `test-archive-feature-off`) still extract and run: nextest
  archives must carry the `.dwo` files or the change applies to local builds only.

## Schedule

After 0.68.0, unless candidate 1 or 2 is measured on a tree that is being rebuilt anyway; the measurement
may use `CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=unpacked` in the environment without changing `Cargo.toml`.
