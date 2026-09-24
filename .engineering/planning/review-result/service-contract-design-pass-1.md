---
format: aep.planning-md/1
id: review-result:service-contract-design-pass-1
kind: review-result
status: active
title: First concrete selected ServiceIr design examination
relations:
- reviews: design:reusable-service-contract
revision: 1
---
needs-revision

Covered design: `docs/design/ess-evolution/service-contract.md` at
`sha256:05fe9e142c8697ccf72bd5abbedba254ba072625e8073e4ed14a574f05cb5bb6`.

## Blockers

### 1. The entity capability closure loses incoming ownership semantics

**Requirement.** The selected contract must preserve ownership versus context and provide a complete
entity/relation capability closure for a subsequent target. The design selects `EntityLifecycle`
for owned/directly-used entities and then follows their relation targets
(`docs/design/ess-evolution/service-contract.md:76`).

**Source evidence.** A relation is stored only on its declaring source entity, while an `owns`
relation's carrier field is on its target (`crates/specify/ess-compiler/src/ir.rs:386` and
`crates/specify/ess-compiler/src/ir.rs:403`). The compiler therefore has to scan every source entity
to answer which relation a target entity's field carries
(`crates/specify/ess-compiler/src/ir.rs:1613`). The domain contract makes the reverse edge deliberate:
an owned target's owner is derived by lookup rather than repeated on the target
(`crates/specify/ess-domain/src/entity.rs:681`). Relation validation is system-wide and does not
constrain owner and target to one domain (`crates/specify/ess-domain/src/entity.rs:1206`).

**Why this prevents implementation.** In a valid model, an entity in the selected component's owned
domain can be the target of an `owns` relation declared by an entity in an unowned domain. Starting
from the selected entity and following only its outgoing `relations[].target` edges never reaches
that foreign owner. The selected entity's carrier field consequently loses the relation name, kind,
cardinality, and owner identity from the selected capability set. A consumer could recover the fact
only by ignoring `capabilities()` as the promised closure and independently rescanning the global
`source()`, which creates the second selection rule this extraction is intended to eliminate.

**Bounded correction.** Define entity closure as a fixpoint that, for every included entity, follows
outgoing relation targets and also adds the declaring source of every incoming `owns` relation whose
carrier is that entity (using `EssIr::relations_carried_by`); keep those added entities contextual and
out of `owned_entities()`. Continue the existing type and relation-target closure from each added
source. Add a fixture where an unowned-domain entity owns a selected-domain child and assert the
foreign owner capability, complete carried relation, and unchanged local ownership surface.

### 2. Binding selection is inconsistent with the independent publication surface

**Requirement.** Binding transformation, delivery, escalation, and their obligations must be selected
from one exact service-output rule. The design exposes `component.publishes` as the public event
surface (`docs/design/ess-evolution/service-contract.md:47`) but selects reacting bindings from events
emitted by selected operation outcomes (`docs/design/ess-evolution/service-contract.md:85`).

**Source evidence.** `ResolvedComponent::publishes` and `ResolvedCommand::outcomes[].emits` are
independent compiler-owned sets (`crates/specify/ess-compiler/src/ir.rs:1310` and
`crates/specify/ess-compiler/src/ir.rs:675`). Component validation expressly permits publication from
a domain the component does not own, including translation and republication, and does not derive it
from accepted commands (`crates/specify/ess-domain/src/component.rs:100`). The existing Rust port
publishes an outcome event only when it is also in the component's `publishes` set
(`crates/generate/ess-synth/src/rust/port.rs:271`), while the system transport's event roster starts
from every component's declared published events
(`crates/generate/ess-synth/src/rust/system.rs:145`).

**Why this prevents implementation.** Both asymmetric cases are valid. A binding caused by an event
the selected component declares it publishes but no selected operation emits is omitted, losing its
delivery or escalation obligation from the public service context. Conversely, a binding caused by
an outcome event the component does not publish is included even though the existing component
transport filters that event out. Billing and gatepass do not settle this because their relevant
published and emitted events overlap. An implementor therefore cannot derive the promised exact
binding roster from the selected surface and the cited synthesis rule without choosing new semantics.

**Bounded correction.** Make selected transport bindings those that invoke a selected operation or
whose event cause is in the selected component's exact `publishes` set. If unpublished operation
emissions are intentionally retained for a later non-transport target, specify them as a separate
contextual rule rather than selecting their `BindingDelivery` capability. Add distinct fixtures for
published-but-not-emitted and emitted-but-not-published events and assert exact binding identities,
dispositions, obligations, and refusals in original plan order.

## Examination boundary

Read the fixed design, its minimal service-contract model, dependency policy, relevant source-routing
facts, and the compiler/synthesis definitions and consumers needed to trace selection, relations,
handles, capabilities, conversions, bindings, ordering, and canonical provenance. No tests, builds,
services, SQL operations, database operations, or network operations were run. The sole outside write
was `home-path:sha256:6f73a084d6d3cb93fa387d2d7262f9a5469bc547c725a8c4c94ac6b700e9f13d`.

```findings
- file: docs/design/ess-evolution/service-contract.md
  line: 76
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Define entity capability closure over incoming owns sources as well as outgoing relation targets, because an owns relation is stored only on its declaring source while its carrier field is on the selected target, so the current forward-only closure loses valid ownership semantics
- file: docs/design/ess-evolution/service-contract.md
  line: 85
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Align reacting-binding selection with the component's independent published-event surface and test both asymmetric publish/emit cases, because current synthesis transports only declared publications while the proposed rule selects outcome emissions and can both omit public delivery obligations and include undeliverable ones
```
