---
format: aep.planning-md/3
id: story:a-component-plays-a-pinned-contract-role
kind: story
status: draft
title: A component plays a component of a pinned contract specification, checked by compose
tags:
- adopter-report
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#525
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
---
## Outcome

A composition can state that one imported service's component plays a component of another
imported service pinned by digest (a contract role), and which of its commands stands where in that
contract. `ess specify compose` checks the claim against both compiled surfaces at their digests and
records it in the composition IR, instead of a comment asserting it. Planned for wave k (0.61.0).

## Evidence

https://github.com/beyond10x/ess/issues/525 (ess 0.57.0, neutral reproducer `repro.sh` in the issue:
system `demo`, component `relay`, contract system `wire`, component `wire-server`). Every form is
refused today: a `plays:` key on the component (unknown field), a binding to
`wire.server.Subscribe` (ESS-BINDING-001), the contract's domain listed in `demo` (ESS-SPEC-004),
`roles:` in `ess-composition/3` (unknown field), and a command-to-command conformance
(`unresolved_semantic_reference`). Composition admits type conformance only
(`docs/design/composition-type-conformance.md`).

## Fit review

Written with `.agents/skills/assessing-external-requests/SKILL.md`.

1. **The need, apart from the syntax.** A service specification depends on a protocol contract
   published as its own system, pinned by digest, and wants the dependency checked: which of its
   components plays which contract component, and which of its commands answers, or gates, which
   contract command.
2. **Class: gap.** No form expresses it (the five refusals above). Not a defect.
3. **Already expressible? Only in part.** Type conformance (`ess-composition/2`, `conformances:`)
   checks that a local type has an imported type's shape. Nothing relates a command to a command or
   a component to a component.
4. **Where it lives: the composition, `ess-composition/4`, `roles:`.** A composition already pins
   every service by system, version and source digest and selects one component per service key
   (`crates/specify/ess-composition/src/lib.rs:215`, `ResolvedService` at 764-777). A role is a
   relation between two such pins, which is what a composition document holds. The requester's
   other form, a `plays:` key on the component in `ess/N`, would put a digest pin of another system
   into the source format: `ess/N` has no construct that pins another specification, it would need
   a new `ess/N` for a cross-system edge the composition already models, and the crate boundary
   keeps composition out of `EssIr` (AGENTS.md, "Boundaries"). Rejected for that reason.
5. **A second adopter?** Any service implementing a published protocol (the OAuth specifications
   behind https://github.com/beyond10x/ess/issues/493 are the same shape). Plausible, not observed.
6. **Cost.** About 3 units: design page, reader and checks with the IR, CLI and docs.

## Decisions

- **adopt as redesigned**: `ess-composition/4` with `roles:`. Shape:

  ```yaml
  format: ess-composition/4
  composition: demo-wire
  services:
    - {key: demo, system: demo, version: v1, source_digest: <sha256>, component: relay}
    - {key: wire, system: wire, version: v1, source_digest: <sha256>, component: wire-server}
  roles:
    - service: demo
      plays: wire
      commands:
        - {local: demo.gate.Admit, stands: gates, contract: wire.server.Subscribe}
  ```

- `stands:` is a closed set: `implements` (the local command answers the contract command: every
  contract input field is a local input field of a conforming type, by the comparison
  `conformance.rs` applies to type fields) and `gates` (the local command decides before the
  contract command succeeds; both ends are checked to exist on their selected surfaces, the
  ordering itself is recorded, not executed).
- Not in this story: delivery of a local event through the contract's delivery (the issue's third
  clause). It needs a transport-level role and is left as a later story; the issue's closing
  comment says so.
- Format consequence: `/4` is a new composition format; `/1` to `/3` documents keep their IR bytes,
  and a `/3` document carrying `roles:` is refused as an unknown field, as today.

## Acceptance

Failing tests first, in a new file `crates/specify/ess-composition/tests/roles.rs`, with the
issue's two systems as a fixture under `crates/specify/ess-composition/tests/fixtures/contract-role/`:

- `a_component_plays_a_contract_component_and_the_role_is_in_the_ir`: the issue's reproducer under
  `/4` composes; the IR lists the role with both services' digests and the command entries in
  order.
- `a_role_naming_an_unimported_service_is_refused` (new code, naming `roles[i].plays`).
- `a_role_command_outside_its_selected_component_is_refused`: a local command `relay` does not
  accept, a contract command `wire-server` does not accept; each names the entry.
- `an_implements_entry_whose_inputs_drift_is_refused`: a contract input field missing locally, or
  of another type, names the field.
- `a_gates_entry_is_recorded_without_an_input_comparison`.
- `a_stands_value_outside_the_set_is_refused`.
- `composition_3_documents_keep_their_ir_bytes_under_the_4_reader` (old-reader compatibility).
- `crates/specify/ess-composition/tests/diagnostic_catalogue.rs` covers each new
  `CompositionCode` variant; `task test-xtask` passes (rendered diagnostics page).
- A design page `docs/design/composition-contract-roles.md` precedes the code.

## Scope

| surface | confidence |
|---|---|
| `crates/specify/ess-composition/src/lib.rs`: format constants (49-55), `CompositionSpec` (432), `CompositionCode` and `CATALOGUE` (549-600), `EssCompositionIr` (855) | cited |
| `crates/specify/ess-composition/src/conformance.rs` (field comparison reused) | cited |
| `crates/specify/ess-composition/tests/roles.rs`, `tests/fixtures/contract-role/` | inferred (new) |
| `docs/design/composition-contract-roles.md` | inferred (new) |
| `website/docs/reference/formats.md`, `website/docs/reference/diagnostics.md` (rendered) | inferred |
| `crates/edge/ess-cli` compose command (accepts `/4` through the library) | inferred |

## Delivery

- depends on: nothing in wave k (no file shared with the `ess-http/1` units)
- du: about 6G (inferred: `ess-composition` and `ess-cli` package gates)
- adversary: yes (a new persisted format)
- closes: https://github.com/beyond10x/ess/issues/525
