---
format: aep.planning-md/3
id: review-result:adversary-d-absent-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit absent
relations:
- reviews: story:a-related-field-the-creator-leaves-absent-is-witnessed
revision: 1
---
unit: absent (beyond10x/ess#239), story:a-related-field-the-creator-leaves-absent-is-witnessed; uncommitted working tree on 300bfd3f5 in ~/.local/state/worktree/trees/b10x/ess/ess-e-absent
verdict: NEEDS-CHANGE
cases: executed 1725→1727, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory (~/.cache/ess-wave-n2/absent/adv1/: review.md, red1.log, red2.log, suite.log, generate.log, clippy.log, base-run.log, base-unit.log); shared build dir ~/.cache/b10x-target/ess-e-absent
needs-coordinator: no

## 1. Diff stat

`git --no-pager diff --stat` (tracked files: the unit's own; I changed none of them):

```
 crates/verify/ess-conformance/src/synthesize.rs    | 47 ++++++++++++++++++-
 .../src/synthesize/related_guard.rs                |  2 +
 .../ess-conformance/src/synthesize/set_effects.rs  |  3 ++
 .../ess-conformance/src/synthesize/subject_fact.rs | 52 +++++++++++++++++++---
 4 files changed, 95 insertions(+), 9 deletions(-)
```

Untracked: `tests/related_guard_absent_field.rs` (the unit's) and `tests/adversary_absent_pass1.rs` (mine, the only file I added). No implementation path touched.

## 2. Cases added

`crates/verify/ess-conformance/tests/adversary_absent_pass1.rs`: the #239 model, plus one `affects:` on
`ConfigureSite` (`where: not defined(site)`, `sets: {site: fallback}`). The admitted construct: every other
configuration with no site gets a fallback. A `Declared` target implements the model as written.

| case | asserts | now |
|---|---|---|
| `a_site_written_by_a_later_creations_affects_is_not_read_as_absent` (:309) | `not-configured` is refused, or its filed scenario passes the declared target | RED |
| `every_filed_scenario_passes_the_declared_affects_target` (:343) | every filed scenario passes the declared target | RED |

Red output, each case run alone before the suite (red1.log, red2.log):

```
commands sent: ["demo.sso.OnboardTenant", "demo.sso.ConfigureSite", "demo.sso.OnboardTenant", "demo.sso.OnboardTenant", "demo.sso.ConfigureSite", "demo.sso.InitiateSignIn"]
...
                input: Some(
                    "demo.sso.InitiateSignIn(tenant = \"00000000-0000-4000-8000-000000000018\")",
                ),
                expected: [
                    "outcome = not-configured",
                ],
                observed: [
                    "outcome = initiated",
                ],
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

```
thread 'every_filed_scenario_passes_the_declared_affects_target' (2826661) panicked at crates/verify/ess-conformance/tests/adversary_absent_pass1.rs:356:5:
filed scenarios failing a target that does what the model declares: [
    "demo.sso.InitiateSignIn/outcome/not-configured",
]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.02s
```

The mechanism is `related_guard.rs:537`, which arranges decoy, row, decoy. The second decoy is a
configured row, so its `ConfigureSite` runs after the row. Its `affects:` writes `fallback` onto the row.
Nothing folds that write into the row's `unwritten`: `still_unwritten` (`synthesize.rs:3043`) only reads
the `sets:` of acts on that row. `row_truth_with` (`subject_fact.rs:378`) then binds `site` as Null. The
pre-command view check passes because `expected_row` (`subject_fact.rs:2357`) skips the unwritten field,
so nothing in the scenario catches the wrong assumption.

## 3. Suite run (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast`: EXIT=101. Totals across binaries: passed 1725, failed 2, ignored 3.

```
error: 1 target failed:
    `-p ess-conformance --test adversary_absent_pass1`
```

The before count, 1725, comes from the implementor's `conf-after2.log` (passed 1725, ignored 3).
Other runs:
- `cargo xtask generate --check`: EXIT=0, `projections are up to date`.
- `cargo clippy -p ess-conformance --test adversary_absent_pass1 -- -D warnings`: EXIT=0.
- `rustfmt --check` on my file: 0.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize.rs:3043 | blocker | NEEDS-CHANGE | introduced | `unwritten` is shrunk only by acts on the row itself. An `affects:` on another row (a later arranged decoy's `ConfigureSite`) writes the field, but the row still reads absent. The unit files a `not-configured` scenario that a correct target fails. | Sound minimum: `unwritten_by` excludes every `Optional` field that any `affects:` `sets:` or `instances:` outcome of the model can write on that entity (these then read `Unknown`, as before). Alternative: fold every `affects:` or `instances:` act a scenario's steps run into every earlier arranged row of that entity. |
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2357 | note | CONFIRMED | introduced | The skip is by design. It does mean the observation before the command cannot catch a wrong `unwritten`: in the red run the `demo.sso.Configurations` check passed on a row holding `site: fallback`. | None required if the finding above is fixed. Otherwise pin absence where the view projects the field. |

What reaches the blocker: `affects:` is an admitted ess/16 construct on `updates:` and `moves:` branches.
The only condition is that the command writing the guarded field also carries an `affects:` on the same
entity, and that is enough for `related_guard::prepare_at`. No flag is involved.

Origin evidence:
- At base, `row_truth` (git show 300bfd3f5) has no `unwritten` and reads an undetermined field as `Unknown`.
- The implementor's base red run (`~/.cache/ess-wave-n2/absent/red1.log`) shows `not-configured` refused at base (`left: None`).
- So at base my case takes its early-return branch.
- My own attempt to run at base through a `git archive` export is void. The shared target dir linked the unit's `ess-conformance` lib: "Compiling ess-conformance … Finished in 0.50s", and the unit's own 7 tests passed "at base". This is the known shared-target hazard. The export has been deleted, so cargo sees its dep-info paths as missing and rebuilds.

Not made into a case (judgement, residue): a binding (`when: TenantOnboarded → invoke ConfigureSite`)
would write the field in a correct target after the creator runs, and `unwritten` ignores it too. This is
the same class as the blocker. The blocker's fix (exclude any field some other writer can reach) covers it
if bindings are counted as writers.

## 5. Attacked and not broken

- `unwritten_by` against a creator `sets:` from an omitted `Optional` input: the field is in `sets`, so it is not unwritten, and settled holds Null (the unit's own case).
- `{generated: true}` / `{related:}` / `{cleared}` / `{increment}` writers: all are `sets:` entries, so they are removed from `unwritten`. Where the value is undetermined the field reads `Unknown`, the safe direction.
- The `instances:` rows changed by `left_by`: `still_unwritten` is applied (set_effects.rs:904).
- Creators cannot carry `affects:` (domain refuses: "outcome `onboarded` creates its subject and declares `affects:`").
- `affects:` on the subject: excluded by the domain.
- Replay, owner placeholder, around_row and isolating use an empty set, the safe direction.
- Byte stability: `cargo xtask generate --check` is clean. Determinism: `BTreeSet` throughout, no hash iteration.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/absent/adv1/review.md
- ~/.cache/ess-wave-n2/absent/adv1/red1.log
- ~/.cache/ess-wave-n2/absent/adv1/red2.log
- ~/.cache/ess-wave-n2/absent/adv1/suite.log
- ~/.cache/ess-wave-n2/absent/adv1/generate.log
- ~/.cache/ess-wave-n2/absent/adv1/clippy.log
- ~/.cache/ess-wave-n2/absent/adv1/base-run.log (void, see section 4)
- ~/.cache/ess-wave-n2/absent/adv1/base-unit.log (void, see section 4)
- ~/.cache/ess-wave-n2/absent/adv1/base/ (git archive export, deleted)
- build output in ~/.cache/b10x-target/ess-e-absent (the assigned dir)

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 3043
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an affects write by a later arranged row's command never leaves the row's unwritten set, so the absent-field branch is filed as a scenario a correct target fails
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2357
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the pre-command observation skips the unwritten field, so no step of the scenario can detect a wrong unwritten assumption before the command answers
```
