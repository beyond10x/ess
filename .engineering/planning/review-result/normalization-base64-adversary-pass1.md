---
format: aep.planning-md/1
id: review-result:normalization-base64-adversary-pass1
kind: review-result
status: active
title: Frozen base64 qualification adversary pass 1
relations:
- reviews: story:go-normalization-pattern-semantics
revision: 1
---
unit: story:go-normalization-pattern-semantics — working tree wt-bf45625a6a50 over 9899faed9eb10eb2e4d9e6dc946f6d732e87c6b6, pass 1
verdict: CONFIRMED
cases: executed 1→2, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: correct incomplete pattern accounting before continuing

`git --no-pager diff --stat` includes the inherited implementation and coordinator-owned planning changes:

```text
 .engineering/planning/journal.jsonl                | 15 ++++
 .../story/go-normalization-pattern-semantics.md    | 24 ++++++-
 .../story/raw-json-normalization-provenance.md     | 80 +++++++++++++++++++++-
 CHANGELOG.md                                       |  6 +-
 crates/edge/ess-cli/tests/normalization.rs         | 47 +++++++++++++
 crates/generate/schema-contract/src/realize.rs     | 23 ++++++-
 .../src/realize/normalize/go_target.rs             | 10 +++
 .../schema-contract/tests/normalization_go.rs      | 76 ++++++++++++++++++++
 .../schema-contract/tests/normalization_rust.rs    |  8 +++
 docs/design/source-pinned-data-normalization.md    | 37 ++++++++++
 website/docs/guides/generate-artifacts.md          | 18 +++--
 11 files changed, 333 insertions(+), 11 deletions(-)
```

Those non-test changes were present before this pass. My sole source-tree addition is the untracked test, shown using `git diff --no-index --stat /dev/null <test>`:

```text
 .../tests/normalization_base64_adversary.rs        | 63 ++++++++++++++++++++++
 1 file changed, 63 insertions(+)
```

The new case, `crates/generate/schema-contract/tests/normalization_base64_adversary.rs:54`, compiles an ordinary model containing `Map<Integer, Bytes>`. Its checked projection has the qualified base64 value pattern **and** the unqualified integer-key pattern `^-?(0|[1-9][0-9]*)$` under `propertyNames`. Reference execution accepts a valid input. The test then requires Go generation to refuse the unqualified pattern at its model-qualified source pointer.

First execution, before any control/suite run:

```text
CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 cargo test -p schema-contract --test normalization_base64_adversary base64_values_do_not_qualify_unrelated_model_property_name_patterns -- --exact --nocapture

running 1 test

thread 'base64_values_do_not_qualify_unrelated_model_property_name_patterns' (3527648) panicked at crates/generate/schema-contract/tests/normalization_base64_adversary.rs:58:9:
Go generation admitted an unqualified pattern under model propertyNames
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test base64_values_do_not_qualify_unrelated_model_property_name_patterns ... FAILED

failures:

failures:
    base64_values_do_not_qualify_unrelated_model_property_name_patterns

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p schema-contract --test normalization_base64_adversary`
```

Exit **101**. Rustfmt subsequently moved the assertion from line 58 to line 54 without changing behavior.

The focused comparison ran after the new case existed. Its baseline explicitly excludes the new target:

```text
CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 cargo test -p schema-contract --test normalization_go qualified_patterns_do_not_hide_other_obligations_or_inspect_annotation_data -- --exact --nocapture

running 1 test
test qualified_patterns_do_not_hide_other_obligations_or_inspect_annotation_data ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.02s
```

Exit **0**. Then the complete new test target:

```text
CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 cargo test -p schema-contract --test normalization_base64_adversary -- --nocapture

running 1 test

thread 'base64_values_do_not_qualify_unrelated_model_property_name_patterns' (3533398) panicked at crates/generate/schema-contract/tests/normalization_base64_adversary.rs:54:9:
Go generation admitted an unqualified pattern under model propertyNames
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test base64_values_do_not_qualify_unrelated_model_property_name_patterns ... FAILED

failures:

failures:
    base64_values_do_not_qualify_unrelated_model_property_name_patterns

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p schema-contract --test normalization_base64_adversary`
```

Exit **101**. The header’s **1→2** counts this focused control plus adversary selection, not a repeated whole-package run.

| Location | Verdict / origin | Finding |
|---|---|---|
| `crates/generate/schema-contract/src/realize/normalize/go_target.rs:129` | CONFIRMED / introduced | Qualifying Bytes admits model Map<Integer, Bytes> while its unqualified propertyNames integer-text pattern is absent from collected obligations, contradicting every-other-pattern refusal. |

**Measured:** generation returns success instead of the required `go_schema_pattern` refusal.

**Reachability:** supported model syntax flows through `ModelTypes::select`, `Plan::check_with_models`, and `Plan::go`; the CLI uses those same APIs. No forged projection or private-state mutation is involved.

**Origin:** this specific `Map<Integer, Bytes>` path becomes publishable through the new whitelist. Reading the base shows its unconditional pattern refusal previously caught the Bytes value pattern. The incomplete `propertyNames` traversal predates this unit; no separate claim about `Map<Integer, String>` was measured.

Correction belongs in schema-aware obligation collection: inspect model `propertyNames` constraints with their exact nested pointers. Do not qualify the integer-key pattern implicitly.

The existing mixed-value-pattern control remains green; the new failure identifies a schema position it does not cover. No full runtime corpus was repeated.

Logs remain inside the assigned scratch directory:

- `target/resume-base64/adversary-pass1-isolated.log`
- `target/resume-base64/adversary-pass1-control.log`
- `target/resume-base64/adversary-pass1-suite.log`

Paths written outside the worktree: **none**.

```findings
- file: crates/generate/schema-contract/src/realize/normalize/go_target.rs
  line: 129
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Qualifying Bytes admits model Map<Integer, Bytes> while its unqualified propertyNames integer-text pattern is absent from collected obligations, contradicting every-other-pattern refusal.
```
