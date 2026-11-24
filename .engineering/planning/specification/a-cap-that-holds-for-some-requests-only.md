---
format: aep.planning-md/3
id: specification:a-cap-that-holds-for-some-requests-only
kind: specification
status: draft
title: A cap that holds for some requests only
summary: Decline of beyond10x/ess#816 with the idiom, run on 0.98.0
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- specifies: story:the-guards-page-shows-a-cap-that-holds-for-some-requests-only-a-row-set-count-refusal-narrowed-by-an-input-guard
revision: 1
---
## Reproduction

Reproduced on 0.98.0. Model `run/cap/system.yaml` (`format: ess/36`; generic names): entity
`cap.a.Seat {seat_id, tenant: String, plan: cap.a.Plan}` (`Plan` an enum `trial | paid`); command
`cap.a.AddSeat(tenant, plan)`:

```yaml
- name: trial-full
  when: input.plan == trial
  when_related:
    entity: cap.a.Seat
    where: tenant == input.tenant
    count: {gte: 2}
  error: cap.a.TrialFull
- name: added
  creates: cap.a.Seat
  ...
```

```console
$ ess specify validate --path cap
cap v1 — 1 file(s), valid
$ ess verify conform synthesize --path cap --out cap-suite.json
2 scenario(s) (0 authored), 0 refusal(s), written to cap-suite.json
$ ess verify conform run --path cap --suite cap-suite.json --target interpreted --report-format 2
  2 scenarios: 2 passed, 0 failed, 0 error, 0 unsupported
$ ess verify conform mutate --path cap --target interpreted
mutation audit of cap v1 against interpreted: 4 mutant(s), 3 killed, 0 survived, 0 inconclusive, 1 stillborn, ...
```

`trial-full` arranges two seats of one tenant and sends `plan: trial` for it; the mutants
`count: {gt: 2}` and `when: not (plan == trial)` are killed.

Finding beside it: no scenario sends `plan: paid` for a tenant that already holds two seats, so a
target that caps every plan passes the suite (the audit has no mutant that drops the input
conjunct). See W5-D1.

