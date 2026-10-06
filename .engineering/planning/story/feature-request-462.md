---
format: aep.planning-md/3
id: story:feature-request-462
kind: story
status: implemented
title: A row-set guard on an upsert command (external create, updating default) synthesizes as Unknown (ESS-SYNTH-001)
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#462
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/row-set-upsert.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/row_set_upsert.rs
- confidence: cited
  path: docs/design/filtered-related-reads.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T23:57:35Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-05T23:57:36Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-06T17:49:23Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#462: A row-set guard on an upsert command (external create, updating default) synthesizes as Unknown (ESS-SYNTH-001).

## Origin
beyond10x/ess#462, filed 2026-10-05. Found while specifying a storage port whose write is an upsert and whose refusal depends on a row of another entity. A follow-up comment reports the same refusal for a deleting default.

## Fit review
1. Need: a row-set refusal whose selector compares with the input (`curator == input.curator`, `exists: true`) must be witnessed on a command that also addresses an existing record through the input. That covers an upsert (external creation, then an updating default), a plain update and a delete. The requester expects `refused` to be arranged on a matching row between decoys, and `replaced` on decoys only. They propose no syntax. Fresh probes are in `<fit-review scratch>/probe-462/` (entities `Library {curator}` and `Book`; commands `Shelve` and `Unshelve`):
   - Installed `ess 0.53.0` (`ess-version.out`) at `ess/22`: `upsert/`, `update/` and `drop/` each validate, then synthesis refuses both branches with ESS-SYNTH-001: "its own row set is Unknown" and "`refused` is not decidedly passed over: its row set is Unknown" (`*/synth.out`). `create/` (creation only) gives 3 scenarios and 0 refusals. So the defect is wider than upserts: any input-reading selector on a command whose subject the input names by instance is affected.
   - A 0.54 development build at `/dev/shm/ess-054/coord/debug/ess` (built 2026-10-05 23:24, after 6ee25379c; the exact commit is not verified, see `devbinary.out`), at `ess/23`: `v23-upsert/`, `v23-update/` and `v23-drop/` each give 4 scenarios and 0 refusals. `refused` arranges a `Library` with `curator-1` and one with the input's `curator`. `replaced` arranges the decoy only (`v23-upsert/suite.json`), which is the issue's Expected. The same binary at `ess/22` still refuses both (`dev22-upsert.out`).
   - The interpreter with the dev build at `ess/23`: `v23-update/` and `v23-drop/` pass 4/4. `v23-upsert/` passes 1 and fails 3, and every failure is at the `Shelve` forced to `added`: "no declared outcome was reached; the target refused for a reason the specification does not model" (`v23-upsert/run.json`). Installed 0.53.0 at `ess/22` fails `Shelve/outcome/added` the same way (`upsert/run.out`). Without the guard, the upsert passes 3/3 on both binaries (`v23-upsert-noguard/`, `upsert-noguard/`).
2. Class: defect, two parts.
   - Synthesis. The design admits `input.<path>` in a selector on any command (`docs/design/filtered-related-reads.md:9-11`, `:75`), and validate accepts the probe. Mechanism, read in code and consistent with the probes but not traced at runtime: below `ess/23`, `Reading::new` binds only literal inputs (`crates/verify/ess-conformance/src/synthesize/row_set.rs:421-425`), which drops the identity sent as an arranged instance. `evaluate_row` then flattens the input as total (`crates/verify/ess-conformance/src/input.rs:73-84`), fails, and answers `Unknown` (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:508-510`). The requester's guess (`consistent()`) is where the error surfaces (`row_set.rs:617`, `:624`), not where it starts.
   - Interpreter. `addressed_row` runs before any row set (`crates/verify/ess-conformance/src/interpret/execute.rs:602-606`). It treats the updating default's subject as addressed, and answers an absent identity with `unknown_instance` (`:1048-1050`), even when a creation branch of the same entity takes that absence. Without a row set that pre-check is skipped and the upsert passes. So the reference target fails a suite ESS itself synthesizes.
3. Existing idiom: for synthesis, declare `ess/23`. On the integration head (520ea83a7), story:feature-request-429 already writes the literal input into the selector (`row_set.rs:439-446`) and resolves an instance input to its literal identity (`row_set.rs:332-351`, `:421-425`), from `ess/23` only (6ee25379c: "gate identity selectors to ess/23"). The probe above shows it working. Nothing documents it: the `ess/23` row (`website/docs/reference/spec-versions.md:65`) names only re-keys and `{subject: state}`. The interpreter part has no idiom; no authored spelling avoids `addressed_row`.
4. Fit: no new surface.
   - Synthesis: keep 429's fix and its `ess/23` gate. The comment at `row_set.rs:435-438` names this very case (an instance input "leaves the rest of the input unflattened") and keeps older documents on the suite they had, and the bytes of `ess/22` suites stay put (the 0.53.0 refusal message is embedded in the suite, `dev22-upsert.json`). Add regression cases for the three shapes.
   - Interpreter: in `addressed_row` (`execute.rs:1016-1059`), an absent identity is not an unknown instance when a creation branch of the same entity takes its identity from the same input field. The answer falls through to the row sets, then to the creation. This is the order the design states: row-set tests come after addressed-row existence and before the accepting branch (`filtered-related-reads.md:82-86`). The fix is format-independent because it changes no suite, only the reference target's verdict, which today contradicts the specification at every format.
   - Siblings: an update or delete default with no creation keeps `unknown_instance` (`v23-update/` and `v23-drop/` pass today). Go and TypeScript generated runtimes run the adopter's implementation, not this check. Whether the entity runtime lowering has the same pre-check, I don't know.
5. Second adopter: a settings store. `PutSetting {key, tenant}` creates a setting when the key is absent and replaces it otherwise, and is refused while a `Freeze` row exists for the tenant (`when_related: {entity: Freeze, where: tenant == input.tenant, exists: true}`). Or a team roster whose `RemoveMember` is refused while any `Hold` exists on the member's team. Both selectors read only the input.
6. Cost: no format bump of its own. `ess/23` is owned by story:feature-request-429. This story reports one sentence for the `ess/23` row and does not edit `spec-versions.md`. No `ess-conformance/N` bump, because no step is new. No keyword and no diagnostic. Suites at `ess/≤22` keep their bytes. Interpreter verdicts change only from fail to pass for upsert commands guarded by a row set. Size S: one guard in `addressed_row`, one fixture, one test file, one design-note sentence.
7. Alternatives:
   - (a) Change nothing. Synthesis at `ess/23` already works, but the reference interpreter fails the synthesized upsert suite, and the gated fix is undocumented.
   - (b) Lift the synthesis fix to `ess/22`. Rejected: it changes `ess/22` suites, which 6ee25379c gated deliberately.
   - (c) Bind the input as partial in `evaluate_row` at every format. This changes the same bytes as (b), and widens a total-input invariant other callers rely on.
   - (d) Chosen: keep 429's gate, fix the interpreter, lock both with tests, and document both.
   - The requester's expectation is taken as stated.

## Decisions
accept as proposed. The synthesis behaviour the issue expects already holds from `ess/23` on the integration head (story:feature-request-429, 6ee25379c). This story:
- adds regression cases for the upsert, update and delete shapes;
- fixes the interpreter's `addressed_row` for an absent identity that a creation branch takes;
- adds one sentence to `docs/design/filtered-related-reads.md` "Subject borrowing and precedence" (`:73-91`);
- reports this sentence for the coordinator to merge into the `ess/23` row: "A row-set selector that compares with the input is decided on a command that also names an existing record through the input (an update, a delete or an upsert); below `ess/23` such a branch stays refused (beyond10x/ess#462)."

`ess/22` documents stay refused, and the reply to the requester says to declare `ess/23`.
- Format: none new; shares `ess/23`.
- depends_on story:feature-request-429 (merged on the integration branch; owns `FormatVersion::V23` and the `ess/23` row).
- No edge to #463. It edits `synthesize/row_set.rs`, and this story does not.
- Shares `interpret/execute.rs` with #429, #457 and #458 at other sites (re-key, `BrokenInvariant`, `SubjectField` payloads). This story touches `:1016-1059` only. Dry-run `git merge-tree` before the second merge.

Noted, not fixed: the ESS-SYNTH-001 `help:` for a row-set gap says "give the field a type that has a finite value" (`crates/verify/ess-conformance/src/synthesize.rs:1104-1106`), which is wrong for a row set. Changing it changes the bytes of `ess/22` suites.

## Acceptance
- row_set_upsert_synthesizes_every_branch: the upsert fixture at `ess/23` synthesizes `added`, `refused` and `replaced` with no refusal. `refused` arranges a selected-entity row with a different `curator` and one with the input's. `replaced` arranges only the decoy.
- row_set_update_and_delete_defaults_synthesize: the update-only and delete-default variants at `ess/23` synthesize both branches with no refusal.
- row_set_input_selector_below_ess23_unchanged: the three variants at `ess/22` still refuse both branches with ESS-SYNTH-001 "its own row set is Unknown", and their suites match the 0.53.0 bytes.
- interpreter_creates_absent_upsert_row_beside_row_set: the interpreted run of the `ess/23` upsert suite passes 4/4 (1/4 today). `Shelve/outcome/added` also passes at `ess/22`.
- interpreter_row_set_refusal_precedes_upsert_creation: on an absent book with a matching library, the interpreter answers `refused` and creates no book.
- interpreter_update_without_creation_keeps_unknown_instance: an update-only command guarded by a row set, sent an absent identity, still takes the unknown-instance answer.
- upsert_suite_catches_faulty_targets: a scripted target that ignores the selector fails `refused`. One that refuses whenever any library exists fails `replaced`. An honest target passes all three.
- upsert_precedence_sentence_in_design_note: `filtered-related-reads.md` states that an absent addressed row which a creation branch takes is the creation's, not an unknown instance. A case in the new test file reads the page and fails without that sentence.

## Scope
- crates/verify/ess-conformance/src/interpret/execute.rs  cited — `addressed_row` (1016-1059), called before row sets (602-606)
- crates/verify/ess-conformance/tests/row_set_upsert.rs  inferred — new cases above
- crates/verify/ess-conformance/tests/fixtures/row-set-upsert.yaml  inferred — the fit-review model at `ess/23`, with update and delete variants
- docs/design/filtered-related-reads.md  cited — Subject borrowing and precedence (73-91)
