---
format: aep.planning-md/3
id: story:the-guards-page-says-a-before-row-external-declared-above-the-input-refusals-answers-before-them
kind: story
status: active
title: 'The guards page says how a refusal answers before the input is checked: an external: marked before_row: true, declared above the input refusals'
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
- depends_on: story:an-external-refusal-before-a-related-row-is-witnessed-at-their-overlap
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:39Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:41Z", actor: "human:timo", revision: 3}
---
## Need

beyond10x/ess#816 item 1. A service that answers 401 for a missing session before it reads the
request body needs its authentication refusal to answer before every input refusal. The reporter
read `before_row: true` as placing an `external:` refusal after the input guards and found no way
to place a refusal before them.

The language already says it: from `ess/27` one precedence rule answers an `external:` marked
`before_row: true` at step 1, beside the input-guarded refusals, and within step 1 the first
declared answers (`website/docs/reference/predicates.md:1413-1425`,
`docs/design/generic-language-features.md:493`). Synthesis witnesses it since 0.60.0
(`Witness::InsideLaterRefusal`, `crates/verify/ess-conformance/src/synthesize.rs:5161`, arranged by
`arranged_inside_later` at 13411; pinned by `tests/adversary_464_pass1.rs`). The guide never says
so: no page under `website/docs/guides/` names `before_row`, and the list under "Which branch
answers" (`website/docs/guides/specify/guards-and-predicates.md:105-133`) says "input refusals
answer first" in item 1 with no word of the external refusal that shares that step.

## Reproduction

Reproduced on 0.98.0. Model `run/auth/system.yaml` (`format: ess/36`; generic names): command
`authfirst.desk.OpenSeat(label: String)` with, in this order, `unauthenticated` (`external: the
request carries no valid session`, `before_row: true`, error `Unauthenticated`), `label-too-long`
(`when: input.label.count > 8`), `opened` (creates `Seat`).

```console
$ ess specify validate --path auth
authfirst v1 — 1 file(s), valid
$ ess verify conform synthesize --path auth --out auth-suite.json
3 scenario(s) (0 authored), 0 refusal(s), written to auth-suite.json
$ ess verify conform run --path auth --suite auth-suite.json --target interpreted --report-format 2
  3 scenarios: 3 passed, 0 failed, 0 error, 0 unsupported
```

The scenario `authfirst.desk.OpenSeat/outcome/unauthenticated` forces the external cause twice:
once with `label: "label"`, and once with `label: "labellabe"`, which `label-too-long` also claims,
requiring `unauthenticated` both times (`run/auth-suite.json`, steps at lines 210-241). A target
that checks the label first fails that second send.

## Outcome

The guards page states the rule where an author looks for it, with a model that runs:

- item 1 of "Which branch answers" names both members of step 1: the input-guarded refusals and an
  `external:` refusal marked `before_row: true`, the first declared answering;
- a new subsection under it, `### Answer before the input is checked`, shows the authenticate-first
  model above (renamed to a generic domain), says that declaring the external refusal below an
  input refusal makes the input refusal answer first, says that an `input_absent:` branch still
  answers before both (step 0, `generic-language-features.md:492`), and links the precedence
  section of the predicates reference.

## Acceptance

1. Red today: `refusal_pair_overlap.rs` asserts that the "Which branch answers" section of
   `website/docs/guides/specify/guards-and-predicates.md` contains "`before_row: true`" in its
   numbered item 1; it fails on the branch (the guides name `before_row` nowhere).
2. The page has a subsection `### Answer before the input is checked` whose first fenced `yaml`
   model validates and synthesizes with 0 refusals; in the external refusal's scenario one send
   lies inside the input refusal's guard, with the external cause forced, and requires the
   external refusal's outcome.
3. Run against the interpreted target the suite passes; run against a target that answers the input
   refusal first (the wrapper pattern of `swapped_order_target_fails_only_overlap`,
   `refusal_pair_overlap.rs:256`) exactly that send's scenario fails.
4. The subsection says, in words the test finds: the external refusal is declared above the input
   refusals; declared below one, the input refusal answers first; `input_absent:` answers before
   both.

## Scope

- `website/docs/guides/specify/guards-and-predicates.md`: item 1 of the list at 109-111; a new
  `###` subsection after the paragraph that ends at 133 and before `### Two input refusals whose
  guards overlap` (218), or directly after the list; the implementor picks the one that keeps the
  section's flow, and the test locates it by heading.
- `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs`: new `#[test]`s only, reusing
  `synthesis`, `scenario`, `sends`, `run`, `not_passed` and `subsection` (344).

## Owned files

- `website/docs/guides/specify/guards-and-predicates.md` (shared with W5, which adds a section
  further down; serial in the one pull request)
- `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs` (shared with W5; new tests only)
- `CHANGELOG.md` (one bullet)

## Not yours

- Any change to synthesis or validation: both already do what the page will say. If acceptance 2 or
  3 is red after the page is written, the step is `LEFT: synthesis does not send the overlap
  (<test output>)` and the triage re-opens it as a synthesis story.
- A refusal that answers before `input_absent:`: not asked; step 0 stays.
- An outcome group's `before_row` (#816 item 2, step 108).

## Tests first

In `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs` (existing target):

- `which_branch_answers_names_the_before_row_external_in_step_one` (acceptance 1).
- `authenticate_first_model_on_the_page_sends_the_overlap_and_requires_the_external`
  (acceptance 2, 3).
- `authenticate_first_subsection_says_where_to_declare_it` (acceptance 4).

Each runs one validation, one synthesis and one interpreted run of a three-outcome model, well under
1 s (bound 60 s per test, 120 s per test step). Time them and name any over 5 s.

## Gate

One at a time; no job in the build pool over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test refusal_pair_overlap`
- `task test-xtask` (`website/` changed: `AGENTS.md`, the package-gate table)
- the sources stamp, checked unchanged:
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
- byte pins: none reachable (no synthesis source changes); `external_beside_held_guard` is run once
  as the control: `cargo test -p ess-conformance --locked --test external_beside_held_guard`
- `task site-build` where the network is available; otherwise CI's `pages.yml` lane

## Depends on

Step 14 (`story:an-external-refusal-before-a-related-row-is-witnessed-at-their-overlap`), as
ORDER places it: the same subsection family of the page. No code dependency.

## Changelog

Under `Changed` (documentation):

- The guards guide says how a refusal answers before the input is checked: declare an `external:`
  refusal marked `before_row: true` above the input refusals (#816).

## Fit review

1. Need: a refusal decided by the request alone (a missing session) must answer before every
   input refusal. The reporter's proposal, theirs: a precedence step before step 1.
2. Class: convenience. The language expresses it (`predicates.md:1413-1425`); the guide omits it,
   which is a documentation defect.
3. Already expressible: yes, run above on 0.98.0; the suite checks the figure (the overlap send
   requires the external refusal, and a target that checks the input first fails).
4. Fit: no new key, no format; the page follows the reference.
5. A second adopter: any API that answers a missing credential, a closed tenant or a rate limit
   before it validates a body.
6. Cost: one page section and three tests.
7. Considered: a new step 0.5 (rejected: step 1 already holds both, ordered by declaration);
   change nothing (rejected: the reporter could not find it, so the next one will not either);
   document it (chosen).

## Decisions

- Decision: decline, with the idiom. The idiom is the page subsection and its tested model; reply
  to the requester with it.
