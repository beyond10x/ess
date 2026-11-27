---
format: aep.planning-md/3
id: story:sets-retarget-mutants-get-a-separating-witness
kind: story
status: implemented
title: 'sets-retarget: with three boolean inputs, the mutant''s suite still witnesses the swapped pair with one value (follow-up to #161)'
refs:
- provider: github
  reference: beyond10x/ess#202
relations:
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T01:53:24Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
# Story: sets-retarget: with three boolean inputs, the mutant's suite still witnesses the swapped pair with one value (follow-up to #161)

## Why

beyond10x/ess#202. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #202 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The defect is still there on this tree (`e932781965`, release 0.40.0). The fix is small: one function in synthesis, plus one design choice about which pairs to separate.

| # | Finding | Basis |
|---|---|---|
| 1 | The mutant generator pairs each `sets:` input with the **first** input of the same type (`.find(...)`). That gives `paid→gift`, `gift→paid` and `rush→paid`, which matches the issue. | cited: `crates/verify/ess-conformance/src/mutate.rs:636-653` |
| 2 | The #161 fix is `distinguished()`. It walks `outcome.sets` in order and, when a source and a sibling of the same type hold one value, moves the **sibling first**. | cited: `synthesize.rs:5363-5421` (loop `'moved: for moving in [sibling, field]` at :5395) |
| 3 | Root cause: `distinguished()` never checks whether a move undoes a pair it already separated. In the `gift` mutant the model's sets are `paid: input.paid, gift: input.paid, rush: input.rush`. So `input.gift` is only ever a sibling, and a later `rush` step that flips `paid` can put `paid == gift` back. With 3 inputs of a 2-valued type, no single witness separates every pair anyway. | cited: code path; inferred: the exact order of moves (I did not run it) |
| 4 | Only one witness is made per call. `freshened()` calls `distinguished()` at :5306, and `freshened()` has 4 callers: :2074, :2109, :4745, :7147. Nothing adds a second witness when separation fails. | cited |
| 5 | Nothing on the branch fixes this. The last related commit is `dda521887a` (#161). | cited: `git log` |
| 6 | Fix location: `distinguished()`, and `equals_a_sibling()` (:5347) if it has to respect pinned pairs. Possibly also the witness loop in the `freshened` callers, for a second scenario. | inferred |
| 7 | The adopter case with an **event** writing 3 booleans probably goes through the same `InputField` path, as an event-triggered command input. | inferred, not traced |

Design decision the implementor must make (I recommend option a):
- **a) Target-name priority.** In a mutated model, a `sets` entry whose target `T` names an input of the same type that is not its source (`gift: input.paid` while an input `gift` exists) marks the pair (source, `input.T`). Separate that pair first and pin both fields so later moves cannot collapse them. This needs no mutant provenance in synthesis. It relies on target and input sharing a name, which is true of every retarget the generator emits only when names line up.
- **b) Pass the mutated pair explicitly** from the `mutate.rs` mutant into synthesis, and separate exactly that pair. This is exact, but it adds an API seam through `--emit`.
- **c) Issue's second option.** Add a second witness per outcome whenever the number of same-typed `sets` sources exceeds the type's cardinality. It is general, costs more scenarios in every suite, and changes baseline counts (the `mutation_audit.rs` expectations).
- Whichever is chosen: pinned separations must never be undone. That is a guard inside the `'moved` loop.

Collisions with the other issues:
- **#196** (a Map input is always witnessed as `{}`, so the `sets:` assertion passes): same neighbourhood, the `sets` witness and `candidates`/`freshened`. Likely conflict. inferred
- **#203** (mutate: a suite that gains synthesis refusals is scored survived) and **#210** (`--collect` refuses to score when the baseline has skipped scenarios): mutate scoring and `mutation_audit.rs` expectations. Low to medium. inferred
- **#198, #199, #204, #209, #193**: same 9096-line `synthesize.rs` but other functions (when_subject, arrangement, view filters). Textual merge risk only. inferred
- **#195, #201, #205, #211**: likely other crates or the model layer. No overlap expected. inferred

Confidence: high on the code path and on it not being fixed (read, not executed); medium on the move order that produces the reported witness.

Paths:
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/mutate.rs (only if option b)
- crates/verify/ess-conformance/tests/connective_and_source_mutants.rs (new case, inferred)
- crates/verify/ess-conformance/tests/mutation_audit.rs (counts, inferred)

Verdict: open

## Sibling inputs that carry one literal (downstream report, 2026-11-27)

## Sibling inputs that carry one literal (downstream report, 2026-11-27)

A downstream hardening pass found that a swap of two sibling inputs passes its whole suite. The rule above separates
same-typed siblings that a `sets:` entry reads (`distinguished`, `source_pairs`, beyond10x/ess#161 and #202); the report
shows shapes it did not reach.

### Need

On 0.99.2, and on `main` at 0.99.8 (`CHANGELOG.md` 0.99.3 to 0.99.8 name no change to how synthesis chooses the
literals of sibling inputs; the 0.99.x `example:` entries decide which leaf keeps an authored example, not whether two
inputs differ), the downstream measured, on the part of its specification that validates alone:

- Inputs of one type carry one literal in a scenario: two `Integer` inputs of one command both `2`, two `Boolean`
  inputs both `false`, a minimum and a maximum size both `1`.
- An update writes the value the row already stores (a policy set to the value it holds).
- 6 `sets-retarget` mutants and 1 `sets-drop` mutant of those commands are `unwitnessed` for it (the mutant's suite
  equals the baseline's), and `value: input.grace_seconds` written in place of `value: input.<own input>` in one of the
  commands passes the baseline suite, 266 of 266, exit 0. The rule is not pinned by any scenario.

What is established on 0.99.2 here: for a command with two `Integer` inputs each read by its own `sets:` entry into a
stored field, with a view that reads both fields, the baseline suite sends different values (1 and 2) and both
`sets-retarget` mutants are killed. So the plain shape is already separated, and the report's cases differ from it. The
cause is not found; readings, each inferred and none checked: the two inputs have different declared type names over one
representation (`type_ref` differs, so `source_pairs` does not pair them); one of them is read only by a guard, a
payload, an invariant or a response and not by a `sets:` entry; a type invariant ties the two (a minimum no larger than
a maximum) and the search settled on equal values; or a two-valued type (`Boolean`) leaves no room for three inputs, the
case #202 accepted. The first step of the story is the minimal pair for each, red on the released binary, and the
shape that does not reproduce is dropped from the acceptance.

### Fit review

1. **Need.** A suite in which a swap of two inputs of one command, read into stored fields or payloads, fails a
   scenario. The consumer is "a downstream control plane".
2. **Class.** Gap in value selection: equal literals cannot tell two fields apart, and a suite that cannot is
   weaker than the model it was synthesized from.
3. **Already expressible?** An authored scenario with distinct values pins one command (`--scenarios`); that is a
   scenario to write by hand for every command with two siblings, which is the work synthesis does for the shapes it
   covers. An input's `example:` sets a literal per input and does not relate two inputs.
4. **Fit.** Extends the separation rule of #161 and #202 to the pairs it leaves out; no new construct, no format
   change. Synthesis output of an existing model moves.
5. **Second adopter.** Any command with two inputs of one type read into different fields, which is common (a minimum
   and a maximum, a source and a target, a start and an end).
6. **Cost.** A pinned suite of an affected model moves when it is synthesized again, like the 0.99.x `example:`
   entries; `synthesis_suite_bytes_table` and the byte-pin targets of `AGENTS.md` move, and the time of a synthesis is
   timed before and after.
7. **Alternatives.** (a) Change nothing; the downstream authors a scenario per command. (b) Separate more pairs
   (chosen, D1). (c) Vary every input of every scenario: larger suites for pairs no output reads.
8. **Generic, not per finding.** One rule over "two inputs of one command that an outcome reads into the row, a payload
   or a response" covers the integer, boolean, bounds and already-stored shapes; it is not one fix per command.

### Decisions

- **D1. Which pairs of inputs are sent apart?** (a) Every pair of inputs of one command with one wire type. (b) The pairs
  an outcome reads into a stored field, a payload or a response, whether or not a `sets:` entry reads them, in the
  priority order `source_pairs` already uses. **Recommended: (b).** A pair no output reads cannot be told apart by any
  observer, so (a) grows the suite for nothing; the pair a guard reads alone is the one most worth checking in the
  first reproduction.
- **D2. Where one witness cannot separate every pair (three `Boolean` inputs, or a type invariant that orders two
  inputs)?** (a) Keep #202's priority order and leave the rest joined. (b) Add a further scenario for the pairs the
  first leaves joined. **Recommended: (b),** because a scenario for the pairs left over is the only way a swap among
  them fails a test; it adds scenarios only for commands with such pairs, and the suite size is measured.
- **D3. Two inputs whose declared types differ over one representation?** (a) Pair them like same-typed siblings: a
  swap type-checks on the wire. (b) Do not pair them. **Recommended: (a);** a target that reads one into the other's
  field passes the same way.
- **D4. An update that writes the stored value?** Recommended: covered by the same story where the reproduction shows
  `freshened` leaving it, and not before.

### Acceptance

Each runs in a test that synthesizes a small model once and not per case, under the 60 s bound; each is red on 0.99.2.

- A command with two inputs of one declared type, each read into its own stored field, passes the baseline suite and
  fails it when `sets:` reads the other input, for the shape that reproduces first.
- The same for an input a guard alone reads beside an input a `sets:` entry reads, and for a payload or response read.
- A minimum and a maximum under an invariant that orders them are sent with different values, or the suite says why one
  scenario cannot.
- A `sets-retarget` and a `sets-drop` mutant of those commands are `killed` by `mutate --target interpreted`, not
  `unwitnessed`.
- An update that writes the value a row already stores is sent a different value where the type has one.
- A suite of a model with no such pair is byte-identical to the previous release; a moved pin names its cause.

## A reproduction in 55 lines, 2026-11-27

One of the shapes above is reproduced on ess 0.99.2 in a specification of 55 lines
(`.engineering/repro/sets-retarget-mutants-get-a-separating-witness/`). One command has an `Integer` input `a`, an
`Integer` input `b` and a `String` identity input; its creating outcome sets stored field `a` from `input.a` and `b`
from `input.b`; a view reads both fields.

- **Trigger.** A first outcome that refuses with an `exists` row refusal on the command's own entity
  (`when_related` with `exists: true`, matched on the identity). With it, synthesis gives every sibling `Integer` of the
  command one value in each scenario (1 and 1, 2 and 2, 3 and 3, 4 and 4).
- **Red.** `ess verify conform mutate --path . --target interpreted --class sets-retarget` from the repro directory
  answers 2 mutants, 2 unwitnessed (`ESS-MUTATE-004`, the mutant's suite equals the baseline), exit 3.
- **Without the trigger.** With the `exists` outcome removed the same command is sent 1 and 2, and both mutants are
  killed. This is the plain shape the section above found already separated.
- **Not needed for it.** `Integer` constraints, `Boolean` inputs, a second command, lifecycle transitions, a newtype
  identity. A model of one command, one entity and one view is enough.

This narrows the readings above: none of the type, the guard-only read or the invariant is needed, and the one thing
the reproduction adds is the refusal on the command's own entity. The cause is still not found. The first step of the
story is a test on this model, red on 0.99.2 (checked there) and green when the two `Integer` inputs differ in every
scenario the command is sent, with both mutants killed; a check on `main` at 0.99.8 is still to be run.
