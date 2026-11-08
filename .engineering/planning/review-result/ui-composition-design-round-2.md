---
format: aep.planning-md/3
id: review-result:ui-composition-design-round-2
kind: review-result
status: active
title: 'Design critic, round 2: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
approve

**What you read**
- 2 artifacts, shown whole with `aep plan artifact show`.
- Also read: `relations`, `graph` (filtered to the two ids, the guard story, P4 and P6, plus the edge lines), `validate` (reports `valid`; its only notes are old unrelated `review-result` follow-ups), the guard story's body, and `docs/design/union-unit-variants.md` § Wire form.
- Confirmed `ess_gen::schema::union_content_key` exists at `crates/generate/ess-gen/src/schema.rs:76`.

**What you could not establish**
- Edges walked: 5 declared edges on the two ids (2 `serves`, 2 `informed_by`, 1 `depends_on`). I also walked outside the set, to `vision:O2`, the epic and the guard story. No cycle, and no chain beyond one P2→P1 edge that has a stated reason.
- Acceptance reads traced to a producer in the set: 1 (P2's "variant field `as: list` with `fields:` edits inside the chosen variant" reads the `Field.fields` that P1 introduces). It is recorded by `depends_on story:a-form-field-edits-a-list-of-records`.
- The guard story is not a missing edge. P2 submits through the existing `union_content_key` helper and adds no authored content key. Its body says it would follow the guard story's key if that story later changes it, so P2 does not read anything the guard story produces.
- Unease, not a finding: P2's acceptance hardcodes `value` as the content key. If the guard story's `content:` lands before P2, the acceptance line would need rewording.
- Unease, not a finding: the guard story is tagged `ess-0.71.0` while the adopter ran ess 0.90.0, so its queue tag looks stale. Both are outside the shape of this set.
- Outside my lane: both stories edit `ess-ui/src/model.rs`, `ess-ui-check/src/model.rs` and `rules.rs`, the React `fields.tsx.tmpl` and `emit.rs`, `ess-ui-tui/src/view.rs`, and the schema and reference doc. The `depends_on` edge already orders them. Whether that is enough is for parallel safety.

```findings
[]
```
