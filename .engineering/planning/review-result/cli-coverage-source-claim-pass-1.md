---
format: aep.planning-md/1
id: review-result:cli-coverage-source-claim-pass-1
kind: review-result
status: active
title: CLI coverage source and claim review pass 1
relations:
- reviews: task:consumer-accounting-cli-coverage
revision: 1
---
needs-revision

# S8 source-to-claim admission review, pass 1 of 2

This review covers the frozen managed tree
`local-evidence:ess-evolution/s8-source-candidate`
at `f1af8280338b97d862a6c474ec50f78d5157d71c`, the inherited-source manifest
`16690c3365219ce9c3babaf7d1a4342a37edb6919ce1430aa9a1f58a909f5d34`, the exact
implementation result
`9708d36f2736d382f4af3213fb3d07a69dd3050b21582f6f4e9c355907f52b0b`, and the exact
implementation manifest
`890e83c0c61bb5574f75fc00e2e6bf8273193315c33e6c9b0a81c4df2dce13be`. Every file in that
manifest and all three root-owned production files in `coordinator-periodic-repair.sha256` matched
their recorded SHA-256. I ran no Cargo, native, or browser command and changed no source or test.

All 4,474 ledger rows were examined. Their identities, shapes, profile pins, row totals, and
disposition counts are internally consistent with the pinned obligation and current-inventory
files. The 64 upstream gaps and 20 aggregate candidates have the correct substantive disposition.
The exact five aggregate identities are retained in each profile and make no authority claim.

| Disposition after examination | Rows |
| --- | ---: |
| `Supported` claims needing revision | 4,390 |
| `UnsupportedAtThisEntrypoint`, substantively correct but with stale control references | 64 |
| `AggregateClosureCandidate`, correct | 20 |
| Total examined | 4,474 |

For each 686-row profile, the affected `Supported` coordinates are
`A1-A344, A351-A620, C1-C20, C22-C27, C30-C33, C36-C37, C44-C54, C57-C64`.
The substantive gaps are `A0, A345-A350, C0, C34-C35, C38-C43`; the aggregate candidates are
`C21, C28, C29, C55, C56`. For final merge, the affected set is exactly every one of the 2,395
rows whose disposition is `Supported`: the complete 2,416-row current inventory minus the same 16
model identities classified as gaps and the same five aggregate identities. The inventory-local
`index` values are not globally unique across the Rust and wire sources, so model identity plus
disposition is the exact coordinate for that profile.

## Findings: model-coordinate attribution

### 1. Final merge promotes 2,395 inventory rows from one Billing run and digest movement

Every final-merge `Supported` row cites either both
`final_merge_preserves_suite5_pretty_compact_lineage_and_pre_output_refusal` and
`final_merge_observes_each_independent_replacement_or_new_input_family`, or the latter alone. The
first case proves real, bounded behavior: suite/5 output, 29 generated Billing scenarios, exact
authored-source digests, parsed pretty/compact equality, canonical compact bytes, and preservation
of a destination on a named upstream refusal
(`evolution_coverage_accounting.rs:302-363`). The source-lineage and compact-byte fault controls
went red and their restorations passed, so those two checks are causal.

The variant case, however, asserts only that each of 16 whole-model overlays changes
`provenance.spec_digest` while the unrelated authored source map stays equal
(`evolution_coverage_accounting.rs:365-397`). It never asserts a generated member, omission,
projection value, refusal, or unchanged output attributable to a ledger model identity. The 1,730
`current-existing` rows all share the generic input text “the complete billing specification” plus
“the sixteen separated ... variants”; this includes dependency-graph relations, compiler handles,
private IR parts, and schema containers that the Billing assertion neither names nor varies. The
other 665 rows use the same family overlays described below. A changed specification digest is an
identity observation before the row's claimed final-merge behavior; it cannot establish all 2,395
model coordinates.

No final-merge `Supported` row is source-to-claim admitted as currently written. A correction must
map each retained supported coordinate to an isolated input and exact merge result, or to an
explicitly justified unchanged complete result at this entrypoint; coordinates the boundary does
not observe need their finite non-Supported disposition.

### 2. Selection carries exact suite bytes but not the 665 claimed model values

`selection_observes_each_independent_replacement_or_new_input_family` generates one authored suite
per overlay and proves that `coverage::select` retains its exact original suite bytes and selects
one scenario (`evolution_coverage_accounting.rs:512-547`). The separate Billing case proves one
requested ID and output preservation on an unknown-ID refusal (`:447-498`). These are valid
selection-boundary facts.

The selected suite does not carry the source model or the 665 model values; for most rows the only
effect of the overlay in the carrier is the specification digest. The authored scenario explicitly
states that it is retained while “unrelated model coordinates vary” and executes only
`workbench.todo.CreateList` with outcome `created`
(`stable-scenario.yaml:1-10`). Exact parent bytes therefore prove carrier preservation, not the
ledger assertion that every named model occurrence was independently changed and observed by
selection. All 665 `Supported` selection rows need coordinate-bearing carrier content or an exact
source rationale for unchanged selection behavior.

### 3. Execution records one unsupported unrelated scenario for all 665 rows

The Billing execution case is sound for its actual boundary: report/1 is refused before output,
report/2 runs the exact admitted Billing suite, and all 29 scenarios pass
(`evolution_coverage_accounting.rs:549-595`). The 665 ledger rows do not cite that case as their
coordinate evidence. They all cite the variant case at `:597-630`, which runs the same unrelated
one-scenario `CreateList` suite under every digest and asserts exactly one `unsupported` result,
zero passes, and overall execution failure.

An exact report bound to an admitted suite is useful evidence for that scenario and the interpreted
target's limitation. It is not successful execution of periodic, selection, accessor, settings,
reading, response, subject-state, naming, or schema coordinates. All 665 `Supported` execution
rows require coordinate-bearing executable scenarios and asserted results, or a non-Supported
disposition where `coverage::execute` cannot observe the coordinate.

### 4. Firefox executes 17 copies of the unrelated scenario, not 665 model coordinates

The browser receipt is real. Firefox 155.0.1 admitted and stepped the base plus 16 overlay sites,
each with one scenario; the test checks cursor movement, scenario count, and visible authored
selection (`evolution_coverage_accounting.rs:730-829`). Rust and Firefox both refused the 12
mutated periodic replay documents made at `:632-728`. Those observations prove actual browser
admission/replay and the named refusal vectors.

They do not prove the row assertions. The browser step is again the unrelated `CreateList`
scenario. The test never asserts the projected value attributable to a ledger row. Even the
`periodic` overlay changes only `every: PT2S` to `PT5S`, while 220 rows are assigned
`variant_inputs: ["periodic"]` and claim their exact authored occurrence was independently changed.
The remaining periodic words, host fields, type references, schema branches, and parent unions are
not independently varied by that overlay. The same many-to-one attribution occurs in every other
family. All 665 browser `Supported` rows need a model-to-projection-to-browser crosswalk with exact
value or justified unchanged behavior and causal controls; one admitted replay step and a changed
digest are insufficient.

## Finding: ledger evidence references

### 5. All 64 gap rows contain nonexistent S7 control-case names

The gap dispositions themselves are correct: two semantic-graph identities are not carried in
`EssIr`, and the 14 named-reading schema coordinates are refused before an S8 source consumer can
receive a valid current model. The actual S8 refusal case is present in every row's
`refusal_cases` and observes all three named-type refusals plus destination preservation.

Their `control_cases` fields are stale. The two graph rows per profile name
`every_s7_profile_has_its_own_complete_finite_ledger`; the 14 reading rows per profile name
`invalid_named_reading_arms_stop_upstream_of_every_s7_entrypoint`. Neither Rust case exists in this
tree. `assert_complete_rows` checks only that unsupported rows have a nonempty reason and owner
(`evolution_coverage_accounting.rs:128-131`), so the green ledger test cannot detect either broken
reference. Replace all 64 stale controls with exact S8 case coordinates and make the ledger test
validate that referenced case names exist.

## Findings: periodic projection and readers

### 6. Periodic replay changes the closed persisted envelope without a new format version

`web.rs:202-212` now correctly projects `ResolvedPeriodic.contract` instead of the compiler-only
`context` and `read` tables, and it leaves the event branch unchanged. `Binding` serialization in
`web_replay.rs:100-111` preserves the event-only member order and omits the absent periodic field.
The retained browser targets also include a passing legacy-player-byte case. Those facts support
the claimed event-only compatibility boundary.

They do not authorize periodic documents under the unchanged
`ess-conformance-replay/1` discriminator (`web_replay.rs:12,182-190`). The repair adds a new
optional member and a new accepted cause to a closed persisted envelope, and an old replay/1 reader
refuses it. Repository policy requires a new format version when the persisted envelope changes
(`AGENTS.md:48-49`), while the admitted periodic design requires explicit new format vocabulary and
previous-reader refusal tests (`docs/design/periodic-binding-triggers.md:90`). “No previous
functioning periodic persisted form” is not an exception to either rule. Preserve event-only bytes
under their existing contract, assign the periodic envelope its coordinated format vocabulary,
and retain the required current/previous-reader observations.

### 7. Rust admits nullable field naming metadata that JavaScript refuses

The two repaired readers do not have the same source grammar. Rust deserializes periodic host fields
through `Field`, whose flattened `Naming` has `Option<String>` members for `wire`, `display`, and
`summary` (`name.rs:196-210`; `types.rs:306-321`). Explicit JSON `null` is therefore admitted as
`None` and omitted when Rust canonicalizes the replay. The browser reader instead calls `text` for
each present naming key (`coverage-admission.js:611`), so it refuses the same `null` value. The
source contract explicitly treats the null form as valid; the existing container design calls for
an `F-null` control.

The 12 parity vectors do not include this case. Add the nullable naming vectors for both host field
tables and make Rust and JavaScript agree before claiming matching source grammar. The rest of the
reviewed repair has no source finding: exactly-one event/periodic, closed periodic and host objects,
canonical positive-u32 period parsing, component/binding/field spelling, type-reference depth and
Binary64-map-key refusal, per-table 64-field limits, and event-only omission are implemented on
both sides.

## Source-path dispositions and retained receipts

| Production path | Disposition |
| --- | --- |
| `crates/verify/ess-conformance/src/web.rs` | Source-only periodic projection is correct; event projection remains byte-shaped as before. Its new periodic output participates in finding 6's format boundary. |
| `crates/verify/ess-conformance/src/web_replay.rs` | Needs revision for unchanged replay/1 vocabulary and participates in the nullable-metadata parity finding. Exactly-one and typed periodic validation are otherwise bounded correctly. |
| `crates/verify/ess-conformance/assets/coverage-admission.js` | Needs revision for nullable naming parity. Its reviewed period, name, type, closure, exactly-one, and host-limit checks otherwise match the Rust source rules. |

The retained logs prove only their executed boundaries, and they are credible for those boundaries:

- `25-new-target-final` passed all 9 S8 cases and records Firefox 155.0.1, 17 one-scenario replays,
  and 12 Rust/Firefox refusals.
- `19-existing-browser-targets` passed 10 coverage-browser cases (including 314 closed-model
  vectors and retained legacy player bytes) and 30 replay-fidelity cases.
- `28-full-ess-cli-package-final` contains 61 runner blocks totaling 609 passed, zero failed, and
  five pre-existing ignored tests; the S8 target appears there with all 9 cases passing.
- `27-ess-cli-clippy-final` and `26-scoped-fmt-final` exited zero. The final formatting receipt is
  an empty success log.
- `09/10` and `11/12` are meaningful red/restored controls for authored-source digest and compact
  canonical bytes. `08` and `15` are the genuine browser failures that exposed the old reader and
  compiler-only projection; `17` is the restored Firefox pass.

These executions establish build health and the bounded facts listed above. They do not turn
generic digest, source-lineage, carrier, count, or unsupported-scenario observations into 4,390
model-coordinate claims. Native qualification, aggregate authority, integration, and global gates
remain outside this review.

```findings
- file: crates/edge/ess-cli/tests/fixtures/evolution_coverage_accounting/non-authority-coverage-final-merge.json
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 2,395 Supported rows are attributed to one Billing merge and sixteen whole-model digest changes without an exact generated, omitted, refused, projected, or justified unchanged result for each model coordinate.
- file: crates/edge/ess-cli/tests/fixtures/evolution_coverage_accounting/non-authority-coverage-selection.json
  line: 58
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows infer model-coordinate selection behavior from exact parent suite bytes whose only coordinate-specific effect is generally provenance; the selected CreateList scenario expressly remains unrelated to the varied model coordinates.
- file: crates/edge/ess-cli/tests/fixtures/evolution_coverage_accounting/non-authority-coverage-execution.json
  line: 58
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows execute the same unrelated one-scenario CreateList suite and assert an unsupported failed report, so no claimed model coordinate has a coordinate-bearing execution result.
- file: crates/edge/ess-cli/tests/fixtures/evolution_coverage_accounting/non-authority-browser-paired-replay.json
  line: 58
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 665 Supported rows reuse one unrelated browser scenario and a family-level overlay without asserting the projected value or independently varied occurrence attached to the row.
- file: crates/edge/ess-cli/tests/evolution_coverage_accounting.rs
  line: 128
  category: contract-drift
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: All 64 UnsupportedAtThisEntrypoint rows retain nonexistent S7 control-case names, and the ledger validator checks only nonempty reason and owner fields rather than resolving evidence references.
- file: crates/verify/ess-conformance/src/web_replay.rs
  line: 12
  category: contract-drift
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The repair admits a new periodic member and cause in the closed persisted replay envelope while continuing to emit ess-conformance-replay/1, contrary to the repository format rule and the periodic design's explicit vocabulary boundary.
- file: crates/verify/ess-conformance/assets/coverage-admission.js
  line: 611
  category: boundary
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Rust admits explicit null for periodic host field wire/display/summary metadata through Option<String>, while JavaScript requires every present value to be text; the claimed Rust/Firefox source-grammar parity is false and the 12 parity vectors omit this case.
```

Original local review SHA256: 340b8e43612160ae71f578bbd44fab320c6dc3f0d6ef38ceba427a52df3ad86a. Only workstation-path text was projected; findings unchanged.
