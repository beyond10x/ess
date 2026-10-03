---
format: aep.planning-md/3
id: story:nested-increment-previous-location
kind: story
status: draft
title: Nested increments read their own previous stored location
relations:
- informed_by: story:feature-request-292
- depends_on: story:feature-request-292
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/values.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
revision: 4
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

Accept as a semantic defect, using existing vocabulary. Keep this story draft until binding design, exact cross-target scope and source admission controls are established. It depends on feature-request-292 for shared executor/proof-surface sequencing; that dependency is not a claim that #292 caused the bug. The same future held-bundle integration owns delivery, with no separate PR. Publication-ancestry and publication-identity holds remain unchanged.

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

- Named nested_previous_value regression exercises compiled source, real update and independent expected observation:100→101; top-level3 stays3; changing top-level8 cannot alter the nested result.
- The same nested increment admits without the unrelated top-level member; a truly missing or nonnumeric nested target remains refused.
- Deeper same-leaf siblings and supported parent/container cases use the correct pre-outcome location, with positive and fault-sensitive controls.
- Exact integer/decimal arithmetic, unknown history values and invariants retain their existing guarantees; no invented observation or partial transition is published.
- Independent generated Rust/Go/TypeScript and actual browser execution plus synthesized assertions cover the location distinction. No shared wrong expectation is accepted as independent evidence.
- Compatibility decision, regression red/green, scoped package checks and independent review precede the single bundle integration.

## Scope

Cited implementation seams in held source: command/value_expression.rs check_increment/existing_subject_field; compiler resolve.rs nested payload lowering; interpret/execute.rs nested location; interpret/execute/values.rs increment; synthesize.rs increment expectation; ess-synth/src/rust/behaviour.rs Increment. Tests and binding-design paths are inferred. Go/TypeScript and browser exact paths need final impact scoping before dispatch. Do not edit these concurrently with #292.

## Exact-main source risk audit

At operator release-handoff request, root inspected exact main e68684efb6a4ac22052c77d3ed8292fd44f9ace5. Its domain checker still invokes existing_subject_field using the nested leaf name at value_expression.rs:727. Rust generated Increment still formats the whole before row plus field.target at rust/behaviour.rs:1546; synthesize.rs:6202 reads before.get(field.target) for the expected increment. These source seams match the location error diagnosed in held runtime and establish a concrete main compiler/generator/expectation risk. No main executable was built or executed for this audit. Main retains the older interpreter that does not derive these executions, so the held-runtime observed4/9 results are not claimed as main-native results. Receiving integrator should reproduce generated-main behavior before deciding the release disposition; no main or source patch was made by this handoff.
