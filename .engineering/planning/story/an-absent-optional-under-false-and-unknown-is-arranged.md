---
format: aep.planning-md/3
id: story:an-absent-optional-under-false-and-unknown-is-arranged
kind: story
status: draft
title: An absent Optional under false-and-unknown is arranged
summary: Arrangement cannot decide false and unknown over an absent Optional input, and its refusal blames an unrelated creating command (beyond10x/ess#728)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#728
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/decision.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/arranged_row_beside_absent_optional_guard.rs
revision: 6
---
## Need

When synthesis arranges a row through a creating command whose sibling refusals read an `Optional`
input, the first candidate it builds without that input and on which a sibling guard is `Unknown`
ends the whole search with ESS-SYNTH-002, although another candidate, one that supplies the input
or one on which the guard is `false` (Kleene `false and unknown = false`), creates the row.

Minimal reproduction (`ess/34`, one file; `m2` in the triage reproduction):

```yaml
commands:
  - name: example.review.Record          # creates example.review.Decision {key, kind, note}
    input:
      - {name: key, type: String}
      - {name: kind, type: example.review.Kind}            # Accept | Erratum
      - {name: check, type: Optional<example.review.Check>} # Reported | Verified
      - {name: note, type: String}
    outcomes:
      - {name: incomplete, when: {all: [kind == Erratum, not defined(check)]}, error: example.review.Incomplete}
      - {name: not-verified, when: {all: [kind == Erratum, check != Verified]}, error: example.review.NotVerified}
      - {name: already-exists, existing_instance: true, error: example.review.AlreadyExists}
      - {name: recorded, creates: example.review.Decision, instance: key, sets: {kind: input.kind, note: input.note}, ...}
  - name: example.review.Annotate
    input: [{name: key, type: String}, {name: note, type: String}]
    outcomes:
      - {name: not-accepted, when_subject: {predicate: kind != Accept}, error: example.review.NotAccepted}
      - {name: annotated, updates: example.review.Decision, instance: key, sets: {note: input.note}, ...}
```

```console
$ ess verify conform synthesize --path m2 --out m2.json
refused: refusal[ESS-SYNTH-002]: outcome example.review.Annotate/not-accepted has no scenario `example.review.Annotate/outcome/not-accepted`
  `(kind == Erratum and check != Verified)` cannot be decided against an input of `example.review.Record`
    - `check != Verified`: `check` is declared, and this candidate supplied nothing for it
refused: refusal[ESS-SYNTH-002]: outcome example.review.Annotate/annotated has no scenario `example.review.Annotate/outcome/annotated`
  (same cause)
4 scenario(s) (0 authored), 2 refusal(s)
```

Without `not-accepted` (`m1`) there is no refusal, and `Record/outcome/recorded` is witnessed, one of
its sends being `{kind: Accept, note, key}` with `check` absent: the outcome search already decides
the guard `false` there. Only the row search steered by Annotate's stored guard fails.

Cause, from the code: `decides` (`crates/verify/ess-conformance/src/synthesize.rs:9687`) returns
`Err(GuardUnevaluable)` on the first `Unknown`, and the candidate loops that check an arranging
creator's input (`selects_branch`, `synthesize.rs:9505`, through `admits_plain`, line 9617) propagate
it with `?`, ending the search. The steered candidates (`hinted`,
`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:4019`) vary `kind` and leave the
`Optional` `check` out. The crate already classifies this cause as one another candidate can fix
(`Reason::ValueAbsent`, `fixable_by_another_candidate`, `crates/verify/ess-conformance/src/decision.rs:257`),
but no search acts on it.

## Acceptance

- `ess verify conform synthesize` on `m2` writes `example.review.Annotate/outcome/not-accepted` and
  `example.review.Annotate/outcome/annotated` with no refusal: the first arranges the row with
  `kind: Erratum` and `check: Verified`, the second with `kind: Accept` (with `check` absent or
  present).
- A candidate whose `Unknown` has any cause other than an `Optional` input the candidate left out
  (a path the input types do not declare, a non-scalar, and the other `Reason`s) still ends the
  search with ESS-SYNTH-002, as `crates/verify/ess-conformance/src/lib.rs:36-58` requires; an
  existing test of that refusal passes unchanged.
- Where every candidate is `Unknown` only for left-out `Optional` inputs, the refusal is the first
  such ESS-SYNTH-002, as today.
- `ess verify conform run --target interpreted` passes the `m2` suite; a suite that did not refuse
  keeps its bytes.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- cited: crates/verify/ess-conformance/src/decision.rs
- cited: crates/verify/ess-conformance/src/lib.rs
- inferred: crates/verify/ess-conformance/tests/arranged_row_beside_absent_optional_guard.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A creating command whose refusals compare an `Optional` input leaves every row of its
   entity unarrangeable for a command with a stored-field guard, so that command's scenarios are
   refused. Reproduction above, brand-free. The requester proposed no syntax; they ask synthesis to
   decide `false and unknown` as false and arrange the row. Their workaround,
   `all: [kind == Erratum, defined(check), check != Verified]`, changes the specification to suit
   the generator.
2. **Class.** Defect. Evaluation is Kleene (`website/docs/reference/predicates.md:46-47`;
   `crates/verify/ess-conformance/src/lib.rs:38`), and the crate names a value the candidate did not
   supply as the one `Unknown` another candidate fixes (`decision.rs:257`); stopping the whole
   search there contradicts that classification. `m1` shows the same guard is decided for the
   outcome scenario.
3. **Already expressible?** The workaround above and `coalesce(check, Reported) != Verified`
   (`predicates.md:448-490`, from `ess/29`) both rewrite a correct guard to suit synthesis; neither
   is needed by the domain, and the reference says `defined(x)` beside the comparison costs a
   witness of each child (`predicates.md:474-477`). Not an answer.
4. **Design fit.** No authored surface. Grid R is unchanged: an `Optional` input is readable in
   `when:` today. The change is internal to candidate searches: a candidate `Unknown` only for an
   `Optional` input it left out is passed over like a refuted one, and the search refuses with that
   cause only when no candidate serves. It applies to every search that calls `decides` over
   creator inputs, not one branch kind. No target, generator, `ess verify diff` class or runtime is
   touched.
5. **Second adopter.** `example.billing.Invoice` is created by `Issue` with
   `Optional<example.billing.Discount>` and a refusal `all: [tier == Gold, discount != Approved]`;
   a `Pay` command guarded `when_subject: {predicate: tier != Basic}` loses its scenarios the same
   way.
6. **Cost.** No format version, keyword or diagnostic. Suites that refused gain scenarios; others
   keep bytes because a candidate that served before still serves first. The crate invariant
   "Unknown stops the search" gets a stated exception for one `Reason`, which the crate doc already
   anticipates (`lib.rs:57-58`).
7. **Alternatives.** (a) Change nothing and document the `defined(...)` workaround: rejected, it
   moves a generator limit into every specification. (b) Always supply every `Optional` input in
   steered candidates: loses the absent case a guard like `not defined(check)` must be witnessed
   with, and changes bytes of suites that pass today. (c) Chosen: pass over a candidate whose
   `Unknown` is only a left-out `Optional`, keep the first such cause for the refusal.

## Decisions

accept, redesigned: the need (arrange the row) is accepted as a defect. The requester framed it as
Kleene evaluation of `false and unknown`; evaluation is already Kleene (`m1`), and the fix is in
the candidate search, which stops at the first candidate the guard is `Unknown` on instead of
trying the next. The misnamed creator in the refusal is split into
`story:an-arrangement-refusal-names-the-cause-of-every-creator-tried`.

## Source

github:beyond10x/ess#728
