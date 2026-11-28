---
format: aep.planning-md/3
id: story:a-binding-maps-an-event-of-one-system-to-a-command-of-another-system-in-one-run
kind: story
status: draft
title: A binding maps an event of one system to a command of another system in one run
summary: Two systems with their own prefixes are admitted for one run, and a binding crosses from one to the other.
tags:
- adopter-report
- composition
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- informed_by: story:cross-system-relation-target
- informed_by: story:a-component-plays-a-pinned-contract-role
- informed_by: story:system-issued-command-is-awaited-not-driven
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-composition/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system_issued.rs
- confidence: inferred
  path: docs/design/composition-contract-roles.md
- confidence: inferred
  path: docs/design/composition-type-conformance.md
- confidence: inferred
  path: docs/design/system-issued-commands.md
revision: 7
---
## Outcome

Two specifications, each its own `system.yaml` with its own domain prefix, are admitted together for
one run, and a binding of one maps an event of its system to a command of the other. Each system
keeps its own names, its own `naming.wire` and its own validation; no source is renamed into one
prefix. Field types of the event and of the command input conform by a declared rule, not by
accident of equal spelling. A command the target system grants only to a `system: true` actor may
be invoked by such a cross-system binding. The runtime side (one process running several
specifications) is another repository's work and is not this story.

## Problem

A downstream simulation that runs two specifications (the consumer) wants an event of the first
system to cause a command of the second. Both
systems compile alone. Reported on ess 0.99.8 (read-only probes of the reporter) and described by the
minimal model under Acceptance:

- One `system.yaml` listing the domains of both systems is refused once per domain of the second
  system: `[conflicting_declaration] domain <second>.<domain> is not inside the system <first>`
  (`check_namespace`, `crates/specify/ess-domain/src/system.rs:1639-1667`). The reporter's pair gave
  191 of them.
- A mechanical rename of the second system into the first system's prefix compiles only after each
  domain's `naming.wire` gets a prefix of its own: without it two systems claim the same command
  route.
- After the rename a bridge binding is refused two ways. `[type_mismatch]` on every field whose type
  is declared in both systems (an identifier newtype, a currency enum, a mode enum): same spelling,
  different named types. And `[conflicting_declaration] <binding> invokes <command>, a command the
  system issues itself` when the target command is granted to a `system: true` actor
  (`crates/specify/ess-domain/src/system_issued.rs:116`), because the stage-1 design left "a binding
  that invokes a system-issued command" out (`docs/design/system-issued-commands.md`, Validation
  table and Out of scope).
- Nothing in `ess-composition` binds across systems: it pins services, checks references, type
  conformances and roles, and a binding to a pinned system's command is `ESS-BINDING-001`
  (`docs/design/composition-contract-roles.md`, "The gap").

Requester's proposal (theirs, not a design): "A system may include the domains of a second system
with their own prefix, and a binding may map an event of one to a command of the other when the
field types conform."

## Acceptance

- `two_systems_with_different_prefixes_are_admitted_for_one_run`: the test model below, two systems
  with two prefixes, validates and compiles as one run without a `[conflicting_declaration] ... is
  not inside the system` for a domain of either system, and without a rename or a shared `naming.wire`
  prefix.
- `a_binding_maps_an_event_of_one_system_to_a_command_of_the_other`: the binding of the test model
  compiles; its compiled form names both systems; the interpreter fires it on a run of both.
- `a_cross_system_binding_field_of_a_declared_conformance_is_not_a_type_mismatch`: the bridge's
  `[type_mismatch]` on an identifier newtype, a currency enum and a mode enum is gone where the
  conformance is declared (decision CS-D2) and is still raised, naming the field and both types,
  where it is not declared; a conformance whose end has drifted is the existing
  `type_conformance_drift`.
- `a_cross_system_binding_may_invoke_a_command_only_a_system_actor_of_the_target_holds`: the
  `[conflicting_declaration] ... a command the system issues itself` refusal is not raised for a
  binding that crosses systems onto a command the target grants only to a `system: true` actor (CS-D3
  decided A); the same binding inside one system is still refused with it. The binding counts as that
  actor: conformance reads the command through the actor's await and `expect_invocation`, not as a
  caller's step.
- Both systems still validate and compile alone, byte for byte as before: a source with no
  cross-system declaration keeps its bytes, and a format version moves only if CS-D1 needs one.
- Every target handles the new binding or refuses it by name: Rust, Go, TypeScript, web, the
  interpreter, `ess verify diff`, generated docs, synthesis.

Test model (brand-free, to be written under `crates/specify/ess-composition/tests/`): system `alpha`
with domain `alpha.orders` (identifier newtype `OrderId` over `String`, enum `Currency`, event
`OrderPlaced {order: OrderId, currency: Currency}`) and system `beta` with domain `beta.billing`
(identifier newtype `OrderRef` over `String`, enum `Currency` with the same variants, command
`OpenAccount {order: OrderRef, currency: Currency}` granted only to a `system: true` actor `Ledger`).
Each has its own `naming.wire`. Binding `open-account-on-order-placed`: event `alpha.orders.OrderPlaced`
to command `beta.billing.OpenAccount`, mapping `order` and `currency` from the event. Variants: the
binding without the declared conformance (`type_mismatch`), with a drifted enum (`type_conformance_drift`),
inside one system onto the same command (the `issues itself` refusal stays), and a model whose
`naming.wire` of the two systems collide on a route (refused by name, as today).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`.

1. Need: a binding whose trigger event and whose invoked command belong to two systems that are
   specified, versioned and validated apart. Reproduction: the test model above; the reporter's pair
   gave 191 refusals on one `system.yaml`. Requester's proposal labelled above.
2. Class: gap. No construct expresses a cross-system binding; the rename path compiles but the
   binding is refused twice, so there is no idiom (skill question 3).
3. Already expressible: no. `ess-composition` checks references, type conformances and roles between
   pinned systems and never executes or binds (`docs/design/composition-contract-roles.md`,
   `docs/design/composition-type-conformance.md`). `ess specify vendor` (`[Unreleased]`,
   beyond10x/ess#663, `story:a-specification-imports-another-specifications-types`) writes a
   consumer's copy of an owner's type under the consumer's own prefix and proves it with a
   conformance: it avoids the type mismatch for one consumer and does not make two systems one run
   or lift the `issues itself` refusal.
4. Fit: grid O/R (`docs/design/language-grids.md`): the new cell is "a binding's trigger and target
   in different systems", the whole row of cross-system references, not one case. It reuses
   `conformances:` and `writer: true` (`ess-composition/6`) for field types, and `system: true`
   (`ess/27`) for the actor. Siblings that must say the same: a view, a relation (`story:cross-system-relation-target`,
   draft, whose own Status leaves open whether the link lives in the importing system or in a
   composition document: CS-D1 settles it for both), a role (`story:a-component-plays-a-pinned-contract-role`).
5. A second adopter: any two products specified apart (a billing system and a provisioning system)
   that want an event of one to start a command of the other.
6. Cost: a new key or a composition format version (CS-D1), an IR dimension that names the system
   of a binding's two ends, new diagnostics for an undeclared cross-system type, and one new
   diff class (a binding that crosses systems is added or removed). The runtime that serves several
   specifications is out of scope and belongs to another repository.
7. Considered: change nothing (the mechanical rename plus `naming.wire` prefixes rewrites a vendored
   specification and still cannot bind); the three options of CS-D1.

CHANGELOG: checked 2026-11-27 on `main` (`a8938f6b6f`), sections 0.99.3 to 0.99.8 and
`[Unreleased]`: no entry admits two systems in one run, binds across systems, or lifts the
`issues itself` refusal for a binding. The only neighbour is `ess specify vendor` in `[Unreleased]`.
Related designs: `docs/design/composition-contract-roles.md`,
`docs/design/composition-type-conformance.md`, `docs/design/system-issued-commands.md`. Related
stories: `story:cross-system-relation-target`, `story:a-specification-imports-another-specifications-types`,
`story:a-component-plays-a-pinned-contract-role`, `story:system-issued-command-is-awaited-not-driven`
(its stage 1 refuses a binding onto a system-issued command; CS-D3 lifts that for a cross-system
binding only).

## Decisions

- **accept, redesigned**: the need is real and a gap; the requester's "include with its own prefix"
  is one of two shapes, decided by CS-D1.
- CS-D1, **decided 2026-11-27 17:10 CET** by the owner (relayed by the operator session), an
  architecture decision. How a run holds two systems: a composition document above both systems.
  The compiler merges the two systems' declarations into one IR in which they do not collide; a
  runtime reads that composition document's directory the way it reads a `system.yaml` today; a
  cross-system binding arrives in the IR as an ordinary binding; and the `system: true` actor of
  CS-D3 arrives through the IR's actor grants. Each system's own source stays valid alone, unchanged,
  with its own prefix and `naming.wire`. Costs, as drafted: a composition format version, and the
  composition gains an executable construct, which `docs/design/composition-contract-roles.md` kept
  out of the source format on purpose; the run takes a composition, not a `system.yaml`.
  - Letter: the relayed answer was "A". In this story the composition-document option below is
    lettered B (and was the recommended one); option A below is the one that adds a key to
    `system.yaml`, and it is not taken. The decision is the content above, not the letter.
  - Out of scope, and an architecture change that would need its own decision: a runtime that loads
    two IRs and joins them itself.
  - Before the design settles, the IR delta is drafted and shown to the runtime that will read it:
    the binding whose trigger and target belong to different systems, the actor grant that carries
    `system: true` across, and the prefixed names that keep the two systems' declarations apart. The
    design page is not final until that runtime has answered on the draft.
  - The three options as put to the owner:
    - A: a system includes a second system's domains, each keeping its own prefix, through a new key
      of `system.yaml` (a source format version). The binding stays in the including system's
      source. Costs: the source format gains a pin of another system, which
      `docs/design/composition-contract-roles.md` ("Where it lives") refused for roles because the
      source format has no construct that pins another specification; the compiler's
      `check_namespace` and the IR gain a system dimension for domains.
    - B (recommended, taken): a composition document above both systems (`ess-composition/N`) holds
      the cross-system binding: `services` already pins each system by version and digest,
      `conformances` and `roles` already relate them, and each system's own source stays valid alone,
      unchanged, with its own prefix and `naming.wire`.
    - C: change nothing; the rename path stays and the two refusals stay. The consumer keeps a
      stand-in step outside the specification, which moves a checked figure out of it.
  - B won: it adds no pin to the source format, reuses the construct that already relates two pinned
    systems, and leaves both systems byte-identical. It also settles where
    `story:cross-system-relation-target`'s link lives: that story's composition option is the same
    document.
- CS-D2, open. How a cross-system binding's field types conform:
  - A (recommended): a declared conformance between the named types, in the composition's
    `conformances:` entries, with the direction `ess-composition/6` already decides (the sender
    writes, the receiver reads). A binding mapping field `f` from type `T1` to input type `T2` is
    admitted when an entry `T1`-to-`T2` exists and holds, else `type_mismatch` naming both. It keeps
    two identifier newtypes distinct unless somebody says they are one, and the drift check
    (`type_conformance_drift`) already exists.
  - B: structural, implicit: two types of the same shape are interchangeable in a binding. Costs: an
    identifier of one entity flows into the identifier of another because both wrap `String`; the
    value of a newtype is lost.
  - C: an explicit mapping in the binding per field, with a conversion expression. Costs: no check
    that the two types agree; every consumer writes the conversion again. The existing binding
    mapping expressions stay available beside A for fields that differ in shape.
- CS-D3, decided by the operator session (2026-11-27, about 17:00 CET), option A: in a composition of
  two systems, a binding of one system may invoke a command of the other that the target grants only
  to a system actor (`system: true`); the binding counts as that actor. Today refused with
  `[conflicting_declaration] ... a command the system issues itself`. Alternatives not taken: B, the
  target drops the grant when composed (edits the second specification); C, a stand-in step outside
  the specification. The refusal stays for a binding that invokes such a command inside one system:
  `docs/design/system-issued-commands.md` ("Two causes of one command need their own design"). With
  CS-D1 decided, the actor reaches the runtime through the IR's actor grants.

## Scope

- cited: crates/specify/ess-domain/src/system.rs
- cited: crates/specify/ess-domain/src/system_issued.rs
- inferred: crates/specify/ess-composition/src/lib.rs
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: docs/design/composition-type-conformance.md
- inferred: docs/design/composition-contract-roles.md
- inferred: docs/design/system-issued-commands.md
- inferred: CHANGELOG.md

## Acceptance after CS-D1

CS-D1 decided on 2026-11-27 (composition document, one merged IR). These lines refine the Acceptance
above and replace none of it:

- `a_composition_of_two_systems_compiles_to_one_ir_without_colliding_declarations`: the test model's
  composition document, read as one run, compiles to one IR in which both systems' domains, types,
  commands and events are present under their own prefixes and nothing is a `[conflicting_declaration]`;
  each system alone still compiles to the bytes it had.
- `a_runtime_reads_a_composition_directory_as_it_reads_a_system_yaml`: the same loader entry takes the
  composition's directory and answers the one IR; no second loader and no join of two IRs.
- `the_cross_system_binding_is_an_ordinary_binding_in_the_ir`: the compiled form of the binding is the
  IR's existing binding shape, with both ends named by their prefixed names; no new binding kind.
- `the_system_actor_of_the_target_reaches_the_ir_through_its_actor_grants`: the `system: true` grant
  of the target's command is in the IR's actor grants, and the cross-system binding counts as that actor.
- The IR delta (the binding, the actor grant, the prefixed names) is written as a draft and shown to
  the runtime that will read it before the design page is final; the story records its answer.

## A second consumer, 2026-11-28

The report of 2026-11-28 that cites the story above (references across systems, on 0.99.2) is a second consumer of this one too: its specification of reusable building blocks names the other system's records and has no binding across systems in one run. The second consumer is a reason to keep this story in the queue.
