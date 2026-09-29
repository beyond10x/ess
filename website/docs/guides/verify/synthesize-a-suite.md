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
