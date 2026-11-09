---
format: aep.planning-md/3
id: review-result:zero-issues-design-round-1
kind: review-result
status: active
title: Design critic, round 1, epic zero-open-issues-2026-11
relations:
- reviews: epic:zero-open-issues-2026-11
- reviews: story:stored-field-equals-returned-response-value
- reviews: story:when-subject-beside-a-row-set-validates-under-the-g2-format
- reviews: story:a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals
- reviews: story:a-value-expression-chooses-between-two-values-by-a-condition
- reviews: story:an-invariant-compares-a-union-field-with-its-variant-labels
- reviews: story:a-text-operator-reads-a-binder-or-a-field-as-its-operand
- reviews: story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body
- reviews: story:expect-error-checks-generated-field-presence
revision: 1
---
needs-revision
epic:zero-open-issues-2026-11 — no child owns the shared format bumps: 28 non-terminal children each open "the next source format" (11 name `ess/37`) and about 55 open a new conformance suite format, all on one branch with no release, so which number each acceptance reads depends on a wave order nothing records; add one task that assigns `ess/37`.. and the suite pairs to each design decision (G1, G2, G3, each single-request construct), with `depends_on` edges from the first story of each decision to it — .engineering/planning/epic/zero-open-issues-2026-11.md:25 (Scope assigns no formats), .engineering/planning/story/aggregate-group-keys-are-expressions.md:68, .engineering/planning/story/a-value-outside-its-declared-type-is-answered-malformed.md:66
story:stored-field-equals-returned-response-value — it claims `ess/37` "allocated to this decision when it lands" while story:aggregate-group-keys-are-expressions (G3, active) claims `ess/37` with no condition, so both acceptances cannot hold; its body must name the format it takes, or the `depends_on` edge that orders it after G3 — .engineering/planning/story/stored-field-equals-returned-response-value.md:66, .engineering/planning/story/aggregate-group-keys-are-expressions.md:68
story:when-subject-beside-a-row-set-validates-under-the-g2-format — its acceptance reads "G2's next source format", which story:navigation-reaches-any-depth-in-view-filters-and-guards opens, and unlike its four G2 siblings it has no `depends_on` edge to that story — .engineering/planning/story/when-subject-beside-a-row-set-validates-under-the-g2-format.md:125, .engineering/planning/story/navigation-reaches-any-depth-in-view-filters-and-guards.md:67
story:a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals — its acceptance needs `again` to be admitted, which story:when-subject-beside-a-row-set-validates-under-the-g2-format produces, and no `depends_on` edge runs to it — .engineering/planning/story/a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals.md:71
story:a-value-expression-chooses-between-two-values-by-a-condition — its acceptance says the mask "validates once the text functions land", which story:text-functions-compute-values produces, and no `depends_on` edge runs to it — .engineering/planning/story/a-value-expression-chooses-between-two-values-by-a-condition.md:57
story:an-invariant-compares-a-union-field-with-its-variant-labels — its acceptance reads the hole report that story:synthesis-offers-an-input-that-breaks-an-entity-invariant adds, and no `depends_on` edge runs to it — .engineering/planning/story/an-invariant-compares-a-union-field-with-its-variant-labels.md:59, .engineering/planning/story/synthesis-offers-an-input-that-breaks-an-entity-invariant.md:56
story:a-text-operator-reads-a-binder-or-a-field-as-its-operand — the `{fact: …}` operand on the ignore-case operators belongs to "whichever of the two stories lands second", so neither story owns it; a `depends_on` edge between it and story:text-shape-operators-ignore-case must fix which one does — .engineering/planning/story/a-text-operator-reads-a-binder-or-a-field-as-its-operand.md:64, .engineering/planning/story/text-shape-operators-ignore-case.md:38
story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body — `query` beside `multipart` belongs to whichever of it and story:a-command-route-takes-a-multipart-request-with-file-parts lands second, so neither owns it; a `depends_on` edge must fix which one does — .engineering/planning/story/a-command-route-reads-chosen-inputs-from-the-query-beside-its-body.md:61, .engineering/planning/story/a-command-route-takes-a-multipart-request-with-file-parts.md:72
story:expect-error-checks-generated-field-presence — it and story:view-and-error-assertions-carry-their-value-shape both give `ExpectError` a shape under a new suite major, and its own body says they "should land in one unit or one after the other", but neither a `depends_on` edge nor a merge records that — .engineering/planning/story/expect-error-checks-generated-field-presence.md:99, .engineering/planning/story/view-and-error-assertions-carry-their-value-shape.md:60

What I read: 283 children (273 non-terminal: 266 draft, 7 active). Commands: `aep plan artifact show` on the epic, `relations`, `list --format json`, `graph --help`, `validate`. Read the Acceptance sections of the stories that open a format or name a sibling, plus a grep of the store and `git show 786d569a9e`.
- Edges: walked all 330 `depends_on` edges in the store plus every `blocks` edge, outside the set as well. No cycle. 88 edges are inside the set and 12 point outside it.
- No serialising chain: depths are 202 at 1, 47 at 2, 18 at 3, 5 at 4, 1 at 5.
- Acceptance reads: 22 name a sibling story by id, and 12 of those are covered by an edge. Separately, 28 read a source format: the G3 group and four of the five G2 stories have edges; the four G1 stories and the single-request stories have none.

What I could not establish:
- I did not read all 273 bodies in full. I traced acceptance reads through sibling ids and format or suite wording, so a read that names no sibling and uses other words (like the one found at a-value-expression:57) could be missed.
- Out of my lane (parallel safety): the shared `FORMAT_HISTORY` in `crates/specify/ess-domain/src/system.rs` and the precedence-plan files that the G2 stories all touch.
- Out of my lane (scope/fit): policy 2 says a construct for one request gets no format of its own, yet about 12 single-request stories each take "the next open" format.
- `validate` prints "valid", plus about 40 warnings about review-results with no recorded outcome. Those are not my findings.

```findings
- file: .engineering/planning/epic/zero-open-issues-2026-11.md
  line: 25
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "no child owns the shared format bumps: 28 non-terminal children each open 'the next source format' (11 name `ess/37`) and about 55 open a new conformance suite format, all on one branch with no release, so which number each acceptance reads depends on a wave order nothing records; add one task that assigns `ess/37`.. and the suite pairs to each design decision (G1, G2, G3, each single-request construct), with `depends_on` edges from the first story of each decision to it"
- file: .engineering/planning/story/stored-field-equals-returned-response-value.md
  line: 66
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it claims `ess/37` 'allocated to this decision when it lands' while story:aggregate-group-keys-are-expressions (G3, active) claims `ess/37` with no condition, so both acceptances cannot hold; its body must name the format it takes, or the `depends_on` edge that orders it after G3"
- file: .engineering/planning/story/when-subject-beside-a-row-set-validates-under-the-g2-format.md
  line: 125
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads 'G2's next source format', which story:navigation-reaches-any-depth-in-view-filters-and-guards opens, and unlike its four G2 siblings it has no `depends_on` edge to that story"
- file: .engineering/planning/story/a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals.md
  line: 71
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance needs `again` to be admitted, which story:when-subject-beside-a-row-set-validates-under-the-g2-format produces, and no `depends_on` edge runs to it"
- file: .engineering/planning/story/a-value-expression-chooses-between-two-values-by-a-condition.md
  line: 57
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance says the mask 'validates once the text functions land', which story:text-functions-compute-values produces, and no `depends_on` edge runs to it"
- file: .engineering/planning/story/an-invariant-compares-a-union-field-with-its-variant-labels.md
  line: 59
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads the hole report that story:synthesis-offers-an-input-that-breaks-an-entity-invariant adds, and no `depends_on` edge runs to it"
- file: .engineering/planning/story/a-text-operator-reads-a-binder-or-a-field-as-its-operand.md
  line: 64
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the `{fact: …}` operand on the ignore-case operators belongs to 'whichever of the two stories lands second', so neither story owns it; a `depends_on` edge between it and story:text-shape-operators-ignore-case must fix which one does"
- file: .engineering/planning/story/a-command-route-reads-chosen-inputs-from-the-query-beside-its-body.md
  line: 61
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "`query` beside `multipart` belongs to whichever of it and story:a-command-route-takes-a-multipart-request-with-file-parts lands second, so neither owns it; a `depends_on` edge must fix which one does"
- file: .engineering/planning/story/expect-error-checks-generated-field-presence.md
  line: 99
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it and story:view-and-error-assertions-carry-their-value-shape both give `ExpectError` a shape under a new suite major, and its own body says they 'should land in one unit or one after the other', but neither a `depends_on` edge nor a merge records that"
```
