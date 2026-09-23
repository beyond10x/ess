---
format: aep.planning-md/1
id: review-result:entity-runtime-lowering-source-pass-2
kind: review-result
status: active
title: 'Final complete lowerer source examination: creation identity across outcomes'
relations:
- reviews: story:entity-runtime-service-lowering
revision: 1
---
# Final whole lowerer source review — pass 2 of 2

**Verdict: NEEDS-CHANGE — one introduced blocker.**

I reviewed the complete 15-path lowerer delta from `be604d874ee9e567ae104e565e7cbddf953885a9` to candidate `faddaffc834bd1f3c6769b901ef986a4fbac1f75` (tree `a231f488bba619086d38923c6f3a8466e3b72af3`) against the original implementation contract and admitted design. This was the final whole-source pass, not a review of only the F1/F2 correction. Production source is unchanged by this review. I added one dedicated reviewer regression.

## Finding

### F3 — later accepting creation outcomes do not reuse the created instance identity

- **File:** `crates/generate/ess-entity-runtime/src/lib.rs`
- **Primary lines:** 1292–1340 and 1773–1793
- **Category:** creation / identity integrity / event value
- **Severity:** blocker
- **Origin:** introduced relative to the base (the lowerer crate is new); present in the original candidate and newly detected in pass 2, not introduced by the F1/F2 correction
- **Verdict:** NEEDS-CHANGE

`creation_coordinate` selects the first creation outcome at line 1297 and returns an `EventFieldCoordinate` containing that outcome's name at lines 1335–1339. The command then has one shared `creation_identity`, but `lower_outcome` substitutes that identity into an emitted field only when the current outcome's complete coordinate exactly equals the first outcome's coordinate at line 1779. An otherwise equivalent later accepting branch has a different outcome name, so its observed identity field falls through to normal payload lowering and receives a separate `Undetermined` event-field slot.

The compiled counterexample uses `contract.local.Run`. It duplicates the admitted `completed` creation branch as an earlier `conditional` branch and changes only the branch name plus `when: note == first`. Both accepting branches create `contract.local.Child`, declare `instance: child_id`, emit `contract.local.PrivateEmission` at occurrence 0, and map `PrivateEmission.child_id` from `{generated: true}`. Their resolved semantic identity sources are therefore the same observed event handle and field; only the outcome coordinate differs. Supplying false external evidence selects the later `completed` branch. The shared logical identity slot receives `278f4f3a-c8b8-4e86-9a16-2c385910fc68`, while the separately exposed `completed`/occurrence-0/`contract.local.PrivateEmission`/`child_id` slot receives `84f3e37f-4e8a-48c4-b019-895572d6c3ec`. The real Entity Runtime decision creates the instance with the first value but emits the second:

```text
left:  String("84f3e37f-4e8a-48c4-b019-895572d6c3ec")
right: String("278f4f3a-c8b8-4e86-9a16-2c385910fc68")
```

The source model is admitted by the ESS parser/compiler, `lower()` succeeds, registry closure validates, and the actual runtime reaches the mismatch. This is a behavioral failure, not a compile failure. It violates the design's creation and identity obligations: every accepting creation outcome declares the observed event field as the subject identity, so the emitted value must equal the created instance's logical identity.

The correction boundary is finite. Keep the command's one shared logical identity value, but associate each accepting creation outcome's own observed event occurrence with that shared value. The admitted semantic source is the resolved event and field, independent of the outcome name. A correction can therefore preserve the authored identity behavior for every accepting branch without arbitrarily choosing the first outcome and without refusing these already valid models.

Dedicated evidence is `crates/generate/ess-entity-runtime/tests/reviewer_source_review_2.rs` at SHA-256 `37a9541d1298412bf19bf98dbf2190513f3d560ef33df5eea623b5675c97a793`. `logs/04-multi-creation-identity.log` records the compiled runtime failure; Cargo exited 101 because the assertion failed after execution.

## First-review disposition

- **F1 resolved.** Slot remapping is now driven by typed `BoundTarget` locations rather than recursive rewriting of arbitrary strings. Literal escaping occurs at ER template/operand use sites. The corrected assertion expects the accepted ER encoding rather than a raw single-dollar target string. The focused correction test passes, and the actual-runtime test proves authored single-dollar and double-dollar strings survive a decision with their intended bytes.
- **F2 resolved.** Direct response reuse now carries the source `ResolvedTypeRef` and source presence. The original F2 assertion was not changed and passes.
- **Carried findings:** none. F3 is a new pass-2 finding with a separate root cause.

The corrected F1 test file has SHA-256 `416021cb43353287fcbff2ce4b89be1f00cc84df41db59d0ccbf5e8e81a1aa78`. Accepted Entity Runtime `250f6993181822ab1e36c17d38dbc909d084d423` interprets a single `$` prefix as an expression and `$$` as a literal dollar. Consequently, encoding authored `"$..."` as `"$$..."` and authored `"$$..."` as `"$$$..."` at target boundaries is required. The runtime check confirms this rather than inferring behavior from lowered JSON alone.

## Complete ten-dimension disposition

| Dimension | Examination and disposition |
|---|---|
| Creation | **F3 blocker.** One logical identity is derived, but only the first accepting outcome's exact coordinate receives it. Creation field sets, set-if-present handling, definition version lookup, and create/update separation otherwise match the design. |
| Updates | Update identities, required and optional field actions, preserves, and update transition selection were traced through lowering and the service fixtures. No additional finding. |
| Transitions | Accept/refuse outcomes, predicate selection, external evidence slots, and ordered runtime outcomes remain explicit. Unsupported or incomplete input produces diagnostics rather than partial output. No additional finding. |
| Predicates | Input, stored fact, relation, and external predicates lower through the declared operands. Literal fact values are escaped only at the ER operand boundary. No additional finding. |
| Invariants | Required, optional, collection, and union type lowering and recursively nested literal templates were examined. No additional finding. |
| Identities | **F3 blocker.** The command-level shared identity is not connected to every semantically identical branch-local observed occurrence. Supplied and bound creation identity handling otherwise follows the admitted source model. |
| Relations and closure | Local/foreign references, relation targets, definition registration, and `Registry::validate_all` closure are covered by the billing/gatepass contract fixture and the executed counterexample. No additional finding. |
| Exact values and fields | F1 and F2 are resolved. Typed slot canonicalization covers external evidence, entity set/set-if-present, event payload/payload-if-present, response/responds-if-present, and bound creation identity locations. Literal `$` handling is source-preserving at runtime. No additional finding. |
| Outcomes and commands | Service 1/2/3 selection, command binding tables, exact selected outcomes, response fields, and precise unsupported diagnostics were traced. F3 is caused by branch-specific coordinates within otherwise admitted command outcomes. No separate finding. |
| Event order and multiplicity | Event occurrences retain authored order and repeated occurrences remain distinct. **F3 also affects the value of the identity occurrence in a later accepting creation branch**, but no independent order or multiplicity defect was found. |

I also traced the opaque fulfillment action sequence through the lowerer. It preserves the accepted ER action order and field targeting; it does not introduce another finding in this delta.

## Executed evidence

All behavioral commands used Rust 1.85.0, `--locked --offline -j 1`, lld, `CARGO_INCREMENTAL=0`, debug info disabled, a separate target directory, and the assigned TMPDIR after the first setup error. Capacity was 22 GiB free disk and 41 GiB available memory.

| Evidence | Exit | Result |
|---|---:|---|
| `logs/01-f1-f2-reviewer.log` | 101 | Environmental setup failure only. The initial TMPDIR path was mistyped and absent; `proc-macro2` could not create a temporary directory. No lowerer behavior compiled or ran. |
| `logs/02-f1-f2-reviewer.log` | 0 | Corrected F1/F2 reviewer target: 2 passed. |
| `logs/03-runtime-literal.log` | 0 | Exact real-runtime literal preservation case: 1 passed, 13 filtered. |
| `logs/04-multi-creation-identity.log` | 101 | Dedicated counterexample compiled and ran; 1 behavioral assertion failed with the identity mismatch above. |

The correction author's retained Rust 1.85 all-target 19-case, strict, documentation, and scoped-format evidence informed the source examination but is not relabeled as execution by this reviewer. The review contract excluded a full workspace, browser, generated-service, provider, administration, or SDK build.

## Source and evidence integrity

- `candidate-source.sha256` is the complete 15-path correction manifest, unchanged at this candidate.
- `reviewer-tests.sha256` covers both reviewer test files.
- `inputs.sha256` covers the governing brief, original implementation contract/design, first review, literal disposition, correction report/manifest, and author coverage records.
- `command-results.tsv` records the exact exit classification for every command.
- `source-state.log` records `git diff --check`, the sole additive reviewer test, and exact candidate commit/tree.
- `logs.sha256` and `evidence.sha256` bind the raw logs and final review evidence.

No production code, author test, fixture, manifest, design, or planning artifact was modified during this review.

## Machine-readable findings block

```yaml
findings:
  - id: F3
    file: crates/generate/ess-entity-runtime/src/lib.rs
    line: 1297
    related_lines: [1335, 1779]
    category: identity-integrity
    severity: blocker
    verdict: NEEDS-CHANGE
    origin: introduced
    message: >-
      creation identity is anchored to the first accepting outcome's full
      branch coordinate, so a later accepting creation outcome observing the
      same event field receives an independent event slot and can emit an
      identity different from the created instance
```
