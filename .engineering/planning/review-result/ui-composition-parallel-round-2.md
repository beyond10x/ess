---
format: aep.planning-md/3
id: review-result:ui-composition-parallel-round-2
kind: review-result
status: active
title: 'Parallel-safety critic, round 2: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
approve

**What you read:** 2 artifacts, via `aep plan artifact show` on each, `aep plan artifact graph`, `aep plan artifact waves` and `git grep union_content_key`. I also read the guard story, which is outside the set, to answer the hand-off question. Surfaces: 2 cited (both bodies name paths and line ranges, though the store marks the `scope` field `inferred`), 0 inferred, 0 unplaced. Acceptance reads traced to a producer in the set: 1 (P2's "a variant field `as: list` with `fields:`" reads the key P1 introduces), and 1 of 1 is recorded by a `depends_on` edge. `waves` puts P1 in wave 3 and P2 in wave 15.

**What you could not establish:**
- P1 and P2 share eight files. Examples are `crates/ui/ess-ui/src/model.rs`, `schemas/ui/ess-ui.schema.yaml`, `crates/ui/ess-ui-react/src/emit.rs` and `crates/ui/ess-ui-tui/src/view.rs`. They do not collide in parallel, because P2's `depends_on` P1 edge orders them and P2's Decisions say P1 introduces the nested `Field` list. That ordering is not a finding.
- The hand-off question: the guard story is not a missing edge. Its `a_union_names_its_content_key` acceptance would change `union_content_key`. P2 reads only the existing function in `crates/generate/ess-gen/src/schema.rs:76`, so P2 reads no state the guard story creates today. P2's `## Decisions` also says the guard story is not a dependency and that the form follows a later key.
- `waves` lists collisions between each story and `story:a-client-paged-collection-offers-sizes-and-a-range`, and with other stories, on the same UI files, all `inferred`. That story is outside the set, so I did not judge it.
- Out of my lane: whether the read side belongs in P2 (scope or design), and whether the acceptance lines are checkable (acceptance).

```findings
[]
```
