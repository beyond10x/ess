# Reusable selected service contract

Authority: ESS evolution revision 1, step 6. This design implements the pure extraction part of
`ess-service-contract`; the ER target and runtime `/4` composition remain separate deliverables.

## Problem and boundary

Service SDK currently selects operations inside its own runtime compiler and flattens command
emissions. ESS already resolves the complete ordered outcomes, response fields, errors, effects,
payloads and entity relations. A reusable service contract must expose those exact values rather
than reconstruct a reduced version of them. The new crate is
`crates/specify/ess-service-contract`, depending on `ess-compiler`, `ess-synth` and `ess-domain`
for their existing public types. It performs no IO, execution, persistence or ER admission.

This is a typed Rust semantic projection, borrowing compiler-owned values. It introduces no
persisted format, reader, generator command or new `EssIr` fields. Runtime and realization `/4`
documents will be separate owned projections of this contract; legacy `/3` bytes are unchanged.

## Input and construction

The public function is `extract<'a>(ir: &'a EssIr, plan: &'a SynthesisPlan,
component: &ComponentName) -> Result<ServiceIr<'a>, Vec<ServiceDiagnostic>>`.

`ServiceIr` has private fields and no public unchecked constructor, mutable accessors or
`Deserialize`. Both inputs remain immutably borrowed for its lifetime. The named component is
looked up in this exact IR, never used as a handle taken from another IR. Unknown selection returns
`ServiceDiagnostic::UnknownComponent { component: ComponentName }`.

Because `SynthesisPlan` has public fields, provenance equality alone is insufficient. Extraction
compares it with `SynthesisPlan::of(ir)` in full, including scope, capability multiplicity, order,
dispositions, obligation/refusal details and provenance. Any mismatch returns
`ServiceDiagnostic::PlanMismatch`. This intentionally refuses hand-edited, incomplete and foreign
plans. The pure planner is the single authority; extraction invents no alternative capability rules.
Independent input diagnostics accumulate in a stable order: unknown selection, then plan mismatch.
There is no successful partial contract when input admission fails.

## Selected surface

`ServiceIr` supplies the following read-only accessors, using ordered collections of references to
the existing resolved types. Names are the existing `QualifiedName` or `ComponentName` types.

| Accessor | Exact meaning |
| --- | --- |
| `component()` | The selected `ResolvedComponent`, including reach, naming, CLI declaration, settings and references. |
| `owned_domains()` | Exactly the component's `owns` handles resolved to `ResolvedDomain`, keyed by qualified name. |
| `operations()` | Exactly `component.accepts`, each whole `ResolvedCommand`, keyed by qualified name. |
| `published_events()` | Exactly `component.publishes`, each whole `ResolvedEvent`; these are public declarations, not every event used by an operation. |
| `views()` | Every view declared by an owned domain, keyed by qualified name; no view from an unowned domain becomes a service query. |
| `owned_entities()` | Every entity declared by an owned domain, keyed by qualified name; a referenced foreign entity never becomes locally owned. |
| `source()` | The exact immutable compiler-minted `EssIr` backing all handles and dependent declarations. This is definition context, not an additional selected service surface. |
| `source_plan()` | The exact admitted complete synthesis plan. Its global dispositions remain explicitly scoped to the source specification. |
| `capabilities()` | Selected service capabilities as defined below, in the original plan order, with the original complete disposition. |

The owned-domain view rule is an explicit reuse of the existing synthesis port rule, documented at
`ess-synth/src/plan.rs::plan_components`: a component port includes the views its domains declare.
CLI grouping is retained as presentation and does not silently remove queries from that semantic
surface. Selection does not infer a component from the SDK service identity, choose the first
component, or promote all commands in owned domains into accepted operations.

Each operation retains its complete input, response, naming, references and ordered outcomes.
Each outcome retains its name, condition, subject/effect, test strategy, ordered zero/one/many
emissions, payload mappings, error/refusal, summary/references and state assignments. No flattening,
deduplication, conversion through `f64`, branch selection or singleton-outcome admission occurs.
Errors, internal emitted events, named types, foreign relation targets and conversion definitions
are resolved through the borrowed source's existing total handle accessors. This keeps a complete
typed definition graph without cloning it or inventing a second handle table. Consumers must not
interpret reachability through `source()` as service ownership or command acceptance.

## Capability selection and obligations

The selected capability roster contains:

- `ComponentPort`, `ComponentTransport` and `Workload` only for the selected component.
- `CommandContract` and `CommandBehavior` only for accepted commands.
- `ViewType` and `ViewQuery` only for selected owned-domain views.
- `EntityLifecycle` starts with owned entities and entities directly used by selected outcome
  subjects or view sources. Compute a fixpoint: include each entity's outgoing relation targets,
  and use `EssIr::relations_carried_by` to include the declaring source of every incoming `owns`
  relation carried by that entity. Continue both directions and the type closure from each added
  entity with visited sets. Foreign owners and targets remain contextual, never newly owned.
- `EventType` for published events and every selected outcome emission/payload event.
- `ErrorType` for errors named by selected outcomes.
- `DomainType` for types used by the selected component settings, commands, response/payload/set
  fields, selected events/errors/views/entities, entity identity/state and transitive named type
  bodies. Follow relation targets and type cycles with visited sets; retain field/variant order.
- `ActorGrants` for actors with at least one grant to a selected operation. Preserve complete grants
  as context; do not add the other granted commands to `operations()`.
- Binding transformation/delivery/escalation capabilities for bindings that invoke a selected
  operation or whose event cause belongs to the selected component's exact `publishes` set.
  Publication is independent of operation emission: published-but-not-emitted events select their
  reacting bindings; emitted-but-unpublished events do not select reacting bindings unless the
  binding independently invokes a selected operation. Unpublished emissions retain their event
  type and complete outcome context, without claiming a component transport delivery. The binding's full
  resolved declaration and global capability disposition remain reachable through `source()` and
  `source_plan()`; following it does not select another component or recursively expose its ports.
- `Conversion` entries for declared conversions used by the selected fields or included bindings,
  using the existing `plan::conversion_source` identity rather than matching prose fragments.

Named dependencies needed by included bindings and conversions enter the contextual type/event/error/
entity capability closure, without changing the selected surface. Match each capability by its
existing kind and exact source identity, never by a string prefix. Preserve original plan order.

Expose `obligations()` and `refusals()` as filtered iterators over these same selected capability
entries, retaining the original `ImplementationObligation` and `SynthesisRefusal`. A generated
source capability is not ER support. In particular external outcome selection, response/generated
payload ownership, binding host reads/conversions and workload refusals are never silently upgraded.
The full source plan remains available so a subsequent target can prove its own complete closure.

ESS compiler inputs contain no unresolved declaration handles. Unknown user selection and mismatched
plan are the bounded extraction diagnostics; target-specific unsupported or unresolved semantics
belong to the subsequent ER target and must remain explicit there. Use existing qualified source
identities in diagnostics; do not fabricate file/line locations absent from `EssIr`.

## Acceptance and stopping condition

Test billing and gatepass through public compilation, planning and extraction. Assert the actual
selected commands, views, domains and public events; compare every retained outcome, response,
effect, mapping and exact literal against the compiler value. Include multiple components with
overlapping dependency types, a foreign relation target, a nonaccepted owned command, zero and
multiple ordered emissions, external outcomes, declared response/generated fields and reused types.
Prove stable repeated results and original ordering, and that the input `EssIr` canonical bytes and
digest remain unchanged. Test unknown component, foreign plan, changed disposition, removed or
duplicated/reordered capability and multiple simultaneous diagnostics.

Include an unowned-domain entity that owns a child in the selected domain: assert the foreign
owner's contextual lifecycle/type capabilities, complete carried relation and unchanged selected
`owned_entities()`. Separate published-but-not-emitted and emitted-but-unpublished fixtures assert
exact reacting-binding identities and complete dispositions/obligations/refusals in plan order;
also retain a binding selected independently because it invokes an accepted operation.

Check contextual closure independently from the extraction implementation: expected fixture
capability identities and full obligation/refusal values must be asserted explicitly. A dropped
outcome/event and a swapped selected field/operation must each make a corresponding test fail;
restore and rerun afterward. This is extraction fault sensitivity, not ER behavior acceptance.

The bounded implementor completes source/tests, scoped formatting, strict crate Clippy, crate tests
and the actual pure dependency closure on Rust 1.85, then yields source hashes and closes its
assignment. Root owns submission, the existing two-pass source-review process, the full ESS
`task check` with consumer coverage, `task site-build` for this design change and local integration.
Complete accounting remains required; this crate does not modify or extend the frozen baseline.
(Parked, 2026-09-25: plan revision 3 removes consumer coverage from the required gate; see
[acceptance](acceptance.md). This paragraph is kept as the record of the original assignment.)

No ER or SDK source, Eventlog code, migration commands, runtime formats, generic registry, JSON
property bag or infrastructure application is part of this assignment. Those remain explicit later
requirements under the unchanged initiative, not an excuse to claim service acceptance here.
