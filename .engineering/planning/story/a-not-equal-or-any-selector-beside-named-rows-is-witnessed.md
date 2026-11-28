---
format: aep.planning-md/3
id: story:a-not-equal-or-any-selector-beside-named-rows-is-witnessed
kind: story
status: active
title: A not-equal or any selector beside named rows is witnessed
tags:
- defect
- ess-d368-close
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_d368_u4_pass1.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:41Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:43Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
---
## Finding

Found by wave D368 U4's adversary pass (`review-result:adversary-d368-u4-pass-1`), out of that
story's Acceptance: a row-set selector with no equality conjunct (only `!=`, or `any:`) beside named
rows refuses every branch of the command, the named-row branches included. The refusal is honest
that no scenario can decide the rows, but such a command has no suite at all, and each refusal
calls the branch itself a `row set` without naming the selector.

The gate is `decidable` (`crates/verify/ess-conformance/src/synthesize/row_set.rs:3128-3140`): a
selection not `scoped` by an equality between a String or Uuid field and the input or the subject
refuses.

Reproduction on 0.92.0: the fixture
`crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml` (a `Connect` command with
four named-row branches on `input.from_port` / `input.to_port` and one row-set branch
`source-taken`) synthesizes 7 scenarios and 0 refusals. Change only the `source-taken` selector:

```yaml
      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port != input.from_port      # or {any: [from_port == input.from_port, to_port == input.to_port]}
          exists: true
        error: wiring.board.SourceTaken
```

```text
$ ess specify validate --path ne
wiring v1 — 1 file(s), valid
$ ess verify conform synthesize --path ne --out suite-ne
refused: refusal[ESS-SYNTH-001]: outcome wiring.board.Connect/no-source has no scenario `wiring.board.Connect/outcome/no-source`
  no witness: `wiring.board.Connect/no-source` is `row set`, which selects rows by no equality between a String or Uuid field and the input or the addressed subject, so the rows it counts are not only the scenario's own
... (the same for no-sink, source-not-output, sink-not-input, source-taken, connected)
1 scenario(s) (0 authored), 6 refusal(s), written to suite-ne
```

The `any:` selector gives the same six refusals.

## Acceptance

- For both variants above, `ess verify conform synthesize` writes a passing scenario (against
  `ess verify conform run --target interpreted`) for each `Connect` branch where an arrangement
  exists: at least `no-source`, `no-sink`, `source-not-output` and `sink-not-input`, which a target
  can reach with no `Wire` row stored.
- Each `Connect` branch still refused is refused naming the `source-taken` selector expression
  (`from_port != input.from_port`, or the `any:`), not calling the branch itself a `row set`.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes with no row of
  `synthesis-suite-bytes-table.tsv` changed: the unmodified fixture keeps its 7 scenarios and its
  row, and no other fixture the table lists declares a row-set selector without an equality
  conjunct (the two variants are inputs of `adversary_d368_u4_pass1.rs`, not table rows).
- `cargo test -p ess-conformance --locked --test adversary_map_witness_pass1 adv196_every_example_still_synthesizes_to_its_committed_suite`
  passes with no file under `suites/generated/` changed (no model under `examples/` declares
  `when_related`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/row_set.rs
- cited: crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/tests/adversary_d368_u4_pass1.rs

confidence: medium

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no byte-pin row moves.

## Adopter report, 2026-11-27 (0.99.8): a row set no equality selects

A downstream adopter, on 0.99.8, with the same gate (`decidable`, a selector not `scoped` by an
equality between a `String` or `Uuid` field and the input or the subject) on a shape that has no
named-row branch beside it and no `!=` or `any:` selector. This story's two variants are one case of
it.

Facts from the adopter's specification (read-only probes):

- An outcome whose guard reads a row set that no input equality selects (rows of another entity
  counted or tested by their own fields and literals, for example a cap on the rows in one state) is
  refused `ESS-SYNTH-001`, "selects rows by no equality between a String or Uuid field and the input
  or the addressed subject", on that outcome and on every transition and state scenario that sends the
  command.
- The workaround the refusal's wording points at, selecting the rows by an input equality, makes the
  refusal go and moves the failure: synthesis then fails to arrange the other entity's rows (the
  scenarios whose setup needs them), and the whole suite falls from 563 to 335 scenarios. The cause
  of that second loss is not investigated; the adopter's model has to be reduced before it is.

Reproduced on 0.99.8 with a minimal model: one entity with a lifecycle and two `String` fields, a
command whose branch reads `when_related` over that entity with `where: {all: [batch_id == "fixed"]}`
(a literal only) and `count: {gt: 0}`. `ess verify conform synthesize` refuses `ESS-SYNTH-001` on the
guarded outcome, on the accepting outcome, and on the transitions the command moves. The same model
with `batch_id != "x"` gives the same refusals. The 563-to-335 effect was not reproduced (it needs the
adopter's other-entity arrangement).

Fit, in the words of `.agents/skills/assessing-external-requests/SKILL.md`:

1. Need: a guard over a row set that the specification selects by its own fields, not by the request,
   gets no scenario at all, so the command has no suite.
2. Class: gap with a documented limit (`website/docs/reference/predicates.md`: "A selector needs an
   equality ... so a scenario counts only its own rows"). A scenario cannot own rows it does not
   select, so a count over every row of the entity is not decided by the scenario's arrangement.
3. Already expressible: partly. An input equality scopes the selector but changes the command's
   input, and per the second fact it moves the loss elsewhere.
4. Fit: generic with this story: one gate, `decidable`, for every selector with no scoping equality.
   Not a second story; the not-equal and `any:` variants are cases.
5. Second adopter: this one; the first finding was an adversary pass's (`review-result:adversary-d368-u4-pass-1`).
6. Cost: either synthesis owns the whole row set of the entity in the scenario (arranging every row it
   counts, with its decoys), or the refusal names the selector expression and the repair. The second is
   already this story's acceptance; the first is the new question.

Acceptance added by this report:

- A guard whose selector holds only literals and `!=` (no equality with the input or the subject) on
  an entity no other row of the run touches is witnessed: the scenario arranges the counted rows
  itself and the interpreted target passes it, or the branch is refused naming the selector
  expression, as the acceptance above says. Test: a fixture with the literal-only selector.
- The suite of a model that selects by an input equality instead keeps every scenario whose setup
  needs the other entity's rows; the failing arrangement is reduced from the adopter's model and gets
  its own test before this story is closed.

CHANGELOG: checked 2026-11-27 on `main` (`ff5c4713e`), sections 0.99.3 to 0.99.8 and `[Unreleased]`: no entry
witnesses a selector without a scoping equality.

## Repro, 2026-11-27 (0.99.8): the row-set refusal and the 563 to 335 drop

Reproduced on 0.99.8 with two minimal models, committed beside this story under
`.engineering/repro/a-not-equal-or-any-selector-beside-named-rows-is-witnessed/` (each with its `README.md`,
the commands, and `out/` holding the observed output, the suite and the scenario ids). Both are one catalog
of three entities (a kind, a variant of a kind, an instance an install creates) and two commands.

- `A1-rowset-no-input-equality/`: a creating outcome behind a row-set refusal whose `where:` holds no
  equality with an input (the input is reached only through an `exists:` quantifier over the variant rows).
  `ess verify conform synthesize` writes 12 scenarios and 3 refusals, all `ESS-SYNTH-001` "selects rows by no
  equality between a String or Uuid field and the input or the addressed subject", on the three outcomes of
  the command, including `no-such-variant`, which is a plain `when_related: {via: input.variant_id, exists:
  false}` existence check and is still called a row set. The same model without the row-set outcome writes
  14 scenarios and 0 refusals, so the one row-set outcome costs every outcome of its command.
  The refusal's help line ("give the field a type that has a finite value, or drop it from the command's
  input") does not match the cause: the cause is the selector, not an input's type, so the suggested fix
  does nothing here. The need is that the accepting and the existence-check outcomes get a scenario.
- `A2-rowset-input-equality/`: the same model with the row set rewritten to select by an input equality
  (the workaround the refusal's wording points at). It writes 9 scenarios and 6 refusals: the same three
  outcomes of the install command, plus three scenarios of the second command (its accepting outcome, one
  state refusal and one transition) refused `ESS-SYNTH-001` "its row set is Unknown": synthesis no longer
  sends the install command to arrange the instance the second command moves, because the arranging send
  now has an undecided row set. This is the likely mechanism of the drop from 563 to 335 scenarios that the report
  above could not reproduce: it needs the downstream command. Without that second command, A1 and A2 both
  write 8 scenarios and refuse the same three outcomes, so the loss shows only where a second command needs the
  row the install command creates: the arranging send of a command whose row set is Unknown is dropped.

So the acceptance above gains: an outcome whose row set is not selected by an input equality is refused
for itself only, naming its selector (not an input type), and a command that creates the row another command
moves is still sent to arrange that row when only a branch it does not need is undecided; test: the A2 model
with its second command's three scenarios present.
