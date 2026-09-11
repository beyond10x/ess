---
format: aep.planning-md/1
id: review-result:list-selection-implementation-adversary-1
kind: review-result
status: active
title: List selection implementation adversary, round 1
relations:
- reviews: story:binding-list-selection-contract
revision: 1
---
unit: story:binding-list-selection-contract working tree wt-09052fd37c43 at base 32c765bc8535d2bf56d5a7fe56871208d5e8d830, executed candidate manifest 71ebfe9875459ea5e9bf5a859b7221e3515fafdc866b76178655cb3f5d2bf906
verdict: CONFIRMED
cases: executed 0→1, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 14 retained file or cache-root paths, listed below
needs-coordinator: record immutable finding and route invariant-authority correction before integration
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 19f052cad5b191d694bb466a0022883896501b18f9597703c900beb001f4fc93, retained as local-evidence:runtime-gaps/publication-replay/snapshots/19f052cad5b191d694bb466a0022883896501b18f9597703c900beb001f4fc93.md. Source creation recorded at 2026-09-11T04:05:24Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 6106dac061d283cebcf79604ea414ba3e7b1bce29663f2fa3084b1844c8d5771, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/6106dac061d283cebcf79604ea414ba3e7b1bce29663f2fa3084b1844c8d5771-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

```text
$ git --no-pager diff --stat
 Cargo.lock                                         |   1 +
 crates/generate/ess-gen/src/asyncapi.rs            |  22 +
 crates/generate/ess-gen/src/docs.rs                |   8 +
 crates/generate/ess-synth/Cargo.toml               |   1 +
 crates/generate/ess-synth/src/failure.rs           |  13 +-
 crates/generate/ess-synth/src/go/accessor.rs       |  57 +-
 crates/generate/ess-synth/src/go/mod.rs            |   1 +
 crates/generate/ess-synth/src/go/system.rs         | 176 +++-
 crates/generate/ess-synth/src/lib.rs               |   1 +
 crates/generate/ess-synth/src/plan.rs              |  46 ++
 crates/generate/ess-synth/src/rust/accessor.rs     | 104 ++-
 crates/generate/ess-synth/src/rust/mod.rs          |   1 +
 crates/generate/ess-synth/src/rust/system.rs       | 196 +++--
 crates/specify/ess-compiler/src/ir.rs              |  21 +
 crates/specify/ess-compiler/src/resolve.rs         | 151 +++-
 .../specify/ess-compiler/tests/oracle_fixture.rs   |   3 +-
 crates/specify/ess-domain/src/accessor.rs          |  85 +-
 crates/specify/ess-domain/src/binding.rs           | 197 ++++-
 crates/specify/ess-domain/src/lib.rs               |   1 +
 .../specify/ess-domain/src/primitive_admission.rs  |  13 +
 crates/verify/ess-conformance/src/accessor.rs      | 242 ++++--
 crates/verify/ess-conformance/src/admission.rs     |  47 +-
 crates/verify/ess-conformance/src/go/runtime.go    | 916 ++++++++++++++++++++-
 crates/verify/ess-conformance/src/lib.rs           |   1 +
 crates/verify/ess-conformance/src/runner.rs        |  21 +-
 crates/verify/ess-conformance/src/scenario.rs      |  13 +-
 crates/verify/ess-conformance/src/synthesize.rs    |   5 +
 crates/verify/ess-diff/src/change.rs               |  13 +
 crates/verify/ess-diff/src/delta.rs                |  12 +-
 crates/verify/ess-diff/src/diff.rs                 |  17 +
 30 files changed, 2195 insertions(+), 190 deletions(-)
```

The stat is the author's pre-existing tracked candidate diff; it does not identify reviewer changes and omits untracked candidate files. The reviewer added only `crates/verify/ess-conformance/tests/list_selection_adversary.rs`. No existing test, implementation, planning or Git state was modified. The exact executed manifest covers 49 changed/new files including this probe; all remained byte-identical between command launch and handback inspection. The author permitted API-stable lint factoring before execution and continues unrelated completion work afterward; this finding is tied to the recorded bytes, not a later moving tree.

Public path-redaction notice: exact managed-checkout prefixes use `worktree-state:ess/` and the cache prefix uses `local-evidence:`. The original first-run log remains unchanged with SHA256 `7eee8b6c4771ab476b6cc0c834c204fff608b52e8926904c7e33c3da0558a7f3`. Exact absolute paths are retained in `paths-private.md`.

## 1. New deciding regression

`unselected_tail_must_satisfy_its_declared_record_invariant` in the new standalone test adds `from != "blocked"` to the existing declared Leg record. The predicate selectors never read `from`. The test first validates and compiles the source, proves the IR retains exactly that invariant, and independently evaluates the invariant as False for the trailing record. It confirms the valid two-record control selects `selected`. It then changes only the unselected tail's `from` from `remote` to `blocked` and requires preflight to refuse the invalid record.

The final assertion is red: the actual result is `Ok(Present(Text("selected")))`. The test ran once, after it was written. No malformed source, dropped invariant during fixture preparation, unknown predicate result or compilation failure explains the result. Source admission and the control assertions passed before the final failure.

Command in `worktree-state:ess/wt-09052fd37c43`:

```text
RUSTUP_TOOLCHAIN=1.98.1
CARGO_TARGET_DIR=local-evidence:ess-evolution-20260910/priority-wave/build-targets/list-selection
CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER=sccache
CARGO_ENCODED_RUSTFLAGS unset
cargo test --offline --locked -p ess-conformance --test list_selection_adversary unselected_tail_must_satisfy_its_declared_record_invariant -- --exact --nocapture
   Compiling ess-domain v0.22.2 (worktree-state:ess/wt-09052fd37c43/crates/specify/ess-domain)
   Compiling ess-compiler v0.22.2 (worktree-state:ess/wt-09052fd37c43/crates/specify/ess-compiler)
   Compiling ess-gen v0.22.2 (worktree-state:ess/wt-09052fd37c43/crates/generate/ess-gen)
   Compiling ess-conformance v0.22.2 (worktree-state:ess/wt-09052fd37c43/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 10.64s
     Running tests/list_selection_adversary.rs (local-evidence:ess-evolution-20260910/priority-wave/build-targets/list-selection/debug/deps/list_selection_adversary-e3d01088c14becab)

running 1 test

thread 'unselected_tail_must_satisfy_its_declared_record_invariant' (3266394) panicked at crates/verify/ess-conformance/tests/list_selection_adversary.rs:88:5:
an invariant-invalid trailing record passed whole-record preflight: Ok(Present(Text("selected")))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test unselected_tail_must_satisfy_its_declared_record_invariant ... FAILED

failures:

failures:
    unselected_tail_must_satisfy_its_declared_record_invariant

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p ess-conformance --test list_selection_adversary`
exit: 101
```

## 2. Subsequent suite boundary

No subsequent or existing suite was run. The coordinator explicitly authorized one new regression and directed “one counterexample first no broad existing tests beforefix.” The header counts this isolated probe scope: zero previously existing cases with this identity, one executed added case, one red. It does not count the author's existing domain, native or Go witnesses as rerun. Compilation completed in 10.64 seconds; the selected test executed in 0.02 seconds. The cache/compiler slot was immediately returned to the implementor.

## 3. Finding and API comparison

| File:line | Verdict | Severity | Origin | What was measured |
|---|---|---|---|---|
| `crates/verify/ess-conformance/src/selection.rs:159` | CONFIRMED | blocker | introduced | Selection observation drops declared record invariants, so an invariant-invalid trailing record passes preflight and a selected value is returned. |

`Observation::of` copies struct fields and newtype representations while discarding their invariant vectors. `Declaration::body` reconstructs both with `invariants: Vec::new()`. Consequently `validate()` rederives a plan from weaker declaration facts, and `evaluate()` can validate every field's shape without checking whether each record is a valid value of its declared nominal type. The contract requires complete input admission before selection and says an early match cannot hide invalid later data. The compiler's own `ResolvedBody` comments identify invariants as conditions every value satisfies (`ess-compiler/src/ir.rs:309,316`). The observed result violates that authority boundary.

The same omission is visible by source inspection in the Go observation declaration (`selectionDeclaration` retains kind/of/fields/variants/tag only) and `validateValue`, and both native validator generators match `ResolvedBody::Newtype { of, .. }` / `Struct { fields, .. }` without checking invariant predicates. This pass executed the Rust conformance API only. It does not claim the native or Go runtime was independently reproduced for this new case. Those paths need corresponding correction and narrowly selected verification; existing successful shape-only witnesses do not settle constrained records.

A correction must retain and enforce the declared constraints before returning any selected value, including unselected records and nested constrained members. If a constraint cannot be evaluated under the bounded capability, return a named unsupported/unknown refusal instead of silently dropping it. The actual direct-list and prepared-input success cases must continue to establish useful supported behavior. Merely checking the final selected record would leave this counterexample's tail unchecked.

Origin is introduced: this candidate adds the selection observation module, whose declaration-copy branch creates the loss. `git show` confirms the module is absent at the assigned base; the base has no corresponding API to execute this scenario. No pre-existing reproduction is claimed.

## 4. Other inspected boundaries

- Prepared helpers accept the exact declared host-converted list type and keep binding-transformation coverage/host conversion obligations explicit. Root/child/parent preparation, raw decoding, authenticated context and stateful updates remain host-owned; no consumer adoption is claimed.
- Exclusion and first-present use earlier selector indices over the same input, retaining original occurrence indices rather than IDs. The source implementation preserves the documented primary-versus-fallback distinction; no additional finding was measured.
- Required list/record shape, null-item handling, declared enum vocabulary, count limits and full-tail shape preflight are present. The reproduced missing layer is nominal invariant authority.
- Source3 and suite6/7 selection routing, original observation retention and Optional projection admission were read. No additional runtime claim is made from source inspection alone.

## 5. Outside-worktree inventory and lease

The list below accounts for every retained scratch file and explicitly used cache root. Exact personal paths appear only in the private inventory. No worktree/cache cleanup was performed. Own lease `ess-list-adversary-01a089ee` was released with exit0, captured in `lease-release.log`; root owns the immutable review record, correction route and integration.

- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/candidate-before-probe-private.json`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/candidate-before-probe.diff`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/candidate-integrity-private.json`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/command-private.json`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/executed-candidate-private.json`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/final-diff-stat.txt`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/first-regression.exit`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/first-regression.log`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/lease-release.log`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/origin.log`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/paths-private.md`
- `local-evidence:ess-evolution-20260910/priority-wave/list-selection/adversary-1/report-public.md`
- `local-evidence:ess-evolution-20260910/priority-wave/build-targets/list-selection`
- `local-evidence:sccache`

```findings
- file: crates/verify/ess-conformance/src/selection.rs
  line: 159
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Selection observation drops declared record invariants, so an invariant-invalid trailing record passes preflight and a selected value is returned.
```