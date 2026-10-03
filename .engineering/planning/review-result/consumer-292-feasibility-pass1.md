---
format: aep.planning-md/3
id: review-result:consumer-292-feasibility-pass1
kind: review-result
status: active
title: 'History feasibility review: unchecked suffix at exact work exhaustion'
relations:
- reviews: story:feature-request-292
revision: 1
---
needs-revision

# Independent review of the #292 feasibility checkpoint

Reviewer execution count: 0. Source and immutable-artifact inspection only; no compiler, test, target, cache use, source edit, AEP mutation or publication. This report is private review prose. The owner's reported 52 integration tests and 4 private proof tests were read as owner evidence, not rerun or independently measured. Full native neighbors remain outstanding.

## Exact reviewed checkpoint

- Report SHA256: `7ca2b64b62594ea5d03465f21b34fe727b09350ab153111eaf097f3fbfe78cc4`.
- Source manifest SHA256: `ecb0f1bd5211a9c21c928aab2b0a6569024e5aca63e4d499ea29da5de3a5c745`.
- Source archive SHA256: `5368f68bb3e06930e6f83885e313bcc3546c40116260b7939c9ebcb2bdcfd9c6`.
- Tracked patch SHA256: `2ef073590506bf48ca6fc3f81796efe4290ab652c5eb7e1c0c49ba368d057d4a`.

All four artifact hashes were verified. `sha256sum -c` verified all 18 manifest entries against the live frozen tree, including previously untracked private history modules. Review used that exact source over base `86b4a994481f4391f7f38cc2677cdc493ca20404` and the retained feasibility traversal policy. Review scope was feasibility/proof validation and bounded witness construction, not approval of the complete outstanding #292 implementation.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/input.rs",
    "line": 824,
    "category": "boundary",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The List and Map validation loops break whenever the proof budget reaches exactly zero, including after a successful child validation, then return Ok when no earlier child was unresolved. Later children can therefore remain unchecked while the validator reports Valid. For List<Boolean> containing [true, an empty text node], a fresh work budget of 6 consumes 1 for setup entry, 3 for the whole-value walk and 2 validating the first Boolean; the zero-budget break skips the invalid second element and returns Ok. The equivalent Map loop at lines 842–846 has the same defect. Exhaustion before all children are checked must produce Unresolved, or traversal must let the next child fail its first charge. Add exact-boundary tests for both containers, including a later semantic-invalid child and an all-valid container. This is independent of the already tracked abstract early-Unknown precision gap. The trace is source-derived and was not executed by this reviewer."
  }
]
```

## Finding detail and minimum repair boundary

`ProofBudget::charge` accepts an exact debit that leaves zero; `exhausted()` then returns true. In `setup_value`, List lines 819–828 and Map lines 836–846 break on that condition after retaining the child result. `retain_validation` deliberately leaves `unresolved` empty for a valid child. Consequently the final `unresolved.map_or(Ok(()), Err)` can validate an unvisited suffix.

A concrete private-validator probe needs no generated witness or fabricated history authority: use a compiler-admitted List<Boolean> field type and call `validate_typed_value_bounded` with `Node::Seq([Node::Bool(true), Node::Text(String::new())])` and a six-unit ProofBudget. At sufficient budget the same value must be Invalid; at the exact partial budget it must be Unresolved, never Valid. A Map<String,Boolean> with keys `a`,`b`, first true and second empty text gives the analogous boundary with budget 8 (entry 1, map/value/key prewalk 5, first child 2). These numerical traces follow the frozen implementation; no test was run here.

The finding is in the proof-validation contract itself. I have not claimed an executed end-to-end false Linearizable verdict, nor established that the current structural witness generator emits this malformed nonempty list. A proof helper reporting Valid for an incomplete validation is nevertheless unsound and unsafe to use for finite/derived-domain guarantees. Existing planned arithmetic proof work would rely on that distinction.

Removing the premature break permits the next child's initial charge to return Unresolved and then stop; alternatively record an explicit Unresolved whenever an unvisited suffix remains. Completing the final child exactly at the budget boundary may still be Valid. Preserve immediate proved Invalid and the existing False-dominant conjunction behavior. No broader semantic change is necessary.

## Assessed soundness outside the finding

The feasibility tri-state and completed-only cache follow the retained policy. `history/values.rs::ProofContext::prove` charges entry/key work, reuses completed proofs, separates active dependencies, removes active state on return, and stores only Inhabited/Empty. Depth/work/active uncertainty is not inserted as a permanent Empty proof. Cache contents hold no witness values or operation-specific observations.

`prove_domain` accepts direct Optional inhabitation, validates structural witness candidates, propagates an Empty required child only when omission is conclusively disallowed, and otherwise uses finite enumeration. Newtype wrapper constraints are not bypassed merely because an inner Optional/List/Map has an empty base. Contradictory Integer bounds are extracted only as necessary constraints. Union witness construction tries alternatives and handles active recursion without treating it as semantic emptiness.

`Enumeration { values, complete }` correctly distinguishes existential usefulness from universal completeness: an actually valid candidate can prove Inhabited despite unresolved candidates, while empty values prove Empty only when complete. The public finite-domain helper exposes values only when complete. Finite candidate validation discards Invalid, records Unresolved as incompleteness, and does not treat the depth-exhaustion errors from the original 17-Struct red as false values. This assessment is conditional on correcting the incomplete-validation finding above.

Resource controls are shared through the proof context rather than reset for nested witness/finite/absence/bounds operations. The structural witness entry is separate from ordinary synthesis, uses depth/active guards, caps witness attempts, and does not call the unbounded synthesis repair path. Finite Cartesian products are capped before construction and object/value copies are charged. Concrete projection is preceded by a budgeted walk matching the existing projector's relevant shape traversal; its Union arm is intentionally opaque in both projector and preflight, with complete Union validation performed separately by setup_body. These are source observations, not a benchmark or proof of an exact wall-clock resource bound.

The concrete typed validator preserves semantic False versus Unknown and scans onward after Unknown within resources to find a definite invalid sibling/invariant. The tested owner matrix covers the intended order reversal. The separately acknowledged abstract `validate_at`/`validate_invariants` early-Unknown issue remains outstanding and is not presented here as a newly introduced finding.

## Native compatibility and limitations

The public `validate_typed_value` String-result wrapper and `validate_entity_setup` remain available. Native mode has no new work allowance, so the exact-zero proof-only defect does not apply to its unlimited budget. Ordinary error detail and newtype error context remain present. The newtype fact-projection failure now formats ShapeErrors through Display rather than the former Debug vector; byte-identical diagnostics on that uncommon failure path are therefore not established by source review. I found no additional demonstrated native acceptance regression in this pass. Run the promised native neighbors before claiming native compatibility; the checkpoint report appropriately does not claim they passed.

The mechanical-crossing/derived Increment work, abstract conjunction precision repair and independently scoped nested Increment target-location defect are explicitly outside this checkpoint's completed behavior. This review does not waive them and does not authorize implementation expansion or publication.
