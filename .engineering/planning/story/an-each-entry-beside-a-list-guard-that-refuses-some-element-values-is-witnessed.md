---
format: aep.planning-md/3
id: story:an-each-entry-beside-a-list-guard-that-refuses-some-element-values-is-witnessed
kind: story
status: draft
title: 'An each: entry beside a list guard that refuses some element values is witnessed'
summary: 'The four elements an each: scenario sends are not steered by the branch''s list guards, so one guard that refuses an element kind with a field set leaves the outcome refused (ESS-SYNTH-001, no four elements)'
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- informed_by: story:a-state-only-an-each-move-reaches-is-arranged-for-the-refusals-of-the-entitys-other-commands
revision: 1
---
## Outcome

An `affects:` entry with `each:` over a command's list input does not leave its outcome unwitnessed because the
command also has a list guard that refuses some element values. The four elements the witness sends are chosen so that
the branch's own list guards pass.

## Origin

A downstream product specification, reduced on 0.99.9 by its owner. With every row-set guard of one command removed,
the success outcome of two commands (a preview and a release of a versioned record, each writing its listed entries
with `affects:` and `each:`) is still refused `ESS-SYNTH-001`; in the reduced model one list guard of the command
is enough to cause it. That guard excludes an entry that is not of the tiered kind and has an optional field defined.
Deleting all seven list guards of the adopter's reduced cut clears the refusal. The adopter's specification is not in the store.

## Reproduction

Run with the released `ess` 0.99.9 on models written for this story, neutral nouns (`shelf.stock`), kept in
`.engineering/repro/an-each-entry-beside-a-list-guard-that-refuses-some-element-values-is-witnessed/`, each with its
`README.md` and observed output. One entity with a draft-to-staged lifecycle, one entity of entries that the staging
command writes with `affects:` and `each:` over its list input `entries`, a `distinct` guard on the entries' identity,
and no row-set guard.

| model | list guard added before `no-such-release` | scenarios | refusals |
|---|---|---|---|
| none | none | 6 | 0 |
| L1 | `exists e in entries: e.kind != tiered and defined(e.unit)` | 6 | 1 |
| L2 | `exists e in entries: e.kind == tiered and not defined(e.unit)` | 7 | 0 |

L1 refuses the success outcome `ESS-SYNTH-001`: "`staged.affects[0]` is `entries`, which offers no four elements that
keep three identities apart, change every member the entry reads on a held row, and reach the branch". L1's list
guard has its own scenario, with a note that its list is sent without spreading. L2, a guard on the other corner of the
same two fields, leaves the outcome witnessed.

A witness exists by hand for L1 (not run): a held row that is tiered with a unit defined; a second call whose element is
not tiered, has no unit, and has another name, so it changes every member the entry reads and does not take the guard.

Cause, read at `main` (`fd6049733`): `each_lists` in `crates/verify/ess-conformance/src/synthesize/set_effects.rs`
builds the four elements (the held row, a second-call element for it, a decoy and a new one) from
`witness::fields` at consecutive distinctions, for `EACH_TRIES` attempts, and keeps an attempt only when both lists
reach the branch (`selects_branch`). Nothing steers an element's values by the branch's list guards. An element whose
kind differs from the held row's is not tiered, and (inferred from the refusal; the choice of the optional field was not read) has its optional field set, so every attempt
sends an element the guard refuses.

The refusal is the third in a chain on the adopter's model, found by deleting what came before: the "no equality"
refusal of the guard (`story:a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed`), then the
each-move refusal (`story:a-state-only-an-each-move-reaches-is-arranged-for-the-refusals-of-the-entitys-other-commands`),
then this. In the adopter's whole specification on 0.99.9 (493 scenarios, 673 refusals) the text does not occur: the
earlier two come first.

## Fit review

Consumer: a downstream product specification.

1. **Need.** An outcome with an each-entry is witnessed on a list that passes the command's own list guards. The
   adopter proposed nothing; its note is that the elements the witness builds ignore the guards it also declares.
2. **Class.** Defect against this story's own sibling (`story:each-entry-moves-the-rows-its-elements-name` and
   `docs/design/set-effects-over-filtered-instances.md`, which name the lists an each-scenario sends) in a shape those
   do not hold: a list guard that excludes values the elements' fields can take.
3. **Already expressible.** Partly. The adopter can delete the guard or move it into the outcome that checks it, which
   moves a checked rule out of the command, so it is not an answer.
4. **Fit.** No new key and no new code. The element search learns the list guards the branch passes, as the
   arrangement of rows learns the guards a creator holds. Siblings: a `forall` list guard and a guard over a member
   of an element (`e.x.y`) have the same question; the change opens the whole row of list guards or none.
5. **Second adopter.** Any command that registers each listed line and refuses a line with an inconsistent pair of
   fields (a unit on a flat price, a currency on a free item).
6. **Cost.** No format bump. Synthesis output changes only for models with this pair: more scenarios.
7. **Alternatives.** (a) Change nothing: the refusal names the list guard that rejected every attempt, so the
   adopter can see it. (b) Raise the number of attempts: no steering, a model needing a specific pair is still
   missed. (c) Steer the elements by the list guards the branch passes.

## Decisions

LG-D1, how the refusal is handled (2026-11-27; recommended: C; no answer yet).

- A: change nothing; document that the elements are not steered.
- B: name, in the refusal, the list guard every attempt took, and fix nothing else. Cheap and honest; the outcome stays
  unwitnessed.
- C (recommended): steer the four elements by the list guards the branch passes (a refuting `exists`, a passing
  `forall`), and name the guard in the refusal where no element can pass it. The guard is the adopter's own and
  declared in the command; the witness should honour it.

## Acceptance

- `an_each_entry_beside_a_list_guard_that_refuses_some_element_values_is_witnessed`, in `ess-conformance`, on model
  `L1`: the success outcome has a scenario, 0 refusals, the interpreted target passes the suite, and the scenario's
  two lists hold no element the list guard refuses. The suite of `L2` and of the model with no list guard keeps its
  bytes (7 scenarios, 6 scenarios).
- A target that ignores the each-entry fails a scenario of the `L1` suite.
- A model whose list guard no element can pass is refused naming the guard (a second fixture).
- Test runs on a model synthesized once for the whole test (under 60 s); the suite-bytes pin targets pass with no row
  moved.

## Scope

- `crates/verify/ess-conformance/src/synthesize/set_effects.rs`: `each_lists`, `EACH_TRIES` (read at `main`).
- `crates/verify/ess-conformance/tests/`: one test and the fixture `L1`.
- `CHANGELOG.md`: one `### Fixed` bullet under Unreleased.

CHANGELOG: checked 2026-11-27 on `main` (`fd6049733`), `[Unreleased]` and 0.99.1 to 0.99.8: no entry steers an each-entry's
elements by a list guard.
