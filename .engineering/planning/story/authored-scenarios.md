---
format: aep.planning-md/1
id: story:authored-scenarios
kind: story
status: draft
title: A specification carries the scenarios an author wrote, not only the ones it obliges
revision: 1
---
## What is missing

A specification can generate the scenarios it obliges. It cannot carry the scenarios an author
wrote, and for the same model those are different claims.

`ess verify conform` offers two verbs, measured against 0.13.1:

```console
$ ess verify conform --help
  synthesize  Generate the suite the specification obliges
  run         Run a generated or committed suite against a built-in reference implementation
```

Neither reads a scenario anybody wrote.

The **suite format already carries them**. `ConformanceSuite::insert` takes any scenario, and this
crate's own tests use it — `crates/ess-cli/tests/go_conformance.rs:382` inserts two hand-written
checks beside the emitted ones and runs them on the emitted runner unchanged. What is missing is an
authoring surface: a file, checked against the model, compiled into the same `ess-conformance/2`
suite.

## Why the generator cannot cover it

Synthesis says so itself. On the ACD model in `sbf/acd`, `ess synthesize` reports 73 capabilities —
54 generated, 16 obligations, **3 refused** — and one refusal reads:

> the contract is declared; the algorithm is not

The router's matching order is not derivable from the model and never will be. An authored scenario
is the instrument for pinning an algorithm a model cannot generate, which is exactly the case a
specification language should have an answer for and currently does not.

## The adopter this comes from

`sbf/acd` and `acd/acd-rs` hold three behavioural oracles between them, none of them generated, all
of them synced by hand:

| oracle | Go | Rust | kept equal by |
| --- | --- | --- | --- |
| routing scenarios | `scenarios/*.yaml`, 11 files | the same 11 | **byte-identical copies** |
| slotmatcher conformance | 2,530 lines | 822 lines, *"ported from the Go"* | **a hand re-expression** |

The second is the one to look at. A copy diffs; a re-expression does not, and nothing checks that
the two still say the same thing.

## What it has to do

- **Refuse at compile time what the model does not declare.** A scenario naming a command, state,
  field or view that is not in the model is refused by name, the way everything else here is. This
  is the whole value over a bespoke runner: today a scenario naming a field that no longer exists
  fails at runtime, in each consumer independently.
- **Compile into the same suite.** Authored and generated scenarios come out of one `synthesize`,
  carry ids of the shapes `ScenarioId` already has, and run on the existing runners with no change
  to `ConformanceTarget`.
- **Be distinguishable in the report.** A reader has to be able to tell what the specification
  obliged from what a person asserted, because the second is only as good as the person.

## Acceptance

- A scenario file naming an undeclared command is refused, naming the command and the file.
- A valid one appears in `ess verify conform synthesize` output beside the generated scenarios and
  runs on the emitted Go runner with no change to the runtime.
- `ess-conformance-report/1` distinguishes the two populations.
- The billing example carries at least one authored scenario, so the feature is pinned by the same
  example everything else here is.
