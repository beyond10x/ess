---
format: aep.planning-md/3
id: review-result:served-store-and-entry-adversary-pass-1-20261002
kind: review-result
status: active
title: 'Server entry adversary pass 1: ordinary memory domain regression'
relations:
- reviews: story:served-store-and-entry
revision: 1
---
unit: served-store-and-entry at d58db28ea230fd5e6a1844c7e65977b95128d6b1
verdict: CONFIRMED
cases: executed 11→13, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: 3 directory roots, enumerated below
needs-coordinator: fix ordinary memory-domain gating; exact-base execution not performed

Publication note: The original raw report has SHA-256 `74d3a96619c9a689d37487526ab92a9308c27f765691737cc8dbc8c3421bbfa7`. This publication copy replaces only literal home-directory path prefixes with `$HOME/`. All findings, counts, line numbers and other report content are preserved. Log excerpts below are path-normalized, not byte-verbatim; descriptions of verbatim capture refer to the preserved original report.

1. git --no-pager diff --stat d58db28ea230fd5e6a1844c7e65977b95128d6b1

```text
 crates/generate/ess-synth/tests/served_entry.rs | 46 +++++++++++++++++++++++++
 1 file changed, 46 insertions(+)
```

Only tests changed. No source, generated output in the worktree, documentation, planning, commits or refs changed. The assigned managed worktree was leased as codex-adversary1-459ab620; that lease is released on return.

2. Cases and first isolated runs

`crates/generate/ess-synth/tests/served_entry.rs:34`, `a_non_network_domain_named_memory_remains_available`: parses, assembles and compiles the accepted fixture with domain `notes.memory` and `reached_by: in_process`, synthesizes Rust, and requires the generated types package to compile. Red now. This is a generated-consumer compilation failure after the fixture and Rust test harness successfully compile, not a broken test harness. It reaches normal public synthesis through a valid domain name and supported reach selection.

Every run used the following environment, in the assigned worktree:

```console
env TMPDIR=$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1 CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-w5-go CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_BUILD_JOBS=2 GOCACHE=$HOME/.cache/ess-w5-go-cache CARGO=/usr/bin/cargo cargo test -p ess-synth --locked --test served_entry a_non_network_domain_named_memory_remains_available -- --exact --nocapture
```

Actual cargo exit status: 101. Complete first isolated output:

```text
   Compiling ess-synth v0.51.0 ($HOME/.local/state/worktree/trees/b10x/ess/ess-w7-store-entry/crates/generate/ess-synth)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.35s
     Running tests/served_entry.rs ($HOME/.cache/b10x-target/ess-w5-go/debug/deps/served_entry-a2ca4a971cfca932)

running 1 test

thread 'a_non_network_domain_named_memory_remains_available' (2665740) panicked at crates/generate/ess-synth/tests/served_entry.rs:34:5:
generated non-network domain remains usable:     Checking notes-types v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2665739/domain-memory/crates/notes-types)
error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:28:72
   |
28 |     fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::memory::NoteSnapshot>;
   |                                                                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:31:40
   |
31 |     fn put(&mut self, snapshot: crate::memory::NoteSnapshot);
   |                                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:38:34
   |
38 |     fn list(&self) -> Vec<crate::memory::NoteSnapshot>;
   |                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:69:16
   |
69 | impl<P> crate::memory::obligations::AddNoteBehavior for Generated<P>
   |                ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:73:42
   |
73 |     fn add_note(&mut self, input: crate::memory::AddNote) -> Result<crate::memory::AddNoteOutcome, UnmetObligation> {
   |                                          ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:73:76
   |
73 |     fn add_note(&mut self, input: crate::memory::AddNote) -> Result<crate::memory::AddNoteOutcome, UnmetObligation> {
   |                                                                            ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:77:27
   |
77 |         let data = crate::memory::NoteData {
   |                           ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:81:50
   |
81 |         NoteStorage::put(&mut self.ports, crate::memory::AnyNote::Open(crate::memory::Note::new(data)).snapshot());
   |                                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:81:79
   |
81 |         NoteStorage::put(&mut self.ports, crate::memory::AnyNote::Open(crate::memory::Note::new(data)).snapshot());
   |                                                                               ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:82:26
   |
82 | ...   return Ok(crate::memory::AddNoteOutcome::Added { note_added: crate::memory::NoteAdded { id: identity.clone(), title: input.tit...
   |                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:82:77
   |
82 | ...   return Ok(crate::memory::AddNoteOutcome::Added { note_added: crate::memory::NoteAdded { id: identity.clone(), title: input.tit...
   |                                                                           ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:87:16
   |
87 | impl<P> crate::memory::obligations::ArchiveNoteBehavior for Generated<P>
   |                ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:91:46
   |
91 |     fn archive_note(&mut self, input: crate::memory::ArchiveNote) -> Result<crate::memory::ArchiveNoteOutcome, UnmetObligation> {
   |                                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:91:84
   |
91 |     fn archive_note(&mut self, input: crate::memory::ArchiveNote) -> Result<crate::memory::ArchiveNoteOutcome, UnmetObligation> {
   |                                                                                    ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:95:30
   |
95 |             return Ok(crate::memory::ArchiveNoteOutcome::Missing { error: crate::memory::NotFound });
   |                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:95:82
   |
95 |             return Ok(crate::memory::ArchiveNoteOutcome::Missing { error: crate::memory::NotFound });
   |                                                                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:99:20
   |
99 |             crate::memory::AnyNote::Open(instance) => crate::memory::AnyNote::Archived(instance.archive()),
   |                    ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:99:62
   |
99 |             crate::memory::AnyNote::Open(instance) => crate::memory::AnyNote::Archived(instance.archive()),
   |                                                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:100:35
    |
100 |             _ => return Ok(crate::memory::ArchiveNoteOutcome::ArchivedAlready { error: crate::memory::AlreadyArchived }),
    |                                   ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:100:95
    |
100 |             _ => return Ok(crate::memory::ArchiveNoteOutcome::ArchivedAlready { error: crate::memory::AlreadyArchived }),
    |                                                                                               ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:104:26
    |
104 | ...   return Ok(crate::memory::ArchiveNoteOutcome::Archived { note_archived: crate::memory::NoteArchived { id: input.id.clone() } });
    |                        ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:104:87
    |
104 | ...tcome::Archived { note_archived: crate::memory::NoteArchived { id: input.id.clone() } });
    |                                            ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:109:16
    |
109 | impl<P> crate::memory::obligations::NotesQuery for Generated<P>
    |                ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:113:42
    |
113 |     fn notes(&self) -> Result<Vec<crate::memory::Notes>, UnmetObligation> {
    |                                          ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:117:32
    |
117 |             .map(|held| crate::memory::Notes {
    |                                ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error: unexpected `cfg` condition value: `memory`
  --> crates/notes-types/src/lib.rs:20:11
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ^^^^^^^^^^^^^^^^^^ help: remove the condition
   |
   = note: no expected values for `feature`
   = help: consider adding `memory` as a feature in `Cargo.toml`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: `-D unexpected-cfgs` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(unexpected_cfgs)]`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `notes-types` (lib) due to 26 previous errors

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test a_non_network_domain_named_memory_remains_available ... FAILED

failures:

failures:
    a_non_network_domain_named_memory_remains_available

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.17s

error: test failed, to rerun pass `-p ess-synth --test served_entry`
```

The first suite then ran (11 passed, 1 failed). While it completed, the coordinator requested the adjacent network-domain branch. That request caused a second new case, not a source change.

`crates/generate/ess-synth/tests/served_entry.rs:13`, `a_network_domain_named_memory_remains_available`: synthesizes a network component under the domain `notes.memory`; builds both Rust workspace and single-crate executables; creates a note and reads it back through HTTP, asserting its exact title and matching identity. Green now. Its first isolated run reused a helper bound to the original domain's HTTP routes and returned 404 after a successful generated build. That was a probe-harness mismatch, not a finding. Only the new case was corrected to derive routes from its own model; the existing cases were untouched. Both the erroneous first run and corrected isolated run are retained verbatim below.

Command suffix for both isolated runs (same environment above): `cargo test -p ess-synth --locked --test served_entry a_network_domain_named_memory_remains_available -- --exact --nocapture`.

First network probe, exit 101, harness mismatch:

```text
   Compiling ess-synth v0.51.0 ($HOME/.local/state/worktree/trees/b10x/ess/ess-w7-store-entry/crates/generate/ess-synth)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.93s
     Running tests/served_entry.rs ($HOME/.cache/b10x-target/ess-w5-go/debug/deps/served_entry-a2ca4a971cfca932)

running 1 test
cd "$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606/network-memory" && env -u CARGO_ENCODED_RUSTFLAGS -u CARGO_TARGET_DIR CARGO_BUILD_JOBS="2" CARGO_INCREMENTAL="0" CARGO_PROFILE_DEV_DEBUG="0" RUSTFLAGS="-D warnings" "$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "build" "--offline" "--bin" "notes-server" "--target-dir" "$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606/rust-target"
     Locking 49 packages to latest compatible versions
      Adding time v0.3.41 (available: v0.3.55)
      Adding time-core v0.1.4 (available: v0.1.9)
      Adding time-macros v0.2.22 (available: v0.2.32)
      Adding uuid v1.23.3 (available: v1.26.1)
   Compiling libc v0.2.189
   Compiling proc-macro2 v1.0.107
   Compiling getrandom v0.4.3
   Compiling cfg-if v1.0.5
   Compiling powerfmt v0.2.0
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling deranged v0.4.0
   Compiling itoa v1.0.18
   Compiling time-core v0.1.4
   Compiling utf8parse v0.2.2
   Compiling num-conv v0.1.0
   Compiling anstyle-parse v1.0.0
   Compiling uuid v1.23.3
   Compiling time v0.3.41
   Compiling colorchoice v1.0.5
   Compiling anstyle v1.0.14
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-query v1.1.5
   Compiling anstream v1.0.0
   Compiling syn v3.0.6
   Compiling notes-types v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606/network-memory/crates/notes-types)
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling notes v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606/network-memory/crates/notes)
   Compiling notes-system v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606/network-memory/crates/notes-system)
   Compiling clap v4.6.7
   Compiling notes-server v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606/network-memory/crates/notes-server)
    Finished `dev` profile [unoptimized] target(s) in 7.78s

generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)

thread 'a_network_domain_named_memory_remains_available' (2707607) panicked at crates/generate/ess-synth/tests/served_entry.rs:351:5:
assertion `left == right` failed: {"refused":"`/notes/commands/AddNote` is not a path this surface declares; `GET /openapi.json` publishes every one that is"}
  left: 404
 right: 202
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test a_network_domain_named_memory_remains_available ... FAILED

failures:

failures:
    a_network_domain_named_memory_remains_available

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 7.83s

error: test failed, to rerun pass `-p ess-synth --test served_entry`
```

Corrected network case alone, exit 0:

```text
   Compiling ess-synth v0.51.0 ($HOME/.local/state/worktree/trees/b10x/ess/ess-w7-store-entry/crates/generate/ess-synth)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.21s
     Running tests/served_entry.rs ($HOME/.cache/b10x-target/ess-w5-go/debug/deps/served_entry-a2ca4a971cfca932)

running 1 test
cd "$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory" && env -u CARGO_ENCODED_RUSTFLAGS -u CARGO_TARGET_DIR CARGO_BUILD_JOBS="2" CARGO_INCREMENTAL="0" CARGO_PROFILE_DEV_DEBUG="0" RUSTFLAGS="-D warnings" "$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "build" "--offline" "--bin" "notes-server" "--target-dir" "$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/rust-target"
     Locking 49 packages to latest compatible versions
      Adding time v0.3.41 (available: v0.3.55)
      Adding time-core v0.1.4 (available: v0.1.9)
      Adding time-macros v0.2.22 (available: v0.2.32)
      Adding uuid v1.23.3 (available: v1.26.1)
   Compiling libc v0.2.189
   Compiling proc-macro2 v1.0.107
   Compiling getrandom v0.4.3
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling cfg-if v1.0.5
   Compiling powerfmt v0.2.0
   Compiling deranged v0.4.0
   Compiling time-core v0.1.4
   Compiling itoa v1.0.18
   Compiling utf8parse v0.2.2
   Compiling num-conv v0.1.0
   Compiling time v0.3.41
   Compiling anstyle-parse v1.0.0
   Compiling uuid v1.23.3
   Compiling colorchoice v1.0.5
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstream v1.0.0
   Compiling notes-types v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory/crates/notes-types)
   Compiling syn v3.0.6
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling notes v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory/crates/notes)
   Compiling notes-system v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory/crates/notes-system)
   Compiling clap v4.6.7
   Compiling notes-server v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory/crates/notes-server)
    Finished `dev` profile [unoptimized] target(s) in 5.15s

generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
cd "$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory-single" && env -u CARGO_ENCODED_RUSTFLAGS -u CARGO_TARGET_DIR CARGO_BUILD_JOBS="2" CARGO_INCREMENTAL="0" CARGO_PROFILE_DEV_DEBUG="0" RUSTFLAGS="-D warnings" "$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "build" "--offline" "--bin" "notes-server" "--features" "server" "--target-dir" "$HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/rust-target"
     Locking 49 packages to latest compatible versions
      Adding time v0.3.41 (available: v0.3.55)
      Adding time-core v0.1.4 (available: v0.1.9)
      Adding time-macros v0.2.22 (available: v0.2.32)
      Adding uuid v1.23.3 (available: v1.26.1)
   Compiling notes v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865/network-memory-single)
    Finished `dev` profile [unoptimized] target(s) in 0.48s

generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
test a_network_domain_named_memory_remains_available ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 5.71s

```

3. Suite runs

Both runs used the exact environment above with command suffix `cargo test -p ess-synth --locked --test served_entry -- --test-threads=1`. No pre-case baseline was executed; the before-count 11 came from the implementor's report. First suite followed the non-network case (12 executed, one red, exit 101). Final suite followed both corrected isolated cases (13 executed, one red, exit 101). Complete outputs, first then final:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/served_entry.rs ($HOME/.cache/b10x-target/ess-w5-go/debug/deps/served_entry-a2ca4a971cfca932)

running 12 tests
test a_non_network_domain_named_memory_remains_available ... FAILED
test an_entry_point_needing_caller_attributes_refuses_to_start_naming_them ... ok
test an_entry_point_with_owed_commands_refuses_to_start_naming_them ... ok
test reusable_types_keep_the_default_dependency_graph_empty ... ok
test the_context_supplies_uuid_newtypes_and_system_clock_timestamps ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_fixture_suite_passes_against_the_generated_go_and_rust_servers ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_generated_context_assigns_distinct_uuids ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_go_entry_point_serves_the_fixture_with_no_hand_written_code ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=none
ok
test the_rust_entry_point_serves_the_fixture_with_no_hand_written_code ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=none
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=none
ok
test the_served_notes_plan_has_no_obligation ... ok
test the_static_directory_is_served_beside_the_api ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_store_lists_in_identity_order ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok

failures:

---- a_non_network_domain_named_memory_remains_available stdout ----

thread 'a_non_network_domain_named_memory_remains_available' (2694356) panicked at crates/generate/ess-synth/tests/served_entry.rs:34:5:
generated non-network domain remains usable:     Checking notes-types v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2694355/domain-memory/crates/notes-types)
error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:28:72
   |
28 |     fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::memory::NoteSnapshot>;
   |                                                                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:31:40
   |
31 |     fn put(&mut self, snapshot: crate::memory::NoteSnapshot);
   |                                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:38:34
   |
38 |     fn list(&self) -> Vec<crate::memory::NoteSnapshot>;
   |                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:69:16
   |
69 | impl<P> crate::memory::obligations::AddNoteBehavior for Generated<P>
   |                ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:73:42
   |
73 |     fn add_note(&mut self, input: crate::memory::AddNote) -> Result<crate::memory::AddNoteOutcome, UnmetObligation> {
   |                                          ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:73:76
   |
73 |     fn add_note(&mut self, input: crate::memory::AddNote) -> Result<crate::memory::AddNoteOutcome, UnmetObligation> {
   |                                                                            ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:77:27
   |
77 |         let data = crate::memory::NoteData {
   |                           ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:81:50
   |
81 |         NoteStorage::put(&mut self.ports, crate::memory::AnyNote::Open(crate::memory::Note::new(data)).snapshot());
   |                                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:81:79
   |
81 |         NoteStorage::put(&mut self.ports, crate::memory::AnyNote::Open(crate::memory::Note::new(data)).snapshot());
   |                                                                               ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:82:26
   |
82 | ...   return Ok(crate::memory::AddNoteOutcome::Added { note_added: crate::memory::NoteAdded { id: identity.clone(), title: input.tit...
   |                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:82:77
   |
82 | ...   return Ok(crate::memory::AddNoteOutcome::Added { note_added: crate::memory::NoteAdded { id: identity.clone(), title: input.tit...
   |                                                                           ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:87:16
   |
87 | impl<P> crate::memory::obligations::ArchiveNoteBehavior for Generated<P>
   |                ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:91:46
   |
91 |     fn archive_note(&mut self, input: crate::memory::ArchiveNote) -> Result<crate::memory::ArchiveNoteOutcome, UnmetObligation> {
   |                                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:91:84
   |
91 |     fn archive_note(&mut self, input: crate::memory::ArchiveNote) -> Result<crate::memory::ArchiveNoteOutcome, UnmetObligation> {
   |                                                                                    ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:95:30
   |
95 |             return Ok(crate::memory::ArchiveNoteOutcome::Missing { error: crate::memory::NotFound });
   |                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:95:82
   |
95 |             return Ok(crate::memory::ArchiveNoteOutcome::Missing { error: crate::memory::NotFound });
   |                                                                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:99:20
   |
99 |             crate::memory::AnyNote::Open(instance) => crate::memory::AnyNote::Archived(instance.archive()),
   |                    ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:99:62
   |
99 |             crate::memory::AnyNote::Open(instance) => crate::memory::AnyNote::Archived(instance.archive()),
   |                                                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:100:35
    |
100 |             _ => return Ok(crate::memory::ArchiveNoteOutcome::ArchivedAlready { error: crate::memory::AlreadyArchived }),
    |                                   ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:100:95
    |
100 |             _ => return Ok(crate::memory::ArchiveNoteOutcome::ArchivedAlready { error: crate::memory::AlreadyArchived }),
    |                                                                                               ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:104:26
    |
104 | ...   return Ok(crate::memory::ArchiveNoteOutcome::Archived { note_archived: crate::memory::NoteArchived { id: input.id.clone() } });
    |                        ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:104:87
    |
104 | ...tcome::Archived { note_archived: crate::memory::NoteArchived { id: input.id.clone() } });
    |                                            ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:109:16
    |
109 | impl<P> crate::memory::obligations::NotesQuery for Generated<P>
    |                ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:113:42
    |
113 |     fn notes(&self) -> Result<Vec<crate::memory::Notes>, UnmetObligation> {
    |                                          ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:117:32
    |
117 |             .map(|held| crate::memory::Notes {
    |                                ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error: unexpected `cfg` condition value: `memory`
  --> crates/notes-types/src/lib.rs:20:11
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ^^^^^^^^^^^^^^^^^^ help: remove the condition
   |
   = note: no expected values for `feature`
   = help: consider adding `memory` as a feature in `Cargo.toml`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: `-D unexpected-cfgs` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(unexpected_cfgs)]`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `notes-types` (lib) due to 26 previous errors

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_non_network_domain_named_memory_remains_available

test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.51s

error: test failed, to rerun pass `-p ess-synth --test served_entry`
```

Final suite:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running tests/served_entry.rs ($HOME/.cache/b10x-target/ess-w5-go/debug/deps/served_entry-a2ca4a971cfca932)

running 13 tests
test a_network_domain_named_memory_remains_available ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test a_non_network_domain_named_memory_remains_available ... FAILED
test an_entry_point_needing_caller_attributes_refuses_to_start_naming_them ... ok
test an_entry_point_with_owed_commands_refuses_to_start_naming_them ... ok
test reusable_types_keep_the_default_dependency_graph_empty ... ok
test the_context_supplies_uuid_newtypes_and_system_clock_timestamps ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_fixture_suite_passes_against_the_generated_go_and_rust_servers ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_generated_context_assigns_distinct_uuids ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_go_entry_point_serves_the_fixture_with_no_hand_written_code ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=none
ok
test the_rust_entry_point_serves_the_fixture_with_no_hand_written_code ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=none
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=none
ok
test the_served_notes_plan_has_no_obligation ... ok
test the_static_directory_is_served_beside_the_api ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok
test the_store_lists_in_identity_order ... generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
generated entry: ephemeral storage; callers=actor-header (demonstration mode; not authentication)
ok

failures:

---- a_non_network_domain_named_memory_remains_available stdout ----

thread 'a_non_network_domain_named_memory_remains_available' (2729714) panicked at crates/generate/ess-synth/tests/served_entry.rs:55:5:
generated non-network domain remains usable:     Checking notes-types v1.0.0 ($HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2728587/domain-memory/crates/notes-types)
error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:28:72
   |
28 |     fn get(&self, identity: &crate::primitives::Uuid) -> Option<crate::memory::NoteSnapshot>;
   |                                                                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:31:40
   |
31 |     fn put(&mut self, snapshot: crate::memory::NoteSnapshot);
   |                                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:38:34
   |
38 |     fn list(&self) -> Vec<crate::memory::NoteSnapshot>;
   |                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:69:16
   |
69 | impl<P> crate::memory::obligations::AddNoteBehavior for Generated<P>
   |                ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:73:42
   |
73 |     fn add_note(&mut self, input: crate::memory::AddNote) -> Result<crate::memory::AddNoteOutcome, UnmetObligation> {
   |                                          ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:73:76
   |
73 |     fn add_note(&mut self, input: crate::memory::AddNote) -> Result<crate::memory::AddNoteOutcome, UnmetObligation> {
   |                                                                            ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:77:27
   |
77 |         let data = crate::memory::NoteData {
   |                           ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:81:50
   |
81 |         NoteStorage::put(&mut self.ports, crate::memory::AnyNote::Open(crate::memory::Note::new(data)).snapshot());
   |                                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:81:79
   |
81 |         NoteStorage::put(&mut self.ports, crate::memory::AnyNote::Open(crate::memory::Note::new(data)).snapshot());
   |                                                                               ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:82:26
   |
82 | ...   return Ok(crate::memory::AddNoteOutcome::Added { note_added: crate::memory::NoteAdded { id: identity.clone(), title: input.tit...
   |                        ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:82:77
   |
82 | ...   return Ok(crate::memory::AddNoteOutcome::Added { note_added: crate::memory::NoteAdded { id: identity.clone(), title: input.tit...
   |                                                                           ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:87:16
   |
87 | impl<P> crate::memory::obligations::ArchiveNoteBehavior for Generated<P>
   |                ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:91:46
   |
91 |     fn archive_note(&mut self, input: crate::memory::ArchiveNote) -> Result<crate::memory::ArchiveNoteOutcome, UnmetObligation> {
   |                                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:91:84
   |
91 |     fn archive_note(&mut self, input: crate::memory::ArchiveNote) -> Result<crate::memory::ArchiveNoteOutcome, UnmetObligation> {
   |                                                                                    ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:95:30
   |
95 |             return Ok(crate::memory::ArchiveNoteOutcome::Missing { error: crate::memory::NotFound });
   |                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:95:82
   |
95 |             return Ok(crate::memory::ArchiveNoteOutcome::Missing { error: crate::memory::NotFound });
   |                                                                                  ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:99:20
   |
99 |             crate::memory::AnyNote::Open(instance) => crate::memory::AnyNote::Archived(instance.archive()),
   |                    ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
  --> crates/notes-types/src/behaviour.rs:99:62
   |
99 |             crate::memory::AnyNote::Open(instance) => crate::memory::AnyNote::Archived(instance.archive()),
   |                                                              ^^^^^^ could not find `memory` in the crate root
   |
note: found an item that was configured out
  --> crates/notes-types/src/lib.rs:21:9
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ------------------ the item is gated behind the `memory` feature
21 | pub mod memory;
   |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:100:35
    |
100 |             _ => return Ok(crate::memory::ArchiveNoteOutcome::ArchivedAlready { error: crate::memory::AlreadyArchived }),
    |                                   ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:100:95
    |
100 |             _ => return Ok(crate::memory::ArchiveNoteOutcome::ArchivedAlready { error: crate::memory::AlreadyArchived }),
    |                                                                                               ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:104:26
    |
104 | ...   return Ok(crate::memory::ArchiveNoteOutcome::Archived { note_archived: crate::memory::NoteArchived { id: input.id.clone() } });
    |                        ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:104:87
    |
104 | ...tcome::Archived { note_archived: crate::memory::NoteArchived { id: input.id.clone() } });
    |                                            ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:109:16
    |
109 | impl<P> crate::memory::obligations::NotesQuery for Generated<P>
    |                ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:113:42
    |
113 |     fn notes(&self) -> Result<Vec<crate::memory::Notes>, UnmetObligation> {
    |                                          ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error[E0433]: cannot find `memory` in `crate`
   --> crates/notes-types/src/behaviour.rs:117:32
    |
117 |             .map(|held| crate::memory::Notes {
    |                                ^^^^^^ could not find `memory` in the crate root
    |
note: found an item that was configured out
   --> crates/notes-types/src/lib.rs:21:9
    |
 20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
    |           ------------------ the item is gated behind the `memory` feature
 21 | pub mod memory;
    |         ^^^^^^

error: unexpected `cfg` condition value: `memory`
  --> crates/notes-types/src/lib.rs:20:11
   |
20 | #[cfg(all(feature = "memory", not(target_arch = "wasm32")))]
   |           ^^^^^^^^^^^^^^^^^^ help: remove the condition
   |
   = note: no expected values for `feature`
   = help: consider adding `memory` as a feature in `Cargo.toml`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: `-D unexpected-cfgs` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(unexpected_cfgs)]`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `notes-types` (lib) due to 26 previous errors

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_non_network_domain_named_memory_remains_available

test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.75s

error: test failed, to rerun pass `-p ess-synth --test served_entry`
```

4. Findings

| Location | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|
| crates/generate/ess-synth/src/rust/mod.rs:370 | contract-drift | blocker | CONFIRMED | undecided | The generated runtime feature gate also hides an ordinary domain named memory in non-network models, causing the generated types crate to fail compilation. |

What was measured: the isolated case and final suite both exit 101 because generated `lib.rs` gates the domain while generated behavior still references `crate::memory`; 25 unresolved module errors plus an unexpected-cfg error occur. The consumer compilation is reached through a fully compiled model, not hand-constructed IR. The network sibling builds and runs in workspace and single-crate layouts because Layout reserves `memory` when runtime generation is enabled.

What reaches it: public Rust synthesis of a valid specification containing a domain whose allocated module is `memory` and no served network component; the new test changes only the accepted notes fixture's qualified domain and reach. No hidden caller or impossible state is required.

Origin remains `undecided` under the role's exact-base execution rule: no base worktree was assigned, and the base was not executed. Reading `git show b4da64e38b770fe74103409fe1fef7ae6ca214f4:crates/generate/ess-synth/src/rust/mod.rs` shows that the baseline emits every ordinary domain module without this cfg. The candidate diff adds the gate at lines 370–371. That source comparison strongly attributes the failure to this unit, but it is not mislabeled as an executed baseline result. Suggested correction: apply the feature gate only to the synthesized runtime module, and keep ordinary domains available without runtime features.

No additional judgement findings.

5. Attacks that did not break

- The adjacent network domain named memory compiles and serves a create/read round trip in both Rust layouts; runtime/domain names are disambiguated there.
- All eleven pre-existing acceptance cases remain green, including the complete five-scenario HTTP conformance run per language and static containment probes.
- Current Rust entry construction clones shared Rc<RefCell> storage handles; the earlier separate-store suspicion does not describe this candidate. No new multi-component execution claim is made.

6. Paths written outside the worktree

All explicit writes were within three approved directory roots, including their generated/compiler descendants:

- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1
- $HOME/.cache/b10x-target/ess-w5-go
- $HOME/.cache/ess-w5-go-cache

Retained evidence files:

- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/report.md
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/memory-alone.log
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/memory-alone.exit
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/network-memory-alone.log
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/network-memory-alone.exit
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/network-memory-corrected-alone.log
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/network-memory-corrected-alone.exit
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/suite.log
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/suite.exit
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/suite-final.log
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/suite-final.exit

Disposable generated consumers, child target directories and binaries are contained recursively in:

- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2665739
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2694355
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2707606
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2722865
- $HOME/.cache/uilab-todo/w7s-codex/adversary-pass-1/ess-served-entry-2728587

No cache or tree was deleted. Free space stayed above the 10 GiB floor (observed low 19 GiB; final 27 GiB). All child tests and server processes exited before return. Worktree lifecycle commands also maintained this agent's own lease in manager-owned state; no other lease was changed.

7. Machine-readable findings

```findings
[{"file":"crates/generate/ess-synth/src/rust/mod.rs","line":370,"category":"contract-drift","severity":"blocker","verdict":"CONFIRMED","origin":"undecided","message":"The generated runtime feature gate also hides an ordinary domain named memory in non-network models, causing the generated types crate to fail compilation."}]
```
