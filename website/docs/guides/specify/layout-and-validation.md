---
title: Lay out and validate a specification
sidebar_position: 1
description: The files a specification is made of, where generated output goes, how to read validation refusals, how to check what resolved, and which names a reader sees.
---

# Lay out and validate a specification

## Layout

```text
system.yaml            format version, the system's name, which domains it has
domains/invoice.yaml   one bounded context: types, entities, commands, events, errors, views
domains/email.yaml     a second, so cross-domain references are real rather than assumed
components.yaml        which component owns which domain, the bindings between them, conversions
topology.yaml          what the system needs at runtime to be correct
```

Those five files are `examples/billing/`, and the `5 file(s)` in every validate line below is them.

One file works too: `ess specify validate --path spec.yaml` reads a single file carrying the header
and the members, and splitting into a directory later changes nothing about invocation. A file names
at most one `domain:`, so the one-file form is for a one-domain system; `components:`, `bindings:`
and `topology:` can sit in it as well.

The header's `domains:` list is checked in both directions, and both refusals name the fix:

```text
- [undeclared_reference] system.domains: `tiny.audit` is listed as a domain of the system, and no source declares it (hint: declared domains: tiny.core)
- [conflicting_declaration] domain tiny.core: `tiny.core` is declared, and the system header does not list it (hint: the header's `domains:` says what the system has; add it there, or drop the source that declares it)
```

Point your editor at `schemas/generated/ess.schema.json` and field names are checked as you type.
The schema is generated from the same Rust types the validator runs. The repository's authoritative
offline gate is `task check`, which exercises the schema contract alongside the workspace.

## Keep sources and generated output together

This configuration was introduced in 0.21.0. For a mixed directory, add an immediate
`ess-inputs.yaml` and pass that directory:

```yaml
format: ess-inputs/1
specification:
  - model/system.yaml
  - model/domains/invoice.yaml
  - model/domains/email.yaml
  - model/components.yaml
  - model/topology.yaml
scenarios:
  - authored/routing/first.yaml
  - authored/e2e/second.scenario
```

```text
ess-inputs.yaml
model/                  the five billing model files above
authored/               the two explicitly listed scenario files
generated/openapi/      unlisted generated YAML
generated/asyncapi/     unlisted generated YAML
output/                 unlisted compiler and runner output
```

`ess specify validate --path .` assembles the `specification` entries. Nested or renamed
headers work, and headerless fragments retain their existing meaning. The selected files must
still assemble into one valid specification. Unlisted generated files, including malformed YAML
or copies of model fragments, are never scanned. List order does not affect selection.

When the `scenarios` list is nonempty, `validate` also compiles each listed scenario against the
model with the checks `ess verify conform synthesize --scenarios .` applies before it runs. A
scenario that step would refuse fails `validate` too, with the same `ESS-AUTHOR-*` refusal on
stderr and exit code `1`. The summary counts what it checked:

```shell-session
$ ess specify validate --path .
billing v3 — 5 file(s), 2 scenario(s), valid
```

With an empty `scenarios` list, or without a manifest, `validate` reads no scenarios and the
summary does not mention them. `--format json` adds `scenarios`, the number listed, and
`scenario_refusals`, one entry per refusal with its `code`, `origin`, `scenario` and `message`.
Each key is present only when it has something to report.

Both lists are required and checked for valid, distinct relative paths, even when one role is
inactive. Only files in the active role must exist. An empty active list refuses. Paths are literal,
case-sensitive UTF-8 identities: no empty or dot segments, backslashes, colons or control characters.
The selected root, manifest, listed files and intermediate directories below the root must not be
symlinks. Use real contained files. The manifest declares selection, not file ownership or authorship.

Without an immediate manifest, existing recursive model discovery still requires `system.yaml`
and reads every lowercase `.yaml`/`.yml` below the directory. Use a separate model input directory
for that layout. An explicit file is always one source, independent of its extension; it never
searches its parent for configuration. A malformed or unsupported `ess-inputs.yaml` refuses without
falling back: rename a legacy source with that reserved filename or adopt this configuration.
See the [complete input format](../../reference/formats.md#directory-input-configuration).

### Name the `ess` release the specification is maintained with

`format: ess-inputs/2` adds one optional field, `requires`, naming the `ess` release the
specification is validated and generated with — an exact release, or a minor line:

```yaml
format: ess-inputs/2
requires: ess 0.32     # or `ess 0.32.1` for exactly that release
specification: [model/system.yaml, model/domains/invoice.yaml]
scenarios: []
```

An older `ess` refuses with exit `1`, naming the required release and how to get it: `b10x upgrade`,
or `/ess:upgrade` in an agent session. A newer `ess` prints one warning per command and continues;
`--strict-requires` refuses instead, which is the spelling for CI. Without `requires` nothing
changes. Generated output records the release that produced it, and a regeneration by a different
release that changes the output prints a `note:` naming both.

## Validate early, read the refusals

```shell-session
$ ess specify validate --path examples/billing
billing v3 — 5 file(s), valid
```

Break a reference and the refusal names what was available. Take a copy of `examples/billing/`,
misspell `InvoiceCreated` as `InvoiceRaised` in the `accepted` branch's `emits:` and `payload:` —
those two occurrences only, not every one in the file — and run it:

```shell-session
$ COPY=$(mktemp -d)/billing && cp -r examples/billing "$COPY"
$ # in $COPY/domains/invoice.yaml, rename `InvoiceCreated` to `InvoiceRaised` in the
$ # `accepted` outcome's `emits:` list and its `payload:` key
$ ess specify validate --path "$COPY"
…/billing was refused:
  - [undeclared_reference] command.billing.invoice.CreateInvoice.outcomes.accepted.emits: `billing.invoice.InvoiceRaised` is not a declared event (hint: declared events: `billing.email.DeliveryEscalated`, `billing.email.EmailSent`, `billing.invoice.InvoiceCancelled`, `billing.invoice.InvoiceCreated`, `billing.invoice.InvoiceIssued`, `billing.invoice.InvoicePaid`)
  - [undeclared_reference] command.billing.invoice.CreateInvoice.outcomes.accepted.instance: outcome `accepted` of `billing.invoice.CreateInvoice` acts on the instance named by `invoice_id`, which is no field of an emitted event of it (hint: the field of an emitted event must be typed `billing.invoice.InvoiceId` — declared: none are declared)
  - error[ESS-COMMAND-001]: `billing.invoice.InvoiceRaised` is not a declared event
  … structured diagnostics continue with source locations and repair hints …
```

Every problem is reported in one run — one typo, two consequences, both stated, and the exit code is
`1`. The second is the more useful of the two: the branch says it creates an invoice and publishes
its identity in an emitted event, and the misspelling took away the event that was carrying it.

## Check what you just wrote resolved

`ess specify compile --path <specification> --format json` emits complete top-level `views`,
including each view's `source`, `fields`, `consistency` and `naming.wire`. A domain's `views`
list contains references into that map. Adapters should consume those declarations directly;
no second YAML reader is needed. `--out <file>` writes the same canonical JSON bytes.

`ess specify validate` says the document holds together. `ess specify inspect` shows what one
declaration *became*,
with every reference in it resolved — the fastest way to find out whether the thing you meant is the
thing the model read:

```shell-session
$ ess specify inspect --path examples/billing billing.invoice.CreateInvoice
commands:
  domain: billing.invoice
  input:
  - name: account_id
    type_ref:
      kind: declared
      name: billing.invoice.AccountId
  - name: customer_email
    type_ref:
      kind: declared
      name: billing.invoice.Email
  - name: amount
    type_ref:
      kind: declared
      name: billing.invoice.Money
  name: billing.invoice.CreateInvoice
  naming:
    display: Create invoice
    wire: create-invoice
  outcomes:
  - condition:
      kind: when
      predicate: amount.amount > 0
    name: accepted
    test_strategy: construct_input
  - condition:
      kind: otherwise
    error: billing.invoice.InvalidAmount
    name: rejected
    test_strategy: default_branch
```

`kind: otherwise` is the line to read: the specification names no condition for that branch, and the
model derived one. `construct_input` and `default_branch` say how a generated scenario will reach
each branch, before any suite exists. The actual YAML includes the resolved payload and subject
records as well; the excerpt above keeps only the fields relevant to that question.

On a binding it resolves the crossing as well, reason included:

```shell-session
$ ess specify inspect --path examples/billing notify-on-invoice-created
bindings:
  command: billing.email.SendEmail
  delivery: at_least_once
  escalation: billing.email.DeliveryEscalated
  event: billing.invoice.InvoiceCreated
  failure: escalate
  mapping:
  - conversion: An invoice's customer email is a deliverable address; the email context validates it again on the way out, so the invoice context does not have to know how.
    target: recipient
    target_type:
      name: billing.email.EmailAddress
    value:
      field: customer_email
      kind: event_field
      type_ref:
        name: billing.invoice.Email
  name: notify-on-invoice-created
```

`--kind` is only needed when one name is used in two namespaces; the seven it accepts are `domain`,
`type`, `command`, `event`, `error`, `binding` and `component`.

## Names

| Name | Example | Who reads it |
|---|---|---|
| qualified name | `billing.invoice.CreateInvoice` | the specification, and only it |
| wire name | `create-invoice` | HTTP paths, topics, generated JSON |
| display name | `Create invoice` | generated documentation, a UI |
| locator | `ep://acme/billing/ess-command/billing.invoice.CreateInvoice` | anything outside |

Conflating any two costs a rename later: an HTTP path that changes because someone improved a domain
term is an outage caused by a wording fix.

Field `wire` overrides change JSON property keys, not logical field identities.
Effective wire keys must be distinct within each object: structs, command inputs,
event/error payloads, entity identity/fields/state and view rows. View parameters have
their own namespace. An entity field or identity cannot use the reserved `state` wire
key. Validation accumulates collisions before any projection can overwrite a property;
display names and identical keys in separate objects do not conflict.
