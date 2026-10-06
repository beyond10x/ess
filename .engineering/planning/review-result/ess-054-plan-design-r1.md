---
format: aep.planning-md/3
id: review-result:ess-054-plan-design-r1
kind: review-result
status: active
title: Design critic, ESS 0.54.0 intake, round 1
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
story:feature-request-429 — five bodies (429, 450, 452, 458, 459) each claim the one `ess/23` bump, and the release plan's "whichever story lands first introduces the version" makes ownership a merge race; 429 should be named owner, or the bump split into its own story, with `depends_on` from 450, 452, 458 and 459 (450 and 458 need the edit but list no `system.rs`) — .engineering/planning/story/feature-request-429.md:83
story:feature-request-459 — "sequenced after #452 (same file)" is prose only, and it also reuses 452's deleting `affects:` entry and `selection-effects.md` section, so `depends_on story:feature-request-452` is missing — .engineering/planning/story/feature-request-459.md:71
story:feature-request-455 — 454 and 455 each describe half of one precedence design ("Same design as #454", "One guide section documents both") and neither owns the section or the witness, so merge them or record `depends_on story:feature-request-454` with 454 as owner — .engineering/planning/story/feature-request-455.md:47
story:feature-request-438 — "build them as one unit or one after the other" leaves 438/439 ordering undecided on the shared `aggregate.rs` parameter selectors; an edge `439 depends_on 438` is needed, and its direction is forced because 441 already needs 438 — .engineering/planning/story/feature-request-438.md:82
story:feature-request-438 — "the next unreleased pair" contradicts 427's "every other 0.54 suite-format change should share this pair", so 438 should say that any suite bump is `ess-conformance/44`/`/45` and `depends_on` 427 — .engineering/planning/story/feature-request-438.md:82
story:feature-request-439 — the acceptance relies on the "shared idioms test (see #446)", but its scope lists none of the shared note, example or test, and no edge points at their owner — .engineering/planning/story/feature-request-439.md:85
story:feature-request-441 — four stories (441, 444, 446, 447) each list `read-api-view-idioms.md`, `.example.yaml` and `read_api_view_idioms.rs` as new `inferred` files, so 441 should declare itself the creator, and 439, 444, 446 and 447 should `depends_on` it, or the four doc-only idiom stories should merge — .engineering/planning/story/feature-request-441.md:74
story:feature-request-441 — its refusal of `param.limit_s * 1000` "with the hint built under #438" cannot be stated without 438's hint, and no edge records it, so `depends_on story:feature-request-438` is missing — .engineering/planning/story/feature-request-441.md:63
story:feature-request-430 — its member-level copy scenario needs "member-level parameter copies" that 428 introduces ("Land with or after #428"), and no edge records it, so `depends_on story:feature-request-428` is missing — .engineering/planning/story/feature-request-430.md:41
story:feature-request-437 — "land in sequence or share one reviewer" leaves the order undecided on the shared `ValidationSummary`; record `depends_on story:feature-request-434`, because 434's body never mentions 437 — .engineering/planning/story/feature-request-437.md:57
story:feature-request-440 — 440 adds a refusal inside the `AuthoredMappingSource` visitor that 445 rewrites to admit scalars, and neither body names the other, so one edge between them (445 first) is missing — .engineering/planning/story/feature-request-440.md:53
story:feature-request-435 — one item holds a docs guide and a new `ess generate --check` verb, whose files and tests appear in neither Scope nor Acceptance. The seam is the verb: split it into its own story, which the guide's drift section then `depends_on` — .engineering/planning/story/feature-request-435.md:61
story:feature-request-436 — the page "links to the #435 page instead of restating it", and `site_navigation` fails if that page is absent, so `depends_on story:feature-request-435` is missing — .engineering/planning/story/feature-request-436.md:49
story:feature-request-449 — its acceptance links 451's design note `stored-rules-boundary.md`, which 451 creates and which "also records #449's pattern boundary", but 449's scope omits it and no edge exists, so `depends_on story:feature-request-451` is missing — .engineering/planning/story/feature-request-449.md:54
story:feature-request-426 — the decision record repeats 426a's acceptance (`generated_case_record_domain_note_published`) and lists the same eight scope files as 426a/426b, so two items own one surface and both would close on the same evidence. Keep only the "children delivered" acceptance on 426 and drop its Scope — .engineering/planning/story/feature-request-426.md:48
story:feature-request-445 — its "quote it" and `sets:` hints come from `scalar_representation` (command.rs:4389), whose hint 426b rewrites, so `depends_on story:feature-request-426b` is missing — .engineering/planning/story/feature-request-445.md:62
story:feature-request-450 — 450 extends `EnumVariant` (types.rs:651-656) and its deserialisation, which 426b's `visit_bool` repair edits in the same visitor (types.rs:778-790), and no edge orders them — .engineering/planning/story/feature-request-450.md:88

What I read: all 35 stories (whole bodies, via `aep plan artifact show`), `release-plan:ess-054`, `aep plan artifact relations`, `aep plan artifact graph`, and `aep plan artifact validate`, which printed `valid`.

What I could not establish:
- **Edges walked.** I walked every edge touching the set, including those to `epic:downstream-reported-gaps` and `vision:O2`. They are only `decomposes`, `serves` and `delivers`; there is no `depends_on` or `blocks` edge between any two stories. The declared graph therefore has no cycle and no serialising chain.
- **Cycle in the implied order.** The order the bodies describe in prose has no cycle if 438 precedes 439 and the ess/23 owner precedes 450, 452, 458 and 459. Written as 438→439 together with 439→441→438, it would be a cycle, which is why the 438 finding fixes the direction.
- **Out of lane, not set as verdict.** `generate_check_*` scenarios are named in 435's Decisions but absent from its Acceptance (acceptance critic). 429, 452 and 459 collide on `set_effects.rs`, `subset.rs` and `system.rs`. `predicates.md` is edited by 438, 439, 442, 449 and 457. `guards-and-predicates.md` is edited by 426a, 451, 454, 455 and 456. `sidebars.ts` is edited by 435 and 436. `main.rs` is edited by 434, 435 and 437. All of that is parallel-safety.

```findings
- file: .engineering/planning/story/feature-request-429.md
  line: 83
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'five bodies (429, 450, 452, 458, 459) each claim the one ess/23 bump and the release plan''s "whichever story lands first introduces the version" makes ownership a merge race; 429 should be named owner, or the bump split into its own story, with depends_on from 450, 452, 458 and 459 (450 and 458 need the edit but list no system.rs)'
- file: .engineering/planning/story/feature-request-459.md
  line: 71
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"sequenced after #452 (same file)" is prose only, and it also reuses 452''s deleting affects: entry and selection-effects.md section, so depends_on story:feature-request-452 is missing'
- file: .engineering/planning/story/feature-request-455.md
  line: 47
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '454 and 455 each describe half of one precedence design ("Same design as #454", "One guide section documents both") and neither owns the section or the witness, so merge them or record depends_on story:feature-request-454 with 454 as owner'
- file: .engineering/planning/story/feature-request-438.md
  line: 82
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"build them as one unit or one after the other" leaves 438/439 ordering undecided on the shared aggregate.rs parameter selectors; an edge 439 depends_on 438 is needed, and its direction is forced because 441 already needs 438'
- file: .engineering/planning/story/feature-request-438.md
  line: 82
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"the next unreleased pair" contradicts 427''s "every other 0.54 suite-format change should share this pair", so 438 should say that any suite bump is ess-conformance/44 and /45 and depends_on 427'
- file: .engineering/planning/story/feature-request-439.md
  line: 85
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the acceptance relies on the shared idioms test (see #446) but its scope lists none of the shared note, example or test, and no edge points at their owner'
- file: .engineering/planning/story/feature-request-441.md
  line: 74
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'four stories (441, 444, 446, 447) each list read-api-view-idioms.md, .example.yaml and read_api_view_idioms.rs as new inferred files, so 441 should declare itself the creator and 439, 444, 446 and 447 should depends_on it, or the four doc-only idiom stories should merge'
- file: .engineering/planning/story/feature-request-441.md
  line: 63
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'its refusal of param.limit_s * 1000 "with the hint built under #438" cannot be stated without 438''s hint, and no edge records it, so depends_on story:feature-request-438 is missing'
- file: .engineering/planning/story/feature-request-430.md
  line: 41
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'its member-level copy scenario needs the member-level parameter copies that 428 introduces ("Land with or after #428"), and no edge records it, so depends_on story:feature-request-428 is missing'
- file: .engineering/planning/story/feature-request-437.md
  line: 57
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"land in sequence or share one reviewer" leaves the order undecided on the shared ValidationSummary; record depends_on story:feature-request-434, because 434''s body never mentions 437'
- file: .engineering/planning/story/feature-request-440.md
  line: 53
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '440 adds a refusal inside the AuthoredMappingSource visitor that 445 rewrites to admit scalars, and neither body names the other, so one edge between them (445 first) is missing'
- file: .engineering/planning/story/feature-request-435.md
  line: 61
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'one item holds a docs guide and a new ess generate --check verb, whose files and tests appear in neither Scope nor Acceptance; the seam is the verb, so split it into its own story that the guide''s drift section depends_on'
- file: .engineering/planning/story/feature-request-436.md
  line: 49
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the page links to the #435 page instead of restating it and site_navigation fails if that page is absent, so depends_on story:feature-request-435 is missing'
- file: .engineering/planning/story/feature-request-449.md
  line: 54
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'its acceptance links 451''s design note stored-rules-boundary.md, which 451 creates and which also records 449''s pattern boundary, but 449''s scope omits it and no edge exists, so depends_on story:feature-request-451 is missing'
- file: .engineering/planning/story/feature-request-426.md
  line: 48
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the decision record repeats 426a''s acceptance (generated_case_record_domain_note_published) and lists the same eight scope files as 426a/426b, so two items own one surface; keep only the children-delivered acceptance on 426 and drop its Scope'
- file: .engineering/planning/story/feature-request-445.md
  line: 62
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'its "quote it" and sets: hints come from scalar_representation (command.rs:4389), whose hint 426b rewrites, so depends_on story:feature-request-426b is missing'
- file: .engineering/planning/story/feature-request-450.md
  line: 88
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '450 extends EnumVariant (types.rs:651-656) and its deserialisation, which 426b''s visit_bool repair edits in the same visitor (types.rs:778-790), and no edge orders them'
```
