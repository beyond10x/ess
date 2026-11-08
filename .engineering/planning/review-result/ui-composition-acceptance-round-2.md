---
format: aep.planning-md/3
id: review-result:ui-composition-acceptance-round-2
kind: review-result
status: active
title: 'Acceptance critic, round 2: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
approve

```findings
[]
```

I read 2 of the 2 ids given, in full: `story:a-form-field-edits-a-list-of-records` (revision 2) and `story:a-form-field-over-a-union-draws-the-chosen-variants-fields` (revision 4). I also read `request.md` and `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`. In the tree I confirmed that `ess_gen::schema::union_content_key` exists (`crates/generate/ess-gen/src/schema.rs:76`) and that `docs/design/union-unit-variants.md` § Wire form describes the same key. I also confirmed that `enum_variants` in `crates/ui/ess-ui-check/src/model.rs` returns `None` for a union.

- **P2 content key (round 1 finding):** P2's acceptance now names the key. The submitted value is `{<tag>: <variant>, value: {…}}`, with `content` where the tag is itself `value`, and a unit variant sends the tag only. That is checkable against an existing function, so P2 does not need the guard story as a dependency.
- **Missing edge:** I judge the guard story not to be a missing edge. It would only let a union name its own content key. P2's acceptance holds on today's fixed key, so P2 and the guard story can land in either order.
- **Acceptance format:** Both acceptances are bullet checklists, not one sentence. The store does the same elsewhere, for example `story/a-view-may-scope-rows-to-the-reading-actor.md`. Each bullet is observable (a React test, an `ess ui check` refusal by node path, or an `ess ui test` case). The byte-identical bullet covers the before/after transition. I treated the format as house convention, not a finding.

**Out of my lane, not counted toward the verdict:**
- **P2's read side:** this is a scope question, and round 1 already raised it.
- **P1/P2 file overlap:** the two stories share the same scope files, which is a parallel-safety question. The `depends_on` edge serialises them.
