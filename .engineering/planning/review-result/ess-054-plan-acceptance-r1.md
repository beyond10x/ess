---
format: aep.planning-md/3
id: review-result:ess-054-plan-acceptance-r1
kind: review-result
status: active
title: Acceptance critic, ESS 0.54.0 intake, round 1
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

story:feature-request-435 — the Decisions add `generate_check_reports_drift_and_exits_nonzero` and `generate_check_passes_on_fresh_output` to the acceptance, but the Acceptance section holds neither, so `ess generate --check` can fail to exist and all four listed bullets still pass — .engineering/planning/story/feature-request-435.md:61
story:feature-request-448 — the Decisions add `predicate_refusal_codes_unchanged_after_per_declaration_parse` to the acceptance, but the Acceptance section lacks it, so a code moving out of `ESS-SPEC-*` passes every listed bullet — .engineering/planning/story/feature-request-448.md:64
story:feature-request-426a — this decline-with-idiom story's acceptance never says `docs/design/generated-case-record-domains.md` exists or what it states (no protocol/1 import in ESS, meaning stays in Canon, "case-record domain" vs `ess specify protocol`); "The note is linked from … guards-and-predicates.md; `task site-build` passes" is satisfied by a link to a file nobody wrote, because the site links design notes by GitHub blob URL — .engineering/planning/story/feature-request-426a.md:56
story:feature-request-426 — "Split into 426a … and 426b …" names no observable outcome, and the one real bullet covers only 426a's note, so nothing at the parent says 426b's diagnostics shipped — .engineering/planning/story/feature-request-426.md:47
story:feature-request-432 — acceptance does not name the guide section "A view served from a replica" or its content (shared-store `requires:`, per-environment binding values, write-primary placement out of scope), and `site_navigation` "passes with the new section's links" before the section exists — .engineering/planning/story/feature-request-432.md:46
story:feature-request-444 — "the shared idioms section shows unit newtypes" names no file or heading, and the guide paragraph in `fields-and-invariants.md` that the Decisions say "states the two limits" is not named, so the documentation deliverable cannot be checked as present — .engineering/planning/story/feature-request-444.md:68
story:feature-request-446 — "the shared idioms section shows the `max` instant, the latest-row view and the flat two-key grouping" names no file or heading, and the Decisions' "`arg_max` is recorded in the design note" has no observable — .engineering/planning/story/feature-request-446.md:71
story:feature-request-447 — "the shared idioms section shows the copied label … and says when each is the right fact" names no file or heading, and "says when each is the right fact" is a reader's judgement, not a check — .engineering/planning/story/feature-request-447.md:75
story:feature-request-451 — "Guide section and design note: held by `task site-build` link checking and the fixture test" cannot hold `docs/design/stored-rules-boundary.md`, because the site links design notes by GitHub blob URL (website/docs/guides/specify/values-and-views.md:270) and the fixture test reads only the guide's fenced model; the note's existence and the boundary it records (including #449's pattern boundary) are unobservable — .engineering/planning/story/feature-request-451.md:70
story:feature-request-449 — "the section links the #451 design note; held by site build link checking" is not held, since the link is an external blob URL that `task site-build` does not resolve, and the heading "Text shapes without patterns" is never named in the acceptance — .engineering/planning/story/feature-request-449.md:54
story:feature-request-453 — the first bullet cites two existing tests that already pass, and the new section's content rests on "makes no claim beyond per-command linearizability (reviewed against `linearize.rs:1-120`)", a reviewer's vote that no check can fail — .engineering/planning/story/feature-request-453.md:51
story:feature-request-456 — "`task site-build` passes with the guide edit" passes before the edit, and the acceptance never says the listed `preserves:` example exists beside `ShipOrder` in `guards-and-predicates.md` — .engineering/planning/story/feature-request-456.md:48
story:feature-request-456 — `in_form_is_refused_with_the_list_hint` hedges "an existing refusal test pins it if one exists (I don't know whether one does)", so it names no test that holds it — .engineering/planning/story/feature-request-456.md:47
story:feature-request-457 — "`task site-build` passes with the guide section in `values-and-views.md`" names no heading or content (stored field written by `sets:`, invariants over `state`, projected field), so it passes before the section exists — .engineering/planning/story/feature-request-457.md:59
story:feature-request-452 — "`selection_effects_page_matches_admitted_forms`: the page's description and refusal list agree. Held by `crates/specify/ess-domain/tests/set_effects.rs`, plus the cross-domain idiom paragraph" names no assertion that reads the page and no heading or content for the cascade paragraph — .engineering/planning/story/feature-request-452.md:78
story:feature-request-452 — "affects_delete_entry_naming_other_entity_is_conflicting and affects_delete_beside_other_entry_same_entity_is_conflicting." is two scenario names with no stated observation (no `conflicting_declaration` code, no before/after) — .engineering/planning/story/feature-request-452.md:74
story:feature-request-433 — `macos_regenerates_over_os_provenance_label` branches ("succeeds after (b). Before (b) it is red, or it reports a named skip"), so it gives no single check for when the story is done if (c) never reproduces the refusal and (b) never lands — .engineering/planning/story/feature-request-433.md:55
story:feature-request-434 — the Decisions list "scenarios outside a component" as `completeness.outside`, but no acceptance bullet reads it, so that key can be omitted and the story still closes — .engineering/planning/story/feature-request-434.md:59
story:feature-request-438 — Decisions item 3 ("The guide documents the list, empty-list and default idioms") has no acceptance; `list_parameter_quantifier_validates` checks the probe shapes validate, not that any page shows them — .engineering/planning/story/feature-request-438.md:88
story:feature-request-438 — `list_view_parameter_openapi_encoding_is_stated` ("states its array encoding") names no encoding or key (`style`/`explode`), although the fit review says "Settle it in this story" — .engineering/planning/story/feature-request-438.md:92
story:feature-request-443 — the documented idiom uses `filter: duration > 0` so a zero total is absent, but the only synthesis bullet covers "the unfiltered form", and the fit review says the filtered form is refused with `ESS-SYNTH-017`, so the zero-total-absent claim is checked by nothing — .engineering/planning/story/feature-request-443.md:55
story:feature-request-430 — the clause "synthesis reverted to the 0.52 behaviour reproduces `existing_instance` answers" is not a check anyone runs at close, and the reproduction it describes is already covered by the first bullet — .engineering/planning/story/feature-request-430.md:49

**What I read:** 35 of 35 ids from `~/.cache/ess-054-fit/ids.txt`, in full, via `aep plan artifact show <id>`. I also read `aep plan artifact kinds`, `aep plan artifact lifecycle story`, `release-plan:ess-054`, and the `assessing-external-requests` skill. I used `git grep` on the tree for `requires: ess`, the design-note link pattern, and the `predicate_reference_page.rs` header.

**What I could not establish:**
- Whether `task site-build` or `site_navigation.rs` resolves GitHub blob links to design notes. I inferred it does not from the link pattern (website/docs/guides/specify/values-and-views.md:270) and did not run the build.
- Whether 0.53.0 already reports `BrokenInvariant` (#457) or sends the 454/455 overlap scenarios. Those fit reviews say "unknown", so I did not judge them.
- The decline stories' Decisions each promise a `specification` artifact, and `release-plan:ess-054` says "declined with its artifact". No acceptance names that artifact and none exists in the store yet. I did not raise it as a per-story finding because you scoped the check to the note, the guide section and the test, but a closer would have no check for it.

**Out of my lane:**
- `docs/design/read-api-view-idioms.md`, its example yaml and `read_api_view_idioms.rs` are claimed as scope by 439, 441, 444, 446 and 447. Stories 454 and 455 both claim the "which branch answers" section. Stories 438 and 439 share `aggregate.rs`.
- 435's metadata scope lists `crates/edge/ess-cli/src/main.rs`, but its body Scope does not.
- The plan-critic-parallel-safety and plan-critic-design critics own these.

```findings
- file: .engineering/planning/story/feature-request-435.md
  line: 61
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the Decisions add `generate_check_reports_drift_and_exits_nonzero` and `generate_check_passes_on_fresh_output` to the acceptance, but the Acceptance section holds neither, so `ess generate --check` can fail to exist and all four listed bullets still pass'
- file: .engineering/planning/story/feature-request-448.md
  line: 64
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the Decisions add `predicate_refusal_codes_unchanged_after_per_declaration_parse` to the acceptance, but the Acceptance section lacks it, so a code moving out of `ESS-SPEC-*` passes every listed bullet'
- file: .engineering/planning/story/feature-request-426a.md
  line: 56
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'this decline-with-idiom story''s acceptance never says `docs/design/generated-case-record-domains.md` exists or what it states (no protocol/1 import in ESS, meaning stays in Canon, "case-record domain" vs `ess specify protocol`); a link plus `task site-build` is satisfied by a link to a file nobody wrote, because the site links design notes by GitHub blob URL'
- file: .engineering/planning/story/feature-request-426.md
  line: 47
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"Split into 426a ... and 426b ..." names no observable outcome, and the one real bullet covers only 426a''s note, so nothing at the parent says 426b''s diagnostics shipped'
- file: .engineering/planning/story/feature-request-432.md
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'acceptance does not name the guide section "A view served from a replica" or its content (shared-store `requires:`, per-environment binding values, write-primary placement out of scope), and `site_navigation` "passes with the new section''s links" before the section exists'
- file: .engineering/planning/story/feature-request-444.md
  line: 68
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"the shared idioms section shows unit newtypes" names no file or heading, and the guide paragraph in `fields-and-invariants.md` that the Decisions say "states the two limits" is not named, so the documentation deliverable cannot be checked as present'
- file: .engineering/planning/story/feature-request-446.md
  line: 71
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"the shared idioms section shows the `max` instant, the latest-row view and the flat two-key grouping" names no file or heading, and the Decisions'' "`arg_max` is recorded in the design note" has no observable'
- file: .engineering/planning/story/feature-request-447.md
  line: 75
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"the shared idioms section shows the copied label ... and says when each is the right fact" names no file or heading, and "says when each is the right fact" is a reader''s judgement, not a check'
- file: .engineering/planning/story/feature-request-451.md
  line: 70
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"Guide section and design note: held by `task site-build` link checking and the fixture test" cannot hold `docs/design/stored-rules-boundary.md`, because the site links design notes by GitHub blob URL and the fixture test reads only the guide''s fenced model; the note''s existence and the boundary it records (including #449''s pattern boundary) are unobservable'
- file: .engineering/planning/story/feature-request-449.md
  line: 54
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"the section links the #451 design note; held by site build link checking" is not held, since the link is an external blob URL that `task site-build` does not resolve, and the heading "Text shapes without patterns" is never named in the acceptance'
- file: .engineering/planning/story/feature-request-453.md
  line: 51
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: 'the first bullet cites two existing tests that already pass, and the new section''s content rests on "makes no claim beyond per-command linearizability (reviewed against `linearize.rs:1-120`)", a reviewer''s vote that no check can fail'
- file: .engineering/planning/story/feature-request-456.md
  line: 48
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"`task site-build` passes with the guide edit" passes before the edit, and the acceptance never says the listed `preserves:` example exists beside `ShipOrder` in `guards-and-predicates.md`'
- file: .engineering/planning/story/feature-request-456.md
  line: 47
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`in_form_is_refused_with_the_list_hint` hedges "an existing refusal test pins it if one exists (I don''t know whether one does)", so it names no test that holds it'
- file: .engineering/planning/story/feature-request-457.md
  line: 59
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: '"`task site-build` passes with the guide section in `values-and-views.md`" names no heading or content (stored field written by `sets:`, invariants over `state`, projected field), so it passes before the section exists'
- file: .engineering/planning/story/feature-request-452.md
  line: 78
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '"`selection_effects_page_matches_admitted_forms`: the page''s description and refusal list agree. Held by `crates/specify/ess-domain/tests/set_effects.rs`, plus the cross-domain idiom paragraph" names no assertion that reads the page and no heading or content for the cascade paragraph'
- file: .engineering/planning/story/feature-request-452.md
  line: 74
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the bullet is two scenario names with no stated observation (no `conflicting_declaration` code, no before/after)'
- file: .engineering/planning/story/feature-request-433.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`macos_regenerates_over_os_provenance_label` branches ("succeeds after (b). Before (b) it is red, or it reports a named skip"), so it gives no single check for when the story is done if (c) never reproduces the refusal and (b) never lands'
- file: .engineering/planning/story/feature-request-434.md
  line: 59
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the Decisions list "scenarios outside a component" as `completeness.outside`, but no acceptance bullet reads it, so that key can be omitted and the story still closes'
- file: .engineering/planning/story/feature-request-438.md
  line: 88
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'Decisions item 3 ("The guide documents the list, empty-list and default idioms") has no acceptance; `list_parameter_quantifier_validates` checks the probe shapes validate, not that any page shows them'
- file: .engineering/planning/story/feature-request-438.md
  line: 92
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: '`list_view_parameter_openapi_encoding_is_stated` ("states its array encoding") names no encoding or key (`style`/`explode`), although the fit review says "Settle it in this story"'
- file: .engineering/planning/story/feature-request-443.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the documented idiom uses `filter: duration > 0` so a zero total is absent, but the only synthesis bullet covers "the unfiltered form" and the fit review says the filtered form is refused with `ESS-SYNTH-017`, so the zero-total-absent claim is checked by nothing'
- file: .engineering/planning/story/feature-request-430.md
  line: 49
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the clause "synthesis reverted to the 0.52 behaviour reproduces `existing_instance` answers" is not a check anyone runs at close, and the reproduction it describes is already covered by the first bullet'
```
