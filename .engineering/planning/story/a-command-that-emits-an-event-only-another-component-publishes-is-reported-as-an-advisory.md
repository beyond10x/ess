---
format: aep.planning-md/3
id: story:a-command-that-emits-an-event-only-another-component-publishes-is-reported-as-an-advisory
kind: story
status: draft
title: A command that emits an event only another component publishes is reported as an advisory, because the component that accepts it publishes nothing for it
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/load.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/validate_implied_relations.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json
- confidence: cited
  path: crates/generate/ess-synth/src/go/port.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/port.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: inferred
  path: docs/design/ess-implementor-design-v0.1.md
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 14
---
## Outcome

`ess specify validate` warns, at the `emits:` entry, when a command's outcome emits an event that
another component publishes and the component that accepts the command does not. The warning
names both components and the two repairs. A model that wants the event published by the
accepting component lists it in that component's `publishes.events`; a model that keeps it
internal is unchanged, apart from the warning being there to read.

## Need

Finding of a downstream review: in a system with components, a writing command of component A
that emits an event whose publisher is component B makes the generated component A publish
nothing. Reproduced on the released binary 0.97.4 with a generic model of two components
(`S/wave-1122/close/drafts/m3/base/`, four files, run by `m3/run.sh base`):

- `order-api` owns `shop.order` and accepts `PlaceOrder` and `ShipOrder`, and publishes
  `shop.order.OrderShipped`. `invoice-api` owns `shop.invoice`, accepts `RaiseInvoice` and
  `PayInvoice`, and publishes `shop.invoice.InvoiceRaised` and `shop.invoice.InvoicePaid`.
- The outcome `placed` of `shop.order.PlaceOrder` has `emits: [shop.invoice.InvoiceRaised]` with a
  payload.
- `ess specify validate --path base` prints `shop v1 — 4 file(s), valid` and one warning that is
  about an unrelated stored field (`ESS-ENTITY-019`, `invoice_ref`). Nothing mentions the event.
- `ess generate synthesize --path base --target rust --out gen-base` prints `29 capabilities: 29
  generated, 0 obligation(s), 0 refused`. In `gen-base/crates/order-api/src/lib.rs` the handler
  of `PlaceOrder` matches `shop_types::order::PlaceOrderOutcome::Placed { .. } => {}`, and the
  crate's `PublishedEvent` has the one variant `OrderShipped`. The generated `shop-system`
  collects the outboxes of the two components only (`crates/generate/ess-synth/src/rust/system.rs`,
  the `collect` and `pump` it writes), so `InvoiceRaised` reaches the transport when `RaiseInvoice`
  runs, never when `PlaceOrder` does.
- `ess verify conform synthesize --path base --component order-api` writes 1 scenario and says
  `4 outside it`, each as ``outside: `shop.order.PlaceOrder/outcome/placed` needs `event
  shop.invoice.InvoiceRaised` `` (and the same for `ShipOrder/outcome/shipped`, the transition
  `ship` and the state scenario `Shipped/refuses/ShipOrder`).
  The suite already sees the gap; the generator and the validator do not.
- Two variants, same command lines: listing `shop.invoice.InvoiceRaised` under `order-api`'s
  `publishes.events` too (`m3/pub2`) makes the generated `order-api` push `InvoiceRaised` from the
  `placed` arm (`gen-pub2/crates/order-api/src/lib.rs:65`) and the component suite has 5
  scenarios, `0 outside it`; the two components then both publish the event, which is allowed
  (`crates/specify/ess-domain/src/component.rs`, module comment: "Two publishers of one event ...
  is not refused here"). Removing it from `invoice-api` as well (`m3/nopub`) generates the same
  nothing for `order-api` and a transport with no `InvoiceRaised` at all, and the same 4 outside.

Verdict from the source: the generator is not wrong. `rust/port.rs:292` and `go/port.rs:137,222`
build the typed outbox from `component.publishes` only, as the design says (an emitted event that
a component does not publish stays inside it, the rule
`story:the-openapi-says-published-only-for-events-the-component-publishes` already relies on, and
`docs/design/ess-implementor-design-v0.1.md` § 9: publication is a source, a component that
translates and re-publishes is a shape the design contemplates). What is missing is any signal
where the two statements of a model disagree: `emits:` says the outcome carries the event,
`publishes:` says the component does not hand it on. A second reading of the same data disagrees
with the generator inside the repository: `ess-composition/1` exports an event from "a component
that publishes it or accepts a command that emits it"
(`crates/specify/ess-composition/src/lib.rs:1149-1150, 2325-2345`), so an importer selecting
`order-api` sees `InvoiceRaised` as `order-api`'s while its generated crate does not publish it.

How common the shape is, from `census/census2.py` over every `components.yaml` in the tree
(`git ls-files`, 103 models, all under `crates/` and `examples/`): 0 models emit an event another
component publishes, 0 emit an event of a domain another component owns, and 29 emit an event the
accepting component does not publish, where that event is of its own domain or of a domain nobody
owns and nobody publishes (internal events, which bindings inside the system may trigger on). So
the advisory below fires on none of them, and a refusal of the wider "emits what it does not
publish" would refuse 29 models of the fixture corpus.

## Fit review

An adopter finding, assessed as a missing diagnostic rather than a generator defect: the generator
follows the documented rule and the repair is a one-line `publishes:` entry, but nothing says it is
needed. It is generic (any model of two components) and covers the finding in full. It is the
event-side sibling of `story:a-command-writes-only-rows-its-own-component-stores`, which refuses a
write across components and leaves this open (its Need, "Also observed"); the two do not overlap:
that story refuses a write whose repair is "emit an event and bind it", this one makes sure the
emitted event is published by the component that emits it, which is what that repair needs to
work. Both change files of `ess-domain`'s component module and both move the pins listed below,
so the second to land re-pins.

## Design

- One advisory, reported by `advise_locating`
  (`crates/specify/ess-compiler/src/resolve.rs:1137`) beside the two implied-relation rules, and
  therefore only by `ess specify validate`, never by `compile`: the model compiles to the same
  bytes with or without it. The rule lives in `ess-domain` next to
  `entity::implied_relations`, as `component::unpublished_emissions(specification)`: for each
  component with `accepts` and each outcome of each accepted command, each `emits:` event that the
  component does not list in `publishes`, and that at least one other component does, yields one
  advisory at the `emits:` entry.
- The message: "outcome `<o>` of `<C>` emits `<E>`, which component `<K>` publishes and `<A>`,
  which accepts `<C>`, does not; the generated `<A>` publishes nothing for it". Notes: "legal and
  compiles as written" (the sentence `advise_locating` already adds). Help: "list `<E>` under
  `<A>`'s `publishes.events` to publish it from `<A>` too, or stop emitting it from `<C>`".
- Not warned: an event no component publishes (the 29 models), an event the accepting component
  publishes, a model without `components.yaml`, an event published by the same component under a
  command of another component.
- A new `ValidationCode`, `EmittedNotPublished` (`"emitted_not_published"`), reported as a warning
  like `ImpliedRelation`, the second code that is. The family and number are the next free
  `ESS-COMPONENT-` entry. Adding a code moves four pins outside the crate that declares it, and
  this change re-pins them: `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` (the
  digest of the `validation_codes!` invocation), `docs/design/review-typed-diagnostics.md` (its
  per-file count of string-located errors), `website/docs/reference/diagnostics.md` (the catalogue
  row) and `task test-xtask` with `cargo test -p ess-compiler --test typed_diagnostics`.
- Format: no `ess/N`. `AGENTS.md` § Determinism and formats asks a new format version when
  meaning, identity, references, canonicalization, names or the persisted envelope changes;
  an advisory changes none, as `implied_relation` (beyond10x/ess#437) did not.
- Docs: `website/docs/` page on components (find it with `git grep -l 'publishes' website/docs`)
  names the rule in one sentence: `emits:` is not `publishes:`, and `validate` warns where the
  first names an event only another component hands on. `docs/design/` gets the same sentence
  where the design page states that an emitted event a component does not publish stays inside
  it.

## Acceptance

- The model of `m3/base` (copied into a test fixture, four files) makes `ess specify validate`
  print exactly one `warning[ESS-COMPONENT-<n>]` at `domains/order.yaml`, line of the `emits:`
  entry of `placed`, naming `shop.invoice.InvoiceRaised`, `invoice-api` and `order-api`, with both
  repairs in its help; exit 0. `crates/edge/ess-cli/tests/validate_implied_relations.rs` is the
  pattern (one test per case, the exact text compared).
- `m3/pub2` and `m3/nopub` print no such warning, and neither does any of the 103 models of the
  census: the test walks every `components.yaml` of the committed tree and asserts the count of
  this advisory is 0 (a test that reads the tree and runs no synthesis: it times under 5 s).
- `ess generate synthesize` and `ess verify conform synthesize` print the same bytes as before on
  `m3/base`: `diff -r` of the output before and after is empty, and the repository's byte pins
  (`AGENTS.md` § "A gate of named test targets includes the repository-wide byte pins") pass
  unmoved.
- The four pins listed in Design are re-pinned, each with its cause named beside the pin, and
  `task test-xtask` and `typed_diagnostics` pass.
- `a-command-writes-only-rows-its-own-component-stores` and this story are each referred to in the
  other's Overlap (the second to land says so).

## Decisions

A. What a model that emits another component's event learns.
   - An advisory at `validate` (recommended: no committed model is touched, the repair is one line,
     it matches `implied_relation`, it needs no `ess/N`).
   - A refusal (`unsupported_construct`, `ESS-COMMAND-009` as the sibling story does) of an event
     another component publishes: it also fires on 0 of 103 committed models, but it refuses a
     legal shape the component design contemplates (a translating component re-publishing), so it
     needs a `**Breaking**` changelog entry and D3 of the sibling's gating question.
   - The generator publishes the event from the accepting component without a declaration:
     rejected; it changes `publishes:` from a declaration into a default and doubles every event
     both components emit (`m3/pub2` shows the doubling).

B. `ess-composition/1`'s export surface, which counts an accepting component that emits the event
   as its publisher.
   - Leave it (recommended: it is a different question, what an importer may select, and a change
     is a `ess-composition/N` question); the advisory's text does not mention composition.
   - Align it with the generator, so only `publishes` exports: a format change, its own story.

C. Whether the advisory is also a `--strict` refusal of `ess generate`.
   - Follow `implied_relation`, whatever `--strict` does with it today (recommended: no new
     rule); a separate decision if `--strict` should refuse warnings in general.
   - Refuse under `--strict` only this one: a second special case.
