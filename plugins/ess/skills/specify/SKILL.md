---
name: specify
description: >-
  Validate Executable System Specifications and guide deterministic JSON Schema or OpenAPI projections through the `ess` command. Use when a repository contains an ESS `system.yaml` or `ess-inputs.yaml`, generated schema or OpenAPI artifacts, an `ess-*` format, when a story or epic introduces an entity — a noun the plan needs a typed home for, whether or not a specification exists yet, including a repository with no `system.yaml` anywhere, where the domain is drafted from nothing — or when the user asks about specification validation, compilation, schema generation, projection drift, adapter coverage, or unsupported semantics. An entity introduction names the noun and something typed about it: an identifier, a field, or a relation to another noun. A story that mentions a noun in passing, an acceptance line, a status change or a plan's prose is not one.
---

# ESS schema validation and projection

Treat the Rust-owned specification model and compiler as the authority. Do not infer a contract
from generated artifacts and do not repair a failed import by inventing lifecycle meaning.

## Starting a domain from nothing

A story or epic can introduce a noun before any specification exists. That is still this skill's
job: the domain is drafted first, so the noun has a typed home before stories are written around it.

Where an OpenAPI document already describes it, do not hand-write the domain. Draft it from the
contract, and read the decisions the draft says it could not take:

```console
aep plan reverse openapi --domain <domain> <openapi-document> --out <file>
```

Otherwise write the smallest document that validates — two files, and nothing that is not required:

```yaml
# system.yaml
format: ess/1
system: warehouse
version: v1

domains:
  - warehouse.shipment
```

```yaml
# domains/shipment.yaml
domain: warehouse.shipment

entities:
  - name: warehouse.shipment.Shipment
    identity:
      name: shipment_id
      type: Uuid
    fields:
      - name: destination
        type: String
    relations:
      - name: lines
        kind: owns
        target: warehouse.shipment.ShipmentLine
        cardinality: many
        via: shipment_id
    lifecycle:
      initial: Draft
      states: [Draft]
      terminal: [Draft]

  - name: warehouse.shipment.ShipmentLine
    identity:
      name: line_id
      type: Uuid
    fields:
      - name: shipment_id
        type: Uuid
      - name: sku
        type: String
    lifecycle:
      initial: Draft
      states: [Draft]
      terminal: [Draft]
```

Then check it before anything is written around it:

```console
$ ess specify validate --path <specification>
warehouse v1 — 2 file(s), valid
```

That is the verbatim output of ESS `0.29.0`, exit status 0, over both files as printed—including the
`relations:` block and its second entity. It is a validated starting point, not an illustrative
shape. Change the names and semantics to match the repository you actually read, then validate the
changed specification again.

Every line above is load-bearing, and the refusals say why. An entity without `identity`, `fields`
or `lifecycle` is refused as a missing field. A state with no outgoing transition must be listed
under `terminal:`, or the compiler refuses it as `dead_end_state`. A transition no command outcome
takes is refused as `missing_causation` — so a first domain has one state and no transitions, and
gains a second state only together with the outcome that moves it. The header's `domains:` list and
the declaring source must agree in both directions; either half alone is a refusal.

**A relation is an entry, not a convention.** Two entities linked by a field that happens to be
named after the other one are not related as far as anything can check; the `relations:` entry is
what makes the link a fact a program reads. Three more refusals come with it, and they are the
reason it is worth writing:

| Refused | What it means |
|---|---|
| an unknown `target` | the far entity is not declared anywhere in the specification. A relation to a noun nobody typed is the failure the guardrail exists to catch, and it now fails at `validate` rather than at the first schema projection |
| a missing or mistyped `via` | the linking field is not there, or it is not the type of the identity it is supposed to carry. `via` lives on whichever side holds the field: on the **target** for `owns` — the child's field typed by the owner's identity — and on the **source** for `references` |
| a second owner | two entities both claiming to `own` the same one. Ownership decides what a delete does, and two answers to that is not a richer model, it is an undecided question written down twice |

`owns` and `references` are not stylistic. `owns` says the far side does not stand on its own — it
has no meaning without its owner, so a delete of the owner cannot leave it behind; `references` says
it does stand on its own and outlives the link. What the delete *does* — refuse while children
remain, or take them with it — is a command outcome and not a field of the relation, so `owns`
narrows that question without answering it. Where you cannot say which kind it is, that is the
`UNMAPPED:` case below and not a coin toss.

Grow it from there — types, commands, events, views, and a component that owns the domain — running
`ess specify validate` after each addition rather than at the end.

`--path` takes one ESS file or a directory. A directory is read through its `ess-inputs.yaml` when
one exists — an exact file list, which is what lets authored inputs sit in nested directories beside
generated files (ESS 0.21.0) — and otherwise through the `system.yaml` layout above, which the CLI
calls legacy and still accepts.

**A draft is a proposal, never a silent completion.** Every relation you could not read from code,
an OpenAPI document or an existing artifact is written with an `UNMAPPED:` marker beside the place
it would go, and named again in the report:

```yaml
      # UNMAPPED: the epic says a shipment has a carrier; no code, contract or
      # artifact here says what a carrier is. Ask before typing it.
```

**A relation whose cardinality or ownership you cannot read is `UNMAPPED:`, not an entry with a
plausible value.** `cardinality: many` and `cardinality: one` project different schemas and imply
different stories, and `owns` against `references` decides whether the far side can stand on its own
at all — so a guess there is not a smaller guess than inventing the relation. Write the marker
where the entry would go:

```yaml
      # UNMAPPED: a shipment has lines, and nothing here says whether deleting the
      # shipment deletes them (owns) or orphans them (references). Ask.
```

Imports never guess, and a domain an agent drafts is an import. Never invent a field type, a
lifecycle state or an edge to make a document validate — leave the marker, and name what would
settle it.

## Read before changing

From the specification root, establish the current answer:

```console
ess specify validate --path <specification>
ess specify compile --path <specification> --format json
```

`validate` and `compile` accumulate diagnostics. Relay every refusal; do not stop at the first or
edit generated output around it.

## Deterministic projections

Use the projection command, never a handwritten parallel generator:

```console
ess generate --path <specification> --kind schema --out <directory>
ess generate project openapi --path <specification> --out <directory>
```

The same typed IR must produce the same ordered files and bytes. Compare a regenerated temporary
tree with the committed tree before replacing anything. A stale committed file is drift; a file no
projection owns is not authority.

## Conformance is a record, not a claim

In the planning store an `executable-system-specification` is `conforming` because a suite ran and
its report says so, never because somebody moved it there (AEP 0.50.0). The report is what crosses
from ESS to AEP:

```console
ess verify conform synthesize --path <specification> --out <suite.json>
ess verify conform run --path <specification> --target <reference> --report-out <report.json>
aep plan artifact evidence <specification-artifact> --from <report.json>
```

`synthesize` writes the suite the specification obliges — nothing for a domain with no commands or
outcomes, so the two-entity draft above yields `0 scenario(s)` and no report worth recording.
`run` holds a built-in reference implementation to it; `--target` lists the ones this binary
carries. `evidence --from` reads the kind, the source and the instant out of the report and refuses
a report of no scenarios or with no `spec_digest`. A report/2 (`--report-format 2`, ESS 0.20.0) is
recorded only beside `--suite <the exact suite JSON it ran>` (AEP 0.55.0). Then
`aep plan artifact set <artifact> --model-digest <hex>` ties the record to the model the suite ran
against, and `aep plan artifact move <artifact> --to conforming` is decided by the store.

## Adapter contract

- Importers produce typed IR plus coverage, diagnostics, and unresolved references.
- Projectors produce artifacts plus obligations and refusals; they never apply infrastructure.
- An importer never guesses missing semantics.
- Round trips are claimed only for an adapter's declared supported subset.
- Source-specific detail belongs in explicit typed structures, not an arbitrary JSON property bag.

When an adapter refuses a construct, keep the refusal visible and name the first concrete type or
semantic rule that would close it. Do not propose a generic facet registry or a new persisted format
merely because one source carries more fields.

## Format changes

A new format version is needed when meaning, identity, reference rules, canonicalization, names, or
the persisted envelope changes. Adding internal Rust capabilities is not enough. Before extending a
strict v1 reader such as `infra-ir/1`, add an old-reader compatibility test; unknown fields are
currently refused.

Finish by running the repository's full gate and report the exact command and exit status.
