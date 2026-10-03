---
format: aep.planning-md/3
id: review-result:filtered-related-read-design-20261003-r2
kind: review-result
status: active
title: Filtered related read design second independent review
relations:
- reviews: story:feature-request-299
revision: 1
---
approve

# Filtered related reads design review — round 2

Publication-safe normalization of the independent design review. The private report SHA256 is
`ceab71938881ec353cb689f2cd0635f64ec554ed064c70dbe8a5aa054a77b888`.

Review type: final design and source-contract inspection. The review began at integration HEAD
`5ae1226fd41f42664c7c932dcb58b910b17be7ac`; the reviewed files were committed concurrently, and
the final bytes were reconciled unchanged at HEAD `8606b103c5ca5ac1809eafecea6c9b65ac909d9f`.
No compiler, test, formatter or AEP mutation was run, and no repository file was changed by the
reviewer.

The reviewed design and #299 story hashes are
`a858956a4921d3e157384767c32cc1a11ad7d441c2e12e212e95ca415773acc4` and
`cb9b972bef2bb5aa603a3a27745b39fc8fb4ea08b5df296e93adb42f95ede08a`, respectively.

All seven round-1 findings are closed at design level:

1. Revision 2 defines the shared #228/#237 row-set grammar and explicit empty, known and
   Unknown-membership truth tables for `exists`, `count` and `forall`, including exact counts,
   possible-member retention, proof correlations, complete-domain authority and bounded synthesis.
2. `subject.<path>` borrows one validated existing common subject and refuses different,
   creation-only and set subjects. Missing addressed identity remains distinct from an empty row
   set.
3. The IR contract adds a typed `RelatedSet` condition and a shared typed selector for
   `related_selection`, while preserving existing `Related`, `related_field` and unaffected bytes.
4. A closed format-neutral raw alternative retains predicate mappings and sequences until source
   version is known. Source22 recognizes the new form; older sources recover their original nested
   payload meaning or original refusal, with locations and duplicate/extra-key behavior preserved.
5. The required execution profile and every target disposition are explicit. Required native,
   history, Rust, Go, Web/WASM, suite-runtime, synthesis and accepted explorer/authoring lanes cannot
   close through an obligation or skip. Entity Runtime's external limitation remains named and does
   not count as conformance.
6. Named healthy scenarios and independent faulty controls cover each executable lane. Dropped
   conjuncts, wrong-row copying and post-effect reads must fail through real execution seams;
   reversed order and decoy-only changes must pass.
7. #299 now records predecessor edges to #285, #228 and #237 and expressly assigns the shared
   family contract to the revised design rather than the older family summary paragraphs.

The #282/#304 precedence is coherent. Required or present identity references keep early missing-row
handling. Present related predicates run after addressed-row existence and held-state checks.
Optional identity absence performs no lookup and selects no related branch. Row-set tests run in the
later stored-read phase, and `exists: false` there means an empty selected set rather than identity
absence. Nonmoving outcomes remain independent of held state. Value reads occur after branch
selection against the same immutable pre-outcome snapshot.

Current source inspection confirms that this remains a prospective design: the repository still has
only identity-based related condition/value IR, exact-shape `{related: {via, field}}` raw recognition
and the existing shared-history partition seam. Revision 2 specifies their extension without
claiming implementation.

No blocking contradiction or missing contract remains. Approval is limited to the design and story
revision. Implementation, old-reader vectors, canonical-byte controls and required target/fault
executions remain future evidence obligations.

```findings
[]
```
