---
format: aep.planning-md/3
id: story:one-time-disclosure-inventory-is-published
kind: story
status: active
title: The one-time disclosure inventory is published, and each cell is built or refused by its real cause
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#573
- provider: github
  reference: beyond10x/ess#574
relations:
- serves: vision:O2
- informed_by: story:disclosure-scenarios-bind-the-caller
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T14:33:21Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-14T14:33:21Z", actor: "human:timo", revision: 3}
---
## Outcome

The one-time disclosure inventory is published and predictable: the one-time response guide lists
the cells synthesized per `one_time_response` field (origin, retry and rotation per originating
actor; each outcome of each granted command, each denied command and each view read per actor),
which cells are refused rather than generated, the scenario id shape, the order of growth, and a
worked count for a small model. Every refused cell names its real cause and the declaration that
would make it reachable; a `retry` cell on a create whose identity the caller supplies is built as
the identical invocation over the origin's post-state where the command declares an
existing-instance refusal.

Fits none of the four generic language features: documentation and synthesis of an existing
construct. Related: `story:disclosure-scenarios-bind-the-caller` (https://github.com/beyond10x/ess/issues/550).

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/573 and
https://github.com/beyond10x/ess/issues/574, and asks 1 and 3 of
https://github.com/beyond10x/ess/issues/570 (confirm the size is intended; document the order of
growth), from a downstream IAM service specification (ess 0.59.0).

1. **Need.** An author cannot predict a suite before synthesizing it: the inventory is written only
   in the unpublished design page (`docs/design/one-time-response-values.md:90-151`), while the guide
   (`website/docs/guides/verify/one-time-responses.md:7-46`) states the rule, not the cells. The
   `retry` cell of a create with a caller-supplied identity is refused with "the retry has no
   established surviving subject" (`crates/verify/ess-conformance/src/synthesize/disclosure.rs:156`),
   a cause that does not apply; the fallback `repeat_exact` redraws the supplied identity and fails
   silently, and a declared existing-key refusal is never consulted.
2. **Class.** Documentation gap (inventory); defect (the refusal names the wrong cause, and a
   declared outcome is ignored).
3. **Already expressible?** The model is; the suite and the guide do not say it.
4. **Fit.** The retry rule is the design's own ("execute the exact same command input and actor
   again", `docs/design/one-time-response-values.md:96-98`); with a supplied identity the repeat is an
   existing-key create, answered by the declared existing-instance refusal, or refused naming it as
   missing. Shares `synthesize/disclosure.rs` with `story:disclosure-scenarios-bind-the-caller`
   (sequence them).
5. **Second adopter.** Any one-time secret issued under a caller-chosen key (a webhook signing
   secret named by the caller).
6. **Cost.** Guide pages; a synthesis fix; suites of affected models gain a retry cell. No format
   change.
7. **Alternatives.** (a) Change nothing. (b) Publish only. (c) Publish, fix the cause, build the
   cell where the model declares the answer (chosen).

## Decisions

Accept. The enumeration in the design page is the intended one; the answer to "is the per-scenario
size intended" is in `story:a-committed-suite-is-compact-drift-checked-and-reviewable`.

## Acceptance

- `one_time_guide_lists_the_cells`: the guide names each cell kind, the refused ones, the id shape
  `<command>/disclosure/<outcome>/<field>/<aspect>/as/<actor>`, the growth actors x (outcomes +
  commands + views), and the counts 5/4, 10/8 and 11/9 for the three minimal variants; a test pins
  the counts against synthesis.
- `supplied_identity_retry_names_its_cause`: without an existing-instance outcome the refusal names
  it as the missing declaration.
- `supplied_identity_retry_expects_the_declared_refusal`: with one, the cell is built over the
  origin's post-state and passes on the interpreted target.
