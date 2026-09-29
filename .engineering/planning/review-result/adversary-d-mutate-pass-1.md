---
format: aep.planning-md/3
id: review-result:adversary-d-mutate-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit mutate
relations:
- reviews: story:unkillable-mutants-are-not-scored-survived
revision: 1
---
unit: mutate-dead-guard (beyond10x/ess#218), working tree ess-d-mutate on 4e7c3867e (uncommitted implementor diff + adversary test file)
verdict: NEEDS-CHANGE
cases: executed 1621→1626, red 5
origin: introduced 4 / pre-existing 0 / undecided 1
wrote-outside-worktree: 7 paths (part 6)
needs-coordinator: the `/3` bump touches coordinator-owned docs (FORMAT_RELEASES, formats.md, spec-versions.md); the transition-class sibling (F4) is in #218 scope or its own story — your call

## 1. git --no-pager diff --stat (worktree)

```
 crates/edge/ess-cli/src/main.rs                    |  28 +-
 crates/edge/ess-cli/tests/mutate_external.rs       |   2 +-
 crates/edge/ess-cli/tests/mutate_report_v2.rs      | 133 ++++++-
 crates/verify/ess-conformance/src/mutate.rs        | 393 +++++++++++++++++++--
 .../tests/adversary_mutate_pass1.rs                |   3 +
 .../tests/adversary_mutate_pass2.rs                |   4 +
 .../verify/ess-conformance/tests/mutation_audit.rs |   9 +-
 .../tests/mutation_gained_refusals.rs              |  31 +-
 website/docs/guides/verify-conformance.md          |  18 +-
 website/docs/reference/cli.md                      |   4 +-
 website/docs/reference/formats.md                  |   4 +-
 website/docs/reference/spec-versions.md            |  10 +-
 12 files changed, 570 insertions(+), 69 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_mutate_dead_guard_pass1.rs   <- mine, the only path I added
```
Identical to the implementor's stat; no implementation path touched by me.

## 2. Cases added — crates/verify/ess-conformance/tests/adversary_mutate_dead_guard_pass1.rs (all red now)

| case | asserts |
|---|---|
| `a_guard_an_admitted_input_satisfies_is_not_called_dead` | spec `gate`: two 6-variant enums + `Code` newtype (`starts_with: "C"`); `flagged` = `any: [kind == Kf, tier == Tf, code != "C1"]`. The `any→all` flip is satisfied by `{Kf, Tf, "Ccode"}` (proved in-test with `input::flatten` + `decide` on the flipped IR), so the manifest must not name it `unsatisfiable_guard` |
| `a_live_guard_synthesis_could_not_witness_stays_unwitnessed_and_fails_the_run` | same mutant, collected with a pass-all stand-in: verdict must stay `unwitnessed` (exit 3), not `equivalent` |
| `a_mutant_on_a_transition_only_a_dead_outcome_performs_is_not_survived` | `escalate` is moved only by `escalated`, whose guard is `all: [High, Urgent]` (dead at baseline, scenario refused). With the built-in interpreter target, `from-drop/…escalate/Open` must not be `survived` |
| `the_new_fields_are_written_under_version_3` | emitted manifest is `ess-mutation-manifest/3`, report is `ess-mutation-report/3` (decisions.md #218 row) |
| `a_version_2_manifest_carrying_a_dead_guard_is_refused` | a manifest labelled `/2` carrying `unsatisfiable_guard` is refused by `collect` |

Red run of the cases alone (verbatim excerpt, `cargo test -p ess-conformance --test adversary_mutate_dead_guard_pass1 --no-fail-fast`, EXIT=101):

```
---- the_new_fields_are_written_under_version_3 stdout ----
assertion `left == right` failed: `ess-mutation-manifest/2` shipped in 0.41.0 without `unsatisfiable_guard`
  left: "ess-mutation-manifest/2"
 right: "ess-mutation-manifest/3"
---- a_version_2_manifest_carrying_a_dead_guard_is_refused stdout ----
a 0.41.0 `/2` manifest has no `unsatisfiable_guard`; one carrying it is refused, as a `/1` manifest carrying it is
---- a_guard_an_admitted_input_satisfies_is_not_called_dead stdout ----
assertion `left == right` failed: {kind: Kf, tier: Tf, code: "Ccode"} satisfies the flipped guard, so it is not dead; the manifest says it is
  left: Some("(kind == Kf and tier == Tf and code != C1)")
 right: None
---- a_live_guard_synthesis_could_not_witness_stays_unwitnessed_and_fails_the_run stdout ----
assertion `left == right` failed: MutantEntry { added_refusals: Some([RefusalKey { code: "ESS-SYNTH-003", scenario: Some("gate.pass.IssuePass/outcome/flagged"), ... }]), ..., unsatisfiable_guard: Some("(kind == Kf and tier == Tf and code != C1)"), unscored: None, verdict: Equivalent }
  left: Equivalent
 right: Unwitnessed
---- a_mutant_on_a_transition_only_a_dead_outcome_performs_is_not_survived stdout ----
from-drop/desk.case.Case.escalate/Open Survived added=None baseline=None stillborn=None
assertion `left != right` failed: only the dead `escalated` outcome performs `escalate`, and the baseline refuses its scenario; the correct target cannot kill this mutant, so it is not a survivor: MutantEntry { ..., change: "`from: [Open, Parked]` becomes `from: [Parked]`", class: FromDrop, ..., verdict: Survived }
test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```
Full log: ~/.cache/ess-wave-n2/mutate-d/adv1/cases-final.log. Clippy `-D warnings` on the file: EXIT=0. rustfmt --check: 0.

## 3. Suite run (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast` (build env as briefed), final lines verbatim:
```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.80s

error: 1 target failed:
    `-p ess-conformance --test adversary_mutate_dead_guard_pass1`
EXIT=101
```
Totals over all `test result` lines: 1621 passed, 5 failed (all five mine), 3 ignored. `before` = 1621 = the same run with my file deselected. Every pre-existing test passes. ess-cli not re-run (I added no CLI case).

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/mutate.rs:1106 | blocker | CONFIRMED | introduced | `satisfiable` treats `inputs.len() < 64` as "the enumeration was not cut short", but `witness::candidates` truncates the walk at 64 and *then* drops inadmissible candidates (`witness.rs:395` `admitted_inputs`). With a newtype whose base witness fails its invariant, the list drops below 64 even though combinations were never walked. A satisfiable guard gets called dead, and the mutant is scored `equivalent`, which lets the run exit 0, instead of `unwitnessed`, which exits 3. Reached by any spec with ≥3 guard-read inputs whose ladder product is >64 plus one invariant-constrained text newtype, e.g. the documented `PhoneNumber` `starts_with: "+"` shape | decide "cut short" from the ladder product (or from a flag `candidates` returns), not from the admitted count. Alternatively, require that every enum/bool leaf's values and every open leaf's non-literal value co-occur, i.e. check the full cartesian product rather than per-leaf presence (`others`, :1113) |
| crates/verify/ess-conformance/src/mutate.rs:63, :2128 | blocker | NEEDS-CHANGE | introduced | Writes `equivalent`, `baseline_refusals`, `unsatisfiable_guard` and `counts.equivalent` into `ess-mutation-report/2` and `ess-mutation-manifest/2`. Decisions #218 requires `/3`, with `--collect` still reading `/1` and `/2`, and a `/2` manifest carrying `unsatisfiable_guard` refused (today only `/1` is refused, :2308) | bump both constants to `/3`, keep `/2` as a read-only manifest format (like `MANIFEST_FORMAT_1`), and extend the `!keyed` refusal at :2308 to `/2`. Docs (formats.md, spec-versions.md, FORMAT_RELEASES) are coordinator-owned |
| crates/verify/ess-conformance/src/mutate.rs:1001 | warning | CONFIRMED | undecided | `outcome_site` returns `None` for `from-drop`/`transition-to`, so a mutant on a transition that only a baseline-dead outcome performs is scored `survived` (exit 1) under `--target interpreted`, even though no scenario of either suite can kill it. This is the #218 comment's class ("a mutant on an outcome the baseline suite does not witness") reached through the transition instead of the outcome: a silent drop of a sibling | map a transition mutant to the outcomes that `moves:` it; `unwitnessed` when every such outcome is unwitnessed at baseline, naming their refusals |
| website/docs/reference/formats.md (mutation rows), spec-versions.md:397 | warning | NEEDS-CHANGE | introduced | Still say `/2` is "Unreleased: in the current source and no release yet" and document the new fields under `/2`. That drifts from decisions #218 (`/2` ships in 0.41.0 without them) | follows the `/3` fix; coordinator-owned |

Judgement notes, not cases:
- mutate.rs:1249 `with_dead_guard` maps `Inconclusive` to `Equivalent` as well. A dead-guard mutant with a changed scenario the target skipped is not proven unkillable: `the_built_in_audit_decides_the_guard_and_a_failure_still_kills` shows the fallback scenario *does* kill the #203 mutant. So `equivalent` there hides a scenario that could have killed it. Decisions fix only kill > dead guard > unwitnessed and say nothing on inconclusive. Worth a line in the decision.
- The implementor edited two earlier adversary files (`adversary_mutate_pass1.rs:226`, `pass2.rs:223,296`) to strip `unsatisfiable_guard` from their manifests. The cases still measure the refusal-delta rule they were written for. This is not weakening, but it is an edit to other adversaries' cases.
- #203 behaviour: beyond the decided precedence, nothing was lost. Checks: `added_refusals` is still named on the equivalent line; a `/1` manifest still scores `unwitnessed` by count; killed still wins.

## 5. Attacked, not broken

- Enum-only guards: `finite::analyze` enumerates every variant up to 64 joint assignments, so they are sound.
- Optional leaf, collection, `.count`, ordering, text operator, and fact-vs-fact comparisons all return `None` (undecided).
- Bool literal against a text/number leaf returns `None`.
- `unwitnessed_outcome`: the scenario key `{command}/outcome/{outcome}` matches `ScenarioId::Outcome`, and it names only that scenario's refusals.
- An outcome dead at baseline that a mutant makes live: the baseline suite already probes the connective children (`filed` sends Mid, Urgent, High), so every baseline-green target kills it. INFEASIBLE, so the case was dropped.
- `--target` vs `--collect`: both call the same `unsatisfiable_guard` and `Ruler::judge`; no divergence found.
- Exit codes: all-equivalent/stillborn → 3; equivalent + killed → 0; equivalent never forces 1 (main.rs:3725).

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/mutate-d/adv1/case-run1.log, case-run2.log, case-run3.log, case-run4.log
- ~/.cache/ess-wave-n2/mutate-d/adv1/cases-final.log, suite-ess-conformance.log, clippy.log
- ~/.cache/ess-wave-n2/mutate-d/adv1/review.md (this file)
- build output into the assigned ~/.cache/b10x-target/ess-d-mutate (not a new directory)

Rule deviation: one `cat >> file <<'EOF'` append of a throwaway probe test to my own file. It completed; the probe was then deleted. No worktree lease was taken: `worktree --help` lists no lease verb.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/mutate.rs
  line: 1106
  category: property
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: satisfiable reads an admitted-candidate count below 64 as an exhaustive walk, so a guard an admitted input satisfies is named dead and scored equivalent instead of unwitnessed
- file: crates/verify/ess-conformance/src/mutate.rs
  line: 63
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the new verdict and fields are written into ess-mutation-report/2 and ess-mutation-manifest/2 instead of /3, and a /2 manifest carrying unsatisfiable_guard is accepted
- file: crates/verify/ess-conformance/src/mutate.rs
  line: 1001
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: a from-drop mutant on a transition only a baseline-dead outcome performs is scored survived because outcome_site excludes transition classes
- file: website/docs/reference/formats.md
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the reference documents the new fields under /2 and calls /2 unreleased, contrary to the /3 decision
```
