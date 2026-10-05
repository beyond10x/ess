---
title: Synthesize a suite
sidebar_position: 1
description: Generate the deterministic semantic suite a specification requires.
---

# Synthesize a suite

## Generate the suite

```shell-session
$ ess verify conform synthesize \
    --path examples/billing \
    --out target/billing-suite.json
```

The suite is deterministic. Its provenance names the model and contract digests from which it was
derived.

A generated scenario pins what the specification declares, not only that a command answered:

| declaration | what the suite requires |
| --- | --- |
| a transition `B → C`, where a view projects `state` | the row is read back in `C` |
| `from: [A, B]` on a transition | the move runs from `A` and from `B`, each on its own instance |
| an update's `sets:` | each field is sent a value the row does not already hold: not what the setup wrote, and not an enum's first variant where nothing wrote it |
| `when: n >= 1` over an input | the branch is taken at `n: 1`, and its default refuses at `n: 0` with every other conjunct satisfied |

Boundaries are probed for comparisons of a number or a timestamp with a literal. Text and equality
comparisons are not. The extra checks reuse the existing steps and scenario ids, so the suite format
does not change.

Add `--compact` to fresh `synthesize` output to write deterministic JSON without
indentation, followed by one newline. The decoded suite is unchanged; its exact
byte digest changes, so retain that original compact file when producing reports
or selecting child suites. Pretty output remains the default. This option does
not rewrite an already committed suite.

## Explicit synthesis seeds

Some valid states no bounded arrangement of commands reaches: a compare-and-swap revision counter
at `9223372036854775807`, or one revision below it. Synthesis refuses those obligations with
`ESS-SYNTH-003` rather than walking a counter a quintillion times. An explicit seed supplies the
initial row instead:

```shell-session
$ ess verify conform synthesize --path model \
    --synthesis-seed seeds/max.yaml at-max \
    --synthesis-seed seeds/below.yaml below-max \
    --out suite.json
```

Each `--synthesis-seed FILE INSTANCE` names one authored `ess-scenario/2` document and one of its
arrangements that has a `setup`:

```yaml
type: ess-scenario/2
domain: counter.model
scenario: counter-at-max
summary: A counter held at its signed maximum.
arrange:
  - instance: at-max
    entity: counter.model.Counter
    setup:
      identity: 00000000-0000-4000-8000-00000000a001
      fields: {revision: 9223372036854775807}
      state: Active
assert:
  - view: counter.model.Counters
    contains: {id: {$instance: at-max}, revision: 9223372036854775807}
```

The whole document is compiled and validated against the model, but **a seed supplies only the
nominated initial setup row**. The document's timeline, its assertions and any state its timeline
would reach are never used, and the document adds no authored scenario; pass it with `--scenarios`
as well if you also want it run as one.

Ordinary arrangement is always tried first. A seed row is offered only where a generated
obligation reading the stored row was left unmet, and only to the exact obligation: the scenario
establishes the row, observes it, sends the real command with the input grounded from the row (a
compare-and-swap's expected revision is the row's own), and asserts the outcome the guards select
there, with its error, events and resulting state. A successful ordinary witness is never replaced,
and a seed that answers no unmet obligation adds nothing. A row that needs an owner or a related row,
or whose identity the scenario already uses, is not applied, and the refusal says so.

A request with seeds writes suite `ess-conformance/42` (`/43` with `--suite-format 5`), or `/44`
(`/45`) where an act also claims one event more than once, and records in its provenance which
files were read, the exact rows admitted, and where each row was used. A target needs the entity
setup capability to run those scenarios. Without `--synthesis-seed` the
suite is what it was, except that a guard over an integer beyond 2^53 that synthesis used to refuse
is now witnessed exactly. The binding contract is
[`docs/design/synthesis-seeds.md`](https://github.com/beyond10x/ess/blob/main/docs/design/synthesis-seeds.md).
