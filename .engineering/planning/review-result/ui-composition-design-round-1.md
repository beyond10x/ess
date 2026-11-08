---
format: aep.planning-md/3
id: review-result:ui-composition-design-round-1
kind: review-result
status: active
title: 'Design critic, round 1: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
approve

**What you read:** 3 artifacts (the 2 in the set, plus the guard story to judge the open question), using `aep plan artifact show` on each, `relations`, `graph` and `validate` (run once). `validate` printed only store-wide items, none naming these stories, so I relay none.

**What you could not establish:**
- Walked 5 declared edges from the two ids: `serves` and `informed_by` each from both, plus one `depends_on` from the union story to the list story. The list story has no outgoing `depends_on`, so there is no cycle. The `serves` and `informed_by` edges end at `vision:O2` and `epic:ess-ui-renderer-neutral-ui`, outside the set, and neither returns to a set item. I read the graph for edges on these ids only. I did not run a cycle search over all 1996 artifacts.
- I traced 1 acceptance read to a producer in the set, and 1 edge records it. The union story's acceptance "A variant field `as: list` with `fields:` edits inside the chosen variant" reads the `Field.fields` key that the list story creates. The `depends_on` edge covers it.
- The list story's acceptances read nothing the union story produces.
- On the open question, the guard story is not a missing edge. The union story's acceptance submits `{<tag>: <variant>, <content>: {…}}` under the union's existing content key. The guard story (`.engineering/planning/story/a-guard-reads-a-union-tag-and-a-union-names-its-content-key.md`) adds an authored `content:` key and guard selection by variant, and no union acceptance reads either. The union story's own Fit review item 4 says it does not know whether this adopter's wire needs it.
- The two stories share `Field` in `crates/ui/ess-ui/src/model.rs`, `fields.tsx.tmpl` and `view.rs`. The `depends_on` edge already orders them, and it also reflects a real design dependency, since the union story reuses the list story's nested `Field` checks. Both are cut vertically through check, React, terminal and docs, so this is not a chain or a split abstraction.
- Out of my lane, and not setting the verdict: whether the union's variants have display names in the model today (the union acceptance says the choice shows "display names"). I did not find where the model defines them. That belongs to the acceptance and scope critics.

```findings
[]
```
