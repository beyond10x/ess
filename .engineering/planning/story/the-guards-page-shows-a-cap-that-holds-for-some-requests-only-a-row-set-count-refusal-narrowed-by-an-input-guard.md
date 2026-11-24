---
format: aep.planning-md/3
id: story:the-guards-page-shows-a-cap-that-holds-for-some-requests-only-a-row-set-count-refusal-narrowed-by-an-input-guard
kind: story
status: active
title: 'The guards page shows a cap that holds for some requests only: a row-set count refusal narrowed by an input guard'
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
- depends_on: story:the-guards-page-says-a-before-row-external-declared-above-the-input-refusals-answers-before-them
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:53Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:55Z", actor: "human:timo", revision: 3}
---
## Need

beyond10x/ess#816 item 7, decided NQ3 A (decline, with the idiom). A capacity refusal that applies
to some requests only ("a trial tenant holds at most two seats; a paid one has no cap") cannot be
written the way the reporter tried: counting a row set inside `when_related: all:` is refused
("outcome `<name>` lists in `all:` something other than a row an input identity names",
`crates/specify/ess-domain/src/command/related_guard.rs:412`; pinned by
`crates/specify/ess-domain/tests/related_rows_all.rs:103`), and the draft
`a-guard-over-the-rows-a-list-of-references-names-…` keeps it refused by name.

The cap is expressible today with one refusal: a row-set `when_related:` with `count:`
(`website/docs/reference/predicates.md:1726`) beside an input `when:` that narrows it to the
requests it holds for. No guide page shows a row-set count at all: the guards page has no section
on it (`website/docs/guides/specify/guards-and-predicates.md`, headings at 9, 105, 218, 236, 332,
346, 350, 385).

## Reproduction

Reproduced on 0.98.0. Model `run/cap/system.yaml` (`format: ess/36`; generic names): entity
`cap.a.Seat {seat_id, tenant: String, plan: cap.a.Plan}` (`Plan` an enum `trial | paid`); command
`cap.a.AddSeat(tenant, plan)`:

```yaml
- name: trial-full
  when: input.plan == trial
  when_related:
    entity: cap.a.Seat
    where: tenant == input.tenant
    count: {gte: 2}
  error: cap.a.TrialFull
- name: added
  creates: cap.a.Seat
  ...
```

```console
$ ess specify validate --path cap
cap v1 — 1 file(s), valid
$ ess verify conform synthesize --path cap --out cap-suite.json
2 scenario(s) (0 authored), 0 refusal(s), written to cap-suite.json
$ ess verify conform run --path cap --suite cap-suite.json --target interpreted --report-format 2
  2 scenarios: 2 passed, 0 failed, 0 error, 0 unsupported
$ ess verify conform mutate --path cap --target interpreted
mutation audit of cap v1 against interpreted: 4 mutant(s), 3 killed, 0 survived, 0 inconclusive, 1 stillborn, ...
```

`trial-full` arranges two seats of one tenant and sends `plan: trial` for it; the mutants
`count: {gt: 2}` and `when: not (plan == trial)` are killed.

Finding beside it: no scenario sends `plan: paid` for a tenant that already holds two seats, so a
target that caps every plan passes the suite (the audit has no mutant that drops the input
conjunct). See W5-D1.

## Outcome

The guards page gains a section, `## A cap that holds for some requests only`, before "One
outcome for many commands" (350): the model above, renamed to a generic domain; one sentence that
the input `when:` narrows the row-set refusal to the requests it holds for, both conditions holding
together; one sentence that a count inside `when_related: all:` is refused and that this form is
the one to write; a link to the row-set test syntax (`predicates.md:1726`).

## Acceptance

1. Red today: `refusal_pair_overlap.rs` asserts that the guards page has a section `## A cap that
   holds for some requests only` whose first fenced `yaml` model declares a refusal with both
   `when:` and `when_related:` holding `count:`; it fails on the branch (no such section).
2. That model validates, and synthesizes with 0 refusals: the refusal's scenario arranges rows up to
   the count for one input identity and sends the narrowed input, requiring the refusal.
3. Run against the interpreted target the suite passes.
4. The section names the refused form (`all:`) and points at this one, in words the test finds.

## Scope

- `website/docs/guides/specify/guards-and-predicates.md`: one new `##` section before 350.
- `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs`: new `#[test]`s only, reusing
  `synthesis`, `scenario`, `run`, `not_passed`; a section helper by heading like `subsection`
  (344).

## Owned files

- `website/docs/guides/specify/guards-and-predicates.md` (shared with steps 15 and 108; serial)
- `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs` (shared with step 15; new tests
  only)
- `CHANGELOG.md` (one bullet)

## Not yours

- Admitting `count:` inside `all:` (NQ3 B or C, not taken).
- The draft `a-guard-over-the-rows-a-list-of-references-names-…`: unchanged.
- The unchecked narrowing (W5-D1): not in this story under its default.

## Tests first

In `crates/verify/ess-conformance/tests/refusal_pair_overlap.rs` (existing target):

- `guards_page_shows_a_cap_narrowed_by_an_input_guard` (acceptance 1, 4).
- `cap_model_on_the_page_validates_synthesizes_and_passes` (acceptance 2, 3).

Each is one validation, one synthesis and one interpreted run of a two-outcome model, well under 1 s
(bound 60 s per test, 120 s per test step). Time them and name any over 5 s.

## Gate

One at a time; no job in the build pool over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test refusal_pair_overlap`
- `task test-xtask` (`website/` changed)
- the sources stamp, checked unchanged:
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
- byte pins: none reachable (no synthesis source changes)
- `task site-build` where the network is available; otherwise CI's `pages.yml` lane

## Depends on

Step 15 (same page and test file; serial). No code dependency.

## Changelog

Under `Changed` (documentation):

- The guards guide shows a cap that holds for some requests only: a row-set `count:` refusal
  narrowed by an input `when:` (#816).

## Fit review

1. Need: a capacity limit that holds for some requests only. The reporter's proposal, theirs: a
   `count:` element inside `when_related: all:`.
2. Class: convenience. The cap is one refusal with two conditions today.
3. Already expressible: yes, run above on 0.98.0. The suite checks the cap (a request at the count
   with the narrowed input is refused). It does not check the exemption (W5-D1).
4. Fit: no new key; the row-set test and the input guard are existing constructs. NQ3 B or C would
   add a row-set test as an `all:` element, a second spelling of a count beside the row-set guard.
5. A second adopter: a free account may keep three projects, a paid one any number.
6. Cost: one page section and two tests.
7. Considered: `count:` in `all:` (NQ3 B); any row-set test in `all:` (NQ3 C); the idiom
   (chosen, NQ3 A).

## Decisions

- Decision: decline, with the idiom (NQ3 A, taken on its default). The idiom is the page section
  and its tested model; reply to the requester with it.
- W5-D1 (decided: A, 2026-11-24): the suite does not check that a request outside the input guard is
  admitted at the count.
  - A (decided): this story stays docs only; the finding goes to the triage as a synthesis defect
    (the accepting branch's witness sends an input outside the narrowing guard with the row set at
    the count).
  - B: this story also adds that witness to synthesis (size M, synthesis-sources stamp and byte
    pins in scope).
  - Filed as its own draft story: `story:synthesis-sends-a-request-the-narrowing-input-guard-excludes-at-the-cap-so-a-target-that-caps-every-request`.
