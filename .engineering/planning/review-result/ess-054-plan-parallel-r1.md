---
format: aep.planning-md/3
id: review-result:ess-054-plan-parallel-r1
kind: review-result
status: active
title: Parallel-safety critic, ESS 0.54.0 intake, round 1
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-426
- reviews: story:feature-request-426a
- reviews: story:feature-request-426b
- reviews: story:feature-request-427
- reviews: story:feature-request-428
- reviews: story:feature-request-429
- reviews: story:feature-request-430
- reviews: story:feature-request-432
- reviews: story:feature-request-433
- reviews: story:feature-request-434
- reviews: story:feature-request-435
- reviews: story:feature-request-436
- reviews: story:feature-request-437
- reviews: story:feature-request-438
- reviews: story:feature-request-439
- reviews: story:feature-request-440
- reviews: story:feature-request-441
- reviews: story:feature-request-442
- reviews: story:feature-request-443
- reviews: story:feature-request-444
- reviews: story:feature-request-445
- reviews: story:feature-request-446
- reviews: story:feature-request-447
- reviews: story:feature-request-448
- reviews: story:feature-request-449
- reviews: story:feature-request-450
- reviews: story:feature-request-451
- reviews: story:feature-request-452
- reviews: story:feature-request-453
- reviews: story:feature-request-454
- reviews: story:feature-request-455
- reviews: story:feature-request-456
- reviews: story:feature-request-457
- reviews: story:feature-request-458
- reviews: story:feature-request-459
revision: 1
---
needs-revision
445 — command.rs entry `4339-4470` spans `primitive_literal`, `scalar_representation` (4389-4416) and `literal_representation` (4426); 426b names `scalar_representation` and 450 names 4426, neither body names the other, and 426b also shares the typed-literals design note with 445 (cited; no ordering edge, so either add one citing `command.rs` or narrow 445 to named functions) — .engineering/planning/story/feature-request-445.md:86
452 — shares `deletion_witness` (`synthesize.rs:10736`) with 429 and `affects_beside` (`set_effects.rs:305-350`) with 459, but names only the `ess/23` overlap, and its `command.rs` entry is "inferred" with no function (cited; edge or split) — .engineering/planning/story/feature-request-452.md:85
459 — records "sequenced after #452 (same file)" in prose only and says nothing of 429, with which it shares `outcome_shapes.rs`, `value_expression.rs`, `set_effects.rs` and the `command.rs` outcome conversion, so the order is not an edge in the store (cited; edge or split) — .engineering/planning/story/feature-request-459.md:71
450 — introduces `ess/23` but its typed scope omits `system.rs` `SUPPORTED_FORMATS` (53-55), which 429, 452 and 459 each list, so the first-lander rule in release-plan:ess-054 hides a fifth claimant on that line (cited; edge naming the file, or split the bump into one story) — .engineering/planning/story/feature-request-450.md:75
458 — also needs `ess/23` (line 61) but lists only inferred `primitive_admission.rs` and not `system.rs:53-55`, the same hidden claim on the format list (cited; edge or split) — .engineering/planning/story/feature-request-458.md:79
458 — `synthesize.rs` is listed as "refusal scenario expected payload values" with no function, so it cannot be placed against 455 (`sibling_refusals` 6017, `boundary_inputs`, `overlap_inputs`) or 429 and 452 (`deletion_witness` 10736) in that 14405-line file (cited file, function unstated) — .engineering/planning/story/feature-request-458.md:83
448 — the predicate-parsing refactor claims `command.rs` `RawOutcome.when` with no range, plus `entity.rs` `Invariant::from_node` and `view.rs`, against 437 (`entity.rs`, `related_guard.rs`), 444 (`view.rs:881-886`), 438 (`predicate.rs` 3080-3087) and the `command.rs` stories, and states no functions or ordering (cited files, functions unstated; the scope is wide and unnarrowed) — .engineering/planning/story/feature-request-448.md:78
437 — its inferred `main.rs` entry for `validate` text/JSON `warnings` lands in the output path 434 claims (`validate`, `ValidationSummary`), and neither body mentions the other (inferred for 437, cited for 434; edge or split) — .engineering/planning/story/feature-request-437.md:75
435 — its typed scope carries inferred `crates/edge/ess-cli/src/main.rs`, which the body never lists and which contradicts "No format, CLI or diagnostic change", and it collides with 434 and 437 for no stated reason (inferred; remove the entry or say what changes) — .engineering/planning/story/feature-request-435.md:49
441 — says `read-api-view-idioms.md` is a "shared design note for #439, #441, #444, #446, #447", but 439 lists no such path, and four stories (441, 444, 446, 447) all create the same note, `.example.yaml` and `tests/read_api_view_idioms.rs` with no creator named; these files do not exist yet (inferred; edge or split) — .engineering/planning/story/feature-request-441.md:74
438 — a conditional suite pair ("otherwise the next unreleased pair") would touch `scenario.rs`, `admission.rs` and `count_json.rs`, which 427 cites and 438's scope omits, so the possible collision with 427 is unrecorded (cited for 427, conditional for 438; edge or split) — .engineering/planning/story/feature-request-438.md:82
426 — has no typed scope, so `aep plan artifact waves` lists it under `unassessed` and never places it, while the release plan delivers it; its prose scope only repeats 426a and 426b (no `scope:` in frontmatter; a container needs a statement saying so, or typed entries) — .engineering/planning/story/feature-request-426.md:55

What I read: 35 stories (`aep plan artifact show`, saved per id), the typed scope of each, `aep plan artifact waves --kind story --status draft --format json` (reduced to pairs where both sides are in the 35), `aep plan artifact graph`, `release-plan:ess-054` and `epic:downstream-reported-gaps`. I checked function ranges in the tree with `grep -n 'fn '`.

- **Surfaces:** 33 stories have at least one cited entry, 1 (426a) has only inferred entries (new files), and 1 (426) is unplaced. Several hot-file entries are inferred and say so in the findings.
- **Collision count:** the waves output gives 215 pair-path collisions inside the set. The three big files hold 28 pairs on `command.rs` (8 stories), 21 on `synthesize.rs` (7) and 10 on `resolve.rs` (5).
- **Edges:** the store has no `depends_on` edge among the 35. Every story only `decomposes` the epic, and the ordering that exists is prose in 459, 430 and 438/439.
- **Function overlap:** `command.rs` overlaps at function level for 426b and 445 (`scalar_representation`). `synthesize.rs` overlaps for 429 and 452 (`deletion_witness`). `set_effects.rs` overlaps for 452 and 459 (`affects_beside` 305-350). Elsewhere the cited ranges are disjoint: 427 vs 455 vs 428 in `synthesize.rs`, and 429 (6181-6197) vs 442 (1624-1638, 1849-1852) in `command.rs`. 442 vs 445 on `command.rs` is disjoint where cited.
- **Not stated:** the plan does not say which functions 448, 452 (`command.rs`), 458 (`synthesize.rs`), 437, 450 and 458 (`resolve.rs`) own.
- **Named overlaps:** 438 and 439 share `aggregate.rs` 1150-1163, and the plan names that overlap in prose.
- **Docs:** I did not raise these as findings, and a person should decide. Six stories share `aggregate-views.md` and five each share `spec-versions.md`, `predicates.md`, `values-and-views.md` and `guards-and-predicates.md`. All are new sections in doc files, so the cost is low, but `waves` serialises them.
- **CHANGELOG:** `CHANGELOG.md` is absent from every typed scope, as intended. It is still listed in the prose `## Scope` of 12 bodies (426, 426a, 427, 428, 430, 433, 434, 437, 438, 439, 440, 448, 451). This is a body-wording matter, not a collision.

Outside my lane: the `ess/23` and `/44-/45` format ownership question (which story introduces each) is a design matter. I only flag the collision.

```findings
- file: .engineering/planning/story/feature-request-445.md
  line: 86
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: command.rs entry 4339-4470 spans primitive_literal, scalar_representation (4389-4416) and literal_representation (4426); 426b names scalar_representation and 450 names 4426, neither body names the other, and 426b also shares the typed-literals design note with 445 (cited; no ordering edge, so either add one citing command.rs or narrow 445 to named functions)
- file: .engineering/planning/story/feature-request-452.md
  line: 85
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: shares deletion_witness (synthesize.rs:10736) with 429 and affects_beside (set_effects.rs:305-350) with 459 but names only the ess/23 overlap, and its command.rs entry is inferred with no function (cited; edge or split)
- file: .engineering/planning/story/feature-request-459.md
  line: 71
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: records "sequenced after #452 (same file)" in prose only and says nothing of 429, with which it shares outcome_shapes.rs, value_expression.rs, set_effects.rs and the command.rs outcome conversion, so the order is not an edge in the store (cited; edge or split)
- file: .engineering/planning/story/feature-request-450.md
  line: 75
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: introduces ess/23 but its typed scope omits system.rs SUPPORTED_FORMATS (53-55), which 429, 452 and 459 each list, so the first-lander rule in release-plan:ess-054 hides a fifth claimant on that line (cited; edge naming the file, or split the bump into one story)
- file: .engineering/planning/story/feature-request-458.md
  line: 79
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: also needs ess/23 (line 61) but lists only inferred primitive_admission.rs and not system.rs:53-55, the same hidden claim on the format list (cited; edge or split)
- file: .engineering/planning/story/feature-request-458.md
  line: 83
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: synthesize.rs is listed as "refusal scenario expected payload values" with no function, so it cannot be placed against 455 (sibling_refusals, boundary_inputs, overlap_inputs) or 429 and 452 (deletion_witness 10736) in that 14405-line file (cited file, function unstated)
- file: .engineering/planning/story/feature-request-448.md
  line: 78
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the predicate-parsing refactor claims command.rs RawOutcome.when with no range, plus entity.rs Invariant::from_node and view.rs, against 437, 444, 438 and the command.rs stories, and states no functions or ordering (cited files, functions unstated; the scope is wide and unnarrowed)
- file: .engineering/planning/story/feature-request-437.md
  line: 75
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: its inferred main.rs entry for validate text/JSON warnings lands in the output path 434 claims (validate, ValidationSummary) and neither body mentions the other (inferred for 437, cited for 434; edge or split)
- file: .engineering/planning/story/feature-request-435.md
  line: 49
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: typed scope carries inferred crates/edge/ess-cli/src/main.rs, which the body never lists and which contradicts "No format, CLI or diagnostic change", and it collides with 434 and 437 for no stated reason (inferred; remove the entry or say what changes)
- file: .engineering/planning/story/feature-request-441.md
  line: 74
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: says read-api-view-idioms.md is a shared note for #439, #441, #444, #446, #447 but 439 lists no such path, and 441, 444, 446 and 447 all create the same note, example yaml and test with no creator named; the files do not exist yet (inferred; edge or split)
- file: .engineering/planning/story/feature-request-438.md
  line: 82
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: a conditional suite pair would touch scenario.rs, admission.rs and count_json.rs, which 427 cites and 438's scope omits, so the possible collision with 427 is unrecorded (cited for 427, conditional for 438; edge or split)
- file: .engineering/planning/story/feature-request-426.md
  line: 55
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: has no typed scope, so waves lists it as unassessed and never places it while the release plan delivers it, and its prose scope only repeats 426a and 426b (no scope in frontmatter; a container needs a statement saying so, or typed entries)
```
