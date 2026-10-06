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
