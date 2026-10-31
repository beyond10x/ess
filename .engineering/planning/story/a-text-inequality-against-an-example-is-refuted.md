---
format: aep.planning-md/3
id: story:a-text-inequality-against-an-example-is-refuted
kind: story
status: draft
title: A text inequality against an example value gets its refuting case
summary: Synthesis tries only guard literals on the text ladder, so a != guard against the example value has no refuting candidate
revision: 1
---
## Problem

Synthesis cannot write the case that refutes a `!=` guard against an example's own text value. The
text ladder tries only the guard's literals (`crates/verify/ess-conformance/src/witness.rs`, the
text ladder near 5510-5525), so where the example already holds the compared value, no candidate
selects the other side. Scalar examples have the same limit. Found by the D325 U1 implementor while
writing `ess-conformance/tests/structured_input_example.rs`, which uses a `==` guard for that
reason.

## Acceptance

- A command whose input carries an `example:` and whose guard is `<path> != <the example's value>`
  synthesizes both branches: one sending the example, one sending a value the guard rejects.
- The same holds for a scalar input with an `example:`.
- Suites of models without such a guard keep their bytes.

## Decisions

None yet.
