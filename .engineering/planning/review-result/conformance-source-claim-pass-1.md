---
format: aep.planning-md/1
id: review-result:conformance-source-claim-pass-1
kind: review-result
status: active
title: S7 whole conformance source and claim review pass 1
relations:
- reviews: task:consumer-accounting-conformance
revision: 1
---
needs-revision

The six S7 ledgers are structurally complete. The authored-compile and suite-admission profiles have
exact entrypoint observations for all 665 Supported rows: the former asserts the complete authored
result is unchanged for every isolated input, and the latter asserts original bytes, the parsed
suite, and the complete parsed inventory equal the supplied carrier. Four other profiles promote
2,660 Supported rows from source/provenance/count consistency without the exact behavior their
boundaries require. The actual Rust behavior receipts cover only two Billing `CreateInvoice`
scenarios and cannot close the 665-row runner ledger.

## Reviewed unit and complete accounting

This first whole-unit examination covers the frozen managed tree
`<managed S7 source checkout>` at base
`f1af8280338b97d862a6c474ec50f78d5157d71c`, with the inherited 108-file snapshot and the exact
35-file S7 manifest recorded by `source-files.sha256` (manifest SHA-256
`7fe3b45ad687ec5ab6355b06e7dcc8850c937a611f5f94626590b5e8f3c66506`). The implementation
report matches its recorded SHA-256
`726c37f67c175a520b508e8d2684971730577d8c1fdacb5dbf6b07aa9b923842`.

All 4,116 rows were examined. Each ledger has the exact 621 A rows and 65 C rows from obligation
source `8222c62c803881d9f6865dfa95850a0125adbd8ca14db9b1ff925647b8335763`, with continuous
set indexes, matching model/shape/family coordinates, and the declared 665/16/5 partition. The
row coordinates, dispositions, variant inputs, refusal coordinates, and aggregate coordinates are
byte-equivalent across profiles after profile-specific evidence text is removed. That proves the
six files agree with one another; it does not prove six entrypoints behave alike.

| Disposition after examination | Rows |
| --- | ---: |
| Supported with no finding: authored compile + suite admission | 1,330 |
| Supported needing revision: generated + authored batch + suite/5 + Rust runner | 2,660 |
| UnsupportedAtThisEntrypoint, correctly explicit | 96 |
| AggregateClosureCandidate, correctly retained | 30 |
| Total examined | 4,116 |

The exact Supported coordinates needing revision in each of the four affected profiles are
`A1-A344, A351-A620, C1-C20, C22-C27, C30-C33, C36-C37, C44-C54, C57-C64`.
The correct explicit gaps are `A0, A345-A350, C0, C34-C35, C38-C43`. The five unchanged aggregate
candidates are `C21, C28, C29, C55, C56`.

## Finding 1: synthesis never asserts the generated requirement attached to a row

Affected ledger: `non-authority-conformance-generated.json`, all 665 Supported coordinates above.
Every row cites `synthesis_receives_each_isolated_valid_model_variant`. The case asserts only that
the suite provenance digest differs from the base and that either the whole suite or the whole
refusal vector is nonempty (`evolution_conformance_accounting.rs:373-389`). It never names the
scenario, step, semantic reference, refusal, or omission produced for any ledger coordinate.

The product entrypoint walks commands and outcomes, then lifecycle, invariants, and bindings
(`synthesize.rs:963-1010`). A changed provenance digest is constructed before that walk and remains
different even when the behavior relevant to a claimed coordinate is absent or wrong. The bounded
correction must assert the exact generated requirement/refusal or exact unchanged generated result
for each supported coordinate, with controls that fail when its relevant branch is dropped or
mis-mapped.

## Finding 2: authored-batch claims exceed accepted/refused/source-count behavior

Affected ledger: `non-authority-coverage-authored-batch.json`, all 665 Supported coordinates above.
Every row cites `authored_batch_receives_each_isolated_valid_model_variant`. For every overlay the
case checks one accepted stable source, zero refusals, and the retained identity/text of that same
source (`evolution_conformance_accounting.rs:392-410`). It does not assert the batch provenance or
any coordinate-specific authored result. The additional base-only case at lines 498-521 proves
duplicate/refusal/source separation once; it is not repeated with, or attributed causally to, the
665 coordinates.

`compile_sources` validates the model, calls `compile_one` for each source, retains provenance,
sources, and candidates (`coverage_build.rs:46-73`). The correction must separately establish the
exact result and provenance behavior attributable to each coordinate, or classify coordinates that
do not affect this boundary as non-Supported.

## Finding 3: suite/5 inventory assertions are self-consistency checks

Affected ledger: `non-authority-coverage-suite5.json`, all 665 Supported coordinates above. Every
row cites `suite5_builder_receives_each_isolated_valid_model_variant`. The case checks changed suite
provenance, one authored-source entry, and that each stored count equals the length of the vector it
counts (`evolution_conformance_accounting.rs:413-448`). It does not assert an expected generated,
authored, outside, or refused member, its outcome, or its source identity for any row. A generator
that omitted or misclassified the same member in both the vector and count would still satisfy
these assertions.

`build` composes `compile_sources`, synthesis, inventory partitioning, and final admission
(`coverage_build.rs:105-176`). The bounded correction must name the exact inventory member and
disposition expected for each supported coordinate, preserve exact source lineage, and include a
causal wrong-member or wrong-disposition control. Count/vector agreement alone is ledger and
implementation self-consistency.

## Finding 4: runner variants do not assert execution success, and the receipts cover two unrelated scenarios

Affected ledger: `non-authority-conformance-rust-runner.json`, all 665 Supported coordinates above.
The variant case really calls `Runner::run_admitted`, but it asserts only suite-byte digest,
provenance equality, and report scenario count (`evolution_conformance_accounting.rs:471-495`). It
never asserts report status, scenario status, check status, check code, diagnostics, or the absence
of unsupported target callbacks. The actual runner executes every admitted scenario and computes a
verdict from the produced scenario checks (`runner.rs:353-380`); a report in which every scenario
failed or errored could still pass this accounting case as long as its length matched.

The retained behavioral receipts come from a different `billing()` model and an explicit selection
of exactly `billing.invoice.CreateInvoice/outcome/accepted` and
`billing.invoice.CreateInvoice/outcome/rejected` (`evolution_conformance_accounting.rs:603-637`).
They establish these finite facts:

- the good Billing target passes those two scenarios (25 checks, zero nonpasses);
- `AcceptInvalidAmount` fails the rejected scenario (three nonpassing checks);
- `WrongEvent` fails the accepted scenario (three nonpassing checks).

Their recorded hashes match the implementation report. They do not execute the 16 accounting
fixture variants and do not observe periodic, selection, accessor, settings, reading, response,
subject-state, error-naming, binding-name, external-reference, component-name, schema-container, or
aggregate coordinates. The guard and wrong-identity red controls therefore protect only those two
Billing scenarios. The bounded correction must execute exact selected scenarios that observe each
claimed runner coordinate, assert the expected report checks/status, and retain causal faulty-target
controls. A coordinate with no executable scenario at this boundary needs a non-Supported
disposition.

## Correct dispositions and retained execution evidence

The two graph rows per profile are correctly explicit gaps: `DependencyRelation` is constructed by
the separate semantic graph and is absent from `EssIr`. The 14 reading rows per profile are also
correctly non-Supported here: the fixture observes the three `NamedType::check_shape` refusals
before an `EssIr` exists, and the ledgers do not count that upstream refusal as consumer support.
The five aggregate identities per profile remain candidates and claim no closure authority.

The authored profile's entrypoint case asserts the complete `Authoring` result for every isolated
variant, including acceptance and refusal behavior, and its second case asserts ordering, one
accepted scenario, one refusal, refusal origin, and original source separation. That is the exact
unchanged behavior this boundary supplies; it does not borrow generated or runner behavior.

The suite-admission profile passes the actual public carrier into `AdmittedSuite::from_json` and
asserts exact original bytes, suite value, and full coverage value for every variant. Its separate
selected-parent case proves direct admission refuses `MissingParent` and the complete carrier
restores the exact selected and parent bytes. This is actual admission behavior at `parse`, even
though the suite was produced upstream; no execution claim is inferred from it.

The retained execution logs are credible as execution records for what they actually ran: the new
target passed 12 tests, affected targets passed 171, the package passed 409, and final Clippy and
formatting exited zero. The ledger-presence, ledger-disposition, guard, and wrong-identity controls
went red as reported and were restored. Those results establish suite health and the three finite
controls; they do not repair the source-to-claim gaps above. This review ran no Cargo, native,
browser, or whole-workspace qualification and grants no M9, aggregate, baseline, or production
authority.

```findings
- file: crates/verify/ess-conformance/tests/fixtures/evolution_conformance_accounting/non-authority-conformance-generated.json
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows rely on changed provenance and a nonempty whole synthesis without asserting the generated scenario, step, semantic reference, refusal, or omission attributable to the row.
- file: crates/verify/ess-conformance/tests/fixtures/evolution_conformance_accounting/non-authority-coverage-authored-batch.json
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows cite one accepted stable source and source-byte retention, while batch provenance and coordinate-specific accepted or refused behavior are unasserted.
- file: crates/verify/ess-conformance/tests/fixtures/evolution_conformance_accounting/non-authority-coverage-suite5.json
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows are backed by provenance movement and count-to-vector self-consistency, without exact inventory member, outcome, disposition, or source-lineage assertions.
- file: crates/verify/ess-conformance/tests/fixtures/evolution_conformance_accounting/non-authority-conformance-rust-runner.json
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows lack asserted execution outcomes because the variant runs check only digest, provenance, and scenario count, and the three behavioral receipts cover only two Billing CreateInvoice scenarios from another model.
```

Local path projection: the managed checkout path above replaces the workstation-specific path. Original report SHA256 3c5390bba8fe073babc9e5b07363a0a1af508bec05761f822971323772272f15 is retained at local-evidence:ess-evolution/waves/0010-opus-accounting/s7/source-claim-review-1.md. Findings and all other review text are unchanged.
