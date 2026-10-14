# A component plays a pinned contract's component

beyond10x/ess#525. Composition format `ess-composition/4`. The source format (`ess/N`), the
client plan (`ess-client-plan/1`) and the suite formats do not move.

## The gap

A service specification (system `demo`, component `relay`) implements the server side of a
protocol that another specification publishes as its own system (system `wire`, component
`wire-server`), released and pinned by digest. The service wants the dependency checked: `relay`
plays `wire-server`, and `demo.gate.Admit` decides `wire.server.Subscribe` before it succeeds.

Nothing expressed it. A `plays:` key on the component is an unknown field, a binding to
`wire.server.Subscribe` is `ESS-BINDING-001`, listing the contract's domain in `demo` is
`ESS-SPEC-004`, `roles:` under `ess-composition/3` is an unknown field, and a conformance between
two commands is `unresolved_semantic_reference`: composition compares types only
([composition-type-conformance.md](composition-type-conformance.md)).

## Where it lives

In the composition, not in `ess/N`. A composition already pins every service by system, version and
compiled-model digest and selects one component per service key; a role is a relation between two
such pins. A `plays:` key on the component would put a digest pin of another system into the source
format, which has no construct that pins another specification, and would pull composition into
`EssIr`, which the crate boundary keeps apart.

## The construct

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

A role says that the component selected for `service` plays the component selected for `plays`.
Both are service keys of this composition, so both components and both digests are the ones the
`services` entries pin. `commands` (optional, empty when absent) says which local command stands
where in the contract. `stands` is a closed set:

| `stands` | meaning | checked |
|---|---|---|
| `implements` | the local command answers the contract command | both ends exist on their selected surfaces, and the inputs compare (below) |
| `gates` | the local command decides before the contract command succeeds | both ends exist on their selected surfaces; the ordering is recorded, not executed |

Any other value is refused when the document is read (`unknown variant`, naming the two).

`/4` is otherwise `/3`: `references`, `conformances` and `reader: true` keep their meaning.

## The checks

In document order, every refusal accumulated, as the rest of `compile` does:

1. `roles[i].service` and `roles[i].plays` each name a key the composition imports, or the role is
   refused as **`unknown_role_service`**, naming `roles[i].service` or `roles[i].plays`.
2. A role whose `service` and `plays` are the same key, a second role for a pair already declared,
   and a command entry repeated within a role are refused as **`conflicting_role`**, naming the
   entry.
3. Each command end is admitted like a reference: the local command against `service`'s selected
   component, the contract command against `plays`'s. A command must be on the component's
   `accepts` surface. A name the model does not declare is `unresolved_semantic_reference`; one it
   declares outside the selected component is `reference_outside_component`. Each detail starts with
   `roles[i].commands[j].local` or `roles[i].commands[j].contract`.
4. An `implements` entry whose ends are both admitted compares their inputs. The contract's caller
   sends the contract command's input, and the local command must accept it:
   - every contract input field is a local input field of the same name and wire name, whose type
     conforms by the field comparison `conformance.rs` applies to type fields (named types compared
     recursively, the one tolerance a local value `Optional` where the contract's is required, the
     same `presence` where both are `Optional`);
   - a local input field the contract does not declare must be `Optional` and not
     `null_when_absent`, because the contract's caller never sends it. The issue's decision states
     the first rule only; the second follows from "answers": a local command that requires a value
     the caller never sends refuses every call.

   A difference is refused as **`role_input_drift`**, naming the entry, both commands and every
   differing field.
5. A `gates` entry compares nothing beyond step 3. Its inputs may differ: the gate decides on its
   own input, and the composition does not say how the contract's input becomes it.

## The compiled form

```json
"roles": [
  {
    "service": {"key": "demo", "system": "demo", "version": "v1", "source_digest": "…", "component": "relay"},
    "plays": {"key": "wire", "system": "wire", "version": "v1", "source_digest": "…", "component": "wire-server"},
    "commands": [{"local": "demo.gate.Admit", "stands": "gates", "contract": "wire.server.Subscribe"}]
  }
]
```

Roles keep document order, and so do their command entries: the order is the author's, and
nothing sorts it. Each end carries the system, version, digest and component it was checked at, so
a reader of the compiled composition sees which pin each role was checked against without joining
`services`.

## Not in this construct

- **Delivering a local event through the contract's delivery** (the issue's third clause). It needs
  a transport-level role, which the composition does not model; it is left to a later story.
- **Outputs.** An `implements` entry compares inputs. Outcomes, emitted events and errors are not
  compared: the contract command's outcomes are what the contract's caller observes, and relating
  them to the local command's needs the same transport-level role.
- **Generated clients.** The client plan does not change; a role adds no operation.

## Compatibility

- `SUPPORTED_COMPOSITION_FORMATS` is `[1, 2, 3, 4]`; `cargo xtask docs` reads it.
- A `/1`, `/2` or `/3` document carrying `roles`, even `[]` or `null` (YAML `roles:` or
  `roles: ~` included), is refused when read as an unknown field, as before `/4` existed: the
  reader records that the key was written, not only its value.
- Under `/4`, `roles: null` reads as the absent key: no roles, nothing written for it, and the
  compiled bytes of the document without the key
  (`tests/roles.rs`, `a_composition_4_document_with_null_roles_reads_as_without_the_key`). It
  states nothing a reader could act on, so refusing it would only refuse an empty YAML key, and
  `conformances: null` already reads the same way under `/2` and `/3`. Every earlier document reads and compiles exactly as before: its
  compiled bytes do not change, because `roles` is omitted when empty
  (`tests/roles.rs`, `composition_3_documents_keep_their_ir_bytes_under_the_4_reader`).
- An older reader refuses a `/4` document: its `roles` key as an unknown field, and a `/4` document
  without one as `unsupported_format`.
- `ess specify compose` reads the document through the same closed reader and writes the compiled
  form with the roles; the CLI needs no change beyond its help text.
