---
format: aep.planning-md/3
id: story:binary64-reachability-follows-references
kind: story
status: archived
title: Binary64 reachability may miss a value under a multi-type term (suspected)
tags:
- not-a-defect
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:52Z", actor: "human:timo", revision: 4}
---
## Suspected finding (not measured)

Read in code by the U2 implementor of wave 2026-10-07b, not run: `reaches_binary64`
(`crates/generate/schema-contract/src/realize.rs:535`) skips nodes it has seen by pointer, and a
multi-type term shares its parent's pointer, so a binary64 value under such a term might not be
found. The U2 fix (`55a03d553e`) uses a reachability walk that tracks references instead.

## Acceptance

A test with a binary64 value under a multi-type term either shows the walk finds it (and this
story is archived as not reproduced) or fails, and the walk is changed to track references.


## Verification on 0.92.0 (2026-11-09)

Result: **not-a-defect**. Checked against main at 4ab2ea1449 and the 0.92.0 release binary by the
zero-open-issues run (epic `epic:zero-open-issues-2026-11`); this draft leaves the plan as archived.

Proof: crates/generate/schema-contract/src/realize.rs:1212-1228; realize.rs:1739-1765; crates/generate/ess-gen/src/types.rs:712-730,985-1031

The walk is now `reaches_marked` (realize.rs:1212-1228): it tests `marked.contains(pointer)` before
the visited check, so a term sharing its parent's pointer is skipped only for *expansion*. A node
shares its parent's pointer in exactly two shapes: a `type: [a, b]` array (realize.rs:1739-1759,
`UnionMode::TypeSet`) and the `type` term of an `Intersection` beside `$ref`/`const`/`enum`/`anyOf`/
`oneOf`/`allOf` (realize.rs:1761-1769). Binary64 marks exist only for model schemas
(`plan.binary64` from `model.binary64_locations()`, realize.rs:723), and the model writer never emits
a `type` array (types.rs `kind: Option<&str>`), never puts `type: object|array` beside a
composition keyword (Optional is a bare `anyOf`, types.rs:720-723; a union or shorthand is a bare
`oneOf`, :999-1001, :1024-1030; `distinct:` `allOf` entries are counts-only and dropped by
`counts_elements`, realize.rs:413-424, 1716-1724); its only intersection is an enum's
`type: string` + `enum`, which holds no number. So no Binary64 can sit under a pointer-sharing term.
Archive as not reproduced; a hand-written schema with such a shape carries no Binary64 marks.
