# A generated seam can answer an unknown instance

Status: implemented (beyond10x/ess#113, follow-up). No source, suite or diff format moves.

## Before

The unknown-instance rule (`typed-literals-and-unknown-instances.md`, section 2) answers a
`moves:`/`updates:` command whose `instance:` names no record with the command's `wrong_state`
outcome. The generated Rust and Go seams have one spelling of that outcome, and it carries the
declared error with every field the error declares:

```rust
AdmitVisitorOutcome::WrongState { error: VisitStateConflict { state } }
```

A visit that does not exist has no `state`. Fabricating one is manufactured evidence, so
`gatepass-realization` and `gatepass-go-realization` returned the unmet obligation (HTTP `501`),
which fails the suite's unknown-instance scenarios, and the billing realization's conformance
adapter answered the rule *before* the seam, so its seam was never asked.

## Rule

A command gets a **second spelling** of its `wrong_state` outcome — the unknown-instance variant —
exactly when `ess_gen::unknown_instance::unknown_instance_answer` returns it. All three must hold:

1. a branch acts on an existing instance the caller names (`moves:` or `updates:`, with
   `instance:` read from input), so an identity naming no record reaches the command;
2. the command declares a `wrong_state` outcome, the rule's answer;
3. that outcome reports an error with at least one field.

The existing variant is unchanged, so a known instance still has to carry every field: the type
does not let a realization omit `state` for a visit it holds. The new variant carries **nothing** —
not the error's other fields either, because the rule's answer is the branch and the error's name,
and a field that happens to be derivable from input is still a description of an instance that
does not exist.

| projection | known instance (unchanged) | unknown instance (new) |
|---|---|---|
| Rust outcome enum | `WrongState { error: E }` | `WrongStateUnknownInstance` (unit) |
| Go outcome | `<Command>OutcomeWrongState{Error: E}` | `<Command>OutcomeWrongStateUnknownInstance{}` |
| served HTTP answer | `409 {"outcome","error","payload"}` | `409 {"outcome","error"}` — no `payload` |
| Rust system wire record | `refusal: {error, payload}` | `refusal: {error}` |
| component port | publishes nothing | publishes nothing |
| `PLAN.md` contract | unchanged branch text | `, and for an instance no record carries, without the error's fields` |
| OpenAPI `wrong-state` response | `payload` required | `payload` not required; the description says when it is absent |

The variant name is the `wrong_state` outcome's own variant name followed by `UnknownInstance`.
It is a symbol in the same scope as every outcome variant, so an outcome that would spell the same
identifier is refused by the existing `symbol-collision` check rather than shadowed.

### Where the case cannot arise, bytes stay

Every command for which the answer is `None` emits exactly what it emitted before, in every
projection: a `creates:`-only command (1), a command acting on an instance without `wrong_state`
(2 — synthesis records a coverage note for it instead), and a `wrong_state` outcome whose error
has no fields or which reports no error (3 — `WrongState { error: E {} }` or `WrongState` already
says it). So a model gains the variant only where its seam could not previously give the answer
the rule demands; `billing` and `gatepass` are such models, and their committed trees move.

## Realizations

`gatepass-realization` and `gatepass-go-realization` return the unknown-instance variant for a
visit they never registered, instead of the unmet obligation. `billing-realization` does the same
in the realization itself, and its conformance adapter no longer checks whether an invoice exists
before calling the seam: the seam now carries that answer.

## Witness

`examples/gatepass-realization/tests/conformance.rs` runs the committed gatepass suite, unchanged,
against the linked Rust realization in process and against the linked Go realization through the
emitted Go runner, and holds both served surfaces to `409 wrong-state` for an unknown visit. Before
this change both suite runs failed exactly the two unknown-instance scenarios and both surfaces
answered `501`.

## Formats

No format moves. The OpenAPI document relaxes one `required` entry for commands that meet the
rule; a client that expected the payload for an unknown instance was already receiving a `501`.
