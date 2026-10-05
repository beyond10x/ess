---
format: aep.planning-md/3
id: review-result:ess-054-plan-acceptance-r2
kind: review-result
status: active
title: Acceptance critic, ESS 0.54.0 intake, round 2
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
story:feature-request-436 — all three bullets (`site_navigation` passes, "every refusal or limit the page names is cited", `task site-build` succeeds) pass on a tree where `website/docs/concepts/adoption-modes.md` was never written; nothing says the page exists, has the table columns or the four modes plus retrofitting, or that `getting-started.md` and `index.md` link it — .engineering/planning/story/feature-request-436.md:53
story:feature-request-435 — the new page `commit-generated-files.md` and its "Why the first diff is large" section are checked only by `site_navigation` and `task site-build`, which pass without them; `default_inventory_is_recorded_from_the_binary` ("its artifact count matches") has no test or named count — .engineering/planning/story/feature-request-435.md:64
story:feature-request-433 — "names `requires: ess X.Y.Z` ... held by the existing site build" passes before the edit, the same defect round 1 fixed in 456 and 457 — .engineering/planning/story/feature-request-433.md:56
story:feature-request-437 — "names the rule and its exclusions, held by the site build" and "lists the class" in `diagnostics.md` pass before the edit; no heading or phrase is named and no test reads either page — .engineering/planning/story/feature-request-437.md:68
story:feature-request-445 — the Decisions promise updates to the binding module table, the guide and the #113 positions table, and no bullet reads any of them, so all three can stay unedited while every bullet passes — .engineering/planning/story/feature-request-445.md:76
story:feature-request-445 — "Rust and Go adapters pass the typed constant, or report the named obligation" is satisfied by either branch and names no obligation, so no single outcome is checked — .engineering/planning/story/feature-request-445.md:83
story:feature-request-458 — "Entity Runtime lowers it to `$from_state` or refuses it by name" accepts either outcome, so the check cannot fail — .engineering/planning/story/feature-request-458.md:75
story:feature-request-458 — the scope lists `commands-and-outcomes.md` (lines 42, 302-331) and the `ess/23` row sentence in `spec-versions.md`, and no bullet checks either — .engineering/planning/story/feature-request-458.md:68
story:feature-request-459 — the Decisions say "design note first", and the scope adds the guide section `## Write one record per element of an input list` and the `spec-versions.md` sentence; none has a bullet — .engineering/planning/story/feature-request-459.md:73
story:feature-request-450 — the guide section in `fields-and-invariants.md` and the `ess/23` row sentence in `spec-versions.md` are scoped and promised, and no bullet reads or names either — .engineering/planning/story/feature-request-450.md:79
story:feature-request-452 — the Decisions add a sentence to the `ess/23` row of `spec-versions.md` and a new section in `set-effects-over-filtered-instances.md`; no bullet checks either, though the `selection-effects.md` page case now does — .engineering/planning/story/feature-request-452.md:70
story:feature-request-457 — `invariant_over_state_catches_a_forgotten_set` ends "If 0.53.0 does not report it, this becomes a defect story of its own", so the bullet has a branch where it never passes and nothing says which branch ends this story — .engineering/planning/story/feature-request-457.md:58
story:feature-request-443 — "synthesis writes `…/aggregate` for both views of the unfiltered form" contradicts the Decisions (a filtered and an unfiltered view; only the unfiltered one synthesizes), so it is unclear which views are checked — .engineering/planning/story/feature-request-443.md:56
story:feature-request-448 — probes `b`, `c` and `e` are files in `~/.cache/ess-054-fit/probe-448`, outside the repository; the bullet names no committed model for the case to run, and scope lists only `validate_parse_refusals.rs` — .engineering/planning/story/feature-request-448.md:65
story:feature-request-454 — "the probe-454 `spec2.yaml` model validates" points at the same kind of uncommitted scratch file, though the committed fixture `held-state-input-refusal.yaml` is in scope; story 455 has the same issue with its `probe-455` bullets — .engineering/planning/story/feature-request-454.md:54

**What I read:** all 36 ids in `~/.cache/ess-054-fit/ids.txt`, in full through `aep plan artifact show`. I also read `aep plan artifact kinds`, `aep plan artifact lifecycle story`, the round-1 report and `OUTCOMES.tsv`. I checked the tree for `BrokenInvariant` (it exists in `interpret/execute.rs:289`), the `ess-check="invariants"` page blocks, and that `docs/design/read-api-view-idioms.md` and `stored-rules-boundary.md` do not exist yet (they are created by the stories).

**Round-1 fixes:** all 22 acceptance fixes in `OUTCOMES.tsv` hold. 435/448 acceptance now lists the moved scenarios, and `ess-generate-check` has four observable bullets (byte-identical, mtime-unchanged, parser refusal, CLI reference). The decline-with-idiom stories name a heading, required phrases and a test that reads the page, and fail naming the missing piece. They are observable: 426a, 432, 440, 441, 442, 443, 444, 446, 447, 449, 451, 453, 454, 455, 456, 457. None of those drops the "page exists" check that 436 and 435 lack. 426's single bullet "426a and 426b are implemented" is acceptable for a parent that only records the split. 433 is now one deciding case, and 438's encoding is settled. The design-lane and scope-lane fixes are not mine to judge.

**What I could not establish:**
- Whether the decline stories' promised `specification` artifacts exist. Round 1 noted this and I did not re-check it.
- Whether the interpreter reports `BrokenInvariant` for a state-entering branch that omits the mapped value (457). I only confirmed the variant exists.
- I did not run `task site-build`, so the claim that it does not resolve design-note links is still inferred.

**Out of my lane:** no new coupling issues. Those I saw, such as the shared `read-api-view-idioms` files and the `ess/23` row edits across 450/452/458/459, belong to the design and parallel-safety critics.

```findings
- file: .engineering/planning/story/feature-request-436.md
  line: 53
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'all three bullets pass on a tree where adoption-modes.md was never written; nothing says the page exists, carries the table columns and the four modes plus retrofitting, or that getting-started.md and index.md link it'
- file: .engineering/planning/story/feature-request-435.md
  line: 64
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the new commit-generated-files.md page and its Why the first diff is large section are checked only by site_navigation and task site-build, which pass without them; default_inventory_is_recorded_from_the_binary has no test or named count'
- file: .engineering/planning/story/feature-request-433.md
  line: 56
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'requires_pin_answers_producing_release is held by the existing site build, which passes before the edit; no test reads the repeated-generation section for the requires pin sentence'
- file: .engineering/planning/story/feature-request-437.md
  line: 68
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'docs bullet (diagnostics.md lists the class, values-and-views.md names the rule and its exclusions) is held only by the site build, which passes before the edit; no heading or phrase is named and no test reads either page'
- file: .engineering/planning/story/feature-request-445.md
  line: 76
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the Decisions promise updates to the binding module table, the guide and the 113 positions table, and no bullet reads any of them'
- file: .engineering/planning/story/feature-request-445.md
  line: 83
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'binding_literal_generated_rust_go_typed_or_obligated passes on either branch (typed constant, or a named obligation) and names no obligation, so no single outcome is checked'
- file: .engineering/planning/story/feature-request-458.md
  line: 75
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'Entity Runtime lowers it to from_state or refuses it by name accepts either outcome, so the check cannot fail'
- file: .engineering/planning/story/feature-request-458.md
  line: 68
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the scoped commands-and-outcomes.md update and the ess/23 row sentence in spec-versions.md have no acceptance bullet'
- file: .engineering/planning/story/feature-request-459.md
  line: 73
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the design note (design note first), the guide section Write one record per element of an input list and the spec-versions.md ess/23 sentence are promised and scoped, and no bullet checks any of them'
- file: .engineering/planning/story/feature-request-450.md
  line: 79
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the guide section in fields-and-invariants.md and the ess/23 row sentence in spec-versions.md are scoped and promised, and no bullet reads or names either'
- file: .engineering/planning/story/feature-request-452.md
  line: 70
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the ess/23 row sentence in spec-versions.md and the new section in set-effects-over-filtered-instances.md are promised in the Decisions with no bullet'
- file: .engineering/planning/story/feature-request-457.md
  line: 58
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'invariant_over_state_catches_a_forgotten_set ends with If 0.53.0 does not report it, this becomes a defect story of its own, so there is a branch where the bullet never passes and nothing says which branch closes the story'
- file: .engineering/planning/story/feature-request-443.md
  line: 56
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'synthesizes group scenarios for both views of the unfiltered form contradicts the Decisions (one filtered view, one unfiltered), so it is unclear which views are checked'
- file: .engineering/planning/story/feature-request-448.md
  line: 65
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'probes b, c and e are files in an uncommitted scratch directory outside the repository; the bullets name no committed model for the case to run and the scope lists none'
- file: .engineering/planning/story/feature-request-454.md
  line: 54
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the probe-454 spec2.yaml model is an uncommitted scratch file although the committed fixture held-state-input-refusal.yaml is in scope; story 455 probe-455 bullets have the same issue'
```
