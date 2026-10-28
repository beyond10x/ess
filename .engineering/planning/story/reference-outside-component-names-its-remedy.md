---
format: aep.planning-md/3
id: story:reference-outside-component-names-its-remedy
kind: story
status: active
title: Reference outside component names its remedy
summary: 'Adopter report N11: reference_outside_component has no hint for an unowned domain (ess#665).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#665
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/outside-component
- confidence: inferred
  path: crates/specify/ess-composition/tests/outside_component.rs
- confidence: cited
  path: website/docs/reference/diagnostics.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:25Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:25Z", actor: "human:timo", revision: 8}
---
## Problem

`reference_outside_component` gives a hint only for the owner-declared case; a reference into a domain no selected component owns gets none. Adopter report N11, ess 0.68.0.

## Fit review

1. Need: `ess specify compose` rejects a reference to a name that the selected component does not export. The rejection should say which domain the name is in, which service it belongs to, and what to change. A repro kept outside the repository (`/4`, 0.74.0) prints only "selected component `relay` does not export it" for `demo.loose.Note` (no component owns the domain) and for `demo.audit.Entry` (`auditor` owns it). Proposed by the requester: "select a component that owns the domain, or add the domain to the selected component's `owns`".
2. Class: gap in the diagnostic. The only hint is the `/1` to `/2` one, at `crates/specify/ess-composition/src/lib.rs:1728-1734`.
3. Already expressible: not applicable. The question is about the message, and no construct changes.
4. Fit: every refusal goes through `Admission::admit` (`lib.rs:1690-1747`), including role ends (`src/roles.rs:302`). The export rule differs by kind (`ResolvedService::exports`, `lib.rs:980-999`): a type or view is exported through `owns`, a command through `accepts`, an event when it is published or emitted by an accepted command, and an error when it is raised by an accepted command. A hint about `owns` alone would be wrong for a command, so the hint names the rule for the name's kind (the sibling rule). The refusal code and the `service` field stay the same.
5. A second adopter: a billing consumer selects `invoicing` and names `billing.ledger.Entry`, which only `ledger-writer` owns. That consumer needs to be told to select `ledger-writer` or to widen `invoicing`'s `owns`.
6. Cost: no format change, no new `CompositionCode`. The `repair` text in the catalogue changes, so `website/docs/reference/diagnostics.md` is regenerated (`lib.rs:769-775`, page at `diagnostics.md:329`). One existing test reads this text, `tests/roles.rs:259-270`, and only through `contains("demo.audit.Record")` and "`relay`", so it still holds.
7. Alternatives: (a) change nothing, which leaves the adopter unable to repair from the message; (b) the requester's hint as written, which is right for types and views and wrong for commands, events and errors; (c) chosen: a hint for each kind. (c) costs no more than (b) and is never wrong.

## Decisions

Accept, redesigned. After the existing detail, the refusal adds: "`<name>` is in domain `<D>` of service `<S>`". It then adds one of the following, depending on kind:

- type or view: "component(s) `<C2>` own it: select one, or add `<D>` to `<C>`'s `owns`", or, when no component owns it, "no component owns `<D>`: add it to `<C>`'s `owns`";
- command: the components whose `accepts` names it, or "add it to `<C>`'s `accepts`";
- event: the components that publish it or accept a command that emits it;
- error: the components that accept a command raising it.

Under `/1`, the existing "referenceable under ess-composition/2" clause stays exactly as it is. Component lists are sorted from the `ir.components()` `BTreeMap`. The domain is found through `ir.domains()[..].{types,commands,events,errors}` (`ess-compiler/src/ir.rs:3204-3224`). Change from the request: the remedy depends on the kind instead of always being about domains.

No format moves: the refusal's code and fields are the same, so this is not a `ess-composition/N` change.

## Acceptance

- `the_refusal_names_domain_and_service`: `demo.loose.Note` under `/4` names `demo.loose`, service `demo` and "add `demo.loose` to `relay`'s `owns`" (`cargo test -p ess-composition --locked --test outside_component the_refusal_names_domain_and_service`).
- `the_refusal_suggests_an_owner`: `demo.audit.Entry` names component `auditor` (same test file).
- `a_command_outside_accepts_names_the_accepting_component`: `demo.audit.Record` names `auditor` and `accepts`, not `owns` (same test file).
- `an_event_outside_the_surface_names_its_publisher`: same test file.
- `the_v1_hint_is_kept`: `demo.gate.Unused` under `/1` still says "referenceable under ess-composition/2" (same test file).
- The diagnostics page matches the catalogue: `cargo xtask diagnostics --check` (run by `task test-xtask`).

## Scope

- `crates/specify/ess-composition/src/lib.rs`: `Admission::admit` hint (1690-1747), catalogue `repair` text for `ReferenceOutsideComponent` (769-775).
- `crates/specify/ess-composition/tests/outside_component.rs`: new.
- `crates/specify/ess-composition/tests/fixtures/outside-component/`: new fixture (the `demo` and `wire` systems, with a `demo.loose` domain no component owns). Do not edit `fixtures/contract-role/`, whose IR bytes are pinned by `expected/composition-3.ir.json`.
- `website/docs/reference/diagnostics.md`: regenerated by `cargo xtask diagnostics`.
- Shared with `story:a-composition-conformance-may-name-an-event-or-command` and `story:a-type-conformance-compares-value-constraints-and-variant-attributes` (`lib.rs` catalogue, `diagnostics.md`).

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
