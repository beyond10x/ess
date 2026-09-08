---
format: aep.planning-md/1
id: review-result:output-ownership-adversary-wave19-pass2
kind: review-result
status: active
title: Output ownership adversary, final pass 2
relations:
- reviews: story:review-output-ownership
revision: 1
---
unit: story:review-output-ownership, final pass2, f00467883985a04caf72bd9364642252212a56a6 plus one additive test delta
verdict: nothing found
cases: executed 4→7, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 write surfaces; 12 retained fixture roots and shared Cargo/lease bookkeeping paths listed below
needs-coordinator: run the new native Mac branches and final integration gate, record this report, integrate and retain the assigned tree

`git --no-pager diff --stat`:

```text
 .../ess-cli/tests/output_ownership_adversary.rs    | 249 +++++++++++++++++++++
 1 file changed, 249 insertions(+)
```

Only `crates/edge/ess-cli/tests/output_ownership_adversary.rs` changed: 249 appended lines, three new cases and a unique fixture helper. The original 230-line prefix retains SHA256 a2fd55ec049f1ad205b114e1053bf868e0912bb142ff2b74cc3ec42e73e88e77. Final file SHA256 c1569ad11cf62484763b719ee4076453a40d440257a458a610179a17321eb8f4. The exact patch written before the first execution equals the final patch byte-for-byte (SHA256 4ebf4c532fd294c4887b2ab1c39edca94e32425492ee6c94ac7038c873961eeb). No original assertion, case, source implementation, Git index/ref or planning artifact was changed.

2. Added cases and their first actual executions

The before count is the four completed cases in the corrected implementor's full CLI package, not a pre-attack suite run. That package report is /home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-wave19/target/review-output-ownership-wave19/native-correction-final-report.md, SHA256 7d5505f6343a35fc8b41e9bb8920a9efc815ad7f70843345fed9c5359130e026. Its complete retained package reports 391 passed across 45 targets; the new local target below supplies the 4→7 count. No historical package or transaction/admission cut matrix was rerun for this attack.

The current AGENTS.md, full exact adversary charter and pass2 brief, active acceptance and complete selected ownership design were read. The review reused unchanged pass1 source/caller/test readings, then read the complete ownership correction, new native admission implementation, publication/adoption callers, added correction/admission tests and affected protocol/route assertions. The final implementor report's semantic sections and original runner summaries, native refusals and native branch outcomes supplied the prior-state context; its full raw historical streams remain at the exact pinned path. The full unit base is b7a0303bd4ca771a409449a0a2b7c59efaf5cffc; f004678 includes the separate already-main synthesis allocation change, which was not attacked as an output-ownership defect. The unchanged pass1 report is retained at target/review-output-ownership-adversary-wave19/report.md, SHA256 28ff2871068e9ddbcfda8bfca33a40d6968749959a6ad7b4e3814e0ffdd6ed8a. Its current AEP review-result body equals that original report byte-for-byte; only the coordinator wrote the AEP record.

All commands ran in /home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19. All Cargo invocations used the same explicit environment below, their own default target, offline locked dependencies and one producer. There were two Rust jobs and two test threads. The existing Cargo configuration supplies one native lld flag; no duplicate flag was added.

```text
export PATH=/home/timo/.cache/ess-review/2026-09-06-resume/wave19-retained-inputs/rust-toolchain/bin:$PATH
export RUSTC=/home/timo/.cache/ess-review/2026-09-06-resume/wave19-retained-inputs/rust-toolchain/bin/rustc
export RUSTDOC=/home/timo/.cache/ess-review/2026-09-06-resume/wave19-retained-inputs/rust-toolchain/bin/rustdoc
export RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_TARGET_DIR
for e19_flag in ${!CARGO_TARGET_@}; do
  case "$e19_flag" in *_RUSTFLAGS) unset "$e19_flag";; esac
done
export CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export TMPDIR=/home/timo/.cache/e19-adversary-tmp
```

- `interrupted_nested_admission_stays_opaque_through_cli_retirement`, test line 245: creates a real nested admission orphan by injecting an error immediately after its native mkdir. It checks every prior path/byte/mode, then drives actual compose regeneration, selected stale retirement and two explicit recoveries. Opaque nested state-like bytes survive, retired files disappear, and a later directory-to-file transition refuses without changing the settled snapshot. First and final outcomes green. The injected error is a callback error, not a measured process death; the old process-cut matrix remains separate. The added opaque bytes are deliberate fixture content, not claimed to have been written by the publisher.
- `native_alias_refusal_preserves_the_owned_file_before_a_shape_transition`, test line 323: measures case and normalization lookup behavior on this filesystem, then combines a selected edited 0400 companion-to-client transition with both requested companions. Actual aliases must refuse before any state/output change; distinct names must publish the entire selected result and preserve the authored neighbor. It checks repeated explicit recovery. First and final outcomes green. Both pairs actually returned aliases=false here; the new aliases=true branch has not run locally.
- `exact_native_file_adoption_in_an_enrolled_readonly_root_needs_no_probe_write`, test line 383: real schema generation creates the exact 255-byte Unicode filename reference and a different existing standalone owner at the target. Metadata-only adoption into that already enrolled 0555 root must preserve the matching 0400 file, other owner, authored neighbor, reference bytes/modes and root mode. Exact repeated adoption and two recoveries remain unchanged. First and final outcomes green. The 255-byte length is a concrete native filename case, not a claimed portable filename limit.

Every new case existed before its own first isolated run; each runner executed exactly one case, with six filtered out. No first run was red, and no red output or failing mutant is claimed.

First isolated command (2026-09-08T12:21:58Z to 2026-09-08T12:22:11Z):

```text
cargo test --offline --locked -p ess-cli --test output_ownership_adversary interrupted_nested_admission_stays_opaque_through_cli_retirement -- --exact --nocapture
```

stdout, verbatim:

```text

running 1 test
retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-147772-0
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled; 3 Rust client artifact(s) written to anchor/client

stderr:

nested admission interruption: Err(interrupt after creating the nested admission namespace)
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/final.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/final.json

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 1
stdout:

stderr:
error: authored descendant blocks owned directory/file transition: client

test interrupted_nested_admission_stays_opaque_through_cli_retirement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.70s

```

stderr, verbatim:

```text
   Compiling ess-synth v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/generate/ess-synth)
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 12.23s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)
```

Actual exit status: 0.

First isolated command (2026-09-08T12:22:54Z to 2026-09-08T12:22:55Z):

```text
cargo test --offline --locked -p ess-cli --test output_ownership_adversary native_alias_refusal_preserves_the_owned_file_before_a_shape_transition -- --exact --nocapture
```

stdout, verbatim:

```text

running 1 test
retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-151532-0
native transition pair "É.json"/"é.json": aliases=false
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client", "--out", "anchor/É.json", "--client-plan-out", "anchor/é.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/É.json; client plan written to anchor/é.json; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-151532-1
native transition pair "é.json"/"e\u{301}.json": aliases=false
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client", "--out", "anchor/é.json", "--client-plan-out", "anchor/e\u{301}.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/é.json; client plan written to anchor/é.json; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

test native_alias_refusal_preserves_the_owned_file_before_a_shape_transition ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 1.14s

```

stderr, verbatim:

```text
    Finished `test` profile [unoptimized] target(s) in 0.14s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)
```

Actual exit status: 0.

First isolated command (2026-09-08T12:23:25Z to 2026-09-08T12:23:26Z):

```text
cargo test --offline --locked -p ess-cli --test output_ownership_adversary exact_native_file_adoption_in_an_enrolled_readonly_root_needs_no_probe_write -- --exact --nocapture
```

stdout, verbatim:

```text

running 1 test
retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-155291-0
CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "reference/éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts"]: status=exit status: 0
stdout:
wrote reference/éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts from record.schema.json

stderr:

CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "anchor/other.ts"]: status=exit status: 0
stdout:
wrote anchor/other.ts from record.schema.json

stderr:

CLI ["output", "adopt", "--ownership-root", "anchor", "--from", "reference", "--owner", "typescript-file", "--file", "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "adopt", "--ownership-root", "anchor", "--from", "reference", "--owner", "typescript-file", "--file", "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

test exact_native_file_adoption_in_an_enrolled_readonly_root_needs_no_probe_write ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.63s

```

stderr, verbatim:

```text
    Finished `test` profile [unoptimized] target(s) in 0.15s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)
```

Actual exit status: 0.

3. Complete touched target, strict target Clippy and formatting

All three first isolated outcomes above were retained before this complete target ran. The complete target executed 7 passed, 0 failed, 0 ignored, 0 measured, 0 filtered; runner duration 2.83 seconds. The unchanged original four cases all passed. No test was deselected from this target.

Command (2026-09-08T12:24:11Z to 2026-09-08T12:24:14Z):

```text
cargo test --offline --locked -p ess-cli --test output_ownership_adversary -- --nocapture
```

stdout, verbatim:

```text

running 7 tests
retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-0
retained fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-0
CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "reference/éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts"]: status=exit status: 0
stdout:
wrote reference/éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts from record.schema.json

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "anchor/other.ts"]: status=exit status: 0
stdout:
wrote anchor/other.ts from record.schema.json

stderr:

CLI ["output", "adopt", "--ownership-root", "anchor", "--from", "reference", "--owner", "typescript-file", "--file", "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "adopt", "--ownership-root", "anchor", "--from", "reference", "--owner", "typescript-file", "--file", "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé.ts"]: status=exit status: 0
stdout:

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

test composition_replaces_its_owned_companion_with_a_client_directory ... ok
retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-1
CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

test exact_native_file_adoption_in_an_enrolled_readonly_root_needs_no_probe_write ... ok
retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-2
native transition pair "É.json"/"é.json": aliases=false
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled; 3 Rust client artifact(s) written to anchor/client

stderr:

nested admission interruption: Err(interrupt after creating the nested admission namespace)
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client", "--out", "anchor/É.json", "--client-plan-out", "anchor/é.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/É.json; client plan written to anchor/é.json; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/final.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/final.json

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

retained pass2 fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-3
native transition pair "é.json"/"e\u{301}.json": aliases=false
CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 1
stdout:

stderr:
error: authored descendant blocks owned directory/file transition: client

test interrupted_nested_admission_stays_opaque_through_cli_retirement ... ok
retained fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-1
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

first installation interruption: Err(injected process-equivalent interruption after first install)
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--client-rust-out", "anchor/client", "--out", "anchor/é.json", "--client-plan-out", "anchor/e\u{301}.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/é.json; client plan written to anchor/é.json; 3 Rust client artifact(s) written to anchor/client

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

CLI ["output", "recover", "--ownership-root", "anchor"]: status=exit status: 0
stdout:

stderr:

test native_alias_refusal_preserves_the_owned_file_before_a_shape_transition ... ok
retained fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-2
CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "out/record.ts/"]: status=exit status: 1
stdout:

stderr:
error: output must name a file, not a directory: out/record.ts/

test standalone_generation_refuses_directory_spelling_before_enrollment ... ok
retained fixture: /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-3
native É.json/é.json alias lookup: false
test rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes ... ok
CLI ["compose", "--path", "/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml", "--service", "todo=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--service", "usage=/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/specify/ess-composition/tests/fixtures/two-components", "--ownership-root", "anchor", "--out", "anchor/É.json", "--client-plan-out", "anchor/é.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/É.json; client plan written to anchor/é.json

stderr:

test unicode_companions_follow_actual_native_alias_behavior_before_any_write ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.83s

```

stderr, verbatim:

```text
    Finished `test` profile [unoptimized] target(s) in 0.14s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)
```

Actual exit status: 0.

Command (2026-09-08T12:24:44Z to 2026-09-08T12:24:47Z):

```text
cargo clippy --offline --locked -p ess-cli --test output_ownership_adversary -- -D warnings
```

stdout, verbatim:

```text
```

stderr, verbatim:

```text
    Checking ess-synth v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/generate/ess-synth)
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 2.86s
```

Actual exit status: 0.

Command (2026-09-08T12:25:25Z to 2026-09-08T12:25:26Z):

```text
cargo fmt -p ess-cli -- --check
```

stdout, verbatim:

```text
```

stderr, verbatim:

```text
```

Actual exit status: 0.

4. Findings, origin and real caller reach

No current findings. There is no judgement-only or unsupported-scenario blocker.

The two pass1 signatures are retained as historical identifiers, not carried findings: `crates/edge/ess-cli/src/output_ownership/mod.rs:280` (NEEDS-CHANGE/introduced, trailing directory spelling) and `crates/edge/ess-cli/src/main.rs:2058` (NEEDS-CHANGE/introduced, owned companion-to-client transition). Their original cases and assertions now pass unchanged on f004678 in the complete target above. Their first red records have not been rewritten. The current named-file validation is at mod.rs:318, and compose enters ownership publication at main.rs:2126.

The new scenarios have concrete CLI reach. Compose at main.rs:2001/2126 combines `--client-rust-out`, `--out` and `--client-plan-out` under `--ownership-root`, then calls the same admission path at output_ownership/mod.rs:873. A new companion under an existing client directory requires the nested namespace exercised by the test-only synthesis-family probe; the callback itself is not exposed as a CLI flag. Subsequent regeneration, retirement, refusal and recoveries in that case all use the real CLI. Schema TypeScript at schema.rs:149/252 uses the standalone named publisher; output management at output_ownership/mod.rs:115/148 drives real reference adoption and its read-only admission call at :260. Existing enrolled roots with ordinary read-only mode and exact native file names are within the selected contract.

This is the second and final assigned attack. There is no third pass, approval, independent-agent evidence claim, AEP mutation or release claim.

5. Boundaries attacked without a break and retained handoff

- Nested admission orphans remained opaque across actual CLI reuse, selected retirement and repeated explicit recovery.
- Case and normalization pairs with distinct actual Linux names completed a combined file-to-directory transition and two companion publications; authored bytes survived.
- Existing-only standalone adoption in an enrolled 0555 root preserved the exact 255-byte filename, matching 0400 file, other owner and untouched reference.
- Both original caller findings stayed fixed under unchanged assertions, and the original multi-owner rollback/native-name controls stayed green.

The local platform result does not establish the new aliases=true branch on macOS. Earlier native f004678 CI reported actual case and normalization aliases on both Mac architectures, but those jobs predate these three new cases. The coordinator must run the added target there and own the final integration gate. This pass adds one injected admission callback scenario; it does not relabel that callback as a process cut or claim to repeat the original 613+613 transaction and 174+174 admission boundaries. No hardware power-loss, hostile-parent, cross-mount or new Linux CASEFOLD execution is inferred.

Exact assigned brief SHA256 9ebc3077053681f22309f4d87278cdcf21c826fee29b7275571ee1a5263c0528; exact installed adversary0.8.1 charter SHA256 75bb7514688c3ba89f5788b6150e1ce35fedb42c85a85d717c4dc979c2e27910. The Worktree skill was applied in the already assigned managed checkout. Own lease is `ess-output-ownership-adversary-wave19-pass2`. Acquisition and heartbeats succeeded; only that lease is released at handoff, with actual output/status retained beside this report. Root owns integration, publication and eventual retirement.

Retained scratch is /home/timo/.local/state/worktree/trees/b10x/ess/ess-output-ownership-adversary-wave19/target/review-output-ownership-adversary-wave19-pass2. Focused archives avoid duplicating the historical full target, which pass1 already sealed:

| Retained object | Scope | SHA256 |
|---|---|---|
| focused-source.tar | 39 full-unit changed files, including the final additive test; 25,139,200 bytes | 3e9426e53cc8163f7175288a43cda91c80bd6793154cf1e93fb63dc5636a9067 |
| native.tar | exact current CLI and touched native test executable; 111,912,960 bytes | c5161cea3b2d3405595d665cb31b96276e51d3f9f4749b122bea2189b21bbf9f |
| fixtures.tar | all 12 newly retained fixture roots, complete native bytes/modes; 194,560 bytes | 6964785002a5385b77f9f514bf7c7d72a775b1de5eae3afd6bf196cb88c84105 |
| all-tracked-source.sha256 | 1,234 exact current tracked source pins | 7d7facf6cb54df03365877fce878e92b59a667327855903826197a8b9abbba24 |
| focused-source.sha256 | 39 archived file pins | edcb2f7c2b17bb056b56deb63ff869084049494eb9b39e1c78b862486a0f1b64 |

All three archive content/metadata readbacks exited 0 without extraction. Checking all 1,234 live source pins also exited 0. Exact CLI SHA256 a786947e2299bd8dfe1d531c3dc06296017329c8112315f84f5f8603677dfd13 matches the final implementor's native CLI. Exact touched executable `target/debug/deps/output_ownership_adversary-f917312615a5063e` SHA256 7f3966f24beafd42e5eaff5c6df327f90ec5ff3ae7e87daefc2b7cca35f45f9e. Both were built and executed in this own default target. Pass1's report and seal manifest retain their original hashes (seal SHA256 342592c5eed0ee624f860731bf4f27891778b2c9d82c734e6dac4faed5505218).

Fresh pre-build allocation was target 1,592,365,056 plus TMP 233,472 bytes; SSD free 87,035,568,128 bytes; MemAvailable 33,717,352 KiB. The monitored build sample was target 1,593,335,808 plus TMP 270,336 bytes, SSD free 87,033,675,776 bytes, MemAvailable 34,594,652 KiB. After Clippy, target was 1,593,454,592 plus TMP 774,144 bytes. After retention, target was 1,731,129,344 plus TMP 774,144 bytes, SSD free 86,279,950,336 bytes, MemAvailable 36,237,176 KiB. The 5 GiB target+TMP cap, 2 GiB SSD free floor and 8 GiB MemAvailable floor held at each sample. These are actual periodic observations, not claimed continuous maxima. No resource stop occurred. Other root-owned work may change global free-space observations.

The compiler slot was returned immediately after the actual target/Clippy/fmt results. All owned producer sessions were consumed and process inspection found none remaining; source/TMP stayed fixed through archive readback. No further producer, mutation build, historical checker, Go/browser work or cleanup was launched. `seal.sha256` and its readback bind this report, original command/status/raw streams, inputs, focused archives and the own lease-release record; the seal hash is returned separately because this report cannot contain its own enclosing checksum.

6. Every outside write surface

The assigned TMPDIR /home/timo/.cache/e19-adversary-tmp also serves any ordinary native compiler temporary files. The 12 newly retained fixture roots are:

- /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-0
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-1
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-2
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-160486-3
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-147772-0
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-151532-0
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-151532-1
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-155291-0
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-0
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-1
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-2
- /home/timo/.cache/e19-adversary-tmp/e19-adversary-pass2-160486-3

These are distinct from the eight retained pass1 fixtures, which were not changed. The fixture archive includes all new descendants, including native Unicode names, opaque admission bytes, modes and settled state.

Ordinary shared Cargo bookkeeping uses /home/timo/.cargo/.global-cache, /home/timo/.cargo/.package-cache and /home/timo/.cargo/.package-cache-mutate. The existing lock files were used and retained. These are shared tool paths, not disposable outputs owned by this attack. No dependency was installed or removed.

The own session hook updates /home/timo/.local/state/worktree/registry.sqlite3 and may use manager-owned /home/timo/.local/state/worktree/registry.sqlite3-wal and /home/timo/.local/state/worktree/registry.sqlite3-shm. The two sidecars were absent at the final inspected bookkeeping census. Only the named own lease was acquired, renewed and released. No other session, worktree lifecycle, Git index/ref or shared planning record was changed.

All source patches, logs, input pins and archives are in the assigned scratch. No /tmp, tmpfs, Atlas/Website, network integration, unassigned scratch, implementation or cleanup write was made.

```findings
[]
```
