---
format: aep.planning-md/3
id: review-result:ui-composition-scope-round-2
kind: review-result
status: active
title: 'Scope critic, round 2: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
approve

**What you read.** 2 stories (P1 and P2), the parent request at `request.md` lines 1-245 (promises at 83-164), and the P6, P4, P3 and P5 sections. I ran `aep plan artifact show` on both stories, `aep plan artifact graph`, `aep plan artifact list`, `aep plan artifact show story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`, and `git grep` on `docs/design/generic-language-features.md` and `docs/design/language-grids.md`.

I extracted 10 promises from the parent and traced all 10 to an item.
- **P1, 7 promises:** a list of records with item fields, add, remove, reorder, min/max, nested lists, and a read-only nested draw in a cell or detail. P1 claims all seven. Its Decisions record the three redesigns: reorder becomes move, min/max come from the model's `.count` invariant, and the add label is the field's `label`.
- **P2, 3 promises:** a group of fields chosen by a type field and written into one typed object, over a typed shape for opaque JSON (EG-S3), giving one form per module type. P2 claims all three. Its Decisions record the refusal of `by:` and `shape:`, and the choice of a model `union` as the typed shape.

Nothing else in the store already claims either outcome. The only title match is the union story, and none duplicates P1 or P2.

**On the open question.** The guard story is not a missing edge. P2's Decisions state it is not a dependency, and the form submits under `value` or `content` (`ess_gen::schema::union_content_key`, `crates/generate/ess-gen/src/schema.rs:76`). The promise it might touch is that the request's `by: module` is a sibling field of `settings`. The union story narrows that, and its `## Decisions` records the narrowing.

**What I could not establish**
- The P2 read side (a union in a column or record field) is not in the request. The request's P2 asks for writes only, and the story says so itself. The Decisions now name it and give a reason, so I did not count it as reach beyond the parent.
- That reason cites "the grid policy". In the tree, that policy (`docs/design/generic-language-features.md:60-61, 213-217`) governs the expression grids R and O, and `language-grids.md` has no UI-field row. The P2 argument therefore applies the policy by analogy. That is a design question for `plan-critic-design`, not a scope finding.
- P2's Fit review item 4 says "whether this adopter's wire needs [the content key]: I don't know". This is an unresolved fact about the adopter, not a coverage defect.

```findings
[]
```
