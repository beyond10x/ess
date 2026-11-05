---
format: aep.planning-md/3
id: story:the-interpreter-and-synthesis-answer-a-guarded-move-before-not-found-alike
kind: story
status: draft
title: The interpreter and synthesis answer a guarded move before a not-found alike
summary: After 0.87.0 U1, synthesis gives a fresh identity with the guarded move to the accepting branch declared first, while the model interpreter answers not-found (from-finding)
tags:
- backlog
- from-finding
revision: 1
---
## Need

Found while implementing `story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair` (0.87.0 wave,
U1). For a command whose accepting branch `when: move == Open` is declared before an `external:` not-found branch,
synthesis now follows the precedence order (`docs/design/input-guard-overlap-precedence.md`): an input with a
fresh identity and `move: Open` is answered by the accepting branch. The model interpreter answers the same
input with `not-found` (`crates/verify/ess-conformance/src/interpret/execute.rs` near :2458-2477, through
`declared_not_found`).

No scenario asserts either answer today, because synthesis no longer builds the witness that sent this input. The
two still contradict each other on the same input, and a hand-written scenario would see the interpreter's answer.

## Acceptance

- Decide which answer the precedence order gives for this input, against
  `docs/design/typed-literals-and-unknown-instances.md` section 2a and the order in
  `input-guard-overlap-precedence.md`, and record the decision in the design text.
- A named test sends `{fresh identity, move: Open}` to the interpreter and to synthesis's expectation and finds the
  same outcome.

## Schedule

Backlog finding (from-finding); no wave takes it before every planned wave is delivered unless the orchestrator
says so.
