---
format: aep.planning-md/3
id: review-result:ui-composition-scope-round-1
kind: review-result
status: active
title: 'Scope critic, round 1: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
needs-revision
story:a-form-field-over-a-union-draws-the-chosen-variants-fields — the story claims a read-side outcome (a column or record field shows a union's variant and its fields) that the request's P2 does not ask for, and the body never says it is added under the grid policy as story 1's does; either add that rationale to `## Decisions` or drop the acceptance bullet and the title clause — .engineering/planning/story/a-form-field-over-a-union-draws-the-chosen-variants-fields.md:119 (request P2 is write-side only: ~/.cache/b10x/conductor/beyond10x/scratch/ess/intake-composition/request.md:146 "a group of fields chosen by the value of a type field, written into one typed object")

**What I read:** 2 stories, the epic, the sibling content-key story, and the whole of `request.md` (245 lines) as parent. I ran `aep plan artifact show` on both stories, the epic and the content-key story, plus `aep plan artifact graph`.

I extracted 8 promises from the parent: P1 to P6, the NONE bucket as an explicit non-promise, and the P2 dependency note. I traced 7 to an item, counting those the drafter named as out of the set (P3 and P5 to the declined specification, P4 and P6 to their backlog siblings). The NONE bucket is not a promise.

- **Gaps:** none. P1 is claimed by story 1 and P2 by story 2.
- **Narrowing:** every narrowing is recorded in `## Decisions`.
  - P1's authored `min`/`max`, `reorder` and `add` become the model's `.count` invariant, no `reorder` key, and the field's `label`.
  - P2's `by:`, `shape:` and opaque-`Json` typing (EG-S3, 2 lines) are refused.
- **Overlap:** none.
- **Other reach (not a finding):** the terminal-renderer acceptances on both stories trace to the epic's outcome that "one document renders as a TUI and as generated React", so I did not call them reach.

**What you could not establish:**
- **Content-key edge:** I judge `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` not a missing edge.
  - The request never asks for a content key.
  - Story 2's fit review names it as conditional ("whether this adopter's wire needs it: I don't know").
  - The union already projects a fixed content key (`value`, or `content` when the tag is `value`), so story 2's acceptance submits `{<tag>: <variant>, <content>: {…}}` with no new key.
  - If the operator wants a non-default content key, that is a separate decision. It is not a dependency, and the edge stays off.
- **Out of my lane:** story 2's acceptance writes `<content>` without saying which key it is under today. That is an acceptance-checkability matter, not scope.
- **Out of my lane:** the P2 grid rationale spans form group and tab positions. Whether the set holds together is the design critic's call.

```findings
- file: .engineering/planning/story/a-form-field-over-a-union-draws-the-chosen-variants-fields.md
  line: 119
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the story claims a read-side outcome (a column or record field shows a union's variant and its fields) that the request's P2 does not ask for (request.md:146, 'written into one typed object'), and the body never says it is added under the grid policy as story 1 does; either add that rationale to Decisions or drop the acceptance bullet and the title clause"
```
