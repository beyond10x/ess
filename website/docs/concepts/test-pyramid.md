---
title: A suite at every level of a test pyramid
sidebar_position: 4
description: Run one conformance suite as a component, integration or end-to-end test, and decide whether to commit or fetch the generated runner.
---

# A suite at every level of a test pyramid

Three questions come up once a team holds an implementation to its suite:

- How do we run the same suite at different levels of a test pyramid?
- Can we run scenarios with some parts of the system real and others replaced?
- Should the generated runner be committed, or fetched when the tests run?

They have one answer: **the level is a property of the target, never of the suite.** The suite a
specification obliges is the same document at every level. What changes is the target: the code
that answers the runner's questions by driving an implementation.

## One suite, many targets

A target reports what it observed, and the runner decides whether the specification is satisfied
([Runners and reports](../guides/verify/runners.md#hold-your-own-implementation-to-the-suite)). An
in-memory target calls functions in the same process. An end-to-end target sends HTTP requests to a
deployed system and reads its event log. Both answer the same questions, so both run the same
suite.

Time shows the pattern. A scenario that states a duration asks the target how much time passed.
An end-to-end target waits for real; an in-memory target advances a clock it owns and returns at
once. Both answer the question truthfully, and neither changes the suite.

## There is no skip

Every scenario in a suite is required. The suite format has no way to mark one optional.

A target that cannot answer a question says so: it returns `ErrUnsupported` in Go, throws
`ErrUnsupported` in TypeScript, or returns `TargetError::Unsupported` in Rust. That is an honest
answer, and the run still does not pass:

- The Rust runner (`ess verify conform run`) fails the run.
- The generated Go and TypeScript runners report the scenario as skipped and the run as
  `inconclusive`, not `passed`. The language's own test command can still exit 0, so a pipeline
  must read the report's status rather than the exit code. With `ESS_REPORT_FORMAT=2` the report
  also counts passed, failed and skipped scenarios separately
  ([explicit outcome counts](../guides/verify/runners.md#opt-into-explicit-outcome-counts)).

So a lighter target does not earn a lighter verdict. To run at a lower level, **narrow the suite**
to what that level can answer:

```shell-session
$ ess verify conform synthesize --path examples/billing --component invoice-service \
    --out target/invoice-service-suite.json
26 scenario(s) for `invoice-service` (0 authored), 6 outside it, 0 refusal(s), written to target/invoice-service-suite.json
$ ess verify conform select --suite target/coverage-suite.json --ids selected-ids.json \
    --out target/selected.json
```

`--component` keeps only the scenarios whose every command, event and view that component accepts,
publishes or owns, and lists the rest. `select` narrows a coverage suite to explicit scenario IDs and
keeps its parent, so a report still says what was left out
([Author scenarios](../guides/verify/author-scenarios.md#run-a-chosen-subset-of-a-coverage-suite)).
Do not hand a partial target the whole suite and read past the red scenarios.

## What each level owes

A target's level is decided by which questions it can answer. The interface is additive: a few
methods are always required, and each further one is owed only by the scenarios that reach it.

| Capability | Owed when |
|---|---|
| Run a command, read a view, observe events | always |
| Force an external outcome | the specification declares an outcome `external:` |
| Deliver an event again | a binding declares `delivery: at_least_once` |
| Report what a binding invoked | a scenario checks a binding's mapping |
| Report instants and elapsed windows | a scenario states a duration |
| Ordered reads that can halt | a view declares an order |
| Establish backend state | an authored scenario arranges entity state |

A target that answers the basics and owns a clock is a component-level target: narrow its suite
to match. A target that drives a deployed system answers more and runs a wider suite. Neither uses
a different suite document, and neither passes by tolerating failures.

## Replace parts of the system by linking, not by a flag

There is no flag that replaces one component with a stand-in. `--component` narrows the suite; it
does not change what `ess generate synthesize` writes.

Replacement happens in your own code. The Rust workspace that `ess generate synthesize --target
rust` writes has a `System` type with one type parameter per component, and each domain module has
an `obligations::Unimplemented` type that refuses every command. "Component A real, B refusing" is a
call to `System::new` with A's implementation and B's `Unimplemented`, and it compiles. Every
command of B answers with a typed refusal naming what it still owes.

ESS never chooses between two implementations of one obligation: there is no priority, default or
"first wins". Exactly one implementation per obligation is the only accepted state. A test level is
assembled by the code that links it, not configured.

## Do not translate scenarios into another test framework

A scenario cannot be turned into a test written in another framework's own language. That
framework's assertions would decide whether the scenario passed, and the suite would stop checking
the implementation.

If you already have a test harness that runs your system, drives it and observes it, make it a
target. It already has what a target needs. The one thing it gives up is deciding the verdict.

## Commit the generated runner

`ess verify conform synthesize --target go` or `--target typescript` writes the runner, the
evaluator and the suite into your repository as one package. Commit it:

- The runner is in version control, so you can search it, diff it and step through it in a
  debugger when a scenario fails for a reason the report does not explain.
- No network access is needed when the tests run.
- `ess verify conform synthesize` regenerates it after every change to the specification; files you
  add beside it, such as your test file, are kept.

The cost is one copy per repository: a runner fix reaches you when you regenerate with a newer `ess`.

The alternatives are not available today:

| Route | Today |
|---|---|
| Import the runner as a library | The `ess-conformance` crate is not published. A Rust target can implement its `ConformanceTarget` trait only from a checkout of this repository. |
| Run a target in another process over a wire protocol | Not supported. `ess verify conform run` reaches only the targets built into `ess`. |
| Run the specification itself (`--target interpreted`) | Runs a command's outcomes, transitions, `sets:`, events and invariants from the model; views, bindings, time, redelivery and established entities come back `unsupported`. It checks a specification, not your implementation. |
