---
format: aep.planning-md/3
id: story:nested-increment-previous-location
kind: story
status: implemented
title: Nested increments read their own previous stored location
relations:
- informed_by: story:feature-request-292
- depends_on: story:feature-request-292
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/browser_response_conformance.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/declared-behaviour-go-harness/main.go
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/declared-behaviour-harness/main.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/declared-behaviour/domains/ticket.yaml
- confidence: inferred
  path: crates/specify/ess-compiler/tests/nested_increment_location.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/value_expressions.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/values.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.test.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/generated_history_values.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/interpreted_value_expressions.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/value_expressions.rs
- confidence: inferred
  path: docs/design/value-expressions.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T15:59:29Z", actor: "human:timo", revision: 8, executor: "agent:codex-ess-bundle-resume-20261003"}
- {from: "proposed", to: "active", at: "2026-10-03T15:59:29Z", actor: "human:timo", revision: 9, executor: "agent:codex-ess-bundle-resume-20261003"}
- {from: "active", to: "implemented", at: "2026-10-08T09:55:09Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

A nested numeric increment reads the same nested stored location before the outcome. A top-level field with the same leaf name must neither supply the previous value nor be required for admission. Preserve the existing documented value-expression semantics across interpreter, generated targets and synthesized expectations.

## Fit review

1. Need: an update to packet.amount must turn 100 into 101 independently of an unrelated amount=3. This is a minimal synthetic model, not adopter data. No new syntax is requested. Actual frozen CLI author/run receipts below show packet.amount=4.
2. Class: defect. docs/design/value-expressions.md E3 says increment uses the field's value before the outcome; E5 nested mapping describes one value expression per struct leaf. The admitted source contradicts this semantic contract.
3. Existing expression: sets: {packet: {amount: {increment: 1}}} is admitted when the unrelated top-level amount exists. Removing that field causes undeclared_reference. A nested literal101 control executes successfully, so replacing the arithmetic with a literal is an experimental control, not a general solution.
4. Fit: retain existing syntax and pre-outcome snapshot semantics, using the actual nested target location. Admission, lowering, native execution, Rust/Go/TypeScript generation and synthesis must agree. Optional parents, deeper nesting, exact numeric arithmetic, set effects and history unknowns require explicit support or existing semantic refusal; no silent leaf-name fallback. Full sibling impact remains to be scoped before implementation.
5. Generality: an inventory package can increment package.quantity while an unrelated total quantity exists; a billing struct can increment payment.attempts without a top-level attempts. Both require location-correct arithmetic, not local policy.
6. Cost: no new authored vocabulary is proposed. Whether the existing resolved representation needs path context must be decided before code; any persisted meaning change must receive the repository's explicit compatibility assessment. Generated behavior and previously wrong synthesized expectations change; this is not permission to silently change a format contract.
7. Alternatives: change nothing preserves demonstrably wrong admitted execution. Ban all nested increments would remove expressible behavior and still needs a compatibility decision. Prefer target-location semantics already stated by the binding design, with explicit handling of cases whose parent is absent. Do not exploit top-level lookup to claim safe unknown arithmetic.

## Decisions

Treat the defect as lost target ancestry, not as missing expression syntax. The target location of
an increment is the sequence of `ResolvedPayloadField.target` names from the root `sets:` field to
the increment leaf. Preserve that sequence as typed segments until the final target-specific read;
do not search by leaf name, parse a dotted author string, or fall back to a top-level field.

Keep the existing recursive IR. It already represents the path, so adding a path member would
change serialized bytes without adding meaning and requires a separate compatibility decision.
This correction changes no source, IR, or suite format version. Regenerate affected synthesized
suites and Rust/Go source because their previously wrong derived bytes must change.

Source admission requires an existing subject, the resolved nested target, and a required numeric
leaf. It does not require a top-level entity field with the leaf's name. Required and deeper struct
parents are supported. An Optional struct parent is supported when present; absence produces the
existing no-value/unmet-obligation result before any state or event publication. Optional numeric
leaves remain refused. List, Map, Enum, and Union traversal remains refused; this story adds no
index, key, or variant syntax.

Native/history and synthesis retain exact Integer and Decimal arithmetic. Unknown history leaves
may transfer an increment only with the existing complete-domain, exact-overflow, and target-
constraint proof. Generated Rust/Go retain their current Integer-only increment capability;
recursive planning must mark nested Decimal behavior owed rather than emit invalid behavior.
Newtype-over-struct keeps its current explicit generated-target disposition.

TypeScript acceptance runs the corrected suite and an old-leaf mutant through the TypeScript
runtime; it does not add a TypeScript command-behavior emitter. Browser acceptance uses the live
generated-Rust product after the browser product is composed. Legacy input replay is not execution
evidence for Struct or Increment.

Implement after the reviewed #292 candidate is composed because both units edit the executor and
history regression surfaces. Finish this bounded correction before broader #282 expression work
touches the same files. It remains part of the single held-bundle integration.

## Evidence

Root executed the retained CLI ess-baseline SHA256 d397e76cd811915e6efb9bfd33e0a7c7483ce65d9518ff229de0cbcab56fd23b. Private evidence directory is ess-nested-increment-probe-20261003. This is measured against frozen held-bundle source, not a freshly built release-main executable.

- model.yaml SHA256 656fb79e380d480d6f9648432ad6f545e0aca3717e711e4173fcbb1fd7fc7a41; scenario.yaml 74a4ec3d55e892cfb37005ce5ed9f8a1c230050707c3d3d90de14cdc79a112f9.
- `ess verify conform author --path model.yaml --scenarios scenario.yaml --out suite.json`: exit0, one authored scenario, zero refusals, suite/34. Exact suite SHA256 0efde37cb05991c8f8c9931c93162d30a0c055ac2732630bf98363d5bac392ac.
- `ess verify conform run --path model.yaml --suite suite.json --target interpreted --report-format 2 --report-out report.json --format json`: exit1, one failed, zero error/skipped/unsupported. Outcome and row-count checks passed; expected packet.amount101, observed4 while top-level amount remained3. run.json SHA256 0a98d78de1940081773f75878db3d03a4d29278739e063ce64745854a67b9388; report.json c94944e6f562ec64b9553a76ee8d777b4bb30962dfae08ee441bd4ea228a2c07.
- Changing only the arranged top-level amount and its preservation assertion from3 to8 produces nested9, still one failed: top-level-eight-run.json e02a598a5e1ca898e0bc8970dd044c5bb915458267ff0894d0a13d12a0ca40a8.
- Replacing nested increment with literal101 passes the same assertion, one passed, exit0: literal-run.json 70cf9543bbf702a205b9334415b4b3bfdabed34713a48d0f551d31e90f1f6b33. This is a control, not a fix.
- Removing the top-level amount from entity/view causes source refusal: no-top-level-validate.log c44988ffb07de4823249a5d02d5fa15b3f842f3770c363f802d9cef7d0a07d21. The nested member remains declared.
- Two earlier fixture-admission errors (outcome without observable event; invalid input-field spelling) are retained separately in author-first and author-second logs. They are not product red results.

Hypotheses after the first admitted red: wrong top-level lookup; update not applied; faulty nested assertion. The top-level variation and literal control support the first and distinguish the other two. Source inspection confirms increment reads before.fields[field.target] in interpret/execute/values.rs; the checker existing_subject_field searches top-level identity/fields. The source-only audit increment-language-disposition-review.md SHA256 eef828749d298be8269847a57b00dab6faffb7229d1da6ad73651e13cb40e87a also identifies corresponding synthesis and Rust-emission lookup sites. Generated-target execution is not yet measured.

## Acceptance

- `nested_increment_uses_its_declared_location`: the source admits without a top-level `amount`;
  compiled ancestry is `packet -> amount -> Increment`; missing/nonnumeric nested targets,
  Optional numeric leaves, creation reads, and list/map/union parents retain their existing
  refusals.
- `nested_increment_reads_its_full_pre_outcome_path`: `packet.amount` changes `100 -> 101` whether
  top-level `amount` is absent, `3`, or `8`; the top-level value is preserved. Deeper paths and two
  same-leaf sibling structs do not alias.
- `nested_increment_preserves_location_and_unknown_arithmetic`: known values above `2^53`, exact
  Decimal values, unknown complete domains, overflow, constraints, and unresolved parents retain
  sound history behavior. A failed read/arithmetic publishes no partial store or event.
- `nested_increment_expectation_uses_the_full_path`: synthesis expects `packet.amount = 101`. A
  deliberate expectation/target mutant implementing the old top-level-leaf lookup fails, so the
  synthesized expectation is not accepted as its own oracle.
- `generated_nested_increment_behaves_in_rust_and_go`: generated Integer behavior builds and runs
  its own suite for required, deeper, and present Optional parents; absent Optional parents return
  an unmet obligation before storage mutation. Nested Decimal is explicitly planned as owed.
- The TypeScript runtime passes the correct nested result and fails the old-leaf mutant. The live
  browser product executes the same case in ordinary and coverage routes and matches native exact
  report/run bytes. Legacy browser replay supplies no acceptance claim.
- Focused tests, affected package tests, format/lint checks, `task check`, and independent review
  pass before the single bundle integration. Any committed executable or checker is Rust.

## Scope

Derived 2026-10-03 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surfaces:** `crates/specify/ess-domain/src/command/value_expression.rs`, `crates/verify/ess-conformance/src/interpret/execute.rs`, `crates/verify/ess-conformance/src/interpret/execute/values.rs`, `crates/verify/ess-conformance/src/synthesize.rs`, and `crates/generate/ess-synth/src/rust/behaviour.rs` — cited
- **Also required:** `crates/generate/ess-synth/src/go/behaviour.rs` and `crates/generate/ess-synth/src/determined.rs` — inferred from the recursive emitter and target-admission call graph
- **Domain tests:** `crates/specify/ess-domain/tests/value_expressions.rs` — inferred from the existing increment and nested-mapping cases
- **Compiler test:** `crates/specify/ess-compiler/tests/nested_increment_location.rs` — inferred as a focused new ancestry regression; compiler production files do not change
- **Native/synthesis/history tests:** `crates/verify/ess-conformance/tests/interpreted_value_expressions.rs`, `crates/verify/ess-conformance/tests/value_expressions.rs`, and `crates/verify/ess-conformance/tests/generated_history_values.rs` — inferred from existing increment suites and the frozen #292 surface
- **Generated-target tests and fixtures:** `crates/generate/ess-synth/tests/declared_behaviour.rs`, `crates/generate/ess-synth/tests/fixtures/declared-behaviour/domains/ticket.yaml`, `crates/generate/ess-synth/tests/fixtures/declared-behaviour-harness/main.rs`, and `crates/generate/ess-synth/tests/fixtures/declared-behaviour-go-harness/main.go` — inferred from the real Rust/Go build-and-suite harness
- **TypeScript acceptance:** `crates/verify/ess-conformance/src/ts/runtime.test.ts` — inferred; the Rust collector already executes every TypeScript test file
- **Browser acceptance:** `crates/edge/ess-cli/tests/browser_response_conformance.rs` — cited from the pending browser product's actual Firefox/native matrix
- **Document:** `docs/design/value-expressions.md` — inferred for the full-target-path and absent-Optional-parent clarification
- **Confidence:** high — the retained probe, story, and each production lookup identify the same local-leaf defect — cited
- **Would collide with:** any unit touching the executor/history, synthesis, or generated-behavior surfaces; compose #292 first and serialize broad expression work on the same files — cited
- **Safety fact:** nested IR ancestry already retains every target segment, so the fix needs no authored syntax or persisted IR member — step 2 (`ir.rs:1026-1048,1191-1205`; `resolve.rs:3004-3080`), unproven

## Exact-main source risk audit

At operator release-handoff request, root inspected exact main e68684efb6a4ac22052c77d3ed8292fd44f9ace5. Its domain checker still invokes existing_subject_field using the nested leaf name at value_expression.rs:727. Rust generated Increment still formats the whole before row plus field.target at rust/behaviour.rs:1546; synthesize.rs:6202 reads before.get(field.target) for the expected increment. These source seams match the location error diagnosed in held runtime and establish a concrete main compiler/generator/expectation risk. No main executable was built or executed for this audit. Main retains the older interpreter that does not derive these executions, so the held-runtime observed4/9 results are not claimed as main-native results. Receiving integrator should reproduce generated-main behavior before deciding the release disposition; no main or source patch was made by this handoff.

## Bundle authority and verification sequencing

The operator's accepted remaining-bundle plan explicitly includes this correction in Batch A and authorizes implementation and verification. Its binding design is now recorded in docs/design/value-expressions.md E3. The read-only scope report was measured against integration 3e7db9aab and frozen history candidate cae6187ec; report SHA256 6b9b9e32281d4c31f965b459fb1d2b68ade8f7a3a65d1e67cb50e74c83cd5762. The retained original story body came from ess-consumer-backlog-20261002 revision4; this integration copy owns subsequent execution and review evidence. The canonical intake copy was not changed.

No decomposition panel is needed for this single existing correction story. Source-based scoping is not execution evidence. Affected package tests, strict lint, fault-sensitive controls and independent review precede unit integration. The complete task check and actual live browser acceptance remain required before the final bundle release; they may run once the pending browser product is composed, consistent with the repository's local/CI gate split. This scheduling clarification does not waive either acceptance obligation. Preserve one integration branch and one PR.

## Reviewed local implementation

The complete implementation candidate 7cba6e0fc3b1c047a8f4318f02e60c0cece1ca59, based on integrated history9f35a2d5d, passed independent full source review2of2 with no findings. Review1's two defects were fixed in the real generated-value emitters and TypeScript test syntax; the independent208-line prospective test patch remains unchanged (SHA2567fb1ef8faf131445af096b85dd627b4c9a74f9b2201129b9ac2161091cea39f1).

Fresh reviewer proof: declared-behaviour20/20 with actual Rust warnings-denied compilation, Go build and150 generated port/HTTP scenarios; focused conformance78/78; TypeScript runtime239/239 plus both configured typechecks with no skip; domain8/compiler1; strict scoped Clippy, formatting and diff checks. Publication report SHA256f9fafc3059af6cc74590c4d19a0a2362a0c27dd4c2154305ec759750266bfb4a. The story remains active until the separately assigned ordinary/coverage browser composition and final bundle gates execute; this evidence does not close an issue or release.
