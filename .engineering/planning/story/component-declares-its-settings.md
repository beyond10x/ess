---
format: aep.planning-md/1
id: story:component-declares-its-settings
kind: story
status: draft
title: A component declares its settings, and the runtime slots are derived from them
relations:
- decomposes: epic:configuration-declared-once
scope:
- confidence: cited
  path: crates/generate/ess-deployment/src/runtime.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 2
---
# Story: a component declares its settings, and the runtime slots are derived from them

## Defect

`ess-runtime/1` types configuration — `ConfigSlot`, `SecretSlot`, `EndpointSlot`,
`crates/generate/ess-deployment/src/runtime.rs:18-67` — and each slot carries
`pub environment: String`, so a slot is an environment variable bound into a container. The document
is hand-authored, and nothing in `ess/1` says what any of those values *are*. Two facts about one
setting live in two places and agree by inspection.

## Shape

`RawComponentSpec` (`crates/specify/ess-domain/src/component.rs`, `deny_unknown_fields`) gains one
optional `settings:` list. Each entry:

| field | meaning |
|---|---|
| `name` | the setting's identity, in the `CliName` charset |
| `type` | any type the specification declares — reuses `types:` entirely, invariants included |
| `required` | absent means optional |
| `secret` | the model may not hold the value; refuses a literal `value:` beside it |
| `summary` | one line, for generated documentation |

One list and nothing else. Configuration is unbounded per application, and borrowing the whole type
system rather than growing a config language is the guard against that.

```yaml
components:
  - component: connectors-cli
    settings:
      - name: state_root
        type: connectors.config.StateRoot
        required: true
      - name: slack_bot_token
        type: connectors.config.Credential
        secret: true
```

## Refusals

Each uses an existing `ValidationCode`; no new codes.

| refused | code |
|---|---|
| a setting whose `type` nothing declares | `undeclared_reference` |
| two settings with one name on a component | `duplicate_declaration` |
| `secret: true` beside a literal `value:` | `conflicting_declaration` |
| a setting typed by an **entity** rather than a type | `type_mismatch` |
| `required: false` with no `Optional<…>` type | `conflicting_declaration` |

The fourth is the load-bearing one: it stops configuration from becoming a second entity model. A
setting has no identity and no lifecycle, and an entity has both.

## The projection that pays for it

`ess/AGENTS.md` requires a projector to establish the semantics before the types are added, so this
story carries exactly one, and it is a document that is hand-authored today:

- derive `ConfigSlot.environment` from the setting name, upper-snake-cased, deterministically;
- derive `ConfigKind` from `required` and the presence of a literal;
- derive `SecretSlot.key` from the setting name;
- **refuse** an `ess-runtime/1` document that hand-authors a slot for a component that declares
  settings, naming both — two sources for one fact is the state this removes.

A component that declares no settings keeps its hand-authored slots unchanged. That is what makes
this additive rather than a migration.

## Format

`ess/1`, on the precedent `reached_by` and `cli:` both set: the field serialises out when unset, so
every existing document digests exactly as before. Part of this story, not a follow-up:

- an old-reader test proving a document without `settings:` produces byte-identical IR;
- the forward-incompatibility written down — an older binary refuses a document that uses it;
- a `changes/*.yaml` fragment, `schema: b10x-change/v1`.

## Acceptance

- `ess validate` accepts a component declaring `settings:`, and each of the five refusals has a test
  that names it.
- `ess specify runtime compile` derives the slots for a component that declares settings, and two
  runs produce identical bytes.
- A hand-authored slot beside a declared setting is refused, naming both.
- The old-reader test passes: IR bytes unchanged for documents without `settings:`.
- `task check` exits 0.

## Out of scope

- Any adapter — a TOML reader, env, keyring, Vault. This story makes them cheap and builds none.
- `EndpointSlot`. It binds to a stack service rather than to a value, and deriving it needs the
  composition layer to have an opinion this story does not.
- A defaults or templating language.


## Integration Provenance

Reconciled through AEP from wt-34496b6990aa at original revision 5 and status draft. Source artifact SHA-256: 5cdc89236cfd4eada84383c5d50bebaa35155dd8b9f964414f294741ec18f6fd. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.
