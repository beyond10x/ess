---
title: Verify conformance
sidebar_position: 3
description: Generate the semantic suite a specification requires, run it, and emit a standalone ESS conformance report.
---

# Verify conformance

A specification declares more than an interface shape. Command outcomes, refused branches, state
transitions, emitted events, and invariants become semantic scenarios an implementation can be
checked against.

## Pages in this guide

1. [Synthesize a suite](verify/synthesize-a-suite.md) — generate the suite a specification requires.
2. [Author scenarios](verify/author-scenarios.md) — add scenarios a person wrote, and what they can
   arrange and observe.
3. [Runners and reports](verify/runners.md) — run a suite against a built-in target, a generated Go
   or TypeScript package or a Rust target, and read the report.
4. [Audit a suite with specification mutants](verify/mutation-audit.md) — measure whether the suite
   notices a changed specification.
5. [Explore command sequences and histories](verify/explore.md) — random sequences and concurrent
   histories.

## Where each section went

This guide used to be a single page. Each of its sections is listed here under its old anchor, so
an older link still finds it.

- <a id="generate-the-suite"></a>[Generate the suite](verify/synthesize-a-suite.md#generate-the-suite)
- <a id="select-authored-scenarios-explicitly"></a>[Select authored scenarios explicitly](verify/author-scenarios.md#select-authored-scenarios-explicitly)
- <a id="expect-an-external-branch-in-an-authored-scenario"></a>[Expect an external branch in an authored scenario](verify/author-scenarios.md#expect-an-external-branch-in-an-authored-scenario)
- <a id="establish-backend-state-in-an-authored-scenario"></a>[Establish backend state in an authored scenario](verify/author-scenarios.md#establish-backend-state-in-an-authored-scenario)
- <a id="name-several-instances-in-one-input"></a>[Name several instances in one input](verify/author-scenarios.md#name-several-instances-in-one-input)
- <a id="observe-outcomes-selected-by-held-state"></a>[Observe outcomes selected by held state](verify/author-scenarios.md#observe-outcomes-selected-by-held-state)
- <a id="observe-retries-of-the-original-result"></a>[Observe retries of the original result](verify/author-scenarios.md#observe-retries-of-the-original-result)
- <a id="observe-selection-periodic-activity-and-clock-evidence"></a>[Observe selection, periodic activity and clock evidence](verify/author-scenarios.md#observe-selection-periodic-activity-and-clock-evidence)
- <a id="deliver-an-event-with-its-context"></a>[Deliver an event with its context](verify/author-scenarios.md#deliver-an-event-with-its-context)
- <a id="run-a-supported-target"></a>[Run a supported target](verify/runners.md#run-a-supported-target)
- <a id="hold-your-own-implementation-to-the-suite"></a>[Hold your own implementation to the suite](verify/runners.md#hold-your-own-implementation-to-the-suite)
- <a id="audit-the-suite-with-specification-mutants"></a>[Audit the suite with specification mutants](verify/mutation-audit.md#audit-the-suite-with-specification-mutants)
- <a id="explore-random-command-sequences"></a>[Explore random command sequences](verify/explore.md#explore-random-command-sequences)
- <a id="check-a-concurrent-history"></a>[Check a concurrent history](verify/explore.md#check-a-concurrent-history)
- <a id="draw-a-history-as-client-lanes"></a>[Draw a history as client lanes](verify/explore.md#draw-a-history-as-client-lanes)
- <a id="import-a-recorded-log"></a>[Import a recorded log](verify/explore.md#import-a-recorded-log)
- <a id="opt-into-explicit-outcome-counts"></a>[Opt into explicit outcome counts](verify/runners.md#opt-into-explicit-outcome-counts)
- <a id="where-passed-failed-and-skipped-live"></a>[Where passed, failed and skipped live](verify/runners.md#where-passed-failed-and-skipped-live)
- <a id="opt-into-declared-coverage"></a>[Opt into declared coverage](verify/runners.md#opt-into-declared-coverage)
- <a id="observe-bounded-binding-accessors"></a>[Observe bounded binding accessors](verify/author-scenarios.md#observe-bounded-binding-accessors)
- <a id="what-the-report-proves"></a>[What the report proves](verify/runners.md#what-the-report-proves)
- <a id="a-target-in-rust"></a>[A target in Rust](verify/runners.md#a-target-in-rust)
