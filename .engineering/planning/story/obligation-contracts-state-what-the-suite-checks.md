---
format: aep.planning-md/3
id: story:obligation-contracts-state-what-the-suite-checks
kind: story
status: active
title: Obligation contracts state what the suite checks
summary: Generated Rust obligation contracts omit view order_by, misprint an empty-string guard, omit refusal precedence and the ASCII-only trimmed rule (0.77.0)
tags:
- defect
- ess-batch-7
- ess-next
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:16Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:17Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Whoever implements an obligation from the contract `ess generate synthesize --target rust` writes
reads the same rules the conformance suite checks: the view's order, the guard as the IR holds it,
the refusal precedence and the trimming rule.

## Problem

Reported 2026-10-29 against ESS 0.77.0 by a downstream consumer; reproduced on 0.92.0.

Model (`spec.yaml`, `format: ess/27`): entity `notes.core.Note {owner, title}`; command
`notes.core.Create` with outcome `blank` `when: {input.title: {trimmed: {in: [""]}}}` (error
`notes.core.Blank`) before `created`; command `notes.core.Close` with `blank-reason` on
`input.reason` the same way before `closed` (a move); view `notes.core.OpenNotes` with
`params: [owner]`, `filter: owner == param.owner`, `order_by: [title asc, note_id asc]`.

```
$ ess generate synthesize --path spec.yaml --target rust --out rs
13 capabilities: 10 generated, 3 obligation(s), 0 refused
```

1. The view-query obligation drops `order_by`. PLAN.md, plan.json and the `OpenNotesQuery` trait
   doc read "…at `read_your_writes` consistency, containing instances where `owner == param.owner`."
   and say nothing of the order, while the suite checks it. Writers:
   `crates/generate/ess-synth/src/plan.rs:1105`, `rust/items.rs:406`, `go/items.rs:526`.
2. The empty-string guard prints as `trimmed(title in [])`, a guard that never holds, in PLAN.md,
   plan.json, the trait doc, and also the generated OpenAPI descriptions and docs.md.
   `Predicate::AnyOf`'s `Display` joins the values unquoted
   (`crates/specify/ess-primitives/src/predicate.rs:4423-4425`), so `""` disappears.
3. The command contract says "Declared outcomes (declaration order, not selection precedence)"
   and states the precedence only for related guards (`plan.rs:709-713`, `related_precedence`
   at `:720`); it does not say that input-guarded refusals answer before not-found and before
   wrong-state, though the suite pins that order.
4. `trimmed` strips ASCII whitespace only (`trim_ascii_whitespace`, `predicate.rs:552-562`), and the
   contract does not say so. An implementer using Rust `str::trim` treats a no-break space as
   empty where the suite does not. An authored scenario for the no-break-space case is refused
   with ESS-AUTHOR-041.

## Acceptance

- Over the model above, the `OpenNotes` view-query contract in PLAN.md, plan.json and the
  generated trait doc (Rust and Go) states `ordered by title asc, note_id asc`, the keys and
  directions exactly as the IR holds them.
- The `blank` guard is printed `trimmed(title in [""])` everywhere `Predicate`'s `Display` reaches
  (PLAN.md, plan.json, trait doc, OpenAPI description, docs); a text value is quoted.
- The `Create` and `Close` contracts state that an input-guarded refusal answers before not-found
  and before wrong-state.
- A contract holding a `trimmed` guard states that `trimmed` strips ASCII whitespace only (space,
  `\t`, `\n`, `\v`, `\f`, `\r`). Synthesized scenarios do not rely on a non-ASCII space.
- One test per item in `crates/generate/ess-synth/tests/` compares the printed contract against
  what the suite checks, so the two cannot drift apart again.

## Scope

- cited: crates/generate/ess-synth/src/plan.rs
- cited: crates/generate/ess-synth/src/rust/items.rs
- cited: crates/generate/ess-synth/src/go/items.rs
- cited: crates/specify/ess-primitives/src/predicate.rs
- inferred: crates/generate/ess-synth/tests/declared_behaviour.rs
- inferred: crates/generate/ess-gen/tests/openapi.rs

confidence: medium
