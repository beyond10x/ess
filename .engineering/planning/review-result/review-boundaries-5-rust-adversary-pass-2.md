---
format: aep.planning-md/1
id: review-result:review-boundaries-5-rust-adversary-pass-2
kind: review-result
status: active
title: Final Rust feasibility adversary pass at a2ff04d
owner: impl_diagnostic
relations:
- reviews: story:review-rust-target-feasibility
revision: 1
---
unit: story:review-rust-target-feasibility — final full pass 2 at a2ff04d70f70a691288b661e4d027eee5da74e42
verdict: NEEDS-CHANGE
cases: executed 190→201, red 7
origin: introduced 0 / pre-existing 2 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this immutable pass, then route the two measured classes for bounded final correction and verification; no third full attack

`git --no-pager diff --stat`:

```text
 crates/edge/ess-cli/tests/feasibility_adversary.rs |  83 ++++++++++++
 .../ess-synth/tests/feasibility_adversary.rs       | 142 +++++++++++++++++++++
 2 files changed, 225 insertions(+)
```

Only those two tracked test files changed. There are no deletions: every pre-existing test and assertion, including the pass-1 regressions, remains. The test patch is `adversary-2-tests.patch`, SHA256 `e64b725eb11e276704709ea37779b05d5531895262469f487222a9f79d1355a2`. All other writes are assigned ignored scratch under this unit's `target/review-boundaries-5`, plus its own Cargo/TMPDIR target. No production, inline-test, planning, Git/index/ref, lifecycle, outside-tree or source-driven preview mutation occurred.

## 2. New cases and first isolated executions

The carried 190-case baseline comes from `correction-report-1.md`, SHA256 `a6de385b57462880e04dfdbbeeb3c6e57c7d433864f864205a1b9273b71415ff`; no baseline suite was run before these cases existed. This pass adds nine synth cases and two CLI cases. Every new case was run by its exact name before the first package run. The five synth failures and two CLI failures remain red in both complete package runs. The four new positive controls remain green. The actual charter used was `/home/timo/.claude/plugins/cache/beyond10x/aep-drive/0.7.0/agents/adversary.md`, the brief's same-version fallback. Acceptance is compiler-admitted valid models yielding compilable output or explicit pre-write refusals; the binding separately requires supported collection recursion and previously compilable output to remain admitted.

| Test file / final line | Case | Final result | What the real runner establishes |
|---|---|---|---|
| synth tests/feasibility_adversary.rs:258 | binding_function_cannot_be_hidden_by_the_delivery_event_parameter | red | Compiler-admitted binding `event` is generated, then rustc E0618 at `let input = event(event)` because the arm binds `event`. |
| synth tests/feasibility_adversary.rs:267 | a_later_binding_function_cannot_be_hidden_by_a_prior_input_local | red | Bindings `[alpha,input]` share one reacting-event arm; the first `let input` captures the later bare function call, rustc E0618. |
| synth tests/feasibility_adversary.rs:276 | a_first_binding_named_input_remains_compilable_on_both_targets | green | Single first binding `input` compiles as pure Rust and paired wasm32 Web; its own let binding does not capture its initializer. |
| synth tests/feasibility_adversary.rs:286 | error_encoder_names_in_separate_outcome_arms_remain_compilable | green | A success event allocating `encode_error_demo_core_broken` and a separate refusal arm calling that encoder compile in actual HTTP Rust. The helper is used in another lexical arm. |
| synth tests/feasibility_adversary.rs:346 | an_unaccepted_outcome_binding_has_no_web_codec_local_scope | green | A command carrying `Out` has no acceptor; pure Rust compiles and Web retains its actual nonempty partial refusal report and compiles without presenting that command codec. |
| synth tests/feasibility_adversary.rs:361 | optional_recursion_behind_nested_collections_keeps_its_size_break | red | `Optional<Map<Integer,List<Optional<Node>>>>` compiles in pure Rust. Its emitted Web decoder passes owned `nested0` into a `&str` map-key parser, E0308. This is a decoder compiler defect, not failure of the size-cycle admission. |
| synth tests/feasibility_adversary.rs:322 | boolean_map_decoder_path_is_borrowed_in_the_supported_codec | red | A nonrecursive `Map<Boolean,String>` compiles in pure Rust; emitted Web `key_bool(key0,nested0)` fails E0308. |
| synth tests/feasibility_adversary.rs:327 | bytes_map_decoder_path_is_borrowed_in_the_supported_codec | red | A nonrecursive `Map<Bytes,String>` compiles in pure Rust; emitted Web `key_bytes(key0,nested0)` fails E0308. |
| synth tests/feasibility_adversary.rs:332 | a_string_map_key_keeps_its_compilable_codec | green | `Map<String,String>` compiles on both targets; its key path does not call those parsers. |
| CLI tests/feasibility_adversary.rs:187 | a_binding_function_capture_is_refused_before_rust_cli_writes | red | Actual CLI exits0, creates absent output and adds files beside an existing unchanged sentinel, with success artifacts instead of a failure envelope. |
| CLI tests/feasibility_adversary.rs:192 | a_binding_function_capture_is_refused_before_web_cli_writes | red | Same actual CLI behavior for Web, despite its invalid Rust prerequisite. |

All source fixtures use real RawSpec parsing, Specification assembly and compile_locating; no hand-built IR or mismatched manual plan is involved. Every compiler witness writes the exact generated artifacts, resolves a local lock offline, checks a fixture-owned target, removes inherited CARGO_TARGET_DIR, and asserts artifact bytes unchanged. Web uses wasm32-unknown-unknown, not the deliberately rejected host target.

Setup records are separate from product failures. The first encoder-name fixture illegally combined emitting and error in one outcome and was rejected as `refusal_mutated_state`. Splitting its arms first exposed a second setup error, two unconditional outcomes (`conflicting_declaration`). The final fixture declares the refusal's external condition, following the actual billing source pattern. It then compiles, preserving the original intended lexical-scope assertion. Neither setup rejection is counted among the seven product reds or included as a finding. The initial strict-Clippy lane later rejected this pass's `push_str(format!(...))` helper; only that newly added test helper was changed to `write!` with the identical source bytes, after which strict Clippy and formatting passed. No existing case was relaxed.

The following raw isolated logs include the two setup attempts and final admitted control. For CLI cases the complete captured child stdout/stderr also remain beside the fixture paths printed by the runner; those exact files are included in the fixture hash manifest.

### adversary-2-isolated-1

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary binding_function_cannot_be_hidden_by_the_delivery_event_parameter -- --exact --nocapture
```

Output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
    Finished `test` profile [unoptimized] target(s) in 0.31s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3304630-0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1/crates/demo-system)
error[E0618]: expected function, found `&demo_types::core::Fired`
   --> crates/demo-system/src/lib.rs:139:29
    |
 53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
137 |             SystemEvent::Fired(event) => {
    |                                ----- `event` has type `&demo_types::core::Fired`
138 |                 // `event`: at_least_once, on failure drop.
139 |                 let input = event(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 1 warning
error: could not compile `demo-system` (lib test) due to 1 previous error; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 1 warning (1 duplicate)
error: could not compile `demo-system` (lib) due to 1 previous error; 1 warning emitted

exit status: 101

thread 'binding_function_cannot_be_hidden_by_the_delivery_event_parameter' (3304631) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test binding_function_cannot_be_hidden_by_the_delivery_event_parameter ... FAILED

failures:

failures:
    binding_function_cannot_be_hidden_by_the_delivery_event_parameter

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.37s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-2

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary a_later_binding_function_cannot_be_hidden_by_a_prior_input_local -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3308583-0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1/crates/demo-system)
error[E0618]: expected function, found `Handle`
   --> crates/demo-system/src/lib.rs:156:29
    |
 65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
151 |                 let input = alpha(event);
    |                     ----- `input` has type `Handle`
...
156 |                 let input = input(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:55:14
   |
55 | pub fn alpha(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:65:14
   |
65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 2 warnings
error: could not compile `demo-system` (lib test) due to 1 previous error; 2 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 2 warnings (2 duplicates)
error: could not compile `demo-system` (lib) due to 1 previous error; 2 warnings emitted

exit status: 101

thread 'a_later_binding_function_cannot_be_hidden_by_a_prior_input_local' (3308584) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test a_later_binding_function_cannot_be_hidden_by_a_prior_input_local ... FAILED

failures:

failures:
    a_later_binding_function_cannot_be_hidden_by_a_prior_input_local

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.39s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-3

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary a_first_binding_named_input_remains_compilable_on_both_targets -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3308777-0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/demo-system)
warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `demo-system` (lib test) generated 1 warning (run `cargo fix --lib -p demo-system --tests` to apply 1 suggestion)
warning: `demo-system` (lib) generated 1 warning (1 duplicate)
    Finished `dev` profile [unoptimized] target(s) in 0.36s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 3 packages to latest compatible versions

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/demo-system)
warning: unused variable: `event`
  --> /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/rust/demo/crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `demo-system` (lib) generated 1 warning (run `cargo fix --lib -p demo-system` to apply 1 suggestion)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-first-3308777-1/generated/web/demo/crates/demo-web)
    Finished `dev` profile [unoptimized] target(s) in 0.43s

exit status: 0
test a_first_binding_named_input_remains_compilable_on_both_targets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.84s

```

Exit: 0

### adversary-2-isolated-4

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary an_error_encoder_called_in_the_outcome_arm_cannot_be_captured -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test

thread 'an_error_encoder_called_in_the_outcome_arm_cannot_be_captured' (3309213) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:36:34:
attack fixture must be compiler-admitted ESS: 2 validation errors:
  - [refusal_mutated_state] command.demo.core.Fire.outcomes.refused: outcome `refused` reports `demo.core.Broken` and also emits `demo.core.EncodeErrorDemoCoreBroken`; a refused command changes nothing, so this is two outcomes wearing one name (hint: split it: one outcome that emits, one that errors, each with its own condition)
  - [undeclared_reference] component worker.accepts.commands: `worker` accepts `demo.core.Fire`, which nothing declares as a command (hint: no commands are declared anywhere in the specification)

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test an_error_encoder_called_in_the_outcome_arm_cannot_be_captured ... FAILED

failures:

failures:
    an_error_encoder_called_in_the_outcome_arm_cannot_be_captured

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-5

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary an_unaccepted_outcome_binding_has_no_web_codec_local_scope -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/worker)
warning: field `behaviors` is never read
  --> crates/worker/src/lib.rs:29:5
   |
28 | pub struct Worker<B> {
   |            ------ field in this struct
29 |     behaviors: B,
   |     ^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `worker` (lib test) generated 1 warning
warning: `worker` (lib) generated 1 warning (1 duplicate)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/demo-system)
    Finished `dev` profile [unoptimized] target(s) in 0.35s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 3 packages to latest compatible versions

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/worker)
warning: field `behaviors` is never read
  --> /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/worker/src/lib.rs:29:5
   |
28 | pub struct Worker<B> {
   |            ------ field in this struct
29 |     behaviors: B,
   |     ^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `worker` (lib) generated 1 warning
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/rust/demo/crates/demo-system)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-unaccepted-out-3309302-0/generated/web/demo/crates/demo-web)
warning: unused import: `crate::json`
  --> crates/demo-web/src/wire.rs:12:5
   |
12 | use crate::json;
   |     ^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unreachable statement
   --> crates/demo-web/src/lib.rs:174:9
    |
171 | /         match command {
172 | |             other => return Err(BridgeError::UnknownCommand(other.to_owned())),
173 | |         }
    | |_________- any code following this `match` expression is unreachable, as all arms diverge
174 |           self.pump()?;
    |           ^^^^^^^^^^^^^ unreachable statement
    |
    = note: `#[warn(unreachable_code)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
   --> crates/demo-web/src/lib.rs:170:13
    |
170 |         let mut out = String::new();
    |             ----^^^
    |             |
    |             help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `input`
   --> crates/demo-web/src/lib.rs:169:38
    |
169 |     fn run(&mut self, command: &str, input: &json::Value) -> Result<String, BridgeError> {
    |                                      ^^^^^ help: if this is intentional, prefix it with an underscore: `_input`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `out`
   --> crates/demo-web/src/lib.rs:170:13
    |
170 |         let mut out = String::new();
    |             ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_out`

warning: `demo-web` (lib test) generated 5 warnings (3 duplicates) (run `cargo fix --lib -p demo-web --tests` to apply 1 suggestion)
warning: `demo-web` (lib) generated 5 warnings (2 duplicates) (run `cargo fix --lib -p demo-web` to apply 3 suggestions)
    Finished `dev` profile [unoptimized] target(s) in 0.42s

exit status: 0
test an_unaccepted_outcome_binding_has_no_web_codec_local_scope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.81s

```

Exit: 0

### adversary-2-isolated-6

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary optional_recursion_behind_nested_collections_keeps_its_size_break -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.13s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:60:61
    |
 60 |                     entries0.insert(json::key_integer(key0, nested0)?, {
    |                                     -----------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:591:8
    |
591 | pub fn key_integer(key: &str, at: &str) -> Result<i64, DecodeError> {
    |        ^^^^^^^^^^^            --------
help: consider borrowing here
    |
 60 |                     entries0.insert(json::key_integer(key0, &nested0)?, {
    |                                                             +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'optional_recursion_behind_nested_collections_keeps_its_size_break' (3309633) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/web/demo
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test optional_recursion_behind_nested_collections_keeps_its_size_break ... FAILED

failures:

failures:
    optional_recursion_behind_nested_collections_keeps_its_size_break

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.39s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-4-corrected

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary error_encoder_names_in_separate_outcome_arms_remain_compilable -- --exact --nocapture
```

Output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
    Finished `test` profile [unoptimized] target(s) in 3.01s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test

thread 'error_encoder_names_in_separate_outcome_arms_remain_compilable' (3368391) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:36:34:
attack fixture must be compiler-admitted ESS: 2 validation errors:
  - [conflicting_declaration] command.demo.core.Fire.outcomes: outcomes `done`, `refused` are all unconditional, so the result of `demo.core.Fire` is not determined by its input (hint: give all but one of them a `when`)
  - [undeclared_reference] component worker.accepts.commands: `worker` accepts `demo.core.Fire`, which nothing declares as a command (hint: no commands are declared anywhere in the specification)

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test error_encoder_names_in_separate_outcome_arms_remain_compilable ... FAILED

failures:

failures:
    error_encoder_names_in_separate_outcome_arms_remain_compilable

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-7

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary boolean_map_decoder_path_is_borrowed_in_the_supported_codec -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.14s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:58
    |
 45 |                     entries0.insert(json::key_bool(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     --------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:604:8
    |
604 | pub fn key_bool(key: &str, at: &str) -> Result<bool, DecodeError> {
    |        ^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bool(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                          +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib test) due to 1 previous error

exit status: 101

thread 'boolean_map_decoder_path_is_borrowed_in_the_supported_codec' (3371461) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/web/demo
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test boolean_map_decoder_path_is_borrowed_in_the_supported_codec ... FAILED

failures:

failures:
    boolean_map_decoder_path_is_borrowed_in_the_supported_codec

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.69s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-8

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary bytes_map_decoder_path_is_borrowed_in_the_supported_codec -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.13s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:59
    |
 45 |                     entries0.insert(json::key_bytes(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     ---------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:621:8
    |
621 | pub fn key_bytes(key: &str, at: &str) -> Result<Vec<u8>, DecodeError> {
    |        ^^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bytes(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                           +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'bytes_map_decoder_path_is_borrowed_in_the_supported_codec' (3371685) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/web/demo
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test bytes_map_decoder_path_is_borrowed_in_the_supported_codec ... FAILED

failures:

failures:
    bytes_map_decoder_path_is_borrowed_in_the_supported_codec

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.40s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-9

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary a_string_map_key_keeps_its_compilable_codec -- --exact --nocapture
```

Output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.13s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-string-3371889-0/generated/web/demo/crates/demo-web)
    Finished `dev` profile [unoptimized] target(s) in 0.20s

exit status: 0
test a_string_map_key_keeps_its_compilable_codec ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.39s

```

Exit: 0

### adversary-2-isolated-10

Command:

```text
cargo test --locked --offline -p ess-cli --test feasibility_adversary a_binding_function_capture_is_refused_before_rust_cli_writes -- --exact --nocapture
```

Output:

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 4.47s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-8390c22bcf99a72e)

running 1 test
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "rust" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3372552-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3372552-0/absent"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "crates", "plan.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3372552-0/absent.stdout and .stderr
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "rust" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3372552-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3372552-0/existing"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "crates", "plan.json", "sentinel.txt"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3372552-0/existing.stdout and .stderr

thread 'a_binding_function_capture_is_refused_before_rust_cli_writes' (3372554) panicked at crates/edge/ess-cli/tests/feasibility_adversary.rs:145:5:
binding collision reached CLI writes:
absent: expected exit 1, got exit status: 0
absent: no typed rust failure
absent: destination was created
existing: expected exit 1, got exit status: 0
existing: no typed rust failure
existing: destination gained files
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test a_binding_function_capture_is_refused_before_rust_cli_writes ... FAILED

failures:

failures:
    a_binding_function_capture_is_refused_before_rust_cli_writes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p ess-cli --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-11

Command:

```text
cargo test --locked --offline -p ess-cli --test feasibility_adversary a_binding_function_capture_is_refused_before_web_cli_writes -- --exact --nocapture
```

Output:

```text
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Finished `test` profile [unoptimized] target(s) in 0.17s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-8390c22bcf99a72e)

running 1 test
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "web" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3375796-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3375796-0/absent"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "README.md", "TARGET.md", "bridge.js", "catalog.json", "crates", "index.html", "plan.json", "target.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3375796-0/absent.stdout and .stderr
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "web" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3375796-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3375796-0/existing"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "README.md", "TARGET.md", "bridge.js", "catalog.json", "crates", "index.html", "plan.json", "sentinel.txt", "target.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3375796-0/existing.stdout and .stderr

thread 'a_binding_function_capture_is_refused_before_web_cli_writes' (3375797) panicked at crates/edge/ess-cli/tests/feasibility_adversary.rs:145:5:
binding collision reached CLI writes:
absent: expected exit 1, got exit status: 0
absent: no typed web failure
absent: destination was created
existing: expected exit 1, got exit status: 0
existing: no typed web failure
existing: destination gained files
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test a_binding_function_capture_is_refused_before_web_cli_writes ... FAILED

failures:

failures:
    a_binding_function_capture_is_refused_before_web_cli_writes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p ess-cli --test feasibility_adversary`
```

Exit: 101

### adversary-2-isolated-4-final

Command:

```text
cargo test --locked --offline -p ess-synth --test feasibility_adversary error_encoder_names_in_separate_outcome_arms_remain_compilable -- --exact --nocapture
```

Output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
    Finished `test` profile [unoptimized] target(s) in 0.32s
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7a8cc9f9c85b11d8)

running 1 test
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-error-encoder-separate-arms-3390881-0" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-error-encoder-separate-arms-3390881-0" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-error-encoder-separate-arms-3390881-0/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-error-encoder-separate-arms-3390881-0/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-error-encoder-separate-arms-3390881-0/crates/demo-system)
    Checking demo-server v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-error-encoder-separate-arms-3390881-0/crates/demo-server)
warning: unused variable: `error`
   --> crates/demo-server/src/worker.rs:170:50
    |
170 |         demo_types::core::FireOutcome::Refused { error, .. } => {
    |                                                  ^^^^^ help: try ignoring the field: `error: _`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `demo-server` (lib test) generated 1 warning (run `cargo fix --lib -p demo-server --tests` to apply 1 suggestion)
warning: `demo-server` (lib) generated 1 warning (1 duplicate)
    Finished `dev` profile [unoptimized] target(s) in 0.51s

exit status: 0
test error_encoder_names_in_separate_outcome_arms_remain_compilable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.53s

```

Exit: 0

## 3. Complete package and ancillary gates

Both package executions occur after all new cases were selected individually. `--no-fail-fast` keeps the complete second package visible despite the CLI failures. The first package run executed 201 (194 passed, 7 failed, 0 ignored); the final run repeats those exact counts after the isolated test-helper Clippy correction. The baseline 190 existing cases all pass. Final totals are CLI 57 (55 passed, 2 failed) and synth 144 (139 passed, 5 failed). No test is ignored or deselected in these whole-package commands. The final fmt/Clippy exits are 0; package exit101 records the seven intentional retained regressions.

The raw first/final package output follows. The intermediate Clippy rejection concerns only the newly authored test helper, and remains preserved. Source/binary-origin work ran after the first package invocation; the final package run followed the helper correction. No full ESS, site, SDK or Atlas gate is claimed by this pass.

### adversary-2-package

Command:

```text
cargo test --locked --offline -p ess-synth -p ess-cli --no-fail-fast
```

Output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.34s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-8390c22bcf99a72e)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... FAILED
test a_binding_function_capture_is_refused_before_web_cli_writes ... FAILED

failures:

---- a_binding_function_capture_is_refused_before_rust_cli_writes stdout ----
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "rust" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3392546-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3392546-0/absent"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "crates", "plan.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3392546-0/absent.stdout and .stderr
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "rust" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3392546-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3392546-0/existing"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "crates", "plan.json", "sentinel.txt"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3392546-0/existing.stdout and .stderr

thread 'a_binding_function_capture_is_refused_before_rust_cli_writes' (3392547) panicked at crates/edge/ess-cli/tests/feasibility_adversary.rs:179:5:
binding collision reached CLI writes:
absent: expected exit 1, got exit status: 0
absent: no typed rust failure
absent: destination was created
existing: expected exit 1, got exit status: 0
existing: no typed rust failure
existing: destination gained files
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- a_binding_function_capture_is_refused_before_web_cli_writes stdout ----
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "web" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3392546-1/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3392546-1/absent"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "README.md", "TARGET.md", "bridge.js", "catalog.json", "crates", "index.html", "plan.json", "target.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3392546-1/absent.stdout and .stderr
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "web" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3392546-1/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3392546-1/existing"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "README.md", "TARGET.md", "bridge.js", "catalog.json", "crates", "index.html", "plan.json", "sentinel.txt", "target.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3392546-1/existing.stdout and .stderr

thread 'a_binding_function_capture_is_refused_before_web_cli_writes' (3392548) panicked at crates/edge/ess-cli/tests/feasibility_adversary.rs:179:5:
binding collision reached CLI writes:
absent: expected exit 1, got exit status: 0
absent: no typed web failure
absent: destination was created
existing: expected exit 1, got exit status: 0
existing: no typed web failure
existing: destination gained files


failures:
    a_binding_function_capture_is_refused_before_rust_cli_writes
    a_binding_function_capture_is_refused_before_web_cli_writes

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p ess-cli --test feasibility_adversary`
     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/target_failure.rs (target/debug/deps/target_failure-2d63596e3e7ff9da)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-e0a1c27a5644dd14)

running 8 tests
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-4a7e263190c9dff9)

running 10 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test the_binary_generates_its_own_completions ... ok
test every_placed_word_is_an_obligation ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test a_string_field_offers_no_values ... ok
test review_second_pass_actual_clap_manifest_retains_complete_comment_admission ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test a_placed_view_becomes_a_verb ... ok
test the_emission_is_deterministic ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/feasibility.rs (target/debug/deps/feasibility-a469b4e3776d4e98)

running 36 tests
test an_empty_domain_cannot_overwrite_the_rust_crate_root ... ok
test a_reserved_type_spelling_is_a_target_limitation ... ok
test a_type_cannot_capture_the_primitive_string_reference ... ok
test component_type_parameters_do_not_shadow_their_port ... ok
test mutual_recursion_is_a_size_cycle ... ok
test fixed_system_fields_are_reserved_only_in_the_system_scope ... ok
test a_mechanical_conversion_cannot_capture_the_from_trait ... ok
test generated_outcome_names_share_the_domain_type_scope ... ok
test generated_port_methods_cannot_capture_a_constructor ... ok
test optional_self_recursion_is_a_size_cycle ... ok
test normalized_fields_share_their_actual_struct_scope ... ok
test path_keyword_repairs_cannot_merge_fields ... ok
test normalized_type_collisions_refuse_before_returning_rust ... ok
test generated_entity_names_and_markers_use_their_own_scopes ... ok
test system_types_receive_a_target_result_instead_of_a_missing_owner_panic ... ok
test variant_names_are_checked_after_pascal_normalization ... ok
test system_level_types_refuse_even_when_referenced ... ok
test additional_optional_wrappers_and_union_size_cycles_refuse ... ok
test reserved_module_repairs_are_checked_as_a_final_set ... ok
test raw_module_identifiers_must_resolve_the_emitted_filename ... ok
test outcome_bindings_cannot_capture_an_encoder_called_in_the_same_arm ... ok
test required_to_optional_binding_assignment_is_not_a_plain_clone ... ok
test codec_fragments_are_checked_as_one_final_function_set ... ok
test wire_aliases_are_checked_only_at_an_emitted_codec_surface ... ok
test collection_indirection_and_acyclic_optionals_compile ... ok
test web_wire_module_cannot_capture_a_component_dependency ... ok
test web_catalog_module_cannot_capture_a_component_dependency ... ok
test independent_names_and_existing_package_repairs_compile ... ok
test distinct_wire_overrides_and_value_union_tag_compile ... ok
test optional_identity_and_omission_bindings_compile ... ok
test neutral_refused_deliveries_keep_the_partial_web_report_and_compile ... ok
test outcome_bindings_in_other_namespaces_or_unused_helpers_compile ... ok
test dependency_helpers_are_reserved_only_where_the_target_uses_them ... ok
test a_web_system_without_any_published_event_has_an_empty_observation ... ok
test a_web_system_with_no_delivery_compiles_with_its_actual_rust_prerequisite ... ok
test complete_committed_valid_artifact_maps_are_unchanged_and_compile ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.70s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7fc4ff76b274d9b9)

running 14 tests
test outcome_event_binding_cannot_capture_the_codec_output_buffer ... ok
test multiple_causes_keep_a_complete_unchanged_plan_and_canonical_order ... ok
test helper_like_event_names_remain_legal_without_the_codec_scope ... ok
test binding_function_cannot_be_hidden_by_the_delivery_event_parameter ... FAILED
test bytes_map_decoder_path_is_borrowed_in_the_supported_codec ... FAILED
test a_string_map_key_keeps_its_compilable_codec ... ok
test a_later_binding_function_cannot_be_hidden_by_a_prior_input_local ... FAILED
test boolean_map_decoder_path_is_borrowed_in_the_supported_codec ... FAILED
test optional_recursion_behind_nested_collections_keeps_its_size_break ... FAILED
test web_component_dependency_cannot_be_hidden_by_the_json_module ... ok
test error_encoder_names_in_separate_outcome_arms_remain_compilable ... ok
test an_unaccepted_outcome_binding_has_no_web_codec_local_scope ... ok
test web_component_dependency_cannot_capture_core_used_by_json_errors ... ok
test a_first_binding_named_input_remains_compilable_on_both_targets ... ok

failures:

---- binding_function_cannot_be_hidden_by_the_delivery_event_parameter stdout ----
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3396995-2
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3396995-13" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3396995-13" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3396995-13/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3396995-13/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3396995-13/crates/demo-system)
error[E0618]: expected function, found `&demo_types::core::Fired`
   --> crates/demo-system/src/lib.rs:139:29
    |
 53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
137 |             SystemEvent::Fired(event) => {
    |                                ----- `event` has type `&demo_types::core::Fired`
138 |                 // `event`: at_least_once, on failure drop.
139 |                 let input = event(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 1 warning
error: could not compile `demo-system` (lib test) due to 1 previous error; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 1 warning (1 duplicate)
error: could not compile `demo-system` (lib) due to 1 previous error; 1 warning emitted

exit status: 101

thread 'binding_function_cannot_be_hidden_by_the_delivery_event_parameter' (3397000) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3396995-13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- bytes_map_decoder_path_is_borrowed_in_the_supported_codec stdout ----
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"


exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.49s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:59
    |
 45 |                     entries0.insert(json::key_bytes(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     ---------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:621:8
    |
621 | pub fn key_bytes(key: &str, at: &str) -> Result<Vec<u8>, DecodeError> {
    |        ^^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bytes(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                           +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'bytes_map_decoder_path_is_borrowed_in_the_supported_codec' (3397002) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3396995-5/generated/web/demo

---- a_later_binding_function_cannot_be_hidden_by_a_prior_input_local stdout ----
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3396995-1
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3396995-14" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3396995-14" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3396995-14/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3396995-14/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3396995-14/crates/demo-system)
error[E0618]: expected function, found `Handle`
   --> crates/demo-system/src/lib.rs:156:29
    |
 65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
151 |                 let input = alpha(event);
    |                     ----- `input` has type `Handle`
...
156 |                 let input = input(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:55:14
   |
55 | pub fn alpha(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:65:14
   |
65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 2 warnings
error: could not compile `demo-system` (lib test) due to 1 previous error; 2 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 2 warnings (2 duplicates)
error: could not compile `demo-system` (lib) due to 1 previous error; 2 warnings emitted

exit status: 101

thread 'a_later_binding_function_cannot_be_hidden_by_a_prior_input_local' (3396997) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3396995-14

---- boolean_map_decoder_path_is_borrowed_in_the_supported_codec stdout ----
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.59s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:58
    |
 45 |                     entries0.insert(json::key_bool(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     --------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:604:8
    |
604 | pub fn key_bool(key: &str, at: &str) -> Result<bool, DecodeError> {
    |        ^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bool(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                          +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'boolean_map_decoder_path_is_borrowed_in_the_supported_codec' (3397001) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3396995-3/generated/web/demo

---- optional_recursion_behind_nested_collections_keeps_its_size_break stdout ----
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.49s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:60:61
    |
 60 |                     entries0.insert(json::key_integer(key0, nested0)?, {
    |                                     -----------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:591:8
    |
591 | pub fn key_integer(key: &str, at: &str) -> Result<i64, DecodeError> {
    |        ^^^^^^^^^^^            --------
help: consider borrowing here
    |
 60 |                     entries0.insert(json::key_integer(key0, &nested0)?, {
    |                                                             +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'optional_recursion_behind_nested_collections_keeps_its_size_break' (3397006) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:115:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3396995-7/generated/web/demo


failures:
    a_later_binding_function_cannot_be_hidden_by_a_prior_input_local
    binding_function_cannot_be_hidden_by_the_delivery_event_parameter
    boolean_map_decoder_path_is_borrowed_in_the_supported_codec
    bytes_map_decoder_path_is_borrowed_in_the_supported_codec
    optional_recursion_behind_nested_collections_keeps_its_size_break

test result: FAILED. 9 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.98s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
     Running tests/go.rs (target/debug/deps/go-65873f08d142fd86)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test emitting_twice_is_byte_identical ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/http.rs (target/debug/deps/http-00497f2eed378b43)

running 9 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test correction_actual_cargo_manifests_keep_their_comment_provenance ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/relations.rs (target/debug/deps/relations-e808695d397e517d)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-7d55f2a670892132)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test grants_are_refused_rather_than_owed ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test the_billing_plan_counts_are_pinned ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test only_the_initial_state_can_be_constructed ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test the_plan_never_names_the_emission_language ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-24af1a90e2d625f3)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 2 targets failed:
    `-p ess-cli --test feasibility_adversary`
    `-p ess-synth --test feasibility_adversary`
```

Exit: 101

### adversary-2-format

Command:

```text
cargo fmt -p ess-synth -p ess-cli --check
```

Output:

```text
```

Exit: 0

### adversary-2-clippy

Command:

```text
cargo clippy --locked --offline -p ess-synth -p ess-cli --all-targets -- -D warnings
```

Output:

```text
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli)
error: `format!(..)` appended to existing `String`
   --> crates/generate/ess-synth/tests/feasibility_adversary.rs:238:9
    |
238 | ...   wiring.push_str(&format!("  - id: {name}\n    when:\n      event: demo.core.Fired\n    invoke:\n      command: demo.core.Handle\n    mapping: {{}}\n    delivery: at_least_once\n    on_failure: drop\n"));
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: you may need to import the `std::fmt::Write` trait
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#format_push_string
    = note: `-D clippy::format-push-string` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::format_push_string)]`
help: consider using `write!` to avoid the extra allocation
    |
238 -         wiring.push_str(&format!("  - id: {name}\n    when:\n      event: demo.core.Fired\n    invoke:\n      command: demo.core.Handle\n    mapping: {{}}\n    delivery: at_least_once\n    on_failure: drop\n"));
238 +         let _ = write!(wiring, "  - id: {name}\n    when:\n      event: demo.core.Fired\n    invoke:\n      command: demo.core.Handle\n    mapping: {{}}\n    delivery: at_least_once\n    on_failure: drop\n");
    |

error: could not compile `ess-synth` (test "feasibility_adversary") due to 1 previous error
```

Exit: 101

### adversary-2-clippy-final

Command:

```text
cargo clippy --locked --offline -p ess-synth -p ess-cli --all-targets -- -D warnings
```

Output:

```text
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
    Finished `dev` profile [unoptimized] target(s) in 0.33s
```

Exit: 0

### adversary-2-format-final

Command:

```text
cargo fmt -p ess-synth -p ess-cli --check
```

Output:

```text
```

Exit: 0

### adversary-2-package-final

Command:

```text
cargo test --locked --offline -p ess-synth -p ess-cli --no-fail-fast
```

Output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth)
    Finished `test` profile [unoptimized] target(s) in 0.83s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-8390c22bcf99a72e)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... FAILED
test a_binding_function_capture_is_refused_before_web_cli_writes ... FAILED

failures:

---- a_binding_function_capture_is_refused_before_rust_cli_writes stdout ----
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "rust" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3435694-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3435694-0/absent"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "crates", "plan.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3435694-0/absent.stdout and .stderr
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "rust" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3435694-0/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3435694-0/existing"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "crates", "plan.json", "sentinel.txt"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-rust-3435694-0/existing.stdout and .stderr

thread 'a_binding_function_capture_is_refused_before_rust_cli_writes' (3435695) panicked at crates/edge/ess-cli/tests/feasibility_adversary.rs:179:5:
binding collision reached CLI writes:
absent: expected exit 1, got exit status: 0
absent: no typed rust failure
absent: destination was created
existing: expected exit 1, got exit status: 0
existing: no typed rust failure
existing: destination gained files
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- a_binding_function_capture_is_refused_before_web_cli_writes stdout ----
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "web" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3435694-1/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3435694-1/absent"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "README.md", "TARGET.md", "bridge.js", "catalog.json", "crates", "index.html", "plan.json", "target.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3435694-1/absent.stdout and .stderr
"/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess" "synthesize" "--target" "web" "--format" "json" "--path" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3435694-1/spec" "--out" "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3435694-1/existing"
exit status: 0; output entries ["Cargo.toml", "PLAN.md", "README.md", "TARGET.md", "bridge.js", "catalog.json", "crates", "index.html", "plan.json", "sentinel.txt", "target.json"]; complete streams: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/edge/ess-cli/../../../target/review-boundaries-5/adversary-cli/pass-2-binding-web-3435694-1/existing.stdout and .stderr

thread 'a_binding_function_capture_is_refused_before_web_cli_writes' (3435696) panicked at crates/edge/ess-cli/tests/feasibility_adversary.rs:179:5:
binding collision reached CLI writes:
absent: expected exit 1, got exit status: 0
absent: no typed web failure
absent: destination was created
existing: expected exit 1, got exit status: 0
existing: no typed web failure
existing: destination gained files


failures:
    a_binding_function_capture_is_refused_before_rust_cli_writes
    a_binding_function_capture_is_refused_before_web_cli_writes

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p ess-cli --test feasibility_adversary`
     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.29s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/target_failure.rs (target/debug/deps/target_failure-2d63596e3e7ff9da)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-e0a1c27a5644dd14)

running 8 tests
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-4a7e263190c9dff9)

running 10 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test a_string_field_offers_no_values ... ok
test every_placed_word_is_an_obligation ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test the_binary_generates_its_own_completions ... ok
test a_placed_view_becomes_a_verb ... ok
test review_second_pass_actual_clap_manifest_retains_complete_comment_admission ... ok
test the_emission_is_deterministic ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/feasibility.rs (target/debug/deps/feasibility-a469b4e3776d4e98)

running 36 tests
test an_empty_domain_cannot_overwrite_the_rust_crate_root ... ok
test a_type_cannot_capture_the_primitive_string_reference ... ok
test a_reserved_type_spelling_is_a_target_limitation ... ok
test component_type_parameters_do_not_shadow_their_port ... ok
test a_mechanical_conversion_cannot_capture_the_from_trait ... ok
test fixed_system_fields_are_reserved_only_in_the_system_scope ... ok
test normalized_fields_share_their_actual_struct_scope ... ok
test additional_optional_wrappers_and_union_size_cycles_refuse ... ok
test normalized_type_collisions_refuse_before_returning_rust ... ok
test path_keyword_repairs_cannot_merge_fields ... ok
test optional_self_recursion_is_a_size_cycle ... ok
test mutual_recursion_is_a_size_cycle ... ok
test outcome_bindings_cannot_capture_an_encoder_called_in_the_same_arm ... ok
test raw_module_identifiers_must_resolve_the_emitted_filename ... ok
test system_types_receive_a_target_result_instead_of_a_missing_owner_panic ... ok
test variant_names_are_checked_after_pascal_normalization ... ok
test required_to_optional_binding_assignment_is_not_a_plain_clone ... ok
test generated_outcome_names_share_the_domain_type_scope ... ok
test system_level_types_refuse_even_when_referenced ... ok
test reserved_module_repairs_are_checked_as_a_final_set ... ok
test generated_port_methods_cannot_capture_a_constructor ... ok
test generated_entity_names_and_markers_use_their_own_scopes ... ok
test collection_indirection_and_acyclic_optionals_compile ... ok
test wire_aliases_are_checked_only_at_an_emitted_codec_surface ... ok
test codec_fragments_are_checked_as_one_final_function_set ... ok
test independent_names_and_existing_package_repairs_compile ... ok
test web_catalog_module_cannot_capture_a_component_dependency ... ok
test web_wire_module_cannot_capture_a_component_dependency ... ok
test distinct_wire_overrides_and_value_union_tag_compile ... ok
test neutral_refused_deliveries_keep_the_partial_web_report_and_compile ... ok
test optional_identity_and_omission_bindings_compile ... ok
test outcome_bindings_in_other_namespaces_or_unused_helpers_compile ... ok
test dependency_helpers_are_reserved_only_where_the_target_uses_them ... ok
test a_web_system_without_any_published_event_has_an_empty_observation ... ok
test a_web_system_with_no_delivery_compiles_with_its_actual_rust_prerequisite ... ok
test complete_committed_valid_artifact_maps_are_unchanged_and_compile ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.75s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-7fc4ff76b274d9b9)

running 14 tests
test outcome_event_binding_cannot_capture_the_codec_output_buffer ... ok
test multiple_causes_keep_a_complete_unchanged_plan_and_canonical_order ... ok
test a_later_binding_function_cannot_be_hidden_by_a_prior_input_local ... FAILED
test helper_like_event_names_remain_legal_without_the_codec_scope ... ok
test web_component_dependency_cannot_be_hidden_by_the_json_module ... ok
test a_string_map_key_keeps_its_compilable_codec ... ok
test optional_recursion_behind_nested_collections_keeps_its_size_break ... FAILED
test boolean_map_decoder_path_is_borrowed_in_the_supported_codec ... FAILED
test bytes_map_decoder_path_is_borrowed_in_the_supported_codec ... FAILED
test binding_function_cannot_be_hidden_by_the_delivery_event_parameter ... FAILED
test error_encoder_names_in_separate_outcome_arms_remain_compilable ... ok
test a_first_binding_named_input_remains_compilable_on_both_targets ... ok
test web_component_dependency_cannot_capture_core_used_by_json_errors ... ok
test an_unaccepted_outcome_binding_has_no_web_codec_local_scope ... ok

failures:

---- a_later_binding_function_cannot_be_hidden_by_a_prior_input_local stdout ----
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3440600-1
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3440600-14" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3440600-14" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3440600-14/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3440600-14/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3440600-14/crates/demo-system)
error[E0618]: expected function, found `Handle`
   --> crates/demo-system/src/lib.rs:156:29
    |
 65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
151 |                 let input = alpha(event);
    |                     ----- `input` has type `Handle`
...
156 |                 let input = input(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:55:14
   |
55 | pub fn alpha(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:65:14
   |
65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 2 warnings
error: could not compile `demo-system` (lib test) due to 1 previous error; 2 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 2 warnings (2 duplicates)
error: could not compile `demo-system` (lib) due to 1 previous error; 2 warnings emitted

exit status: 101

thread 'a_later_binding_function_cannot_be_hidden_by_a_prior_input_local' (3440602) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:116:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3440600-14
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- optional_recursion_behind_nested_collections_keeps_its_size_break stdout ----
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.44s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:60:61
    |
 60 |                     entries0.insert(json::key_integer(key0, nested0)?, {
    |                                     -----------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:591:8
    |
591 | pub fn key_integer(key: &str, at: &str) -> Result<i64, DecodeError> {
    |        ^^^^^^^^^^^            --------
help: consider borrowing here
    |
 60 |                     entries0.insert(json::key_integer(key0, &nested0)?, {
    |                                                             +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'optional_recursion_behind_nested_collections_keeps_its_size_break' (3440611) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:116:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3440600-7/generated/web/demo

---- boolean_map_decoder_path_is_borrowed_in_the_supported_codec stdout ----
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.54s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:58
    |
 45 |                     entries0.insert(json::key_bool(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     --------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:604:8
    |
604 | pub fn key_bool(key: &str, at: &str) -> Result<bool, DecodeError> {
    |        ^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bool(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                          +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'boolean_map_decoder_path_is_borrowed_in_the_supported_codec' (3440606) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:116:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3440600-4/generated/web/demo

---- bytes_map_decoder_path_is_borrowed_in_the_supported_codec stdout ----
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/rust/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.54s

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

     Locking 1 package to latest compatible version

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/web/demo" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target" "--target" "wasm32-unknown-unknown"

    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/web/demo/crates/demo-web)
error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:59
    |
 45 |                     entries0.insert(json::key_bytes(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     ---------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:621:8
    |
621 | pub fn key_bytes(key: &str, at: &str) -> Result<Vec<u8>, DecodeError> {
    |        ^^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bytes(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                           +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `demo-web` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `demo-web` (lib) due to 1 previous error

exit status: 101

thread 'bytes_map_decoder_path_is_borrowed_in_the_supported_codec' (3440607) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:116:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3440600-3/generated/web/demo

---- binding_function_cannot_be_hidden_by_the_delivery_event_parameter stdout ----
source witness: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3440600-2
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3440600-13" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache

exit status: 0
cd "/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3440600-13" && env -u CARGO_TARGET_DIR "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "check" "--locked" "--offline" "--workspace" "--all-targets" "--target-dir" "target"

    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3440600-13/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3440600-13/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3440600-13/crates/demo-system)
error[E0618]: expected function, found `&demo_types::core::Fired`
   --> crates/demo-system/src/lib.rs:139:29
    |
 53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
137 |             SystemEvent::Fired(event) => {
    |                                ----- `event` has type `&demo_types::core::Fired`
138 |                 // `event`: at_least_once, on failure drop.
139 |                 let input = event(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 1 warning
error: could not compile `demo-system` (lib test) due to 1 previous error; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 1 warning (1 duplicate)
error: could not compile `demo-system` (lib) due to 1 previous error; 1 warning emitted

exit status: 101

thread 'binding_function_cannot_be_hidden_by_the_delivery_event_parameter' (3440605) panicked at crates/generate/ess-synth/tests/feasibility_adversary.rs:116:5:
admitted generated workspace failed its actual compiler: /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/crates/generate/ess-synth/../../../target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3440600-13


failures:
    a_later_binding_function_cannot_be_hidden_by_a_prior_input_local
    binding_function_cannot_be_hidden_by_the_delivery_event_parameter
    boolean_map_decoder_path_is_borrowed_in_the_supported_codec
    bytes_map_decoder_path_is_borrowed_in_the_supported_codec
    optional_recursion_behind_nested_collections_keeps_its_size_break

test result: FAILED. 9 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s

error: test failed, to rerun pass `-p ess-synth --test feasibility_adversary`
     Running tests/go.rs (target/debug/deps/go-65873f08d142fd86)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/http.rs (target/debug/deps/http-00497f2eed378b43)

running 9 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test correction_actual_cargo_manifests_keep_their_comment_provenance ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/relations.rs (target/debug/deps/relations-e808695d397e517d)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-7d55f2a670892132)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test grants_are_refused_rather_than_owed ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test only_the_initial_state_can_be_constructed ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test the_billing_plan_counts_are_pinned ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test the_plan_never_names_the_emission_language ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-24af1a90e2d625f3)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test emitting_twice_is_byte_identical ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 2 targets failed:
    `-p ess-cli --test feasibility_adversary`
    `-p ess-synth --test feasibility_adversary`
```

Exit: 101

## 4. Findings, reachability and exact-base origin

Subject remains `a2ff04d70f70a691288b661e4d027eee5da74e42`; opening base is `4b66aac7b608b1deee9de88942390d4a6c5ec745`. The copied immutable opening CLI was used locally, not another worktree's evolving binary. SHA256 `69f63f3addf84e59982701d8cfc6254b00ce34db378604b076863f3a5d330bf3`; exact provenance is retained in `base-cli-for-pass-2/provenance.json`. Every origin below is backed by actual opening emitter plus compiler execution.

| File:line | Category | Severity | Verdict | Origin | Message |
|---|---|---|---|---|---|
| crates/generate/ess-synth/src/rust/feasibility.rs:583 | acceptance | blocker | NEEDS-CHANGE | pre-existing | Binding feasibility omits delivery-arm locals, allowing event and later input transformation calls to be captured while Rust and Web CLI output succeeds. |
| crates/generate/ess-synth/src/rust/wire.rs:670 | acceptance | blocker | NEEDS-CHANGE | pre-existing | Supported Integer, Boolean and Bytes map decoders pass an owned nested path to a borrowed parser parameter, so admitted Web output fails compilation. |

Finding 1 measured: both compiler cases are admitted through `synthesize` then fail E0618 in actual generated `demo-system`; CLI Rust and Web each return0 for absent and sentinel-existing output and emit successful artifacts. The original sentinel bytes remain unchanged, but directories gain files. Rust adds four root entries (five including sentinel); Web adds ten (eleven including sentinel). The preflight inventories the transformation only in `system values` at feasibility.rs:583–590, while the renderer calls it unqualified at system.rs:881. `deliver_fn` binds the source event as `event` at system.rs:825–827 and invokes reacting bindings sequentially; each delivery introduces `input`. The `[alpha,input]` witness demonstrates the precise prior-local condition. A first `input` binding compiles in both targets and also on the exact base: blanket reservation of that name would violate the positive control.

Finding 1 reaches: normal compiler-admitted `ess/1` documents, public `synthesize`/`synthesize_for`, checked `rust::workspace` rendering through the facade, and actual `ess synthesize --target rust|web --path ... --out ... --format json`. CLI main.rs:2391–2405 returns early only for typed Err; the missed allocation arrives as Ok and reaches `write_artifacts`. No manually constructed plan, invalid source, live service or external consumer assumption is involved. Direct Rust/ Web functions share the reviewed checked allocation; the new assertions exercise the facades and CLI, not a separate direct-API execution.

Finding 2 measured: pure Rust compiles all three key cases, including recursive Node behind Optional/Map/List. Web compiler E0308 names `key_integer`, `key_bool`, or `key_bytes`, each requiring `at: &str` while emitted `nested0` is String. The source at wire.rs:670 passes `&nested` as a renderer string argument, emitting the bare local token; its neighboring value decoder explicitly emits `&nested0`. String-map codecs compile. The new cases require supported map emission to remain admitted; they must not be weakened into blanket map refusal. This is a generated decoder defect, separate from the correct collection size-cycle admission. No claim about numeric precision, map semantics, replay or universal target support is made.

Finding 2 reaches: actual Web synthesis and its public type codecs, even a catalog-only source with no component; actual current CLI Web emission of the exact nested-map source was additionally executed into absent and sentinel-existing output and exits0. The source-shared HTTP codec also calls the same wire renderer (`rust/http.rs::server_crate`), but no separate fresh HTTP map compile is claimed here. All three Web map witnesses were run against the exact opening CLI. Opening source also has its separately known missing catalog-only INPUT/OUTPUT E0425 errors; the same E0308 parser failure is visible alongside those, so it is not attributed to the new catalog-only repair.

Five emitted failing source files are byte-identical opening/candidate, verified by cmp exit0 and SHA256: binding-event system lib `47b539fa4092b1412d42d95d7e49822acd0c939abf3a23dc4d926d5e40bd938d`; later-input system lib `e8a02122c1972cde12b22297cf9eb230aa5dc438d71c1545f47e937646c80eb5`; nested Integer map wire `6f3c55d32373bd0d005319c2ab9d20ff7d76079c7f67cf8f831635080dd888ce`; Boolean map wire `a7e6d1abe2fcc5e4a07a849e04444f082820e7e81f6ec7240d009be483446998`; Bytes map wire `2c6e410ac6b29ed3ae888f2abd64d3e2cb3e244e1c8cc87e0c3cc7a15c61481a`. This is an exact file comparison in addition to real base execution, not an inferred origin from source similarity.

### Original finding comparison

- Resolved: pass-1 `crates/generate/ess-synth/src/web/mod.rs:304`, acceptance/blocker/NEEDS-CHANGE/pre-existing, message: “Web feasibility admits a json component whose dependency is hidden by the generated json module, so the CLI writes a workspace that fails wasm32 compilation.” Both retained generated-compiler and CLI cases now pass unchanged; catalog/wire class controls and `core` Rust/Web control also pass in the package suite.
- Resolved: pass-1 `crates/generate/ess-synth/src/rust/feasibility.rs:845`, acceptance/blocker/NEEDS-CHANGE/pre-existing, message: “Codec feasibility omits outcome pattern bindings, allowing an Out event to shadow the output buffer and the CLI to write compiler-invalid Rust HTTP code.” Both retained compiler and CLI cases now pass unchanged; Out without codecs and harmless/unused helper locals remain green.
- Carried: none of those two original signatures remains red. Newly measured this pass: the two table rows above, both pre-existing by actual base execution. Root computes and records the formal signature ledger; this report does not mutate it.

### Exact opening CLI, generated compiler and byte-comparison records


### target/review-boundaries-5/adversary-2-base-binding-event-rust-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo/crates/demo-system)
error[E0618]: expected function, found `&demo_types::core::Fired`
   --> crates/demo-system/src/lib.rs:139:29
    |
 53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
137 |             SystemEvent::Fired(event) => {
    |                                ----- `event` has type `&demo_types::core::Fired`
138 |                 // `event`: at_least_once, on failure drop.
139 |                 let input = event(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn event(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 1 warning
error: could not compile `demo-system` (lib test) due to 1 previous error; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 1 warning (1 duplicate)
error: could not compile `demo-system` (lib) due to 1 previous error; 1 warning emitted
```

Exit: 101

### target/review-boundaries-5/adversary-2-base-binding-event-rust-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target rust --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3304630-0' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo'
```

Output:

```text
7 capabilities: 6 generated, 1 obligation(s), 0 refused
12 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-binding-event-rust-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-binding-input-first-rust-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo/crates/demo-system)
warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:53:14
   |
53 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `demo-system` (lib test) generated 1 warning (run `cargo fix --lib -p demo-system --tests` to apply 1 suggestion)
warning: `demo-system` (lib) generated 1 warning (1 duplicate)
    Finished `dev` profile [unoptimized] target(s) in 0.54s
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-binding-input-first-rust-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target rust --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3308777-0' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo'
```

Output:

```text
7 capabilities: 6 generated, 1 obligation(s), 0 refused
12 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-binding-input-first-rust-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-first/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-binding-input-later-rust-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo/crates/demo-types)
    Checking worker v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo/crates/worker)
    Checking demo-system v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo/crates/demo-system)
error[E0618]: expected function, found `Handle`
   --> crates/demo-system/src/lib.rs:156:29
    |
 65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
    | ------------------------------------------------------------------------- this function of the same name is available here, but it's shadowed by the local binding
...
151 |                 let input = alpha(event);
    |                     ----- `input` has type `Handle`
...
156 |                 let input = input(event);
    |                             ^^^^^-------
    |                             |
    |                             call expression requires function

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:55:14
   |
55 | pub fn alpha(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `event`
  --> crates/demo-system/src/lib.rs:65:14
   |
65 | pub fn input(event: &demo_types::core::Fired) -> demo_types::core::Handle {
   |              ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

For more information about this error, try `rustc --explain E0618`.
warning: `demo-system` (lib test) generated 2 warnings
error: could not compile `demo-system` (lib test) due to 1 previous error; 2 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-system` (lib) generated 2 warnings (2 duplicates)
error: could not compile `demo-system` (lib) due to 1 previous error; 2 warnings emitted
```

Exit: 101

### target/review-boundaries-5/adversary-2-base-binding-input-later-rust-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target rust --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-binding-source-3308583-0' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo'
```

Output:

```text
9 capabilities: 8 generated, 1 obligation(s), 0 refused
12 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-binding-input-later-rust-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-boolean-map-rust-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.17s
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-boolean-map-rust-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target rust --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/rust/demo'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
7 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/rust/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-boolean-map-rust-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-boolean-map-web-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target --target wasm32-unknown-unknown
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo/crates/demo-web)
error[E0425]: cannot find value `INPUT` in this scope
  --> crates/demo-web/src/lib.rs:87:5
   |
87 |     INPUT.with(|held| {
   |     ^^^^^ not found in this scope

error[E0425]: cannot find value `INPUT` in this scope
  --> crates/demo-web/src/lib.rs:99:19
   |
99 |     let request = INPUT.with(|held| String::from_utf8_lossy(&held.borrow()).into_owned());
   |                   ^^^^^ not found in this scope

error[E0425]: cannot find value `OUTPUT` in this scope
   --> crates/demo-web/src/lib.rs:101:5
    |
101 |     OUTPUT.with(|held| {
    |     ^^^^^^ not found in this scope

error[E0425]: cannot find value `OUTPUT` in this scope
   --> crates/demo-web/src/lib.rs:111:5
    |
111 |     OUTPUT.with(|held| held.borrow().len() as u32)
    |     ^^^^^^ not found in this scope

warning: unused import: `std::cell::RefCell`
  --> crates/demo-web/src/lib.rs:36:5
   |
36 | use std::cell::RefCell;
   |     ^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:58
    |
 45 |                     entries0.insert(json::key_bool(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     --------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:604:8
    |
604 | pub fn key_bool(key: &str, at: &str) -> Result<bool, DecodeError> {
    |        ^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bool(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                          +

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
warning: `demo-web` (lib) generated 1 warning
error: could not compile `demo-web` (lib) due to 5 previous errors; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-web` (lib test) generated 1 warning (1 duplicate)
error: could not compile `demo-web` (lib test) due to 5 previous errors; 1 warning emitted
```

Exit: 101

### target/review-boundaries-5/adversary-2-base-boolean-map-web-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target web --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
14 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-boolean-map-web-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
     Locking 1 package to latest compatible version
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-bytes-map-rust-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.23s
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-bytes-map-rust-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target rust --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/rust/demo'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
7 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/rust/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-bytes-map-rust-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-bytes-map-web-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target --target wasm32-unknown-unknown
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo/crates/demo-web)
error[E0425]: cannot find value `INPUT` in this scope
  --> crates/demo-web/src/lib.rs:87:5
   |
87 |     INPUT.with(|held| {
   |     ^^^^^ not found in this scope

error[E0425]: cannot find value `INPUT` in this scope
  --> crates/demo-web/src/lib.rs:99:19
   |
99 |     let request = INPUT.with(|held| String::from_utf8_lossy(&held.borrow()).into_owned());
   |                   ^^^^^ not found in this scope

error[E0425]: cannot find value `OUTPUT` in this scope
   --> crates/demo-web/src/lib.rs:101:5
    |
101 |     OUTPUT.with(|held| {
    |     ^^^^^^ not found in this scope

error[E0425]: cannot find value `OUTPUT` in this scope
   --> crates/demo-web/src/lib.rs:111:5
    |
111 |     OUTPUT.with(|held| held.borrow().len() as u32)
    |     ^^^^^^ not found in this scope

warning: unused import: `std::cell::RefCell`
  --> crates/demo-web/src/lib.rs:36:5
   |
36 | use std::cell::RefCell;
   |     ^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:45:59
    |
 45 |                     entries0.insert(json::key_bytes(key0, nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                     ---------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:621:8
    |
621 | pub fn key_bytes(key: &str, at: &str) -> Result<Vec<u8>, DecodeError> {
    |        ^^^^^^^^^            --------
help: consider borrowing here
    |
 45 |                     entries0.insert(json::key_bytes(key0, &nested0)?, json::text_at(element0, &nested0, "a string")?.to_owned());
    |                                                           +

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
warning: `demo-web` (lib) generated 1 warning
error: could not compile `demo-web` (lib) due to 5 previous errors; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-web` (lib test) generated 1 warning (1 duplicate)
error: could not compile `demo-web` (lib test) due to 5 previous errors; 1 warning emitted
```

Exit: 101

### target/review-boundaries-5/adversary-2-base-bytes-map-web-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target web --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
14 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-bytes-map-web-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
     Locking 1 package to latest compatible version
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-nested-map-rust-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/rust/demo/crates/demo-types)
    Finished `dev` profile [unoptimized] target(s) in 0.25s
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-nested-map-rust-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target rust --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map-spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/rust/demo'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
7 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/rust/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-nested-map-rust-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/rust/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-nested-map-web-check

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo' && env -u CARGO_TARGET_DIR cargo check --locked --offline --workspace --all-targets --target-dir target --target wasm32-unknown-unknown
```

Output:

```text
    Checking demo-types v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/rust/demo/crates/demo-types)
    Checking demo-web v1.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo/crates/demo-web)
error[E0425]: cannot find value `INPUT` in this scope
  --> crates/demo-web/src/lib.rs:87:5
   |
87 |     INPUT.with(|held| {
   |     ^^^^^ not found in this scope

error[E0425]: cannot find value `INPUT` in this scope
  --> crates/demo-web/src/lib.rs:99:19
   |
99 |     let request = INPUT.with(|held| String::from_utf8_lossy(&held.borrow()).into_owned());
   |                   ^^^^^ not found in this scope

error[E0425]: cannot find value `OUTPUT` in this scope
   --> crates/demo-web/src/lib.rs:101:5
    |
101 |     OUTPUT.with(|held| {
    |     ^^^^^^ not found in this scope

error[E0425]: cannot find value `OUTPUT` in this scope
   --> crates/demo-web/src/lib.rs:111:5
    |
111 |     OUTPUT.with(|held| held.borrow().len() as u32)
    |     ^^^^^^ not found in this scope

warning: unused import: `std::cell::RefCell`
  --> crates/demo-web/src/lib.rs:36:5
   |
36 | use std::cell::RefCell;
   |     ^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0308]: mismatched types
   --> crates/demo-web/src/wire.rs:60:61
    |
 60 |                     entries0.insert(json::key_integer(key0, nested0)?, {
    |                                     -----------------       ^^^^^^^ expected `&str`, found `String`
    |                                     |
    |                                     arguments to this function are incorrect
    |
note: function defined here
   --> crates/demo-web/src/json.rs:591:8
    |
591 | pub fn key_integer(key: &str, at: &str) -> Result<i64, DecodeError> {
    |        ^^^^^^^^^^^            --------
help: consider borrowing here
    |
 60 |                     entries0.insert(json::key_integer(key0, &nested0)?, {
    |                                                             +

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
warning: `demo-web` (lib) generated 1 warning
error: could not compile `demo-web` (lib) due to 5 previous errors; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
warning: `demo-web` (lib test) generated 1 warning (1 duplicate)
error: could not compile `demo-web` (lib test) due to 5 previous errors; 1 warning emitted
```

Exit: 101

### target/review-boundaries-5/adversary-2-base-nested-map-web-emit

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/base-cli-for-pass-2/ess' synthesize --target web --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map-spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
14 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo
```

Exit: 0

### target/review-boundaries-5/adversary-2-base-nested-map-web-lock

Command:

```text
cd '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo' && env -u CARGO_TARGET_DIR cargo generate-lockfile --offline
```

Output:

```text
     Locking 1 package to latest compatible version
```

Exit: 0

### target/review-boundaries-5/adversary-2-comparison-binding-event

Command:

```text
cmp target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo/crates/demo-system/src/lib.rs target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1/crates/demo-system/src/lib.rs
```

Output:

```text
47b539fa4092b1412d42d95d7e49822acd0c939abf3a23dc4d926d5e40bd938d  target/review-boundaries-5/adversary-2-base/binding-event/generated/rust/demo/crates/demo-system/src/lib.rs
47b539fa4092b1412d42d95d7e49822acd0c939abf3a23dc4d926d5e40bd938d  target/review-boundaries-5/adversary-compiler/pass-2-binding-event-3304630-1/crates/demo-system/src/lib.rs
```

Exit: 0

### target/review-boundaries-5/adversary-2-comparison-binding-input-later

Command:

```text
cmp target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo/crates/demo-system/src/lib.rs target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1/crates/demo-system/src/lib.rs
```

Output:

```text
e8a02122c1972cde12b22297cf9eb230aa5dc438d71c1545f47e937646c80eb5  target/review-boundaries-5/adversary-2-base/binding-input-later/generated/rust/demo/crates/demo-system/src/lib.rs
e8a02122c1972cde12b22297cf9eb230aa5dc438d71c1545f47e937646c80eb5  target/review-boundaries-5/adversary-compiler/pass-2-binding-input-later-3308583-1/crates/demo-system/src/lib.rs
```

Exit: 0

### target/review-boundaries-5/adversary-2-comparison-boolean-map-wire

Command:

```text
cmp target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo/crates/demo-web/src/wire.rs target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/web/demo/crates/demo-web/src/wire.rs
```

Output:

```text
a7e6d1abe2fcc5e4a07a849e04444f082820e7e81f6ec7240d009be483446998  target/review-boundaries-5/adversary-2-base/boolean-map/generated/web/demo/crates/demo-web/src/wire.rs
a7e6d1abe2fcc5e4a07a849e04444f082820e7e81f6ec7240d009be483446998  target/review-boundaries-5/adversary-compiler/pass-2-map-Boolean-3371460-0/generated/web/demo/crates/demo-web/src/wire.rs
```

Exit: 0

### target/review-boundaries-5/adversary-2-comparison-bytes-map-wire

Command:

```text
cmp target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo/crates/demo-web/src/wire.rs target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/web/demo/crates/demo-web/src/wire.rs
```

Output:

```text
2c6e410ac6b29ed3ae888f2abd64d3e2cb3e244e1c8cc87e0c3cc7a15c61481a  target/review-boundaries-5/adversary-2-base/bytes-map/generated/web/demo/crates/demo-web/src/wire.rs
2c6e410ac6b29ed3ae888f2abd64d3e2cb3e244e1c8cc87e0c3cc7a15c61481a  target/review-boundaries-5/adversary-compiler/pass-2-map-Bytes-3371684-0/generated/web/demo/crates/demo-web/src/wire.rs
```

Exit: 0

### target/review-boundaries-5/adversary-2-comparison-nested-map-wire

Command:

```text
cmp target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo/crates/demo-web/src/wire.rs target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/web/demo/crates/demo-web/src/wire.rs
```

Output:

```text
6f3c55d32373bd0d005319c2ab9d20ff7d76079c7f67cf8f831635080dd888ce  target/review-boundaries-5/adversary-2-base/nested-map/generated/web/demo/crates/demo-web/src/wire.rs
6f3c55d32373bd0d005319c2ab9d20ff7d76079c7f67cf8f831635080dd888ce  target/review-boundaries-5/adversary-compiler/pass-2-map-recursion-3309632-0/generated/web/demo/crates/demo-web/src/wire.rs
```

Exit: 0

### target/review-boundaries-5/adversary-2-candidate-map-absent

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess' synthesize --target web --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map-spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-candidate-map/absent'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
14 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-candidate-map/absent
```

Exit: 0

Destination inventory:

```text
Cargo.toml
PLAN.md
README.md
TARGET.md
absent
bridge.js
catalog.json
crates
index.html
plan.json
target.json
```

### target/review-boundaries-5/adversary-2-candidate-map-existing

Command:

```text
'/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/debug/ess' synthesize --target web --format text --path '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-base/nested-map-spec' --out '/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-candidate-map/existing'
```

Output:

```text
1 capabilities: 1 generated, 0 obligation(s), 0 refused
14 artifact(s), written to /home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5/adversary-2-candidate-map/existing
```

Exit: 0

Destination inventory:

```text
Cargo.toml
PLAN.md
README.md
TARGET.md
bridge.js
catalog.json
crates
existing
index.html
plan.json
sentinel.txt
target.json
```

## 5. Attacks that did not break

- The original json module and Out codec regression assertions pass unchanged in both package runs; helper and partial-target controls are preserved.
- Per-arm encoder helper accounting admits an identically named value when only another arm calls that helper. The valid fixture compiles as an actual Rust HTTP workspace.
- Missing command acceptors remain ordinary partial Web accounting; an unpresented Out command does not create a false local-codec refusal.
- A first transformation called input compiles on both candidate targets and on opening Rust. Binding repair must account for actual lexical ordering.
- The nested recursive source is valid and its pure Rust representation compiles, as do the nonrecursive primitive-key map representations; String map Web codecs compile. These narrow the decoder repair without broadening target refusal.
- All 190 existing package assertions, including canonical error/unchanged-plan checks, direct checked APIs, historical generated-byte controls and corrected empty-event/no-delivery/catalog-only Web cases, pass. This is the package runner's result; it is not an independent/human approval or a claim about unexecuted external consumers.

## 6. Resources, preserved evidence and relinquishment

All pass-2 fixtures and output copies are in this managed unit. Scratch root: `/home/timo/.local/state/worktree/trees/b10x/ess/review-rust-target-feasibility/target/review-boundaries-5`. New paths use `adversary-2-*`, `adversary-compiler/pass-2-*`, `adversary-cli/pass-2-*`, `adversary-2-base` and `adversary-2-candidate-map`. Existing pass-1 report SHA256 remains `344184edfa8144f2f8331dd690c173810babce29e0ca9f5225a90415b1915723`, and correction report remains the assigned hash. The opening binary and provenance copy were only read and invoked.

Each generated Cargo invocation used its own fixture target. Package Cargo used this unit's target; TMPDIR was `$PWD/target`; RUSTC_WRAPPER `/usr/bin/sccache`; SCCACHE_SERVER_UDS `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock`; CARGO_INCREMENTAL=0; CARGO_PROFILE_DEV_DEBUG=0; CARGO_PROFILE_TEST_DEBUG=0; CARGO_CACHE_RUSTC_INFO=0; CARGO_NET_OFFLINE=true. No CARGO_TARGET_DIR was set, no target shared and no cache-server lifecycle operation was performed. The coordinator-owned cache service was used through its prescribed socket; no out-of-tree source/scratch path was written by this pass.

New pass-2 fixture/destination roots total 7,236,249 apparent bytes at the recorded measurement (generated targets included). The full assigned review scratch, including prior passes and the opening binary, measured 150,216,194 apparent bytes before this report tail. Those measurements are not a claim that the package build target is absent. Free disk was 41 GiB before pass-2 builds and 15 GiB after they ended, above the assigned 8 GiB floor; the coordinator was informed. No cleanup occurred.

Artifacts retained:

- `adversary-2-tests.patch` and `adversary-2-source-hashes.txt`: exact test delta and immutable prior report/binary hashes. Final synth test SHA256 `e99e89f66733454af836efffd274e35ce2c33186fe543d081ae6a4f99f2fd2d9`; CLI test SHA256 `ae249bfbf342988f05e0d9c443cac1255481fa6ce23c4c4d62bbb5bea8acf096`.
- `adversary-2-fixture-hashes.txt`: 1,364 retained non-target fixture/output files with exact hashes, SHA256 `a775ea0151dfd10b56e830efaae50103c7e92fea27a6be3cbad97ead5a535c58`. Includes each CLI child's complete success-artifact stdout/stderr and all emitted sources. Target binaries are retained on disk, not mislabeled generated source.
- `adversary-2-evidence.sha256`: command/log/exit/metadata/patch hashes, SHA256 `cc37886860c80f6ef593ce5c3e31b36825b01e2c2164fe89e655bedc67097ba3`.
- `adversary-2-timing.txt`: actual log modification times and bytes. First isolated case finished 2026-09-05 23:20:05 +0200; first package finished 23:24:40; final strict Clippy finished 23:26:20; final package finished 23:26:30. These are observed timestamps, not an invented total active-work duration.
- Every `.command`, `.log`, `.exit`, compiler source, destination inventory, sentinel and byte-comparison record cited above remains in assigned scratch. Logs are complete even where a terminal preview was bounded.

Every path written outside the worktree: **none**. All test/build/CLI processes launched by this pass have exited. No long-running process remains; the coordinator-owned cache server is not this pass's process. The tree remains at the frozen subject with only the two uncommitted test additions. No approval, production correction, publication, store mutation, pin upgrade or source-driven-preview review is claimed. The coordinator owns immutable recording, bounded final repairs/gates and lifecycle cleanup. This final full attack is complete; all writes and processes are relinquished on handoff.

```findings
- file: crates/generate/ess-synth/src/rust/feasibility.rs
  line: 583
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: Binding feasibility omits delivery-arm locals, allowing event and later input transformation calls to be captured while Rust and Web CLI output succeeds.
- file: crates/generate/ess-synth/src/rust/wire.rs
  line: 670
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: Supported Integer, Boolean and Bytes map decoders pass an owned nested path to a borrowed parser parameter, so admitted Web output fails compilation.
```
