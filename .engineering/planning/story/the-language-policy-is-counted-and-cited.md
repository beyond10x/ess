---
format: aep.planning-md/3
id: story:the-language-policy-is-counted-and-cited
kind: story
status: active
title: 'The language policy: a counted general share, the fit-review idiom condition and the grids page'
tags:
- policy
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- decomposes: epic:generic-guard-precedence
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T02:54:29Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-21T02:54:30Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The record's § Policy holds as documentation and release tooling, with no change to the language,
the IR or any target (`docs/design/generic-language-features.md` § Policy, § "The two grids",
§ Delivery wave 1 item 5). Visibility issue: https://github.com/beyond10x/ess/issues/554.

What holds after:

- **The general share is counted every release (policy 3).** From the first release cut after
  this story lands, each dated section of `CHANGELOG.md` that changes the source language ends
  with one line, `Language: G general, S one-to-one, P part (general share X%; cumulative Y%).`,
  counted as the record defines (an `Added` or `Changed` entry admitting a new source spelling
  counts once; general where it opens or closes a whole row or column of grid R, grid O or G2's
  effect × target grid, or relaxes a refusal across every position; one-to-one where it opens
  fewer cells; part where one design opens a row in more than one release; defects, tooling and
  refusal-only changes not counted), each counted entry naming its grid row or column in its
  text. The cumulative starts at the record's baseline: 18 general, 59 one-to-one, 4 part, 22%
  (`docs/analysis/2026-10-13-language-request-history.md` § 3). `cargo xtask docs`, which already
  reads `CHANGELOG.md` (`crates/edge/ess-xtask/src/docs.rs:431-432`), refuses a section that lacks
  the line, a malformed line, a share that does not follow from its own counts, and a cumulative
  that does not follow from the baseline and every counted section before it.
- **No decline whose idiom moves a checked figure out of the specification (policy 1).** Question 3
  of `.agents/skills/assessing-external-requests/SKILL.md` ("If an idiom exists, the answer is the
  idiom", `:40-42`) gains the condition: an idiom is a valid answer only if the suite still checks
  the figure the specification's reader sees; otherwise the request is absorbed into a generic
  feature or refused as out of the language with the reason, and the fit review says which.
- **One source format per design decision (policy 2) and the closing rule.** The same skill's
  fit-review questions triage a request onto cells of the grids: it lands in the open format only
  if its row or column is the one being opened, otherwise it waits for the next decision; a
  construct for one request gets no format of its own; opening one cell is refused at review, and
  a refusal of a whole row or column names why (record § "Closing rule"). A release in the middle
  of a decision ships the format as it stands and the rest moves to the next format, counted under
  the same decision (record § "Open questions, answered", answer 2).
- **The grids are a tracked page the fit review cites.** Grid R and grid O, with today's state, the
  citation of each row and the targets, live on one page under `docs/design/` (proposed name
  `docs/design/language-grids.md`; the record names no path), which the record's § "The two grids"
  links instead of carrying its own copy. Each change that opens or refuses a row or column
  updates that page in the same change; the fit-review skill cites it.

## Acceptance

`cargo xtask docs` refuses a release section that changes the source language without a correct
count line, and the fit-review skill states the idiom condition and cites the grids page. Each
line names the failing test written first.

- `crates/edge/ess-xtask/src/docs.rs` unit tests:
  `a_release_changing_the_source_language_needs_the_count_line`,
  `a_count_line_whose_share_does_not_follow_from_its_counts_is_refused`,
  `the_cumulative_follows_the_baseline_and_every_counted_release`,
  `releases_before_the_policy_need_no_count_line` (every dated section up to 0.61.0, the newest
  release when this story was drafted, passes unchanged), and
  `a_section_with_only_defects_tooling_or_refusals_needs_no_count_line`.
- `crates/edge/ess-xtask/tests/layout.rs`:
  `the_fit_review_skill_states_the_idiom_condition_and_cites_the_grids_page` and
  `the_record_links_the_grids_page_instead_of_copying_it`.
- `task test-xtask` passes.
- Tests are modules of existing test binaries; no new tests/*.rs file.

## Scope

- `crates/edge/ess-xtask/src/docs.rs` (the changelog reader beside `newest_release`, `:640-647`);
  `Taskfile.yml` unchanged (`cargo xtask docs` already runs in the gate, `:282-283`).
- `.agents/skills/assessing-external-requests/SKILL.md` questions 3, 4 and 6.
- New page under `docs/design/` (grids R and O, moved from
  `docs/design/generic-language-features.md:186-268`); that record's § "The two grids" becomes a
  link and the closing rule.
- `CHANGELOG.md`: no line is added by this story; the first count line is written by the release
  that follows it.
- No crate under `crates/specify`, `crates/generate`, `crates/verify` or `crates/infra` changes.
- Adversary: no (documentation and release tooling).

## Order

No dependency: runs in parallel with every other story of the wave, from the start. It shares no
file with them except `CHANGELOG.md`, which it reads and does not edit.

## State 2026-10-21

Shipped before 0.63.0 was cut: `cargo xtask docs` holds the count line, the fit-review skill states
the idiom condition, and `docs/design/language-grids.md` is the grids page. Every acceptance test
named above passes on the 0.67.0 batch branch (`task test-xtask`, 19 test binaries, 0 failed).
