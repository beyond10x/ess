---
format: aep.planning-md/3
id: story:an-arrangement-refusal-names-the-cause-of-every-creator-tried
kind: story
status: draft
title: An arrangement refusal names the cause of every creator tried
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#728
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 2
---
## Need

When an entity has several creating branches and none arranges the row a scenario needs, the
ESS-SYNTH-004 refusal names only the first creator tried, in command-name order, and its cause. A
creator that fails for an unrelated reason therefore hides the cause the author can act on: the
reader is sent to the wrong command.

Minimal reproduction (`ess/34`, one file; `m4` in the triage reproduction): `example.review.Decision`
is created by two commands.

```yaml
commands:
  - name: example.review.Confirm        # sorts first
    input: [{name: key, type: example.review.DecisionKey}, {name: draft, type: example.review.DraftKey}, ...]
    outcomes:
      - {name: no-draft, when_related: {via: input.draft, exists: false}, error: example.review.NoDraft}
      - {name: already-exists, existing_instance: true, error: example.review.AlreadyExists}
      - {name: confirmed, creates: example.review.Decision, instance: key, ...}
      # example.review.Draft has no creating command
  - name: example.review.Record
    outcomes:
      - {name: incomplete, when: kind == Erratum, error: example.review.Incomplete}
      - {name: not-verified, when: kind == Accept, error: example.review.NotVerified}
      - {name: already-exists, existing_instance: true, error: example.review.AlreadyExists}
      - {name: recorded, creates: example.review.Decision, instance: key, ...}
  - name: example.review.Annotate      # updates the Decision; not-accepted: when_subject: {predicate: kind != Accept}
```

```console
$ ess verify conform synthesize --path m4 --out m4.json
refused: refusal[ESS-SYNTH-004]: outcome example.review.Annotate/annotated has no scenario `example.review.Annotate/outcome/annotated`
  it needs an instance of `example.review.Decision` to change without moving, and the route runs through `example.review.Confirm/confirmed`, which no input reaches
  help: the route to this state runs through a branch no input reaches; see that outcome's own refusal
```

`Record/recorded` is also unreachable (`no candidate of the 4 tried satisfies none of: kind == Erratum, kind == Accept`),
and the refusal does not say so. `m3` shows the same misdirection where `Record` fails on an
`Optional` guard (see `story:an-absent-optional-under-false-and-unknown-is-arranged`).

Cause, from the code: `arrange` (`crates/verify/ess-conformance/src/synthesize.rs:7071`) tries every
creator in route-length then name order and keeps `first`, the first creator's `Unreachable`
(lines 7085 and 7108); `arrange_first` (line 7030) does the same across target states.

## Acceptance

- `ess verify conform synthesize` on `m4` refuses `example.review.Annotate/outcome/annotated` with
  ESS-SYNTH-004 whose text names both `example.review.Confirm/confirmed` and
  `example.review.Record/recorded`, each with its own cause, in the order they were tried.
- A refusal where only one creator exists, or where every creator fails for the same cause, keeps
  its current text: the existing ESS-SYNTH-004 tests in `crates/verify/ess-conformance/tests/`
  pass unchanged.
- No suite that synthesized before changes bytes.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/arrangement_refusal_names_every_creator.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A refusal must name the cause the author can repair; with two creators it names the
   first one only. Reproduction above, brand-free. The requester's proposal: "the ESS-SYNTH-004
   text should name the guard it could not decide rather than a different command's branch".
2. **Class.** Defect. The ESS-SYNTH-004 help sends the reader to "that outcome's own refusal"
   (`synthesize.rs:3004-3007`), which for `Confirm` is an unrelated cause, and the crate's own rule
   is that a refusal "always says something" the reader can act on
   (`crates/verify/ess-conformance/src/decision.rs:110-115`).
3. **Already expressible?** Not a language question; the author can only remove the other creator to
   see the real cause, which the issue shows the requester had to do.
4. **Design fit.** No authored surface, no new diagnostic code: ESS-SYNTH-004 keeps its code and
   help, and its text lists each creator's `Unreachable` instead of the first. The same `first`
   rule in `arrange_first` gets the same treatment so that sibling path agrees. Every target is
   unaffected; only synthesis refusal text changes.
5. **Second adopter.** `example.stock.Item` is created by `Import` (needs a related `Batch` nothing
   creates in a test) and by `Receive` (an `Optional` guard). Any refusal about `Item` names only
   `Import` and hides `Receive`.
6. **Cost.** No format version, keyword or diagnostic code. Refusal text in refusing suites changes,
   as for earlier refusal-text fixes (beyond10x/ess#278 in `CHANGELOG.md`).
7. **Alternatives.** (a) Change nothing: rejected, the reader is sent to the wrong command. (b) Keep
   the last cause instead of the first: arbitrary in the other direction. (c) Name the creator whose
   own branch guard failed, preferring it over route failures: needs a ranking of causes nobody has
   defined. (d) Chosen: list every creator tried with its cause; complete, and short in practice
   because entities have few creators.

## Decisions

accept, redesigned: the requester asked the refusal to name the undecided guard instead of another
command's branch; the refusal names every creator tried with its own cause, which includes that
guard without ranking causes.

## Source

github:beyond10x/ess#728
