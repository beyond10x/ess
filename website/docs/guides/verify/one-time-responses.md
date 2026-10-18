---
title: One-time response values
sidebar_position: 6
description: Declare response fields whose values may be disclosed only by the response that first issues them.
---

# One-time response values

`one_time_response`, introduced by the `ess/21` format (0.53.0), marks response fields whose
values may appear only in the response that issues them. A retry, later command, view or event
must not disclose those values again. This is useful for credentials that a caller sees once.

```yaml
format: ess/21
system: credentials
version: v1
domain: credentials.api
commands:
  - name: credentials.api.Issue
    response:
      - {name: secret, type: String}
    outcomes:
      - name: issued
        returns: true
        one_time_response: [secret]
```

The field must be a required `String`, or a transparent newtype over `String` whose declared
constraints can be checked. Optional fields, non-string fields and opaque readings are refused.
Each listed field is a separate origin. A replay that retains a marked response is incompatible
with the policy; fresh issuance must return a value different from every previously issued value
in that scenario.

## The disclosure scenarios synthesis adds

`ess verify conform synthesize` adds disclosure scenarios beside the ordinary outcome scenarios.
For each outcome that lists a field in `one_time_response`, and for each field it lists, it adds
these cells:

- For each actor granted the originating command (one anonymous caller when no actor is granted
  it):
  - `origin`: the command runs once and its response is checked.
  - `retry`: the identical invocation runs again, as the same actor, over the state the origin
    left, and must not return the value again.
  - `rotation`: the command runs again with a fresh witness, and the second value must differ
    from the first.
- For each declared actor (one anonymous caller when the model declares no actor), after an
  origin:
  - `command/<command>/<outcome>`: each outcome of each command the actor is granted, the
    originating command included.
  - `denied/<command>`: each command the actor is not granted, invoked as that actor and expected
    to be refused for lack of a grant.
  - `read/<view>`: each declared view. The cell is refused when the actor is not granted the
    originating command, because that actor cannot issue the value it would read.

Each cell has its own scenario id:
`<command>/disclosure/<outcome>/<field>/<aspect>/as/<actor>`, where `<aspect>` is one of the names
above and `<actor>` is `actor/<actor name>`, or the caller is written `as/anonymous`.

A cell the synthesizer cannot arrange is refused, with its id and the reason, rather than left
out. The common refusals are:

- `retry` when the origin creates a record whose identity the caller supplies and the command
  declares no `existing_instance:` outcome. The identical retry names the record the origin
  created, so it is a create of an existing key, and the model does not say how that is answered.
  Declare `existing_instance: true` with an `error:` on the command, and the retry cell expects
  that refusal.
- `read/<view>` for an actor not granted the originating command, as above.
- Any cell whose arrangement is refused for the same reason the ordinary outcome scenario would
  be.

A cell refused for the same cause as an ordinary outcome scenario it sends or arranges through
(its follow-up outcome, the command it is denied, or its origin) is reported under that outcome's
refusal, not as a refusal of its own. A gap such as an entity no command creates then reads once,
however many actors and follow-ups repeat it. A cell refused for its own reason is linked to the
first cell of the same field and aspect refused for that reason. Each `read/<view>` cell of an
actor not granted the origin is linked that way. In this model `vault.keys.EditNote` updates a
`vault.keys.Note` that no command creates, and two of three actors are granted nothing:

```shell-session
$ ess verify conform synthesize --path system.yaml
refused: refusal[ESS-SYNTH-004]: outcome vault.keys.EditNote/edited has no scenario `vault.keys.EditNote/outcome/edited`
  it needs an instance of `vault.keys.Note` to change without moving, and no outcome creates one
  help: give some command outcome `creates:` for this entity; nothing can act on an instance nothing brings into existence
  and 3 disclosure scenario(s) that send `vault.keys.EditNote`, refused for the same cause
refused: refusal[ESS-SYNTH-001]: outcome vault.keys.IssueToken/issued has no scenario `vault.keys.IssueToken/disclosure/issued/secret/read/vault.keys.Notes/as/actor/vault.keys.Auditor`
  the disclosure cell cannot be arranged or observed: the read's originating caller has no grant to issue this value
  help: the line above names what the cell lacks; a `read/<view>` cell for an actor not granted the originating command is refused by design and needs no change
  and 1 more disclosure scenario(s) refused for the same cause
8 scenario(s) (0 authored), 6 refusal(s), 2 distinct cause(s), nothing written
```

The refusal count still counts every refused cell. `ess specify validate --format json` lists
each one under `completeness.unscenarioed`. A linked cell carries `caused_by`, the id of the
refusal it repeats.

The number of cells per marked field grows as
actors × (outcomes + commands + views), where outcomes counts the outcomes of the commands each
actor is granted and commands counts the commands it is not granted, plus up to three cells for
each actor granted the originating command.

### A worked count

This model has one command and two actors. `Admin` may issue a key and `Viewer` may not:

```yaml
format: ess/22
system: keys
version: v1
domain: keys.api
entities:
  - name: keys.api.ApiKey
    identity: {name: key_id, type: Uuid}
    fields:
      - {name: issued_by, type: String}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
actors:
  - name: keys.api.Admin
    attributes:
      - {name: user_id, type: String}
    may: [keys.api.IssueKey]
  - name: keys.api.Viewer
    attributes:
      - {name: user_id, type: String}
    may: []
commands:
  - name: keys.api.IssueKey
    input:
      - {name: key_id, type: Uuid}
    response:
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: keys.api.ApiKey
        instance: key_id
        sets: {issued_by: {caller: user_id}}
        returns: true
        one_time_response: [secret]
        emits: [keys.api.KeyIssued]
        payload:
          keys.api.KeyIssued: {key_id: input.key_id}
events:
  - name: keys.api.KeyIssued
    fields:
      - {name: key_id, type: Uuid}
```

The suite has five scenarios: the ordinary `keys.api.IssueKey/outcome/issued` and these four
disclosure scenarios:

```text
keys.api.IssueKey/disclosure/issued/secret/command/keys.api.IssueKey/issued/as/actor/keys.api.Admin
keys.api.IssueKey/disclosure/issued/secret/denied/keys.api.IssueKey/as/actor/keys.api.Viewer
keys.api.IssueKey/disclosure/issued/secret/origin/as/actor/keys.api.Admin
keys.api.IssueKey/disclosure/issued/secret/rotation/as/actor/keys.api.Admin
```

The `retry` cell is refused, because `key_id` is supplied by the caller and `IssueKey` declares no
`existing_instance:` outcome:

```text
keys.api.IssueKey/disclosure/issued/secret/retry/as/actor/keys.api.Admin
```

Adding a second command, `LabelKey`, which updates the key and is granted to `Admin` and to a
third actor, `Operator`, adds one ordinary scenario and four disclosure cells: `command` cells for
`Admin` and `Operator`, and `denied` cells for `Operator` on `IssueKey` and for `Viewer` on
`LabelKey`. Adding one view of `ApiKey` adds one `read` cell for `Admin`; the `read` cells for
`Operator` and `Viewer` are refused.

| model | scenarios | disclosure scenarios |
|---|---|---|
| 1 command, 2 actors | 5 | 4 |
| 2 commands, 3 actors | 10 | 8 |
| 2 commands, 3 actors, 1 view | 11 | 9 |

## What the suite observes

The ordinary `ess-conformance/34` and coverage `ess-conformance/35` formats carry the response
authority and the event observation windows. The runner holds issued values privately for the
scenario and checks subsequent responses, view rows and independently observed events. It also
checks unexpected JSON members and object keys: placing a value inside a larger string or under
an extra field does not exempt it. Only the exact originating response field may contain the
newly issued value; an older value or another marked field's value is not exempt there.

An event adapter must observe the implementation's event log independently of a command's direct
events. A declared window includes its closing boundary, even when an earlier observation already
returned events. Missing observation capability is `Unsupported`; it cannot qualify as a passing
non-disclosure check. Ordinary adapter errors remain `Error`.

The private observation has finite limits: 256 captured values and 1 MiB of captured UTF-8 bytes
per scenario; each observed JSON payload is limited to 1 MiB, depth 128 and 65,536 members. Exceeding
a resource bound is an explicit unsupported result, not a partial successful scan. Diagnostics
describe the failed obligation without including captured values or data-derived JSON paths.

## What the implementation must guarantee

Consume disclosure authority atomically and durably with the first response attempt. A lost
network reply does not restore that authority. Rotation creates a new value; it does not make an
old value safe to return again.

A finite serial suite cannot prove durable storage, log hygiene, every encoding of a value,
restart safety or concurrent atomicity. Those remain implementation obligations; what a
specification promises when commands race, and the history check that holds it, is in
[when commands race](../../concepts/ess.md#when-commands-race). Recording and
history exploration paths that cannot preserve the policy refuse marked models before invoking
the target; they must not record plaintext as ordinary history data.

## Projections and browser display

Documentation and structural schema projections retain the policy and state its obligations.
JSON Schema, OpenAPI and AsyncAPI alone do not enforce temporal non-disclosure. Implementation
synthesis refuses a marked model when its target cannot implement durable consumption and fresh
issuance; use an implementation-owned conformance adapter to test the actual behavior.

The ordinary browser scenario player preserves the source policy even when the selected scenarios
do not exercise it. Its declarations are unexecuted and provide no non-disclosure evidence.
The closed coverage replay format currently refuses this policy explicitly. Use the execution
runner and its report to assess the implementation.
