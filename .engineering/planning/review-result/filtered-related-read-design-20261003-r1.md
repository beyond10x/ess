---
format: aep.planning-md/3
id: review-result:filtered-related-read-design-20261003-r1
kind: review-result
status: active
title: Filtered related read design independent review round 1
relations:
- reviews: story:feature-request-299
revision: 1
---
needs-revision

story:feature-request-299 — the design imports family F's overlap, exhaustiveness, typing and precedence without a binding family-F contract, leaving the `exists`/`count`/`forall` truth tables for empty, multiple and Unknown-membership sets undefined — docs/design/filtered-related-reads.md:30

story:feature-request-299 — `subject.<path>` is defined only as the existing addressed subject, so refusal branches that borrow a sibling subject, commands with multiple candidate subjects, and their unknown-instance, wrong-state and identity-related precedence have no rule — docs/design/filtered-related-reads.md:9

story:feature-request-299 — the design specifies a `related_selection` payload-value IR but no typed `ResolvedCondition` representation or byte-preservation rule for the new row-set guard, leaving guard consumers to invent its serialized shape — docs/design/filtered-related-reads.md:75

story:feature-request-299 — the compatibility section requires old struct meanings but does not bind a format-neutral raw parse and back-conversion for `{related: {entity, where, field}}`, although current parsing recognizes related shapes before the format is known and cannot carry predicate sequences as nested payload sources — docs/design/filtered-related-reads.md:70

story:feature-request-299 — generated-target closure is conditional on an undefined support advertisement and never names which Rust, Go, Web, Go-suite and TypeScript-suite lanes must execute versus may return an obligation, so an all-obligation implementation could satisfy the prose — docs/design/filtered-related-reads.md:86

story:feature-request-299 — the acceptance names fault classes but no named controls that run correct and faulty implementations in every required lane, so a self-consistent selector and suite can pass without detecting a dropped conjunct, a post-outcome read or a wrong-row copy — docs/design/filtered-related-reads.md:99

story:feature-request-299 — the body requires #285 and the family row-set work first but the governed graph records no `depends_on` edge to `story:feature-request-285`, `story:feature-request-228` or `story:feature-request-237` — .engineering/planning/story/feature-request-299.md:39

What I read: the #299 and #285 stories, both #304 slices, the #228/#237 family-F decisions, the filtered-read design, and the current typed parser/IR/interpreter/generator/lowering surfaces at `168521d87`; the unchanged later tree confirms the same bytes. The governed store validates, so validator output is not restated as a finding.

What I could not establish: no separate binding family-F design or generated-target support-advertisement contract exists in the reviewed tree. This was a read-only design review; no execution evidence is claimed.

```findings
[
  {
    "file": "docs/design/filtered-related-reads.md",
    "line": 30,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the design imports family F's overlap, exhaustiveness, typing and precedence without a binding family-F contract, leaving the `exists`/`count`/`forall` truth tables for empty, multiple and Unknown-membership sets undefined"
  },
  {
    "file": "docs/design/filtered-related-reads.md",
    "line": 9,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "`subject.<path>` is defined only as the existing addressed subject, so refusal branches that borrow a sibling subject, commands with multiple candidate subjects, and their unknown-instance, wrong-state and identity-related precedence have no rule"
  },
  {
    "file": "docs/design/filtered-related-reads.md",
    "line": 75,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the design specifies a `related_selection` payload-value IR but no typed `ResolvedCondition` representation or byte-preservation rule for the new row-set guard, leaving guard consumers to invent its serialized shape"
  },
  {
    "file": "docs/design/filtered-related-reads.md",
    "line": 70,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the compatibility section requires old struct meanings but does not bind a format-neutral raw parse and back-conversion for `{related: {entity, where, field}}`, although current parsing recognizes related shapes before the format is known and cannot carry predicate sequences as nested payload sources"
  },
  {
    "file": "docs/design/filtered-related-reads.md",
    "line": 86,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "generated-target closure is conditional on an undefined support advertisement and never names which Rust, Go, Web, Go-suite and TypeScript-suite lanes must execute versus may return an obligation, so an all-obligation implementation could satisfy the prose"
  },
  {
    "file": "docs/design/filtered-related-reads.md",
    "line": 99,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance names fault classes but no named controls that run correct and faulty implementations in every required lane, so a self-consistent selector and suite can pass without detecting a dropped conjunct, a post-outcome read or a wrong-row copy"
  },
  {
    "file": ".engineering/planning/story/feature-request-299.md",
    "line": 39,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the body requires #285 and the family row-set work first but the governed graph records no `depends_on` edge to `story:feature-request-285`, `story:feature-request-228` or `story:feature-request-237`"
  }
]
```
