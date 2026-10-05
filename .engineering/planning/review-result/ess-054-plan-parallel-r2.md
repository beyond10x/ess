---
format: aep.planning-md/3
id: review-result:ess-054-plan-parallel-r2
kind: review-result
status: active
title: Parallel-safety critic, ESS 0.54.0 intake, round 2
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
- reviews: story:ess-generate-check
revision: 1
---
needs-revision
story:feature-request-448 — `TryFrom<RawOutcome> for Outcome` (command.rs 5858-6070) is edited by 448 (`raw.when` / `outcome_condition` call, 5900-5946), by 452 (`set_effects::affects` 5878, `affects_beside` 6008) and by 459 (6008). 448 says the other `command.rs` stories touch only `subject_of`, the literal functions and input coverage, "none of these", so the shared function is unstated. 448 has no edge to 452 or 459 and the line ranges are disjoint (cited, both sides; state the overlap and order it, or split) — .engineering/planning/story/feature-request-448.md:77
story:feature-request-437 — `resolve.rs` entry reads "bridge domain advisories to warning diagnostics", with no function. That file is claimed unordered by 448 (`family_of_kind` 1025-1055, `family_of` 1095-1116), 450 (`body` 1679-1739, `condition_of`, `entities`, `views`), 458 (`expression_field` 2777-2889) and 445 (4388, 4584, 5108), so it cannot be placed against them (inferred for 437, cited for the others; name the function or split) — .engineering/planning/story/feature-request-437.md:75
story:feature-request-437 — `related_guard.rs` entry reads "guard path for rule (b)", with no function. 448 claims the same file unordered ("`when_related` predicate", inferred), and neither body names the other (cited for 437, inferred for 448; name the function or add an edge) — .engineering/planning/story/feature-request-437.md:73
story:feature-request-450 — `command.rs` entry is "literal rule (4426)", and 445 makes `literal_representation` (4426) `pub(crate)`, leaving "bodies to 426b and 450". 445 and 450 are unordered and 450 does not name 445 or say whether it edits the signature or the body (cited both; name the function and state the order, or split) — .engineering/planning/story/feature-request-450.md:93
story:feature-request-450 — the `fields-and-invariants.md` entry is "guide section", with no heading or placement. 455 edits the same page at lines 35-46, the two are unordered, and no heading for 450 appears in its Decisions or Acceptance (inferred for 450, cited for 455; name the section) — .engineering/planning/story/feature-request-450.md:101
story:feature-request-450 — 450, 452 and 458 each add "this story's sentence" to the `ess/23` row of `spec-versions.md`, and 459 does too. 450, 452 and 458 are unordered with each other and 459 is unordered with 450 and 458 (452 to 459 is an edge). The `ess/22` row is a single table line (spec-versions.md:61), so concurrent additions are one-line merge conflicts. The row is stated but nothing orders or splits the sentences (cited; edge or one paragraph per story) — .engineering/planning/story/feature-request-450.md:102
story:feature-request-459 — `value_expression.rs` entry is "element binder as a `sets:` source" (inferred), with no function. 458 edits `existing_subject_field` (952-1010) in the same file and 458 and 459 are unordered. 429's `sets:` source is at 257-262, and 459 does not say which function takes the element binder (inferred for 459, cited for 458; name the function) — .engineering/planning/story/feature-request-459.md:87
story:feature-request-459 — `existence.rs` entry is "one-row-per-identity check reuse" (inferred), with no function. `one_row` (765) is private to the module, so reuse from `synthesize/set_effects.rs` implies a visibility edit. 454 edits `held_state_refusals` (506) and `refusals_on_a_stored_row` (1157) in the same file, and no edge connects them (inferred for 459, cited for 454; name the function and whether it is edited, or drop the entry) — .engineering/planning/story/feature-request-459.md:90
story:feature-request-441 — 441 creates `read-api-view-idioms.example.yaml` and 439, 443, 444, 446 and 447 each add "their own models" to it. The dependents are unordered with each other, and none names the entities, views or domain it adds. It is a single validated document, so the additions are same-region appends and risk name clashes (`Millis` in 444, "row entity" in 443, "latest-row view" in 446). The note's `##` sections and the test cases are named, but the yaml models are not (inferred, all six bodies; name one namespace per idiom, or split into one example file per idiom) — .engineering/planning/story/feature-request-441.md:78

What I read: 36 stories (`aep plan artifact show` for each id, run from the worktree) and `aep plan artifact graph` (22 `depends_on` edges inside the set). `aep plan artifact waves --kind story --status draft --format json` gave no cycles and 9 waves for the set. I closed the edges transitively and listed every typed-scope path shared by an unordered pair (about 40 paths). I read each entry's stated functions or sections against the tree at `087935463f`. I also read `release-plan:ess-054`, `critic-parallel-r1.md` and `OUTCOMES.tsv`.

Of the 12 round-1 findings, 11 are fixed in the bodies. The edges now exist (429 owns `system.rs`; 450, 452, 458 and 459 depend on it; 459 depends on 452; 438 on 427; 440 on 445; 437 on 434), and the functions are named in 445, 452, 458, 448 and 435. 426 states "none; delivered by 426a and 426b". The twelfth, 448, is partly fixed: it names its functions but still omits 452 and 459 (first finding above).

Surfaces: 34 stories have at least one cited entry, 1 (426a) has only inferred entries (new files), and 1 (426) has no entry and says so. `waves` lists 426 under `unassessed`; that is a stated container, not a defect.

What I could not establish:
- Whether 433's `output_ownership/mod.rs` entry ("`image` callers") edits any function that `ess-generate-check` touches. That story's entry reads "`check` (75) guard reused; `publish` (87)", which I took as read-only. `image` has about 10 callers in the file and 433 names none. Not raised: the sibling reads only, and the cited functions are far apart.
- 457's placement of the new `ess-check` example on `predicates.md` (the entry gives only line 29). 449's new section is at 830-982, so I did not raise it.
- The `ess-conformance/tests/read_api_view_idioms.rs` appends from 439, 442, 443, 444, 446 and 447 are named cases. I judged them stated and did not raise them.
- Outside my lane: 449's scope omits `docs/design/stored-rules-boundary.md`, which its test reads. It is ordered after 451, so not a collision.

```findings
- file: .engineering/planning/story/feature-request-448.md
  line: 77
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'TryFrom<RawOutcome> for Outcome (command.rs 5858-6070) is edited by 448 (raw.when and outcome_condition call, 5900-5946), by 452 (set_effects::affects 5878, affects_beside 6008) and by 459 (6008). 448 says the other command.rs stories touch only subject_of, the literal functions and input coverage, so the shared function is unstated; no edge to 452 or 459 and the line ranges are disjoint (cited on both sides; state the overlap and order it, or split)'
- file: .engineering/planning/story/feature-request-437.md
  line: 75
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'resolve.rs entry reads bridge domain advisories to warning diagnostics, with no function, while 448 (family_of_kind, family_of), 450 (body, condition_of, entities, views), 458 (expression_field) and 445 (4388, 4584, 5108) claim the same file unordered, so it cannot be placed against them (inferred for 437, cited for the others; name the function or split)'
- file: .engineering/planning/story/feature-request-437.md
  line: 73
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'related_guard.rs entry reads guard path for rule (b), with no function, and 448 claims the same file unordered (when_related predicate, inferred); neither body names the other (cited for 437, inferred for 448; name the function or add an edge)'
- file: .engineering/planning/story/feature-request-450.md
  line: 93
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'command.rs entry is literal rule (4426) while 445 makes literal_representation (4426) pub(crate) and leaves bodies to 426b and 450; 445 and 450 are unordered and 450 does not name 445 or say whether it edits the signature or the body (cited for both; name the function and state the order, or split)'
- file: .engineering/planning/story/feature-request-450.md
  line: 101
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'fields-and-invariants.md entry is guide section with no heading or placement, while 455 edits the same page at lines 35-46 and the two are unordered; no heading for 450 appears in Decisions or Acceptance (inferred for 450, cited for 455; name the section)'
- file: .engineering/planning/story/feature-request-450.md
  line: 102
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '450, 452 and 458 each add this story''s sentence to the ess/23 row of spec-versions.md, as does 459; 450, 452 and 458 are unordered with each other and 459 is unordered with 450 and 458 (452 to 459 is an edge). The ess/22 row is a single table line (spec-versions.md:61), so concurrent additions are one-line merge conflicts and nothing orders or splits them (cited; edge, or one paragraph per story)'
- file: .engineering/planning/story/feature-request-459.md
  line: 87
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'value_expression.rs entry is element binder as a sets: source with no function, while 458 edits existing_subject_field (952-1010) in the same file and 458 and 459 are unordered; 429 reads sets: sources at 257-262 and 459 does not say which function takes the binder (inferred for 459, cited for 458; name the function)'
- file: .engineering/planning/story/feature-request-459.md
  line: 90
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'existence.rs entry is one-row-per-identity check reuse with no function; one_row (765) is private to the module so reuse from synthesize/set_effects.rs implies a visibility edit, while 454 edits held_state_refusals (506) and refusals_on_a_stored_row (1157) and no edge connects them (inferred for 459, cited for 454; name the function and whether it is edited, or drop the entry)'
- file: .engineering/planning/story/feature-request-441.md
  line: 78
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '441 creates read-api-view-idioms.example.yaml and 439, 443, 444, 446 and 447 each add their own models to it, unordered with each other, naming no entities, views or domain; it is one validated document so the additions are same-region appends with possible name clashes (inferred, all six bodies; name one namespace per idiom, or split into one example file per idiom)'
```
