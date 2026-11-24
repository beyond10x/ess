---
format: aep.planning-md/3
id: story:every-target-names-what-it-does-with-a-guard-over-an-expression-of-several-facts-generated-rust-and-go
kind: story
status: active
title: 'Every target names what it does with a guard over an expression of several facts: generated Rust and Go behaviour keep the command an obligation named by it, the Go and TypeScript runtimes evaluate it, and Entity Runtime lowering refuses it by name'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-ess-39-a-command-guard-compares-a-fact-with-an-expression-of-several-facts-and-every-expression-reads-the
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:52:01Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T03:52:01Z", actor: "human:timo", revision: 5}
---
## Need

`sx12-guard-arithmetic` adds, at `ess/39`, a several-fact operand to command guards and a list's
length to every expression. The fit-review rule binds every target: it handles the construct or
refuses it by name; none may ignore it silently (`.agents/skills/assessing-external-requests/SKILL.md`,
question 4 and the red flags). Today each target matches the one-fact `Derived` operand:

- generated Rust and Go behaviour resolve a derived operand's kind and value
  (`crates/generate/ess-synth/src/rust/behaviour.rs:3126`, `:3249-3264`;
  `crates/generate/ess-synth/src/go/behaviour.rs:4121`, `:4269-4285`), and the determined-value
  analysis reads its one parent (`crates/generate/ess-synth/src/determined.rs:1173-1203`, `:1289`);
  for the digest functions the generated behaviours keep the command an obligation (the `ess/27`
  row of `website/docs/reference/spec-versions.md:70`);
- Entity Runtime lowering rewrites a derived operand through its parent
  (`crates/generate/ess-entity-runtime/src/lib.rs:4750`) and refuses a function call as
  `FunctionCallUnsupported`;
- the Go and TypeScript conformance runtimes carry predicate evaluators
  (`crates/verify/ess-conformance/src/go/predicate.go`, `crates/verify/ess-conformance/src/ts/predicate.ts`).

Sixth list, item G12 (rank P2), the targets' half.

## Outcome

- **Generated Rust and Go behaviour** keep a command whose guard holds a several-fact operand an
  obligation, named by the expression, as they do for a guard reading a digest function and for an
  `{expr}` value; a computed value reading `<list>.count` is generated where the value expressions
  are generated today, or kept an obligation by the same rule.
- **Entity Runtime lowering** refuses the guard by name (`FunctionCallUnsupported`'s family, a
  message naming the expression), and `website/docs/reference/entity-runtime-lowering.md` lists it.
- **The Go and TypeScript conformance runtimes**, wherever they evaluate a guard predicate of the
  model, evaluate the operand with the same rules as the Rust evaluator (exact integers within
  signed 128 bits, `Unknown` outside, Decimal exact, `/` absent at zero), or, where a runtime path
  does not evaluate guards at all, nothing changes; a shared case table pins the three evaluators to
  one answer.
- **`ess verify diff`** reports a changed several-fact operand as it reports any changed guard
  operand; a test pins one case.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- First: in `crates/generate/ess-synth/tests/`, the reproduction of `sx12-guard-arithmetic`
  generated for Rust and Go. Before (with the domain unit landed): the generator either fails on
  the new `Derived` arm or emits code that ignores it. After: `Admit` is listed as an obligation
  naming `input.cells.count != lines.count * sites.count`, and the generated code compiles
  (`cargo` and `go build` in the existing generated-tree tests).
- Entity Runtime lowering of the reproduction is refused, naming the expression; a test in
  `crates/generate/ess-entity-runtime/tests/` pins the text.
- A case table of ten operands (products, sums, a quotient with `coalesce`, an overflow, an absent
  `Optional` list) gives one answer in the Rust evaluator and in each Go and TypeScript runtime path
  that evaluates guards; the TypeScript cases run in `predicate.test.ts`, the Go cases in the
  existing Go test harness.
- `ess verify diff` between the reproduction and a copy whose operand is `lines.count + sites.count`
  reports one changed guard; a test in `crates/verify/ess-diff/tests/`.
- Committed generated trees and the byte pins named in `AGENTS.md` § Gate (`ess-synth`'s
  `served_codec_names`, `go`; `feasibility`, `adv319`, `http_binding_rust`) are unchanged.
- Each new test is timed and expected under 10 s; none may run over 60 s, each test step stays
  within 120 s, and each build-pool job within 10 minutes.

## Fit review

Verdict: **accept**, the targets' half of `sx12-guard-arithmetic` (sixth list, item G12). Tree
`bcfc51555`, read-only.

1. **Need.** No target may ignore a guard it does not read.
2. **Class.** Gap that follows from the domain unit.
3. **Already expressible?** n/a.
4. **Fit.** The same per-target answers the digest functions received in `ess/27`: obligation,
   named refusal, shared evaluator semantics.
5. **Second adopter.** Every adopter generating Rust or Go behaviour from a model that uses the
   operand.
6. **Cost.** No format. Generated code changes only for models that use the operand.
7. **Considered.** Generating the arithmetic in Rust and Go behaviour now: larger, and the value
   expressions are not generated either; left to the decision that generates `{expr}`.

## Decisions

- None of its own.

- FMT-B (decided: A, 2026-11-24): this story moves to `ess/38` with the rest of the pull request (D77: no `ess/39`); the Acceptance line on the format says so.

## Scope

- cited: `crates/generate/ess-synth/src/rust/behaviour.rs`, `crates/generate/ess-synth/src/go/behaviour.rs`,
  `crates/generate/ess-synth/src/determined.rs`
- cited: `crates/generate/ess-entity-runtime/src/lib.rs` (`:4750`),
  `website/docs/reference/entity-runtime-lowering.md`
- inferred: `crates/verify/ess-conformance/src/go/predicate.go`, `crates/verify/ess-conformance/src/ts/predicate.ts`,
  `predicate.test.ts`
- inferred: `crates/verify/ess-diff/src/` (guard operand comparison), one test
- Size: M, about 380 changed lines.
- Could not establish by reading: which Go and TypeScript runtime paths evaluate command guards
  (explore mode reads predicates, `explore.go`, `explore.ts`); the implementor lists them first.

## Depends on

- `sx12-guard-arithmetic`. Independent of `sx12-arithmetic-witness`; both may run side by side.
