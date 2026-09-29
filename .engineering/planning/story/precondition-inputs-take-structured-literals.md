---
format: aep.planning-md/3
id: story:precondition-inputs-take-structured-literals
kind: story
status: implemented
title: 'A precondition cannot open a session whose command takes a list: literal refused as not a scalar, fixture refused by the explorer'
refs:
- provider: github
  reference: beyond10x/ess#205
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:49Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:49Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T01:53:27Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":10,"verification":1}}}
---
# Story: A precondition cannot open a session whose command takes a list: literal refused as not a scalar, fixture refused by the explorer

## Why

beyond10x/ess#205. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #205 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

**Reproduces on this tree. Nothing on it fixes the defect.** Tree: `e932781965`, release 0.40.0.

- (cited) A literal list is refused because `precondition_input` passes every non-fixture literal to `example_admitted`, at `crates/specify/ess-domain/src/command/outcome_shapes.rs:716-727`. That function sends `TypeRef::List | Map | Optional` to "is not a scalar" at `crates/specify/ess-domain/src/command.rs:3971-3972`. It sends a named struct to "reaches `…`, which is not a scalar" at `command.rs:3962-3966`.
- (cited) Leaving the input out is refused by the required-input loop at `outcome_shapes.rs:729-745`, which exempts only fixture inputs and `Optional` fields.
- (cited) With a fixture, validation passes. Both explorers then refuse the precondition: Go at `crates/verify/ess-conformance/src/go/explore.go:1459-1461` (`explorePreconditions`), TS at `crates/verify/ess-conformance/src/ts/explore.ts:1182-1186`.
- (cited) The design doc names the explorer refusal as a known limit, at `docs/design/outcome-shapes.md:23-26`.
- (cited) The rest of the pipeline already carries structured literals:
  - The IR holds `input: BTreeMap<String, Node>` (`crates/specify/ess-compiler/src/ir.rs:2105-2107`).
  - Synthesis sends each value as `ScenarioValue::literal(value.clone())` (`crates/verify/ess-conformance/src/synthesize.rs:6805-6809`).
  - `precondition_branch` ignores `Seq`/`Map` literals when it builds facts (`outcome_shapes.rs:794`). A guard that reads one is left undecided and refused (`:806-814`), so the branch choice stays sound.
- (inferred) The Go explorer copies the IR input into `Row` unchanged (`explore.go:1470-1473`), so a list literal should flow into the model's `sets:` as it is. I have not confirmed that model/target row comparison handles list equality.

**Where the fix lands (recommended option A: admit structured literals)**
- (inferred) Add a function in `ess-domain`, for example `precondition_literal_admitted(types, declared, node)`, in `command.rs` beside `example_admitted`. It recurses through List (each element), Map (each value; keys typed as String), Struct (declared fields, required ones present, no unknown keys) and Optional (`null` or the inner type). At each scalar leaf it calls `example_admitted`.
- (inferred) `outcome_shapes.rs:716` switches to the new function. `example_admitted` stays scalar-only, because `example:` needs it that way.
- (inferred) Update `docs/design/outcome-shapes.md:18`, which should say what a precondition literal may be.
- (inferred) Add tests: an `ess-domain` validation case for `[]`, a non-empty list and a struct; a synthesis case; an extension of `crates/verify/ess-conformance/tests/explore_preconditions.rs` with a list precondition through the Go and TS drivers.

**Design decision for the implementor**
- (inferred) Option A (structured literals) is small and deterministic, and it is the one I recommend. Option B (explorers resolve fixtures) needs a fixture-provider hook on the explorer `Target` in both Go and TS, and the explorers are designed to run without one. Leave B as a later milestone. The TS/Go "reads fixture inputs" refusal stays either way.
- (inferred) Open sub-question: should a literal that resolves to `Json` or `Binary64` still be refused? I would keep both refused at the leaf, matching `example_admitted`.

**Collisions**
- (inferred) #211: a new cross-entity guard most likely extends `precondition_branch` (`outcome_shapes.rs:761+`), since it would need refusing as unobservable, and the explorer model in `explore.go`/`explore.ts`. Same files, different functions, so the merge risk is moderate.
- (inferred) #204: selection-authority validation in `command.rs`. Same file, a region far from `:3896`, so low risk.
- (inferred) #196: Map witnesses in `synthesize.rs`. That is witness selection, not preconditions (`:6796`), so low risk.
- (inferred) #209, #198, #199: arrangement search in `synthesize.rs`, away from `preconditions()`. Low risk.
- (inferred) #210, #203, #202, #201, #195, #193: no overlap found.

Confidence: high on reproduction and landing site (every step of the code path read); medium on explorer list equality (not traced).

Paths:
- crates/specify/ess-domain/src/command/outcome_shapes.rs
- crates/specify/ess-domain/src/command.rs
- crates/verify/ess-conformance/src/go/explore.go (inferred: check only)
- crates/verify/ess-conformance/src/ts/explore.ts (inferred: check only)
- crates/verify/ess-conformance/tests/explore_preconditions.rs (inferred)
- docs/design/outcome-shapes.md (inferred)

Verdict: open (small design choice: structured literals over explorer fixture resolution).
