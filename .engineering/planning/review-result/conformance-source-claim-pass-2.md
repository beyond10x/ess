---
format: aep.planning-md/2
id: review-result:conformance-source-claim-pass-2
kind: review-result
status: active
title: Final S7 whole conformance source and claim review pass 2
relations:
- reviews: task:consumer-accounting-conformance
revision: 1
---
needs-revision

# S7 source-to-claim admission review, pass 2 of 2

Review unit: the final complete S7 source/claim set in the frozen managed tree
`<managed S7 source checkout>`.
This is the terminal source review. I examined all 4,116 ledger rows, all six profile boundaries,
the four first-review finding classes, all 64 added exact-observation fixtures, the four
compile-valid causal controls, and the retained final execution records. I did not run Cargo or
alter repository source, tests, fixtures, planning, or authority.

The final pins are exact:

- immutable first review: `3c5390bba8fe073babc9e5b07363a0a1af508bec05761f822971323772272f15`
- correction result: `d37765528ec65c120869cc3d5b60822c28626b8c71bebbd407fcf8b5f9dd3768`
- corrected 99-path source manifest: `8f9ddf9f5ec758cfde71aca7c3fa776302d9466c756fce1af4a88ae35e22668e`
- 64-observation manifest: `add051b737fe507e0cc8d9f0acadd7f7f6e7e66482dd5646c18dd03960d0b7d3`
- final ledger audit: `66971f6c6696b87fa7504ecd6a29b1d09cb2e085f533f086bd05cf4ec116950a`
- unchanged inherited 108-path vector: `28a9de14aa4bc408af9444e2b74669edca039da6521d9a2add5035d02f6717fc`

All six ledgers retain the same ordered 686 model/set/index/current-shape/old-shape/family,
disposition, and variant-input tuples. Each has 665 `Supported`, 16
`UnsupportedAtThisEntrypoint`, and five `AggregateClosureCandidate` rows. The exact Supported
coordinates are `A1-A344, A351-A620, C1-C20, C22-C27, C30-C33, C36-C37, C44-C54, C57-C64`.
The explicit gaps remain `A0, A345-A350, C0, C34-C35, C38-C43`; the aggregate candidates remain
`C21, C28, C29, C55, C56`.

| Disposition after final examination | Rows |
| --- | ---: |
| Supported with no finding: authored compile, generated synthesis, authored batch, suite/5, and suite admission | 3,325 |
| Runner rows still lacking coordinate-bearing execution evidence | 665 |
| UnsupportedAtThisEntrypoint, correctly explicit | 96 |
| AggregateClosureCandidate, correctly retained | 30 |
| Total examined | 4,116 |

## First-review finding dispositions

### 1. Generated synthesis exact observations — resolved

`synthesis_receives_each_isolated_valid_model_variant` now compares the complete synthesized suite,
refusals, and outside set for every one of the 16 isolated valid models. The fixtures retain exact
scenario ids, steps, semantic references, omissions, and refusal causes. The direct entrypoint
receives each varied `EssIr`; variants whose synthesized semantics remain unchanged are pinned as
complete unchanged results rather than inferred from a changed digest. The dropped-member control
compiled and failed the exact periodic observation, and the restored target is green.

### 2. Authored-batch exact observations — resolved

`authored_batch_receives_each_isolated_valid_model_variant` now pins the complete `AuthoredBatch`
for every varied `EssIr`: exact provenance, two ordered original sources, the accepted authored
scenario, and the duplicate refusal with its source origin. These are complete unchanged consumer
results under isolated direct inputs. The changed-provenance control compiled and failed, and the
restored target is green.

### 3. Suite/5 inventory exact observations — resolved

`suite5_builder_receives_each_isolated_valid_model_variant` now pins the complete original suite/5
document for every varied `EssIr`, including generated and authored members, outcomes, refusals,
dispositions, counts, source identity/digest/lineage, and provenance. The wrong authored-source
disposition control compiled and failed. Exact whole-document comparison closes the former
count-to-vector self-consistency gap.

### 4. Rust runner coordinate attribution — unresolved

All 665 `Supported` rows in
`non-authority-conformance-rust-runner.json` still cite
`rust_runner_receives_each_isolated_valid_model_variant`; the first such claim is at ledger line 46.
The case at `evolution_conformance_accounting.rs:470-551` builds with `Origins::Authored` and always
executes the one `workbench.todo/authored/create-list` scenario from `stable-scenario.yaml:1-10`.
That fixture expressly describes itself as stable while **unrelated** model coordinates vary, and
its only executable facts are `CreateList`, one literal input, and the `created` outcome.

The actual entrypoint does not receive the varied `EssIr`. `Runner::run_admitted` at
`runner.rs:353-380` reads the admitted suite, iterates only its scenarios and steps, and copies its
provenance into the report. With authored-only construction, every runner input has the same one
scenario, same source references, same steps, and empty generated/refused/outside inventory; only
the model-derived provenance digests differ. Independently normalizing 64-hex digests in the 16
retained runner observations yields one identical input/good-report/wrong-report document. Thus the
periodic, selection, accessor, settings, reading, response, subject-state, naming, component,
schema-container, and parent-union coordinates do not reach runner execution.

The exact reports do establish a smaller real fact: for each digest-labelled copy of this one
scenario, `WorkBenchTarget::Created` produces one passed `ESS-CF-OUTCOME` check and
`WorkBenchTarget::WrongOutcome` produces one failed check with a diagnostic. The false-success
control proves that good-versus-wrong target distinction. It does not make the scenario observe any
of the 665 attributed coordinates. The separate Billing receipts remain limited to their two
`CreateInvoice` scenarios and do not repair this mapping.

The required final disposition is therefore finite: none of the runner profile's 665 current
`Supported` rows is source-to-claim admitted. They require coordinate-bearing executable scenarios
at `run_admitted`, or a non-Supported disposition where that boundary cannot execute the coordinate.
Because this is pass 2 of 2 and no further correction unit is permitted, this report records the
unresolved class for root's final rejection/adoption decision rather than requesting another pass.

## Retained checks and limits

The retained correction records are internally consistent: the dedicated target reports 12 passed,
the affected targets 171 passed, the complete package 409 passed across 38 result blocks, strict
all-target Clippy and scoped formatting exited zero, and the inherited and corrected source hash
checks exited zero. The four causal controls each exited 101 for their stated assertion and were
restored. These records establish the source and suite outcomes they actually ran; they do not
convert the runner's unrelated stable scenario into coordinate-bearing execution evidence.

The two graph rows and 14 upstream named-reading rows per profile remain correct explicit gaps, and
the five aggregate identities per profile remain candidates without closure authority. This review
grants no native, global, baseline, aggregate, production, or M9 qualification.

```findings
- file: crates/verify/ess-conformance/tests/fixtures/evolution_conformance_accounting/non-authority-conformance-rust-runner.json
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows execute one stable authored CreateList scenario that contains none of their varied model coordinates; after digest normalization all 16 pinned runner inputs and outcomes are identical, so run_admitted observes only the unrelated scenario and provenance.
```

Public-safe projection: only the absolute managed checkout path is replaced. Original report SHA2561f7fc769406486c0fdb41b02f475a8bae790272b71216001980942734b430e9e is retained locally unchanged.
