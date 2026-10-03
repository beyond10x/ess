---
format: aep.planning-md/3
id: component-design:conditional-aggregate-measures-source22
kind: component-design
status: in_review
title: Typed per-measure selection and aggregate observation contract
relations:
- designs: story:feature-request-363
revision: 3
transitions:
- {from: "draft", to: "in_review", at: "2026-10-03T20:41:01Z", actor: "human:timo", revision: 2}
---
## Proposal

The complete initial design is docs/design/conditional-aggregate-measures.md, inspected against74a67d7cd. It binds source22 sibling where, the common view-row/param predicate environment, independent measure membership on unchanged groups, empty/Optional/Unknown behavior, aggregate38/39 and expression40/41 reader compatibility, real target and causal-cut authority, and named decisive faults. This records a proposal for the existing pinned363 obligation, not a new task outside the50 ledger.

## Review and completion

Round1 is preserved in review-result:conditional-aggregate-measures-363-20261003-r1 (needs-revision, six blockers and one warning). The design now binds exact lexical/domain/IR representations and canonical output, preserves Aggregate traits through a structural resolved key, preserves paging conflicts, requires typed semantic diff/rendering, expands execution to all six functions and list/composite predicates across every required lane, charges shared byte/work budgets, and records exact governed implementation dependencies and scope. Each finding has a fixed design outcome, subject to final independent round2; no production or execution approval is implied. Maximum two whole-design rounds remains unchanged. Implementation must independently prove the complete contract and compatibility seams.
