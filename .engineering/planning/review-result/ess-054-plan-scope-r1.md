---
format: aep.planning-md/3
id: review-result:ess-054-plan-scope-r1
kind: review-result
status: active
title: Scope critic, ESS 0.54.0 intake, round 1
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
story:feature-request-426 — its acceptance `generated_case_record_domain_note_published` claims the same design-note outcome as 426a's last acceptance bullet, so both stories would be marked done for one note. Reduce 426 to the split and delegate the note and its link to 426a — .engineering/planning/story/feature-request-426.md:48
story:feature-request-435 — Decisions add a new public flag, `ess generate --check` (a "coordinator decision"). The issue asks only for a documentation page, and the flag is missing from the body Scope (:69) and Acceptance, though main.rs is in the frontmatter scope. Either the flag leaves this story or the body carries it — .engineering/planning/story/feature-request-435.md:61
story:feature-request-440 — a decline story also changes validation: it adds a refusal-text change for `{related:}` and `{subject:}` mapping values. The issue offers only "or a design note placing it out of scope", and no sentence in it asks for a diagnostic change — .engineering/planning/story/feature-request-440.md:43
story:feature-request-442 — a decline story adds a new refusal for extra keys beside a `related:` mapping, which Decisions call "In passing". The issue offers only "or a design note placing it out of scope" and does not ask for it — .engineering/planning/story/feature-request-442.md:88

**What I read.** I read 37 artifacts: the parent, the 35 stories in `ids.txt`, and `epic:downstream-reported-gaps` through the graph. I also read the 34 issue JSONs (#426–#430, #432–#459, plus #431 to confirm it is the pull request), the critic procedure and rubric, and `.agents/skills/assessing-external-requests/SKILL.md`. Commands: `aep plan artifact show` for the parent and all 35 stories, `aep plan artifact graph`, and `jq` over the issue files.

**Promises and tracing.** I extracted 38 promises from the parent and traced 36 to a story. The 38 are:
- 33 issue resolutions
- the 426a/426b split
- the Canon-side generator
- the shared `ess/23` bump
- the shared suite-format pair
- one Fit review plus Decisions per story
- the #430 handoff

The two I did not trace are the #430 handoff to the Secrets session and "comment on and close every shipped issue". Both are release-time steps the release-plan owns, and the parent lists them under its own completion sections. I did not count them as gaps.

**Checked and holding:**
- All 33 issues are claimed (#426 by 426, 426a and 426b), and each is a `delivers` edge from `release-plan:ess-054`.
- All 35 stories carry `ess-0.54.0`, `## Fit review` and `## Decisions`.
- The `ess/23` stories are 429, 450, 452, 458 and 459, and 442 declines without a bump. 427 holds the `/44`–`/45` pair.
- For every issue that offered a design-note alternative (#432, #439, #440, #442, #446, #447, #449, #451), the story answers it. Each takes the decline-with-idiom path, or for #439 the "caller resolves windows" note.
- Narrowings are recorded in Decisions:
  - #434 declines the `UNMAPPED:` markers and open questions
  - #438 does not build `default:`, `empty:` or `{in: param.L}`
  - #452 declines cross-domain cascade
  - #459 leaves replace-a-set out
- 426's Decisions say the Canon generator is Canon's to file and ESS does not file it.

**Not established or out of my lane (none of these set the verdict):**
- Acceptance lane: 435 and 448 name "Acceptance adds …" scenarios in Decisions (`generate_check_*`, `predicate_refusal_codes_unchanged_after_per_declaration_parse`) that are not in their Acceptance sections.
- Acceptance lane: 454 says "A follow-up builds the missing witness" while its Acceptance and Scope build it in the story.
- Parallel-safety lane: 439, 441, 444, 446 and 447 share `read-api-view-idioms.md`, its example yaml and `read_api_view_idioms.rs`. 429, 452 and 459 share `system.rs` and `set_effects.rs`.
- Judged traceable, so not flagged:
  - 426b's diagnostics (the story says the requester asked for none of it, but the issue lists those pitfalls)
  - 439's range-selector synthesis
  - 438's trailing-text hint, which 441 relies on and which overlaps 441's `param.limit_s * 1000` refusal check. I left that overlap unflagged because 441 only asserts the refusal stays.

```findings
- file: .engineering/planning/story/feature-request-426.md
  line: 48
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance `generated_case_record_domain_note_published` claims the same design-note outcome as 426a's last acceptance bullet, so both stories would be marked done for one note; reduce 426 to the split and delegate the note and its link to 426a
- file: .engineering/planning/story/feature-request-435.md
  line: 61
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: Decisions add a new public flag `ess generate --check` as a coordinator decision, which the documentation-only issue does not ask for and the body Scope (line 69) and Acceptance do not carry; either the flag leaves this story or the body carries it
- file: .engineering/planning/story/feature-request-440.md
  line: 43
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: a decline story also changes validation by adding a refusal-text change for `{related:}` and `{subject:}` mapping values, while the issue offers only "or a design note placing it out of scope" and no sentence asks for a diagnostic change
- file: .engineering/planning/story/feature-request-442.md
  line: 88
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: a decline story adds a new refusal for extra keys beside a `related:` mapping ("In passing"), which the issue offers no sentence for and its design-note alternative does not include
```
