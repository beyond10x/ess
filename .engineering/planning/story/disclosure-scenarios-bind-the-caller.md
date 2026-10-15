---
format: aep.planning-md/3
id: story:disclosure-scenarios-bind-the-caller
kind: story
status: implemented
title: One-time disclosure scenarios bind the caller attributes the outcome reads
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T08:45:12Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-15T08:45:13Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-15T08:45:14Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Every disclosure scenario synthesized for a `one_time_response:` outcome binds the caller
attributes that outcome reads (`{caller: <attribute>}` in `sets:`, a payload or a response), as the
outcome's own scenario already does, so the scenarios run and pass against the interpreted target
instead of returning `unsupported`.

Closes https://github.com/beyond10x/ess/issues/550.

## Evidence

https://github.com/beyond10x/ess/issues/550 (ess 0.59.0, ess/22): `keys.api.IssueKey` creates an
`ApiKey` with `sets: {issued_by: {caller: user_id}}` and `one_time_response: [secret]`.
`ess verify conform run --target interpreted` reports 4 of 5 scenarios unsupported
(`command`, `denied`, `origin`, `rotation` disclosure scenarios), each
`ESS-CF-TARGET: protected observation did not satisfy the declared rule`. The same model with the
value from an input passes 5 of 5. The synthesized `disclosure/...` scenarios carry no `caller` key
on any `execute_command` step; the `outcome/issued` scenario carries `"caller": {"user_id": …}`.
An adopter's specification of this shape had 772 of 918 scenarios unsupported from it.

## Acceptance

- `the_disclosure_scenarios_bind_the_caller_the_outcome_reads`: the issue's model synthesizes
  disclosure scenarios whose `execute_command` steps carry the caller attributes the outcome reads,
  and `conform run --target interpreted` reports 5 scenarios, 5 passed.
- A model without a `{caller: …}` source keeps its synthesized suite bytes.
- A target that drops the caller-recorded value or discloses the secret twice still fails the
  disclosure scenarios.

## Out of scope

The `retry` disclosure scenario's `ESS-SYNTH-001` refusal, present with and without the caller
source (noted on the issue as unrelated).
