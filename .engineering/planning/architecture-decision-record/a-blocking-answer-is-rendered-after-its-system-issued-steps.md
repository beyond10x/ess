---
format: aep.planning-md/3
id: architecture-decision-record:a-blocking-answer-is-rendered-after-its-system-issued-steps
kind: architecture-decision-record
status: proposed
title: A blocking answer is rendered after the system-issued steps its command started
summary: Proposes an outcome answer read from a view after settlement (ess/N), so synthesis checks the settled answer an adopter now rebuilds by hand (row 24).
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 1
---
Status: proposed (draft). Not accepted; nothing under it is dispatched before acceptance.

## Context

A command's answer is rendered from the outcome its own port returned. The generated Rust server
runs the command, then `settle` pumps every binding that reacts to what the command published
(including commands the system issues itself, `system: true`, `docs/design/system-issued-commands.md`),
and then renders `answer_<command>(&outcome)` from the outcome captured before the pump
(`crates/generate/ess-synth/src/rust/http.rs:1170-1180`, `settle` at `:1205-1225`).

So when a command starts work the system finishes, the caller is answered with the state before
that work, even though the work has already run by the time the answer is written.

Adopter repro (beyond10x/a2a, tracker row 24, ESS 0.67.0): A2A `SendMessage` with
`configuration.blocking: true` must answer with the task after the agent's system-issued steps have
moved it (to `completed`, `input-required` or `failed`). The generated answer carries the task as
`submitted`. The adopter's router discards the generated answer and re-reads the task to build a
new one (a2a `router.rs:700-709`, `:781-792`), which nothing in the conformance suite checks.

The model has no way to say "answer after the system-issued steps this command started". Every
view is read separately; an outcome's response fields come from the command's own input and sets.

## Options

| | what it is | cost |
|---|---|---|
| A | **Answer from a view read after settlement.** An outcome may declare `answer: {view: <V>, settled: true}`: the response is the named view's row for the command's subject, read after the pump. Synthesis checks the answer against the view's row once every awaited system-issued step has run. | New `ess/N` key and a format version; synthesis must order the awaited steps before the answer assertion (the awaiting machinery from `system: true` already exists); generated Rust, Go and TypeScript servers render after `settle` instead of before. The answer's shape becomes a view's shape. |
| B | **Binding-only: `ess-http/1` declares a blocking answer.** The binding names a view to re-read after settlement; the model is unchanged. | No format version for `ess/N`; but the rule is invisible to the interpreted target and to conformance, so the one property the adopter needs checked stays unchecked. Every other binding (`ess-jsonrpc/1`, `ess-grpc/1`) repeats it. |
| C | **Keep it; document the re-read.** The answer is the command's own outcome; an adapter that needs the settled state reads a view. | Nothing to build; the adopter keeps hand code nothing checks, and every A2A-like protocol repeats it. |

## Recommendation

A. The answer timing is observable behaviour a caller depends on, so it belongs in the model where
synthesis can witness it, by the same argument that put `system: true` in `ess/27`. B is the
fallback if A's format cost is judged too high for one adopter.

## Effect

- **Format:** next open `ess/N` (one key on an outcome); older formats refuse it by name; without it
  every artifact keeps its bytes.
- **Synthesis:** a settled answer is asserted after the awaited system-issued commands of the
  scenario, against the named view's row; a step the system never issues refuses the outcome by
  name (`ESS-SYNTH-*`).
- **Generated code:** servers render a settled answer after `settle` by reading the view; the
  outcome renderer keeps today's path for every other outcome.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.
