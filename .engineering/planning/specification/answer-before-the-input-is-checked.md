---
format: aep.planning-md/3
id: specification:answer-before-the-input-is-checked
kind: specification
status: draft
title: Answer before the input is checked
summary: Decline of beyond10x/ess#816 with the idiom, run on 0.98.0
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- specifies: story:the-guards-page-says-a-before-row-external-declared-above-the-input-refusals-answers-before-them
revision: 1
---
## Reproduction

Reproduced on 0.98.0. Model `run/auth/system.yaml` (`format: ess/36`; generic names): command
`authfirst.desk.OpenSeat(label: String)` with, in this order, `unauthenticated` (`external: the
request carries no valid session`, `before_row: true`, error `Unauthenticated`), `label-too-long`
(`when: input.label.count > 8`), `opened` (creates `Seat`).

```console
$ ess specify validate --path auth
authfirst v1 — 1 file(s), valid
$ ess verify conform synthesize --path auth --out auth-suite.json
3 scenario(s) (0 authored), 0 refusal(s), written to auth-suite.json
$ ess verify conform run --path auth --suite auth-suite.json --target interpreted --report-format 2
  3 scenarios: 3 passed, 0 failed, 0 error, 0 unsupported
```

The scenario `authfirst.desk.OpenSeat/outcome/unauthenticated` forces the external cause twice:
once with `label: "label"`, and once with `label: "labellabe"`, which `label-too-long` also claims,
requiring `unauthenticated` both times (`run/auth-suite.json`, steps at lines 210-241). A target
that checks the label first fails that second send.

