---
format: aep.planning-md/3
id: story:generated-behaviour-reads-several-related-rows
kind: story
status: draft
title: Generated behaviour answers a command that reads several related rows, in the interpreter order
relations:
- serves: vision:O2
revision: 2
---
## Outcome

`ess generate synthesize --target rust` (and `go`, which shares the plan) generates the behaviour
of a command whose `when_related:` branches read rows through two or more different input fields,
instead of leaving it an `obligation` of class `undetermined`. The generated code applies the order
the interpreter applies: missing rows in the declaration order of their `exists: false` branches
before anything else, then the first declared present-related predicate refusal, before every
accepting branch. Today `related_composition` refuses it whenever `several_rows` holds
(`crates/generate/ess-synth/src/determined.rs:1482`, `:1555-1561`), and the reason text cites
https://github.com/beyond10x/ess/issues/283, which closed on 2026-10-05 when the language and the
interpreter gained the construct, so a reader is sent to a finished issue.

Requested by an adopter on 2026-10-08: a queue-assignment command refuses `repository-not-found`
(`when_related: {via: input.repository_id, exists: false}`) beside two goal guards addressed by
`input.goal_id`. Synthesis writes 204 scenarios with 0 refusals; generation reports
`136 capabilities: 135 generated, 1 obligation(s)` (ess 0.56.0, `ess/22`), and the adopter's
generator admits only fully generated behaviour. Reproduction kept outside the store.

## Fit review

1. **Need.** A command checks two referenced records by their input ids and refuses when either is
   missing; generated code must give the same answer as the specification.
2. **Class.** Generator coverage of a construct the language (`ess/22`) and the interpreter already
   have. No new syntax.
3. **Already expressible?** Yes in the specification; only generation is missing.
4. **Fit.** The order is already normative (`ess-domain/src/command/related_guard.rs:1778`,
   `:2097`; `ess-conformance/src/interpret/execute.rs:675`). The generator reuses it; no second
   ordering is written.
5. **Second adopter.** An order command refusing `customer-not-found` and `product-not-found` by
   two input ids.
6. **Cost.** `ess-synth` `determined.rs` and `rust/behaviour.rs`, `go/behaviour.rs`; the test
   `tests/related_guard_several_rows.rs` flips from "stays owed" to "generated", with the generated
   crate held by the synthesized suite. No format change.
7. **Alternatives.** (a) Change nothing: adopters split the command or hand-write it, which the
   spec-first rule forbids. (b) Rust only: the shared plan would mark Go owed for one more release.
   (c) This story, both targets.

## Decisions

- Accept. Until it ships, the obligation text names the construct without the closed issue.

## Acceptance

- The `related-guard-multiple.yaml` fixture's `StartRun` is generated for Rust and Go, and its
  synthesized suite passes against both generated crates.
- A generated crate for two missing rows answers the first declared `exists: false` refusal.
- No obligation reason or contract text cites a closed issue.

## Scope

- `crates/generate/ess-synth/src/determined.rs` (`several_rows` :1482, `related_composition`
  :1550-1575), `src/rust/behaviour.rs`, `src/go/behaviour.rs`, `src/plan.rs`.
- `crates/generate/ess-synth/tests/related_guard_several_rows.rs`.
- Held files: none (`ess-conformance/src/interpret/**` and `synthesize/related_guard*` are read,
  not edited).
