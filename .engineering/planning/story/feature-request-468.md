---
format: aep.planning-md/3
id: story:feature-request-468
kind: story
status: active
title: A CLI result field may be typed Json
tags:
- ess-0.55.0
refs:
- provider: github
  reference: beyond10x/ess#468
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T11:58:58Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-06T11:58:58Z", actor: "human:timo", revision: 4}
---
# A CLI result field may be typed Json

## Acceptance

`ess generate cli` admits a CLI result (output) field typed `Json`, alone or under `Optional`,
`List` or a `String`-keyed `Map`: the compiled binding carries a `json` shape, the generated
adapter writes the value as JSON, not as JSON text, and its result check accepts any well-formed
JSON value there and refuses anything else as `cli_result`. A `Json` input field is still refused,
naming result fields as the place `Json` is admitted. An older `ess` refuses a compiled binding
carrying the new shape by name.

## Context

beyond10x/ess#468, filed 2026-10-06 against 0.53.0 by the connectors repository; it blocks
beyond10x/connectors#105.

## Fit review

1. **Need.** A command whose answer carries an arbitrary JSON document (a schema, a provider's
   result) can only declare it `String` in its CLI result, so callers parse JSON text out of a JSON
   answer. Minimal reproduction: a result type with `{name: payload, type: Json}` bound by an
   `ess-cli/1` presentation binding; `ess generate cli` exits 1 with
   ``unsupported CLI primitive `Json` ``. The requester's proposal: a `Json` shape on result fields,
   checked only as well-formed JSON; inputs may stay refused.
2. **Class: gap.** `Json` is a source primitive since `ess/15`
   (`crates/specify/ess-domain/src/types.rs:66`), and the CLI contract maps only `String`, `Boolean`
   and `Integer` (`crates/specify/ess-cli-contract/src/resolve.rs:37-42`), so a declared model type
   has no CLI representation.
3. **Already expressible?** Only as `String` holding JSON text, which is the double parse the
   request removes.
4. **Fit.** No source-format change: the authored type already exists; the change is a new variant
   of the compiled binding's `Shape` (`crates/specify/ess-cli-contract/src/wire.rs:9`), resolved in
   `type_shape` (`resolve.rs:122-137`) and read by the generated runtime emitted by
   `crates/generate/ess-cli-project`. It composes with `Optional`, `List` and `Map` the way the
   other primitives do. Inputs stay refused because an argv word has no JSON spelling the binding
   declares; the refusal says so.
5. **Second adopter.** A deployment CLI whose `describe` answers a rendered manifest as a JSON
   object.
6. **Cost.** One `Shape` variant; generated CLI packages that declare no `Json` field keep their
   bytes, packages that do gain the check; no source-format or suite-format bump.
7. **Considered.** (a) Change nothing and keep JSON text: rejected, the answer stays doubly
   encoded. (b) `Json` on inputs and results: deferred, inputs need an argv spelling. (c) The
   requester's results-only shape: taken.

## Decisions

**Accept as proposed**: results only; inputs refused by name.

## Milestone

`release-plan:ess-055`. Reply to the connectors session when a release carries it.
