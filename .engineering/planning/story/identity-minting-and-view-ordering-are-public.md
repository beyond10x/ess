---
format: aep.planning-md/3
id: story:identity-minting-and-view-ordering-are-public
kind: story
status: active
title: Identity minting and view row ordering are public for consumers
summary: 'Adopter report row 28: minting and view ordering are private, so consumers copy them.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:52Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:52Z", actor: "human:timo", revision: 7}
---
## Problem

The interpreted target mints identities and orders view rows with functions consumers cannot call: `fn minted` (`crates/verify/ess-conformance/src/interpret/execute.rs:2827`) is private and `compare` (`crates/verify/ess-conformance/src/interpret/views.rs:340`) is `pub(crate)`. A downstream renderer that must agree with the interpreter copies both; before its copy it measured 86 mismatches against the interpreter (adopter report, row 28, ess 0.67.0). A copy drifts silently when either function changes.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/verify/ess-conformance/tests/interpreted_typed_identities.rs`:
  `a_consumer_mints_the_identity_the_interpreter_mints`: for a Uuid, a String newtype with a prefix and an Integer identity, the interpreted target creates three instances in a fixture minting nothing else, and their identities equal `interpret::values::mint(ir, type, 1..=3)`, and `mint` of an `Optional` type is `None`.
- `crates/verify/ess-conformance/tests/interpreted_views.rs`:
  `a_consumer_orders_view_rows_as_the_interpreter_answers_them`: for a view ordered by a Timestamp descending and a newtype over Integer, with a second key breaking ties, rows shuffled and passed to `order_view_rows` come out in the order the interpreted target answers the view.
- same binary, `rows_tied_on_every_order_key_keep_the_callers_order`.

## Fit review

1. **Need.** A downstream renderer that already depends on `ess-conformance` and drives the interpreted target needs to mint and order values exactly as the interpreter does. Today it copies both functions. Code on main (0.69.2): `fn minted` is private (`crates/verify/ess-conformance/src/interpret/execute.rs:2827`). `Store::mint` is `pub(super)` (`:2864`). `compare` is `pub(crate)` (`interpret/views.rs:340`) and is re-exported only crate-wide as `rank_compare` (`interpret.rs:76`). `views` is a private module (`interpret.rs:75`). Verbatim report: "identity minting and view ordering private … copies them …; 86 mismatches before copy" (`batch0690/triage40/ess-reports.before.md:41`). The copy has already drifted for String and Integer identities (triage `batch0690/triage/rows27-28.md`, row 28 detail).
2. **Class.** Gap in the library surface. No authored document reaches it.
3. **Already expressible?** No. A consumer can only observe minted values by running `interpret::execute::execute` and reading answers back. It cannot compute the value for a type and a counter tick, or order rows it holds itself.
4. **Fit.** No authored surface. The functions already exist and the interpreter calls them; only their visibility and a stable signature change. Siblings: synthesis already shares `compare` for ordered heads (`synthesize/ordered_head.rs:131`), which is the one-implementation property the request wants. The Go and TypeScript runners mint nothing, because they are runners, not targets. Not covered: the history-operation mint path (`execute.rs:2814-2821`, `history::generated`) and building a `Store` from outside (the consumer's own note); both are separate requests.
5. **Second adopter.** A model-preview tool shows "what the interpreter would answer" for a draft specification and must show the same generated identifiers and row order as a later conformance run.
6. **Cost.** No format, keyword or diagnostic. `ess-conformance` gains two public functions, which is additive. The functions return the interpreter's current choice, not a specification rule (`interpret.rs:62-66`: "the runner never compares one against an expected value"). A later change to the mint scheme changes their output, and the doc comment says so.
7. **Alternatives.** (a) Change nothing: the copy keeps drifting. (b) Make `minted` and `compare` `pub` as they are: this leaks `&mut dyn FnMut() -> u64` and a resolved-type walk, the requester's literal ask. (c) A small public module with `mint(ir, type, tick) -> Result<Option<Node>, Undetermined>` and `order_view_rows(ir, view, &mut [ViewRow])` (stable sort by the view's `order_by`), which the interpreter itself calls (chosen).

## Decisions

- Accept, redesigned. Two public functions in a public `ess_conformance::interpret` submodule (for example `interpret::values`):
  - `mint(ir, target, tick)`, the value the counter's `tick`-th draw yields, `None` for an `Optional` target.
  - `order_view_rows(ir, view, rows)`, the order `query` applies (`views.rs:68`). Ties keep the caller's order, which is a stable sort.

  `minted`, `Store::mint` and `views::query` call them, so there is one implementation of each.
- The doc comments say the values are the interpreted target's choice and are not checked against any implementation.
- No `ess/32`, suite or IR format move.

## Scope

| unit | files | confidence |
|---|---|---|
| mint | `crates/verify/ess-conformance/src/interpret/execute.rs` `minted` (`:2827-2858`), `mint` (`:2809`), `Store::mint` (`:2864`), `Undetermined` (`:278`, already `pub`) | cited |
| order | `crates/verify/ess-conformance/src/interpret/views.rs` `query` sort (`:68`), `rank` (`:311`), `compare` (`:340`) | cited |
| module surface | `crates/verify/ess-conformance/src/interpret.rs` (`:66-76` module list and `rank_compare` re-export) | cited |
| other caller | `crates/verify/ess-conformance/src/synthesize/ordered_head.rs:131` (keeps calling the same `compare`) | cited |
| docs | `crates/verify/ess-conformance/README.md` or crate docs | inferred |

Crate: ess-conformance (sizing: S). Format gate: none. Tests: `crates/verify/ess-conformance/tests/interpreted_typed_identities.rs` and `crates/verify/ess-conformance/tests/interpreted_views.rs`.

confidence: high
