---
format: aep.planning-md/3
id: story:gained-refusals-are-unwitnessed-not-survived
kind: story
status: active
title: 'mutate --emit/--collect: a mutant whose suite gains synthesis refusals is scored survived'
refs:
- provider: github
  reference: beyond10x/ess#203
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 3}
---
# Story: mutate --emit/--collect: a mutant whose suite gains synthesis refusals is scored survived

## Why

beyond10x/ess#203. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #203 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

**#203: `mutate --emit/--collect` scores a mutant whose suite gained synthesis refusals as `survived`**

**Reproduces on this tree (HEAD `e932781965`, 0.40.0 candidate). Not fixed.**

- (cited) `synthesized()` at `crates/verify/ess-conformance/src/mutate.rs:1544-1549` reduces `synthesis.refusals` to `.len()` and drops the codes and subjects.
- (cited) `Ran.refusals: usize` (`mutate.rs:1245`) and `EmittedSuite.refusals: usize` (`mutate.rs:1401`) carry only the count. `MutantEntry.refusals: Option<usize>` is at `mutate.rs:1029`.
- (cited) `evaluate()` (`mutate.rs:1269-1315`, the built-in `--target` path) sets the verdict from scenario statuses alone, through `Verdict::classify` at `mutate.rs:989-1000`. It never compares refusals with the baseline.
- (cited) `collect()` (`mutate.rs:1755-1826`) copies `suite.refusals` into the entry (`:1786`) and then classifies statuses only (`:1802`). No baseline delta is computed.
- (cited) `render_text()` (`mutate.rs:1097-1140`) prints refusals for the baseline only (`:1101`, `:1110`), not per mutant.
- (cited) CLI exit status keys on `counts.survived` (`crates/edge/ess-cli/src/main.rs:3695`), so a new verdict changes the exit code.
- (cited) `synthesize::Refusal { subject, scenario, cause }` has `code()` (`crates/verify/ess-conformance/src/synthesize.rs:339-362`). Refusal identity by code and outcome can be built from it without touching synthesis.

**Where the fix lands**

- (cited) `mutate.rs`:
  - `synthesized` returns the refusal list, or keys of the form (code, scenario/subject).
  - `Ran` and `EmittedSuite` gain the list.
  - `evaluate` and `collect` compute the mutant-minus-baseline delta.
  - `Verdict` gains a variant.
  - `Counts`, `render_text` and `MutantEntry` change to match.
- (cited) Both format IDs are pinned: `REPORT_FORMAT` is `ess-mutation-report/1` (`mutate.rs:63`) and `MANIFEST_FORMAT` is `ess-mutation-manifest/1` (`mutate.rs:1380`). The manifest is `deny_unknown_fields` (`mutate.rs:1396`, `:1410`, `:1466`). A new per-mutant field therefore needs `/2` of both, with `/1` still readable.
- (cited) `main.rs:3626-3705` needs the emit summary line and the exit mapping (the new verdict should exit 3, not 0).
- (cited) The format rows at `website/docs/reference/formats.md:319-320` must be updated.
- (inferred) Also update `website/docs/guides/verify-conformance.md` (mutation audit section), `docs/design/mutation-audit-and-model-runner.md` and `CHANGELOG.md`.
- (inferred) Tests: `crates/verify/ess-conformance/tests/mutation_audit.rs`, `crates/verify/ess-conformance/tests/mutation_external.rs`, `crates/edge/ess-cli/tests/mutate_cli.rs`, `crates/edge/ess-cli/tests/mutate_external.rs`. Add a fixture from the issue's shape, with both enum orders.

**Collisions**

- (cited) **#210** (collect with a skipped baseline) edits the same `collect()` baseline check (`mutate.rs:1759-1765`), the same `Verdict::classify` path and the same report format and exit mapping. This is a hard collision: sequence the two, or put both in one unit. Both likely bump `ess-mutation-report/2`, and they should share one bump.
- (inferred) **#202** (sets-retarget witnesses) is a change to synthesis witnesses in `synthesize.rs`, and maybe mutant generation in `mutate.rs`. Soft collision.
- (inferred) **#193, #196, #198, #199, #209** touch `synthesize.rs` refusal and witness logic. #203 only reads `Refusal`, so these are soft unless one of them changes the `Refusal` shape.
- (inferred) **#195, #201, #204, #205, #211** are compiler/specification semantics. No overlap expected.

**Design decisions for the implementor**

- (inferred) Choose a new verdict (for example `unwitnessed`: the mutant's suite lost a scenario the baseline had) or reuse `stillborn`. A new verdict is the smaller honest option. `stillborn` means "the compiler refused" and carries `ESS-MUTATE-002` with a compiler cause, so reusing it would overload it. Precedence: `killed` beats the new verdict (a failing scenario is a real kill), and the new verdict beats `survived` and `inconclusive`.
- (inferred) Refusal identity for the delta should be (code, scenario id, or subject when there is no scenario). A count delta alone misses the case where a mutant loses one refusal and gains another.
- (inferred) Choose a new `MUTATE` code (for example `ESS-MUTATE-004`) or no code for the new entry field `added_refusals: [{code, scenario}]`.
- (inferred) The witness-order dependence the issue describes goes away once the verdict keys on the refusal delta. Fixing the witness search itself is out of scope.

Confidence: high on reproduction and location (every code path read). Medium on the verdict naming and format bump, which are the implementor's call.

Paths:
- crates/verify/ess-conformance/src/mutate.rs
- crates/verify/ess-conformance/src/synthesize.rs (read only)
- crates/edge/ess-cli/src/main.rs
- website/docs/reference/formats.md
- website/docs/guides/verify-conformance.md (inferred)
- docs/design/mutation-audit-and-model-runner.md (inferred)
- CHANGELOG.md (inferred)
- crates/verify/ess-conformance/tests/mutation_audit.rs (inferred)
- crates/verify/ess-conformance/tests/mutation_external.rs (inferred)
- crates/edge/ess-cli/tests/mutate_cli.rs (inferred)
- crates/edge/ess-cli/tests/mutate_external.rs (inferred)

Verdict: open (small design choice on the verdict name and format bump; pair with #210)
