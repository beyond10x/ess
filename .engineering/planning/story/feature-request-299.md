---
format: aep.planning-md/3
id: story:feature-request-299
kind: story
status: draft
title: A row selected by a filter can be read in a guard and in sets
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#299
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-285
- depends_on: story:feature-request-228
- depends_on: story:feature-request-237
revision: 5
---
## Outcome

A guard and a `sets:` value read a field of the row selected by a filter over an input.

## Origin

beyond10x/ess#299, from a downstream hardening run (triage item 8c).

## Fit review

1. Need: a newly created attempt copies a delay from an earlier row sharing worker and batch, without requiring the caller to know that row's identity. The request asks for guard and value access by filter (beyond10x/ess#299). The neutral probe uses demo.jobs.Attempt, no adopter source. Requester suggestion: read a row by field equality or supersession; it does not specify an ordering or uniqueness policy.
2. Class: gap. website/docs/reference/predicates.md:27 explicitly restricts when_related to identity lookup; docs/design/value-expressions.md E8 describes exactly one via reference. No documented filter source is being repaired.
3. Existing language: instances/affects reuse a typed where filter but write rows; they cannot copy a scalar into a new row (docs/design/set-effects-over-filtered-instances.md, The constructs). The pre-history combined CLI SHA256 facf3de7d49ceade8c7eaf4a4c6c94ab9b404449e554a33c380ecd8e19471586 refused the neutral filtered-source attempt (exit1), while replacing only that source with literal0 validates (exit0). Final probe SHA256 d60bed4fd1e346312c9e17c4a061abb43ca45773a07ea24de4c415e8995d10ef; literal control26e028ae1a457c4b2331bf4a35245cf99c5cf63f170c9f90efc1dd9d41a632e4. The first literal control lacked instance and was corrected before this comparison; its diagnostic is retained and is not feature evidence.
4. Fit: docs/design/filtered-related-reads.md binds one entity/where selector to the family's when_related row-set guard and a related value source. It reuses existing field/input/subject vocabulary and exact pre-outcome snapshot authority. Exactly one match is required for a value; zero and multiple matches never choose an arbitrary row. Optional values preserve the selected field's Optional type, separately from Optional/chained addresses in #285. The design states dispositions for native/history, Rust, Go, web, TypeScript/Go suite runtimes, diff, Entity Runtime, docs, synthesis, authoring and explorers. Named obligations remain coverage limitations; they do not satisfy required executable acceptance.
5. Second adopter: a replacement shipment copies its insured amount from the one open shipment for the same order and destination. A zero-match branch supplies the initial amount; multiple matches require the authored ambiguity refusal. This is a domain selection fact, independent of the attempt example's delay policy (design Cardinality is explicit).
6. Cost: coordinated source22, one new typed related_selection IR variant with entity/typed predicate/field/result type; existing related_field and unaffected IR bytes unchanged. EssIr already carries source format and is serialized rather than read back (ir.rs:828-832,2365-2368). Keep existing suite steps where adequate; any new step needs explicit envelope version and old-reader refusal. New selector/source changes receive explicit behavior-diff classification. No view join, query language, coalesce source, or generated wire API is introduced. Old-format struct-shaped related fields retain their interpretation.
7. Alternatives: change nothing leaves policy outside the executable specification; require a via identity makes a caller invent knowledge the command currently owns; add a view join expands every query for a command-local read; choose first/latest silently invents order and ambiguity policy. Accept the need redesigned as the existing related wrapper with exactly one filtered selection, sharing the family row-set selector. Two consumers reuse one semantic operation instead of separate guard/value query grammars.

## Decisions

Accept, redesigned, within the operator-authorized remaining bundle. Binding design: docs/design/filtered-related-reads.md. Add `{related: {entity, where, field}}` beside #285's via alternative and compose it with family F `when_related: {entity, where, exists|count|forall}`. Value selection requires exactly one match; zero/multiple rows provide no value and no transition, while application outcomes come from explicit guards. All reads use the immutable pre-outcome store. Source allocation22 supersedes historical family21 prose; one-time responses retain21.

This story remains draft pending the shared family-design review, typed source scope and predecessor implementation. It must not be dispatched from the decision paragraph alone. Implement after #285 and together with family row sets; preserve one branch/PR. The current probe establishes missing admission, not successful implementation. Entity Runtime's externally blocked authority remains a visible target limitation.

## Shared contract revision 2

The coordinator addressed the seven findings in review-result:filtered-related-read-design-20261003-r1 in docs/design/filtered-related-reads.md: (1) closed exists/count/forall grammar, empty-set and Unknown truth tables, complete-domain authority and scope/cap rules; (2) validated common-subject borrowing and #282/#304 precedence, with row-set emptiness distinct from identity absence; (3) typed RelatedSet condition and shared selector with byte preservation; (4) format-neutral raw capture and pre22 nested-source back-conversion including predicate sequences; (5) an explicit executable target/profile matrix; (6) named healthy and independently faulty controls across those lanes; and (7) governed predecessor edges to #285/#228/#237. All seven outcomes are fixed by this revision; approval remains pending a second independent pass. No runtime completion is claimed.

Source22 is coordinated with the bundle. This story is not dispatchable until its scoped implementation predecessor work is ready; a named unsupported result never closes a required executable target. The full shared guard contract is owned in docs/design/filtered-related-reads.md rather than inferred from older family-F summary paragraphs.

## Current design disposition

Second independent review approved the complete shared row-set/filtered-value contract at 8606b103c (review-result:filtered-related-read-design-20261003-r2). All seven first-round findings were fixed. Design review is complete; implementation and target acceptance remain pending. This supersedes the earlier pending-design-review wording, not the predecessor gates or draft implementation status. The shared normative page is docs/design/filtered-related-reads.md and the syntax allocation is source22. No source21 family allocation remains current. #299 depends on #285 and #228/#237; the shared page also binds the latter stories' row-set implementation.
