# Specification surface checks

`task fuzz-check` runs the independent stable workspace's format check, strict Clippy, tests and
finite replay. Neither this workspace nor `engine` is a root workspace member. The engine has its
own lock and nightly dependencies; the ordinary offline gate needs only the stable graph.
Keep every crate version in `fuzz/Cargo.lock` equal to the root `Cargo.lock`: CI fetches only
the root graph before the offline gate, so a version present in this lock alone fails
`fuzz-check` with "attempting to make an HTTP request, but --offline was specified" (PR #17,
run 34285219066). Align with `cargo update --manifest-path fuzz/Cargo.toml -p <crate> --precise <root version>`.

The input is closed JSON containing ordered `documents` with `label` and `text` strings. Labels
are opaque diagnostics and never filesystem paths. The decoder enforces the accepted byte and
count sampling bounds before ESS parsing. These limits do not change production admission.

Every compiled ess/1 bundle calls the five registered checked artifact generators, separate
pretty docs-ir serialization, and Rust, Go, Web and Clap synthesis. Returned errors are recorded
and later independent calls continue. A panic fails. Generated programs are not compiled or run.
The production format after assembly decides whether the source is in-domain; an admitted ess/2
bundle is separately recorded. Invalid source controls retain their actual refusal stage.

`regressions/source` retains all original mandatory source bytes. Its hash manifest and exact
carrier translation are checked. The sixteen structured vectors have separately pinned rendered
source hashes; changing their renderer requires reviewing those identities. `readiness/expected`
retains complete pre-repair Go artifact maps, plans and source/IR records for nine positive cases.

Each stable bundle runs in a separate child with a ten-second monotonic kill-and-reap deadline.
Fresh output lives below `fuzz/target/replay`, or an explicit `--out` directory. Preserve failed
outputs: direct child status, stdout/stderr, original input, typed bounded observation frames and
SHA256-addressed input/rendered/refusal blobs. A successful process without its complete admitted
stream fails. The tests also exercise explicit broken dispatch/translation/observation controls,
returned errors, panic propagation, and a real stalled child. These controls never activate in
ordinary replay.

## Local instrumented lane

Provision the selected dated nightly and cargo-fuzz before working offline; no gate downloads
them. The reviewed lane uses nightly-2026-07-28, cargo-fuzz 0.13.2, ordinary AddressSanitizer,
two build jobs, incremental disabled, and codegen-units 16. It does not use build-std, careful mode
or MemorySanitizer. Explicitly disable all configured compiler wrappers; an inherited Cargo
configuration can otherwise select one even with a private CARGO_HOME.

Build from the repository root, retaining the engine lock before and after:

```console
cargo run --manifest-path fuzz/Cargo.toml --locked --offline --bin replay -- --starting-inputs fuzz/target/starting-inputs
cargo +nightly-2026-07-28 fuzz build --fuzz-dir fuzz/engine --sanitizer address --codegen-units 16
```

Use `CARGO_NET_OFFLINE=true`, `CARGO_BUILD_JOBS=2`, and task-owned storage. This cargo-fuzz version
has no `--locked` option; compare the complete engine lock bytes afterward. Cargo's default engine
target is `fuzz/engine/target`; do not override it or share another checkout's target.

Run the resulting `byte_carrier` and `structured` native binaries sequentially. For each, set
`ESS_FUZZ_OUTPUT` to a different fresh observation directory and give a private mutable copy of
its matching starting corpus. Keep originals and crashes separately. The exact engine arguments
are `-runs=2048 -max_total_time=120 -max_len=65536 -timeout=10 -rss_limit_mb=2048
-malloc_limit_mb=512 -seed=1592590347`, with a task-owned `-artifact_prefix` directory. No workers
or jobs option is needed for the ordinary single-process worker. Inspect actual native binary
locations from the build output rather than borrowing an alias from another tree.

After a successful engine exit, run:

```console
cargo run --manifest-path fuzz/Cargo.toml --locked --offline --bin replay -- --admit-live byte-carrier OBSERVATION_DIRECTORY
cargo run --manifest-path fuzz/Cargo.toml --locked --offline --bin replay -- --admit-live structured OTHER_OBSERVATION_DIRECTORY
```

The adapter retains a separately framed callback receipt before calling the shared pipeline.
Admission matches every callback's exact original bytes and entry to exactly one pipeline attempt;
then it checks complete per-stage sequencing and requires at least one compiled live callback.
The finite replay's accepted models do not qualify live work. LibFuzzer iteration counters and
actual callback counts are separate evidence: startup and mutation can replay inputs more than
once. Only distinguish those classes when the actual engine records support it.

For the deliberately broken adapter control, `ESS_FUZZ_CONTROL=no-work` records callbacks while
omitting the shared pipeline. Engine success must fail `--admit-live` even when stable replay is
green. Unset this variable for both real campaigns. Unknown controls refuse; there is no normal
catch-and-ignore-panic option. Retain original control failures and restored real execution.
