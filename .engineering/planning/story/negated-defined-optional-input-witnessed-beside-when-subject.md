---
format: aep.planning-md/3
id: story:negated-defined-optional-input-witnessed-beside-when-subject
kind: story
status: draft
title: 'Synthesis witnesses when: not defined(<optional input>) on a command whose when_subject reads that input'
relations:
- serves: vision:O2
revision: 2
---
## Outcome

A branch guarded by `when: not (defined(<optional input>))` on a command whose `when_subject:`
reads the same optional input gets a synthesized witness, so `ess verify conform mutate` can kill
its mutants. Today synthesis refuses it as `ESS-SYNTH-003` (no witness). Reported by an adopter on
2026-10-08: a goal-satisfaction command whose `stale-revision` branch is guarded that way.

## Fit review

1. **Need.** "A request that omits the expected revision is refused as stale" while the subject row
   is selected through the same optional field. The witness must send the input absent and still
   arrange the subject the other branches need.
2. **Class.** Synthesis coverage of a construct the language already accepts; no new syntax.
3. **Already expressible?** Yes in the specification; only the witness is missing.
4. **Fit.** The `defined` decision added to `mutate.rs` `satisfiable` for
   https://github.com/beyond10x/ess/issues/501 is the presence reasoning this needs; the witness
   lives in synthesis (`synthesize.rs` and `synthesize/subject_fact.rs`), which choose input values
   for `when_subject` rows. Inferred, to be confirmed by the reproduction.
5. **Second adopter.** An update command refusing `missing-etag` when the optional `if_match`
   input is absent, while `when_subject` selects the row by an input the same command carries.
6. **Cost.** Synthesis only; committed suites with such a guard gain scenarios. No format change.
7. **Alternatives.** (a) Change nothing: the branch stays unwitnessed. (b) This story.

## Decisions

- Accept. Starts after https://github.com/beyond10x/ess/issues/501 is on `main` (it shares
  `mutate.rs`); ships in the minor after 0.58.0's current order.

## Acceptance

- A minimal reproduction under `.engineering/repro/` (brand-free domain) shows `ESS-SYNTH-003`
  on the release this starts from, written first.
- After the change, synthesis writes a scenario that sends the optional input absent and observes
  the guarded refusal; `ess verify conform mutate` kills the branch's `guard` mutants.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`, `src/synthesize/subject_fact.rs` (inferred).
- `crates/verify/ess-conformance/tests/` (new test).
- Held files: none at the time of writing.
