---
title: Write a specification
sidebar_position: 1
description: Author an ESS document — the layout, the constructs the model insists on, and the validation errors that teach the model fastest.
---

# Write a specification

This guide covers authoring an Executable System Specification. The normative example is
`examples/billing/` in the repository — deliberately the smallest system that exercises the
current `0.27.0` model. Concepts are covered in [ESS](../concepts/ess.md); this page is about writing
one.

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
See the [complete input format](../reference/formats.md#directory-input-configuration).

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

The `ess/2` format, introduced in 0.20.0, adds `Binary64` for finite IEEE-754 values. Use it
when a source contract requires binary floating-point rounding and signed zero:

```yaml
format: ess/2
system: sample
version: v1
domains: [sample.settings]
domain: sample.settings
types:
  - name: sample.settings.Ratio
    kind: newtype
    of: Binary64
```

`Integer` retains exact signed integer identity; `Decimal` retains its patterned
string representation. Neither implicitly assigns to `Binary64`. Map keys cannot
be Binary64. Format 1 refuses the new primitive, including fields in headerless
fragments. Authored numeric predicates may compare Binary64 to numeric literals,
using the existing Number predicate rules; that comparison does not construct a
floating value.

The qualified executable boundary is [format-5 normalization](generate-artifacts.md).
Standalone structural Rust/Go codecs and whole-system/conformance targets currently
refuse Binary64; TypeScript structural output reports the finite codec obligation.
Adding a Binary64 type to a model selected in full therefore requires checking
every intended target's support before adopting it.

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

## What the model insists on

These are the authoring decisions that surprise people coming from OpenAPI-first or prose designs.
Each exists to keep a generated test honest. Every block below is an excerpt of
`examples/billing/`, abridged to the construct being explained.

### A command that can be refused says so

Not an `emits` list — **outcomes**. From `domains/invoice.yaml`:

```yaml
outcomes:
  - name: accepted
    when: amount.amount > 0
    creates: billing.invoice.Invoice
    instance: invoice_id
    emits:
      - billing.invoice.InvoiceCreated

  - name: rejected
    error: billing.invoice.InvalidAmount
```

A command with a precondition has at least two results. A specification recording only the happy one
generates a suite that never checks the branch where the money does not move.

Two guards may both hold of one input. A refusal with a `when:` over the input is taken before any
accepting branch whose guard it overlaps, whatever order they are written in. With `closed: open ==
false` and `id-required: ticket_id == ""`, the input `{ticket_id: "", open: false}` is refused as
`id-required`, and the generated suite sends it and requires that. An accepting branch cannot read
the identity to step aside, so this precedence is how such a command is written.

### An invariant reads only what every creation sets

An entity invariant that reads a required field needs every `creates:` branch of that entity to set
the field. Without a value, `reminder_count >= 0` would hold only if the implementation happened to
pick a value that satisfies it. `validate` refuses the gap with `ESS-COMMAND-018` at the creating
outcome. The fix is to set the field there:

```yaml
- name: accepted
  creates: billing.invoice.Invoice
  instance: invoice_id
  sets:
    account_id: input.account_id
    total: input.amount
    reminder_count: "0"
```

Or declare the field `Optional<…>` if an instance may lack it. The identity, the lifecycle `state`
and `Optional` fields are never asked for. A field an invariant reads anywhere counts, including
inside `any:`.

A literal in `sets:` or `payload:` may also be written as the YAML value it means:
`reminder_count: 0` over an `Integer` and `paused: false` over a `Boolean` compile to exactly what
`"0"` and `"false"` do. An unquoted number or boolean over a text field or an enum is refused with
the repair, `quote it: label: '0'`. Over any other type it gets the same refusal as its quoted form.
A decimal such as `1.5` is never a literal, quoted or not; read it from an input.

### Cover every declared enum value

Since 0.23.0 a command may omit its default when its input guards
select exactly one outcome for every value of a required, closed enum. For example,
if `status` has the declared variants `Ready` and `Stopped`, the two guards
`status == Ready` and `status == Stopped` cover that input. Synthesis uses the same
declared domain to construct a witness for each reachable branch. An omitted value
or overlapping guards produce a concrete failing assignment.

This proof is bounded to 64 joint assignments and 128 predicate nodes. Required
enum fields, transparent wrappers, equality, membership and supported Boolean
combinations participate. Every referenced input must fit that finite domain;
Optional paths, open types, unsupported expressions and unknown results retain the
requirement for a default. Existing defaults and external outcomes keep their behavior.

### Order two instants

A right-hand side without a dot is a literal, so `when: ends_at > starts_at` compares `ends_at`
with the text `"starts_at"`. Validation refuses it and names the spelling that reads a field:
declare the two `Timestamp` fields in one struct and compare its members.

```yaml
input:
  - {name: window, type: rooms.booking.Window}   # struct {starts_at, ends_at: Timestamp}
outcomes:
  - name: booked
    when: window.ends_at > window.starts_at
```

A `Timestamp` is ordered by the RFC 3339 instant it names, so `+01:00` and `Z` spellings compare
correctly. Against a literal, write an instant: `when: ends_at > "2020-01-01T00:00:00Z"`. Ordering
a `Timestamp` against text that is not an instant is refused. `Duration` has no ordering yet.

### Say which characters a text may hold, and how long it may be

A `String` the implementation restricts to a character set says so with `alphabet:` on its newtype,
and a length limit is `.count`, the number of Unicode scalar values. Both need `format: ess/11`.

```yaml
types:
  - name: keypad.dial.KeySequence
    kind: newtype
    of: String
    alphabet: "0123456789*#ABCD"
commands:
  - name: keypad.dial.SendKeys
    input:
      - {name: keys, type: keypad.dial.KeySequence, example: "12#"}
    outcomes:
      - name: too-long
        when: keys.count > 64
        error: keypad.dial.UnsupportedKey
      - name: sent
        emits: [keypad.dial.KeysSent]
```

Every character of a value is one of the alphabet's, compared exactly, with no normalization or case
folding. Synthesis builds the input's witness from those characters, and a guard such as
`keys.count > 64` from texts of 65 and 64 characters. An `example:` on a command input is the value
the first witness starts from. It is not a constraint, it has to be a value of the input's type, and
only a scalar input takes one. Generated code documents an alphabet and does not enforce it, and
Entity Runtime refuses both an alphabet and a text length by name.

### Say what a text starts with

A `String` newtype whose every value starts with fixed text says so with `prefix:`. It needs
`format: ess/15`.

```yaml
types:
  - name: demo.msgs.Channel
    kind: newtype
    of: String
    prefix: "/"
```

The prefix is literal text, not a pattern. The published JSON Schema carries it as an anchored
`pattern` (`^/`), every witness synthesis builds starts with it (`/channel` for a field named
`channel`), and a literal written for the field that does not start with it is refused. Beside an
`alphabet:` every character of the prefix has to be in the alphabet, and a newtype of a newtype may
declare a longer prefix that starts with the inner one. Entity Runtime lowers the prefix to a
`starts_with` rule.

### Carry any JSON value

`Json` is a primitive for a value the specification does not structure: a body delivered as it
arrived. It needs `format: ess/15`.

```yaml
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
```

It projects to the empty JSON Schema, which every value satisfies, and a suite compares it
structurally: an object with the same members in another order is the same value. It is never a map
key, a predicate never reads one, and no literal spells one, so a payload fills a `Json` field from
an input. Entity Runtime stores it as its own JSON field kind. The Rust, Go, web and CLI code
targets refuse a model that uses it, at every position, until they have a representation for it.

### Select an outcome from the held subject state

`ess/3`, introduced in 0.23.0, allows `when_subject_state` beside an ordinary input predicate:

```yaml
- name: preserved
  when: incoming == Ringing
  when_subject_state: Bridged
  updates: calls.core.Call
  instance: call_id
  emits: [calls.core.Observed]
```

The surrounding model declares that event, entity and input types. Both
conditions must hold. Omitting `when` selects any admitted input in the
named held state. The state comes from the existing subject row; it is not an
extra caller field. All ordinary, state-guarded and default branches must name
the same entity and input identity. A missing row does not become an initial row.

The guard cannot accompany `creates`, `external` or `wrong_state`. Guarded moves
must start in the declared guard state. Validation checks up to 64 joint
state/input assignments for gaps and overlaps using the shared finite coverage
proof; unsupported or open input domains require a genuine default. Runtime
witnesses additionally validate their concrete inputs and invariants.

### Guard an outcome by the subject's stored fields

"Express parcels over 20 kg are refused at dispatch" depends on two fields stored when the parcel
was created, not on anything the dispatch request carries. `ess/9` (since 0.34.0) states it with
`when_subject: {predicate: …}`, a predicate over the declared fields of the entity the command
addresses:

```yaml
commands:
  - name: shipping.parcel.Create
    input:
      - {name: service, type: shipping.parcel.Service}   # enum: Standard | Express
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.parcel.Parcel
        instance: parcel_id
        sets: {service: input.service, weight_kg: input.weight_kg}
        emits: [shipping.parcel.Created]
        payload:
          shipping.parcel.Created:
            parcel_id: {generated: true}

  - name: shipping.parcel.Dispatch
    input: [{name: parcel_id, type: Uuid}]
    outcomes:
      - name: refused-overweight
        when_subject:
          predicate:
            all:
              - service == Express
              - weight_kg > 20
        error: shipping.parcel.ExpressOverweight
      - name: dispatched
        moves: shipping.parcel.Parcel.dispatch
        instance: parcel_id
        emits: [shipping.parcel.Dispatched]
```

The predicate reads the entity's declared fields and nothing else: not the input, which stays in
`when:` beside it, and not `state`, which stays with `when_subject_state:`. The refusal names no
subject of its own and reads the parcel its sibling moves. An `Optional` field may be read; an
absent value is unknown and selects no branch, so write `not defined(field)` to select on absence.

Validation partitions closed enum fields jointly with the input, so two branches that split an enum
need no default. An open comparison such as `weight_kg > 20` needs a genuine default, here
`dispatched`. Declare an unfiltered view that projects the identity, `state` and every guarded
field: conformance arranges a parcel through `Create`'s `sets:` mappings — Express at 21 kg for the
refusal, Express at 20 kg and Standard at 21 kg for the default — observes it through that view,
and dispatches it. A `read_your_writes` view is read once; where the only such view is `eventual`,
the observation goes in an `eventually` block that waits until the view shows the arranged parcel.
A refused parcel is asserted unchanged only through a `read_your_writes` view: an `eventual` view
that has not caught up shows the old row too, so without one that check is left out.
In a state the command does not move from, no branch is selected by the stored fields, and the
refusal scenario sends the command as it does for a command without `when_subject`. The older
`when_subject: {field, equals}` form keeps `ess/6`.

### An outcome the input cannot decide says that too

Whether a mail provider accepts an address is not a function of the request. From
`domains/email.yaml`:

```yaml
- name: failed
  external: the provider rejects the recipient address
  error: billing.email.Undeliverable
```

Writing `when: false` would claim the branch is unreachable — a different statement, and a false
one. A generator reads `external` and injects a fault instead of trying to construct an input.

### One outcome for many commands

When a remote service carries out every command on the caller's behalf, every command can end the
same way: the service rejects the session credential. `ess/12` (since 0.34.0) declares that
outcome once, in a top-level `outcome_groups:` list that any file may carry:

```yaml
format: ess/12
outcome_groups:
  - name: remote-backed
    actor: calls.Agent              # or  commands: [calls.Hold, calls.Resume]
    except: [calls.Park]            # only beside actor: or domain:
    outcomes:
      - name: credential-rejected
        external: the service rejects the session credential
        error: session.Unauthenticated
        summary: The remote service refused the session.
        refs: [issue:beyond10x/ess#105]
```

A group names its members in exactly one way: an explicit `commands:` list, `actor:` for every
command that actor `may:` invoke, or `domain:` for every command that domain's files declare. Each
member gains the group's outcomes after its own, in ascending group-name order when several groups
select it. That happens before validation, so validation, conformance and every generator see
ordinary outcomes, and a group compiles to exactly what copying the outcome into each command by
hand would. A group of 28 commands therefore still adds 28 scenarios. A group's outcome is always an
external refusal: `external:` and `error:`, with an optional `summary:` and `refs:`, and nothing that
depends on the command it lands in.

`except:` is only for `actor:` and `domain:`; with an explicit list, leave the command out of
`commands:` instead. A member that already declares an outcome of the same name is refused at the
group, never silently overridden: rename one of the two, drop the command's own, or list the command
under `except:`. Two groups that would give one command outcomes of the same name are refused too,
and neither group expands into that command.

### Illegal lifecycle moves are illegal by absence

`Paid` cannot become `Cancelled` because no transition says it can. There is no forbidding rule,
because a rule would be a second place for the same truth to live, and two places eventually
disagree. The generated documentation lists the absent pairs, derived from the same transitions.

### A command says what it answers when invoked in the wrong state

One key and one error name — everything else is derived:

```yaml
- name: issued
  moves: billing.invoice.Invoice.issue      # `issue` runs from [Draft]
  instance: invoice_id
  emits:
    - billing.invoice.InvoiceIssued

- name: wrong-state
  wrong_state: true
  error: billing.invoice.InvoiceStateConflict
```

`wrong_state:` names no state: `issue` already declares it runs from `Draft`, so the refused states
are derived. The `error:` is required — without it a generated scenario could only assert that
*nothing happened*, which also passes against an implementation refusing for the wrong reason.

An **unknown instance** — the input selects a `moves:` or `updates:` branch and its `instance:`
names no record — is answered by the command's **not-found outcome** when it declares one: an
`external:` refusal whose `error:` carries a field of the identity's type, such as
`not-found` reporting `DoorNotFound { door_id }`. The suite checks it
under that outcome's id with an identity no other scenario sends, in place of injecting the
external cause. Only a command declaring no not-found outcome falls back to its `wrong_state`
branch, checked under the branch's own id (`…IssueInvoice/outcome/wrong-state`). Input guards are
decided first, so `PayInvoice` with a non-positive amount still answers `rejected` whatever invoice
it names. Either scenario requires the outcome, its error by name and no error field, and that no
declared event is published. A command that acts on an input-named instance and declares neither
has no declared answer, and one declaring two not-found candidates has an ambiguous one.
`ess verify conform synthesize` prints a `note:` for each, not a refusal.

An error field that describes the current state, such as `InvoiceStateConflict.state`, has no value
for an instance that does not exist. Where the `wrong_state` error declares fields, the generated
Rust and Go behaviour seams add a second variant for this answer that carries none of them:
`IssueInvoiceOutcome::WrongStateUnknownInstance` in Rust, `IssueInvoiceOutcomeWrongStateUnknownInstance`
in Go. The served surface answers it with the branch's `409`, the outcome and the error, and no
`payload`. The `WrongState` variant still requires every field, so a realization cannot leave out
the state of an instance it holds.

### An unknown instance can have its own outcome

From `format: ess/15`, an identity the system never held can answer differently from one in a state
no move starts from:

```yaml
- {name: no-such-call, unknown_instance: true, error: example.call.CallNotFound}
- {name: already-ended, wrong_state: true, refuses: false}
```

`unknown_instance:` sits beside `wrong_state:`, at most once per command, on a command with a
`moves:`, `updates:` or `deletes:` branch that reads `instance:` from input. It names an `error:`,
or declares `refuses: false` for an accepted no-op, and takes no other condition and no effect. It
is the first answer for an identity no record carries — before a declared not-found refusal and
before `wrong_state`. The suite checks it under its own outcome id, sending an identity no other
scenario sends and arranging nothing; the `wrong_state` branch keeps its scenarios in the states it
answers. The generated seams carry its variant like any other outcome's, and the served surface
answers a refusing one with `404`.

### A request with no input can have its own outcome

From `format: ess/16`, a command whose implementation answers a request that arrives with no body at
all, before any field is validated, says so without making its fields `Optional`:

```yaml
- {name: body-missing, input_absent: true, error: demo.notes.BodyMissing}
```

`input_absent:` sits beside `wrong_state:` and `unknown_instance:`, at most once per command, on a
command that takes input. It names an `error:` and takes no other condition, no effect and no
`refuses:`. The command's fields keep their types, so every other branch keeps its contract. The
suite checks it under its own outcome id with an `execute_command_without_input` step, which sends
no input document at all and is not `execute_command` with `input: {}`; it requires the outcome,
the error and that no declared event is published. A target that cannot send a request without a
body reports that scenario `unsupported`. The `OpenAPI` projection marks the request body not
required and documents the answer as `400`. Every generated code target (Rust, Go, Web, Clap) and
Entity Runtime lowering refuse the branch by name.

From `format: ess/16`, a guard that cannot hold because it needs a required input to be absent —
`not defined(text)` or `missing(text)` where `text` is not `Optional`, or a conjunction with one —
is refused as a type mismatch, with a hint naming `input_absent:`. A guard that can still hold
another way, such as `any: [text == "x", missing(text)]`, is admitted.

### An outcome can be selected by whether the record exists

From `format: ess/16`, a command that addresses a record by an identity the caller supplies can say
what it does when that record exists and when it does not. Two forms, both built on
`unknown_instance:`.

**Create or update** (`PUT` semantics). Mark the creating branch `unknown_instance: true`, beside the
branch that updates or moves the record the same input names:

```yaml
- name: updated
  updates: demo.items.Item
  instance: item_id
  emits: [demo.items.ItemStored]
  payload:
    demo.items.ItemStored: {item_id: input.item_id, label: input.label}
  sets: {label: input.label}
- name: created
  unknown_instance: true
  creates: demo.items.Item
  instance: item_id
  emits: [demo.items.ItemStored]
  payload:
    demo.items.ItemStored: {item_id: input.item_id, label: input.label}
  sets: {label: input.label}
```

The creation is taken when no row carries the identity, and the update when one does. The creation
must publish its identity from the same input field the updating branch reads as `instance:`, on the
same entity; an identity `{generated: true}` is never one a caller names again, and is refused. The
creating branch names no `error:` and no `refuses:`, and it is the command's one `unknown_instance:`
branch. The pair is exhaustive, so the command is not refused as undetermined by its input.

**Create or refuse.** Declare the refusal for an identity a record already carries beside the
creation:

```yaml
- name: booked
  creates: demo.items.Slot
  instance: slot_id
  emits: [demo.items.SlotBooked]
  payload:
    demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}
- {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}
```

`existing_instance:` is at most once per command, names an `error:` and takes no other condition, no
effect and no `refuses:`. It needs a `creates:` whose identity comes from an input field — `input.f`,
or an optional id written `{input: f, else: {generated: true}}` — and it
cannot sit beside a branch that acts on the existing record (`moves:`, `updates:`, `deletes:`,
`wrong_state:`) — that command is create-or-update instead.

In both forms an input-guarded refusal (`when:` with an `error:`) is answered first: a request such
a refusal claims is refused whether or not the record exists, and only the rest is answered by
existence.

The suite checks both with two calls that share one identity. For create-or-update, the creating
branch is sent an identity no other scenario sends; the updating branch's scenario creates the record
through the same command, sends the identity again with other values, and requires the update, the
new values in the views and exactly one row for the identity. For create-or-refuse, the scenario
creates the record through each creating branch, snapshots it, sends the identity again with other
values, and requires the declared error, no event and the row unchanged. An input-guarded refusal
beside either form is checked twice: for an identity nothing stored, and for one a record carries. No new suite step is used. The served surface answers
`existing_instance:` with `409`. Every generated code target (Rust, Go, Web, Clap) refuses both forms
by name, and Entity Runtime lowering refuses them with `ExistenceSelectionUnsupported`. Below
`ess/16` both are refused with `unsupported_format_version`.
A system precondition cannot invoke a command of either form, because which branch it takes depends
on a record it cannot observe before it runs.

### An outcome can delete its subject

From `format: ess/15`, a record the implementation removes at the end of its lifecycle is declared
as removed rather than moved to a terminal state no row ever holds:

```yaml
- name: ended
  deletes: example.call.Call
  instance: call_id
  when_subject_state: Connected     # optional, as on any subject outcome
  emits: [example.call.CallEnded]
```

`deletes:` is an effect beside `creates:`, `moves:`, `updates:` and `preserves:`, on an existing
subject named by an input. It may emit events, may not `sets:`, and names no error. Its scenario
requires that no immediate (`read_your_writes`) view of the entity still holds a row with that
identity, then sends the command for it again and requires the unknown-instance answer. A suite
holding the absence check is `ess-conformance/22` (coverage `/23`); Go and TypeScript runners
refuse those majors by version.

### A creation can land in a declared state

From `format: ess/15`, a record that first appears already past `initial` — a call announced as
ringing — says so:

```yaml
- name: offered
  creates: example.call.Call
  instance: call_id
  into: Ringing
  emits: [example.call.CallOffered]
```

`into:` names a declared state of the created entity and is admitted only beside `creates:`; a
terminal state is allowed. Omitted, creation lands in `initial` as before. The scenario asserts
`Ringing` on the created row wherever a view projects `state`, and an arrangement that needs a
`Ringing` call reaches it through this creation rather than through the moves from `initial`. The
state must still be reachable from `initial` by some transition: the entity's own reachability
check does not yet count creation states.

### An accepted request can change nothing

From `format: ess/15`, a request answered with success and no effect is declared as such:

```yaml
- name: acknowledged
  when: flag == true
  accepts: nothing
```

`accepts: nothing` is admitted under `when:` or as the default, with no subject, no event, no error,
no assignment and no replay. Its scenario requires no error and no direct event, and that every
immediate view without parameters holds exactly the rows it held before the command. A command with
a subject says `preserves:` instead. The whole-view check is `ess-conformance/22` (coverage `/23`).

### A system can run inside ambient preconditions

From `format: ess/15`, the source carrying the system header can declare commands every command runs
inside — an open session whose user row exists:

```yaml
preconditions:
  - command: example.call.OpenSession
    as: example.call.Agent
    input: {user_id: 00000000-0000-4000-8000-000000000152}
```

Each names a declared command, optionally the actor it runs as (who must be granted it), and a
literal for every required input. Its literal input must select exactly one branch that reports no
error — a precondition whose `when:` guards pick a refusal, several branches, none, or cannot be
decided from the literals is refused. An input the command declares under `fixture_inputs:` may be left
out or written `{fixture: name}`, and is resolved like any fixture input. Synthesis prepends the
list, in order, to every scenario and requires each to take its success branch; the generated Go
and TypeScript explorers run it before every sequence, so the model starts from the state it leaves.
A precondition the target, or the explorer's model, does not answer with that branch fails the
scenario, or the exploration, as setup. A scenario that itself sends the precondition's command for
the identity it creates runs without the prelude, so it does not create that record twice. The explorers do
not resolve fixture inputs and refuse a precondition that needs one.

### An event's values need a declared source

`emits:` declares which facts a branch announces; `payload:` declares what fills their fields:

```yaml
- name: accepted
  when: amount.amount > 0
  creates: billing.invoice.Invoice
  instance: invoice_id
  emits:
    - billing.invoice.InvoiceCreated
  payload:
    billing.invoice.InvoiceCreated:
      account_id: input.account_id
      customer_email: input.customer_email
      amount: input.amount
```

Without this, an implementation announcing an amount nobody submitted contradicts nothing. In source formats 1–3, the block
is optional per field: `invoice_id` has no line because the identity is the implementation's to assign.

Source `ess/4`, introduced in 0.23.0, requires every field of each emitted event to have a mapping. Use
`invoice_id: {generated: true}` for an explicitly implementation-generated identity.
Events with no emitting outcome remain valid; they may have an external producer.

A command may declare a closed typed `response` record and map one of its fields with
`item: {response: item}`. This reads the returned response; the string `response.item`
retains its historical literal meaning. Input mappings keep their existing spelling.
Missing fields, unknown response members and incompatible types are refused. Conformance
checks compare mapped values with the actual response from the same command invocation.

### Value expressions

Source `ess/14`, introduced in 0.36.0, lets a `payload:` or `sets:` value come from more than
the input or a literal:

```yaml
- name: retried
  updates: demo.orders.Order
  instance: order_id
  emits: [demo.orders.Retried]
  payload:
    demo.orders.Retried:
      order_id: input.order_id
      previous: {subject: retries}          # the stored value before this outcome
      seq: {input: seq, else: {generated: true}}
      ref:                                  # a struct, one source per field
        id: input.order_id
        label: {generated: true}
  sets:
    retries: {increment: 1}                 # previous value plus one
    stamp: {generated: true}                # the implementation decides
    discount: 0.0                           # a Decimal literal
```

| Source | Where | Admitted when |
|---|---|---|
| `{subject: <field>}` | `payload:`, `sets:` | the outcome `moves:` or `updates:` an existing subject; `creates:` has no row before it |
| `{increment: <number>}` | `sets:` | the target is a required `Integer` (a whole number) or `Decimal`; a negative number decrements |
| `{input: <field>, else: {generated: true}}` | `payload:`, `sets:` | the input is `Optional<…>` |
| `{input: <field>, else: <literal>}` | `payload:`, `sets:` | source `ess/16`; the input is `Optional<…>` and the literal is one the target admits |
| `{related: {via: <field>, field: <field>}}` | `payload:`, `sets:` | source `ess/16`; `via` is a field of an existing subject, or `input.<field>`, typed as exactly one entity's identity |
| `{caller: <attribute>}` | `payload:`, `sets:` | source `ess/16`; every actor that may invoke the command declares the attribute, at one type the target admits |
| a nested mapping | `payload:`, `sets:` | the target is a struct; every struct field has a source |
| `{generated: true}` | `sets:` | always (`payload:` has admitted it since `ess/4`) |

A mapping is a source when every key is one of `response`, `generated`, `cleared`, `subject`,
`increment`, `input` and `else`; any other key makes it a nested mapping. The text
`subject.note` is refused in every format: it used to compile as the literal text
`"subject.note"`.

Synthesis asserts each value where the arrangement determined what it reads: the row the
arrangement wrote, the input the scenario sent. Where it did not, a payload field is checked for
presence and type only, and a `sets:` target is not asserted on the row. Entity Runtime lowering
refuses these sources.

From source `ess/16` a fallback can be a literal instead of `{generated: true}`:

```yaml
sets:
  tier: {input: tier, else: Standard}       # the caller's tier, or Standard when none is sent
  rank: {input: rank, else: 3}
```

The literal is checked against the target exactly as a literal written there is: a variant of the
enum, text for a String-backed type, `3` over an `Integer`, `'3'` rather than `3` over a `String`.
Nothing else follows `else:` — not `input.<field>`, not `{subject: …}`. Below `ess/16` a literal
fallback is refused with `unsupported_format_version`. The outcome's synthesized scenario checks
both halves: it first sends the input and asserts the sent value, then invokes the branch again
without the input and asserts the literal. An implementation that ignores the input fails the
first check, and one that stores another default fails the second.

From source `ess/16` a value can come from a field of the row the subject references:

```yaml
- name: dispatched
  updates: demo.shipping.Shipment
  instance: shipment_id
  emits: [demo.shipping.ShipmentDispatched]
  payload:
    demo.shipping.ShipmentDispatched:
      shipment_id: input.shipment_id
      region: {related: {via: customer_id, field: region}}   # the region of the shipment's customer
```

`via` is a field of the subject as it was before the outcome (on `creates:`, a field the
branch sets from its input), or `input.<field>`, and its type is the identity of the entity it
names — exactly, not `Optional<…>` or a list. Where several entities share that identity type, the
relation on the subject field says which one: a `references` relation of cardinality `one` that
the subject declares on it, or the `owns` relation of the subject's owner; an input is settled by
the relation on the field the branch sets from it. `field` is a field of that entity, typed as the
target admits. One hop only. `{related: …}` is written alone and holds exactly `via` and `field`;
any other mapping under `related` is a nested mapping, and below `ess/16` so is this one.

The scenario creates the referenced row between two others of its entity, points the subject at
it, and asserts that row's value, so an implementation that reads another row, the first or the
last, fails. Where the specification has an `updates:` branch that changes the field read, the
scenario runs it on the referenced row just before the branch and asserts the new value, so an
implementation that copied the value earlier fails too. Below `ess/16` the source is refused with
`unsupported_format_version`, and Entity Runtime lowering refuses it.

A literal over a `Decimal` target is admitted in every format, quoted (`'0.25'`) or unquoted
(`0.25`): an optional `-`, digits without a leading zero, optionally a point and digits.

### Read the caller's credential

From source `ess/16` an actor may declare `attributes:`: typed fields its credential carries, such as
the account it acts for. A command reads one as `{caller: <attribute>}` in `payload:` and `sets:`, and
as `caller.<attribute>` in a guard, compared with an input field in `when:` or with a stored field in
`when_subject:`:

```yaml
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}   # not the note's agent
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
```

The attribute is not an input: the credential is its authority, not the request. It is readable
only where every actor whose `may` names the command declares it, at one type; a command whose
actors disagree, or that no actor may invoke, is refused where it reads the attribute. A guard
compares a caller attribute with `==` or `!=` against a field of the same declared type and nothing
else. An input or a stored field named `caller` keeps being read as itself. Below `ess/16` actor
attributes and a `caller.` operand are refused with `unsupported_format_version`, and
`{caller: …}` is the nested mapping it always was.

A refusal the caller decides answers `403` in the generated `OpenAPI` contract, and every operation
names the caller attributes it reads under `x-ess-caller`. The conformance suite (suite/26) says
which caller sends each command, and the target sends it authenticated as that caller; a target
that cannot answers `unsupported`. Synthesis uses two callers: the refusal is sent by one caller on
the other's note, and every scenario of a command that reads the caller runs a second time with the
two callers' roles swapped, so an implementation that records one fixed account or admits one
caller by name fails. Entity Runtime lowering refuses a caller read with `CallerUnsupported`.

### An input refused when absent is present afterwards

From source `ess/16`, an `Optional<T>` input reads as `T` in a branch that is only ever taken with
the input present. There are two such branches:

- the default branch, written with no `when:`, when a sibling refuses exactly the input's
  absence with `error:` — `when: not defined(x)` or `when: missing(x)`;
- a branch whose own guard requires the input: `when: defined(x)`, or an `all:` with it as a
  member.

```yaml
commands:
  - name: demo.notes.SubmitNote
    input:
      - {name: account_id, type: Optional<demo.notes.AccountId>}
      - {name: text, type: String}
    outcomes:
      - name: account-missing
        when: not defined(account_id)
        error: demo.notes.AccountMissing
      - name: submitted                       # the default: the account is present here
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: input.account_id, text: input.text}
        emits: [demo.notes.NoteSubmitted]
        payload:
          demo.notes.NoteSubmitted: {note_id: {generated: true}, account_id: input.account_id, text: input.text}
```

`input.account_id` fills the required `account_id` fields with no `conversions:` entry. A
conversion from `Optional<AccountId>` to `AccountId` would admit the same copy on every command,
whether anything refuses the absence first or not. Narrowing does not depend on the order outcomes
are declared in. It applies to `input.<x>` in `payload:`, in `sets:` and in a leaf of a nested
mapping, and only to a top-level input.

Nothing else narrows. A refusal of the absence *and* something else, such as
`{all: ["not defined(x)", "kind == Draft"]}`, leaves some absent requests to the default branch. A
sibling that succeeds when `x` is absent refuses nothing. A guarded branch other than the default can
match a request the refusal also matches. That includes a branch written `when: true`: it is a
guard that always holds, not the default, and a runtime that tries branches in declared order would
take it before a refusal declared after it. Write the default with no `when:`.

The synthesized suite checks the narrowing: the refusal's scenario sends no `account_id` and requires
`account-missing`, and the success scenario sends one. Below `ess/16` the copy is refused as a
`type_mismatch`, as it always has been.
[Design](https://github.com/beyond10x/ess/blob/main/docs/design/optional-input-narrowing.md).

### A view declares its consistency

`consistency: eventual` on a view is what decides that a generated assertion is `eventually` rather
than immediate. Getting it wrong produces a suite that passes on a laptop and flakes in CI.

When several views expose the same row, declare the row once as a named struct and reference it with
`shape:`:

```yaml
types:
  - name: todo.list.ItemRow
    kind: struct
    fields:
      - name: item_id
        type: todo.list.ItemId
      - name: list_id
        type: todo.list.ListId
      - name: state
        type: todo.list.Item.State

views:
  - name: todo.list.ItemById
    source: todo.list.Item
    shape: todo.list.ItemRow
    consistency: read_your_writes

  - name: todo.list.ListItems
    source: todo.list.Item
    shape: todo.list.ItemRow
    consistency: read_your_writes
```

A view declares exactly one of `shape` or inline `fields`. The named type must be a struct, and ESS
still checks each of its fields against the source entity. Compiled IR carries both the shape handle
and the checked expansion; OpenAPI uses the handle as a real `$ref`, so the row schema is emitted
once rather than copied per view.

### A view can be paged

A list endpoint that answers one page of its rows at a time declares `paging:` beside its
`order_by:`. It needs `format: ess/16`.

```yaml
views:
  - name: demo.jobs.JobList
    source: demo.jobs.Job
    consistency: read_your_writes
    params:
      - {name: type, type: Optional<demo.jobs.JobType>}
      - {name: page, type: Integer}
      - {name: size, type: Integer}
    filter: type == param.type
    order_by: [job_id asc]
    paging: {page: page, size: size, total: true}
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: type, type: demo.jobs.JobType}
```

`page` and `size` name two declared parameters, each `Integer`, a newtype of it or
`Optional<Integer>`. A paged read answers `size` rows of the rows the filter admits, in `order_by:`
order, starting at `page * size`; `first_page: 1` numbers pages from 1, so the first row is at
`(page - 1) * size`. With `total: true` the answer also carries how many rows the filter admits. A
read that sends neither parameter answers every row, in order. A filter may not read a paging
parameter, and a view without `order_by:` cannot be paged: a slice of an unordered view names no
particular rows. Below `ess/16`, `paging:` is refused with `unsupported_format_version`.

The synthesized suite arranges four rows and reads two pages of one row, two pages of two rows,
and one page larger than all of them, which must hold at least those rows: each page holds exactly
its size, the total is at least the rows the scenario made, and the second page
continues the first — ranked no earlier, and not the same row. Each claim holds on a target other
users share. A free-form filter expression the caller supplies is not something `paging:` or
`params:` can declare; a closed set of filter fields can still be declared one optional parameter
at a time. [Design](https://github.com/beyond10x/ess/blob/main/docs/design/view-paging.md).

### Aggregate views

A read API that reports counts, sums and extremes over one entity's rows is a view with `group_by:`
and a field-level `aggregate:`. It needs `format: ess/10`.

```yaml
views:
  - name: metrics.session.TalkTimeByAgent
    source: metrics.session.Session
    consistency: eventual
    filter: state == Completed
    group_by: [agent_id]
    fields:
      - {name: agent_id, type: String}
      - {name: sessions, type: Integer, aggregate: {count: {}}}
      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}
      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}
      - {name: distinct_callers, type: Integer, aggregate: {count_distinct: caller}}
      - {name: mean_talk, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}
```

The filter runs on each source row first, the admitted rows are grouped by the `group_by` fields,
and each aggregate is computed per group. A group with no admitted row is absent. A view without
`group_by` returns exactly one row: `count` is `0` and `min`, `max` and `avg` are absent when no row
passes the filter. Every field without `aggregate:` must be listed in `group_by`.

Each field declares its result type exactly, and validation names the one it expects: `Integer`
for `count`, `count_distinct` and `sum` of an `Integer`; `Decimal` for `sum` of a `Decimal`;
`Optional<T>` for `min` and `max`, keeping a newtype; `Optional<Decimal>` for `avg`, which is
rounded to 6 fractional digits, ties to even. An aggregate reads one top-level field of the source
that every row holds, so an `Optional` argument or group key is refused, and so is grouping by a
`Timestamp` or ranking an aggregate view with `order_by:`.

Conformance creates the rows itself, through the declared creating outcome, and asserts every
group's exact numbers. Because a target may be shared, the rows are kept apart from every other
scenario's by a group key or a parameter compared with one (`queue_id == param.queue_id`) that is a
`String` or `Uuid` the creating command sets from its input. An ungrouped view with no parameter
is over every row, including other scenarios' rows, so conformance reads it before creating its rows
and asserts only how much each `count` and `sum` changed; its other aggregates are not asserted. A
grouped view with neither, or an ungrouped one with no `count` or `sum`, gets no scenario and the
refusal `ESS-SYNTH-016`.

### A binding says what happens when it fails

Bindings live above the domains, in `components.yaml`:

```yaml
bindings:
  - id: notify-on-invoice-created
    when:
      event: billing.invoice.InvoiceCreated
    invoke:
      command: billing.email.SendEmail
    mapping:
      recipient: event.customer_email
      template: invoice-created
    delivery: at_least_once
    on_failure:                   # retry | drop | escalate
      escalate:
        emits: billing.email.DeliveryEscalated
```

`delivery:` and `on_failure:` are required words, not defaults — a binding that can fail silently is
the difference between specifying a system and specifying a demo. `drop` is legal: losing work is a
decision, and the decision has to be findable in the document that made it. `escalate` must name a
declared event, because "surface it to a person" is not something a generated test can observe.

`delivery:` accepts two values, and they are not two points on one scale — they say which side of
the invocation carries the risk.

| word | what it promises | what it costs |
|---|---|---|
| `at_least_once` | the command may run more than once for one event | the handler must be idempotent, and the generated OpenAPI document makes `Idempotency-Key` required on that command |
| `at_most_once` | one attempt, and nothing redelivers it | the handler is owed no repeat, and a lost attempt is `on_failure:`'s to answer |

Neither is "exactly once", and nothing here will ever spell that word: "exactly once" is what
everyone believes they have until a retry proves otherwise. Write `at_most_once` where the
invocation really is a single attempt — one HTTP call with no retry, or one whose response nobody
reads — because a specification claiming the stronger guarantee is a claim the system does not
keep. A conformance suite for an `at_most_once` binding contains no redelivery scenario, since
redelivery is the thing that word says will not happen.

### Read a field inside an event envelope

The `ess/3` format adds bounded binding accessors. Set `format: ess/3`
in the specification header, then use two or three field segments after `event`:

```yaml
mapping:
  status: event.data.status
  text: event.data.body.text
```

Every segment names a declared field. For example, if `status` has
`wire: upstream_status`, the mapping still spells `event.data.status`; the
generated adapter uses the wire name when reading a serialized payload. Existing
single-field mappings such as `event.customer_email` keep their meaning in every
supported source format. Formats `ess/1` and `ess/2` refuse the new paths.

A path can pass through structs and newtypes. Traversing an `Optional` or a union
can leave the path unavailable: the Optional is absent, or the selected union
variant does not declare that field. Such a mapping requires an Optional command
input. A miss produces an absent input; a malformed union payload remains an
error. At least one union branch must contain the complete path, and all branches
that reach it must agree on its leaf type.

The value at the end is copied whole, including a struct, collection or Optional.
Reading `event.data.last_reason`, where `last_reason` is `Optional<String>`, differs
from traversing that Optional to reach another field. Exact source/target type
matches preserve the value; supported Optional lifting adds the required outer
presence layer. Declared conversions still govern other type crossings.

Accessors do not index or search lists or maps, compute values, or read session
context. A recipient identifier absent from the event needs its own declared
authority; adding it to an event that never carries it would misdescribe the wire.
For execution and observation limits, see
[Verify conformance](verify-conformance.md#observe-bounded-binding-accessors).

### Select ordered records in a binding

`ess/3` adds binding-local `selection_inputs` and ordered `selections`.
For an event whose `candidates` field already has type `List<example.calls.Leg>`:

```yaml
selection_inputs:
  - name: candidates
    from: event.candidates
    as: List<example.calls.Leg>
selections:
  - name: first_agent
    first:
      in: candidates
      where: item.role == Agent
  - name: first_external
    first:
      in: candidates
      excluding: [first_agent]
      where: item.role == External
  - name: preferred
    first_present: [first_external, first_agent]
mapping:
  selected_id: {selection: preferred, path: [id]}
```

The record type must declare those fields and enum variants. Exclusion removes a
selected list occurrence by its original index, so equal-valued records remain
distinct. References point backward; `first_present` alternatives use the same
list. The result is optional and must fit the command input's declared type.
Whole records use `path: []`; field paths reuse bounded accessor rules.

Malformed records are refused before selection, including records after an early
match. Executable selection currently refuses inputs with unsupported declared
invariants or clock-reading attachments, including nested members and list
aliases, instead of dropping their constraints. Predicates use a bounded declared fragment: defined checks, typed literal
equality/inequality and Boolean combinations. Selection does not sort or invent
values. If the event carries a different representation, declare the exact
conversion to the local list type. The host prepares that value once and can
pass it to generated Rust/Go selection helpers; the local list is not a new
field on the wire.

### Declare a periodic host cause

A periodic cause belongs to a named host instance with an explicit owner and
authority. This `ess/3` binding fragment requires the declared command, component
and mapped input types:

```yaml
when:
  periodic:
    every: PT2S
    anchor: host_activation
    first: after_period
    cadence: fixed_rate
    overlap: serial_per_instance
    missed: one_pending_drop_excess
    lifetime: host_instance
    cancellation: stop_acknowledged
    host:
      owner: poll-service
      authority: authenticated-session-status
      eligibility: host_boolean
      context_fields: [{name: agent_id, type: String}]
      read_fields: [{name: status, type: String}]
invoke: {command: example.poll.Refresh}
mapping:
  agent_id: host_context.agent_id
  status: host_read.status
delivery: at_most_once
on_failure: drop
```

Context is constant for the host lifetime; reads are fresh for each eligible
occurrence. The first tick follows one period, work is serial, and excess busy
ticks coalesce to one pending tick. Stop acknowledgement means the loop and its
work have quiesced. Native generation reports `PeriodicHostRequired` until that
real host capability is supplied; it does not fabricate a scheduler or event.

### Preserve clock-reading provenance

An `ess/3` newtype can attach a reading contract while retaining its scalar wire
representation:

```yaml
- name: example.clock.LocalReading
  kind: newtype
  of: String
  reading:
    encoding: local_date_time_millis_literal_z
    origins: [{role: producer_process, offset: requires_observation}]
```

Supported encodings are offset date-time text, local millisecond text with a
literal `Z`, and exact integer Unix seconds. A literal `Z` on local text does not
establish UTC. The observation adapter supplies the process instance, clock
epoch, origin and actual formatter offset for the particular occurrence.
Comparison requires the same observed source and epoch. Unknown evidence and
cross-source calibration remain unsupported; generic input predicates cannot
silently discard the attachment.

Normalization accepts years 1970–9999 and fixed offsets within ±14:00. Offset
text allows no fraction or exactly three millisecond digits; local literal-Z
text requires exactly three. Other precision, leap seconds and inferred host
timezone settings are outside this bounded contract.

### Crossing contexts takes a declared conversion

A binding's `mapping:` is the one place two independently-written contexts must agree about a type,
so both sides are checked. `billing.invoice.Email` and `billing.email.EmailAddress` are distinct
newtypes, and the model refuses to treat one as the other unless you say so — with a reason:

```yaml
conversions:
  - from: billing.invoice.Email
    to: billing.email.EmailAddress
    because: >-
      An invoice's customer email is a deliverable address; the email context validates it again on
      the way out, so the invoice context does not have to know how.
```

`because:` is required, and conversions are directional — declaring `Email → EmailAddress` does not
grant the reverse, which is usually the unsafe one. The reason is not decoration: it is what
`ess specify inspect` prints back at the crossing, so the person reading the binding a year later reads the
argument for it rather than reconstructing one.

### An enum variant can carry its own wire spelling

`ess/5`, introduced in 0.27.0, lets a variant declare the name it is called on the wire. Until it
existed, `variants:` was a list of strings, so a variant whose wire form is not derivable from its
name could not be declared at all — the naming had to be written again in every target.

```yaml
types:
  - name: calls.recording.Action
    kind: enum
    variants:
      - name: Stop
        wire: ""
      - name: Flag
        wire: flag
      - Tags
```

Both forms are admitted in one list. A variant that declares nothing stays a bare name, and a
variant that declares something takes the `wire`, `display`, `summary` and `code` members every
other named thing already has. An authored empty string is a spelling rather than an absent one:
`Stop` is the route with no suffix, and falling back to the variant's name would spell the route
that does not exist.

A generated JSON Schema enumerates the wire spellings; the CLI contract keeps the authored names,
which are what an operator types. Formats `ess/1` … `ess/4` refuse a variant that declares naming
with `unsupported_format_version` at `types.<type>.variants.<variant>`, and a bare list is admitted
by every format, so nothing written before this moves. `ess verify diff` reports a moved spelling as
`VariantWireNameChanged` under [`ess-diff/5`](../reference/formats.md#change-and-conformance-records).

### A field can carry its own wire name

A field of a struct, an event or a command input that travels under another name declares it, flat
or nested the way commands and events write their own naming:

```yaml
events:
  - name: demo.orders.Placed
    fields:
      - name: order_id
        type: demo.orders.OrderId
        naming: {wire: orderId}   # the same as `wire: orderId` on the field
```

Writing both spellings on one field is refused. The model keeps the declared name, so a payload
mapping and a predicate still say `order_id`, and JSON Schema, OpenAPI and AsyncAPI key the property
`orderId`. The field is written back flat, so the two spellings are one model with one digest.

### Say whether an absent Optional is sent as null

An `Optional<T>` field says how its absent value travels with `presence:` (`format: ess/15`):

```yaml
types:
  - name: demo.orders.OrderReceipt
    kind: struct
    fields:
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
```

`null_when_absent` makes the JSON Schema property required and nullable; `omitted_when_absent`
keeps it optional and not nullable, which is what an `Optional` field without a policy already
publishes. A suite carries the policy on the field's payload leaf, so an implementation that sends
`null` for `discount_code` or leaves `partner_ref` out fails; such a suite is written as
`ess-conformance/24` (or `/25` with coverage), which the Go and TypeScript runners refuse by version.
`presence:` on a required field, or in a command's, event's or type's own `naming:`, is refused.

## Three layers above the domains

A domain says what the software *means*. Three further layers say how it is put together, kept apart
because conflating them is how a domain model turns into a description of a deployment:

| Layer | Says | Does not say |
|---|---|---|
| **component** | `invoice-service` owns `billing.invoice` and accepts four commands | whether it is a process or a module; which protocol it speaks |
| **binding** | `InvoiceCreated` causes `SendEmail` | which queue carries it |
| **topology** | the system is not correct with one instance | how many pods to start |

`reached_by:` is a closed set of `in_process` (the default), `network` and `command_line`. It
declares where a component's callers reach its surface. `examples/billing/` omits it and gets
`in_process`; `examples/gatepass/components.yaml` declares `network`, which selects HTTP server
generation in the current Rust and Go targets. Projecting an OpenAPI document alone does not run
a server.

For `command_line`, supply a `cli:` block naming the binary and placing every accepted command
exactly once at the root or in a group. Command words derive from their wire names, and flags from
input fields and their wire names. A CLI block without `command_line`, or `command_line` without a
CLI block, is refused. Reach and CLI layout belong to the authored/compiled model and participate
in its identity; physical argv or URL invocation belongs to realization.

Each logical component currently has one reach value. Several physical entrypoint records do not
add simultaneous semantic CLI and HTTP surfaces, and duplicating domain ownership is refused.
See [Logical, interface and delivery owners](../concepts/ess.md#logical-interface-and-delivery-owners)
for the separate contracts, identity consequences and bounded examples.

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

## Next

* [Verify an implementation](./verify-conformance.md) — generate the suite this specification
  obliges, run it, and turn the result into evidence.
* [A specification and its contracts](../examples/specification-to-contracts.md) — the billing
  example's source next to its generated output.
