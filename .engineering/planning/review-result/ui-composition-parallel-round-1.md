---
format: aep.planning-md/3
id: review-result:ui-composition-parallel-round-1
kind: review-result
status: active
title: 'Parallel-safety critic, round 1: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
approve

**What you read:** 3 artifacts (the two stories and the guard sibling), with `aep plan artifact show` on each, `graph`, `waves` and `relations`. I also ran `git grep` on `crates/ui/ess-ui-check/src/model.rs` and `crates/generate/ess-gen/src/types.rs`, and read `request.md`. I did not read `classification.tsv`.

**What you could not establish:**
- Surfaces: 2 items placed cited, 0 inferred, 0 unplaced. Both bodies name their paths and symbols in Need and Fit review, and I confirmed `ResolvedBody::Union => None` at `ess-ui-check/src/model.rs:2049` and `:2065`. Each Scope section tags its paths `(inferred)` and the store's `scope` field does the same, so the Need and Fit paths are the cited evidence.
- Pair collision: the two stories share `ess-ui/src/model.rs`, `ess-ui-check/src/model.rs`, `rules.rs`, `emit.rs`, `fields.tsx.tmpl`, `ess-ui-tui/src/view.rs`, the schema, `docs/design` and `website/docs/reference/ess-ui.md`. This is a legitimate shared surface, not a finding. P2 has a `depends_on` edge to P1, and its body says it builds on P1's nested `Field` list. Its Decisions section gives the edge a reason but does not name the shared files. The other half of the pair, the P1 body, does not mention P2, but the edge already serialises them.
- Reader beside producer: 1 acceptance read traced to a producer in the set, and 1 of 1 is recorded by `depends_on`. The line is P2's "A variant field `as: list` with `fields:` edits inside the chosen variant", which reads the `fields:` key P1 creates.
- The open question (guard story as a missing P2 edge): I did not find a parallel-safety edge. `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` is tagged `ess-0.71.0` and cites `crates/generate/ess-gen/src/types.rs:195`, where the content key is fixed as `value`. It has no `ess-ui` surface, so there is no file collision. P2's acceptance writes `{<tag>: <variant>, <content>: {…}}` from whatever key the model projects today and never reads a configurable content key. Whether P2 needs that story is a scope or design call, so it is out of my lane. If a person decides it does, P2 would then read state the guard story creates and would need the edge.
- Collisions with other stories outside the set: `aep plan artifact waves` lists many, for example `a-toggle-shows-the-row-state-it-flips`, `a-value-renders-in-the-form-its-field-declares`, `a-refused-node-is-reported-at-its-own-path` and `a-used-by-list-reads-several-views`. All are on the same `ess-ui` files. I did not assess them because they are outside the set.
- The `waves` output puts P1 in wave 3 and P2 in wave 15, and the 0.93.0 activation may reorder that. This is outside my lane.

```findings
[]
```
