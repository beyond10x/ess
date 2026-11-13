---
format: aep.planning-md/3
id: story:the-text-output-of-a-failed-scenario-names-the-failed-check
kind: story
status: draft
title: The text output of a failed conformance scenario names the check that failed
summary: conform run prints only `failed <scenario>` and the report holds only the id; expected, observed and input are in --format json or yaml only (a consumer's conformance run, ess 0.94.0)
tags:
- backlog
- conformance
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: story:the-swapped-caller-run-expects-the-error-field-it-redrew
revision: 1
---
## Need

A scenario of a conformance run fails. The text output of `ess verify conform run` and the report
written by `--report-out` say which scenario, and nothing more: not the step, not the check, not
the value expected and the value the implementation returned. The reader opens the suite, finds
the scenario's steps (80 in the reported case) and compares them with the implementation by hand.

The run knows all of it. `--format json` and `--format yaml` print `ess-conformance-run/2`, in
which every check has a status and a failed check has a `diagnostic` with `expected`, `observed`
and `input`. The text output, which is the default and what a CI log shows, drops it.

Evidence: a consumer's conformance run (2026-11-13, ess 0.94.0): the run log held
`failed <scenario id>` and the report the id under `outcomes.failed`; the cause (an `expect_error`
field, `story:the-swapped-caller-run-expects-the-error-field-it-redrew`) was found by reading the
suite's 80 steps.

## Reproduction

The model of `story:the-swapped-caller-run-expects-the-error-field-it-redrew` (ess 0.94.0).

Text output, whole:

```
demo v1 against interpreted 0.94.0 — failed
  passed demo.desk.AnswerTicket/outcome/answered
  failed demo.desk.AnswerTicket/outcome/no-such-note
  passed demo.desk.AnswerTicket/outcome/no-such-ticket
  passed demo.desk.OpenTicket/outcome/opened
  passed demo.desk.Ticket/transition/answer/by/demo.desk.AnswerTicket/answered
  passed demo.desk.WriteNote/outcome/written
  6 scenarios: 5 passed, 1 failed, 0 error, 0 unsupported
```

`--report-format 2 --report-out report.json` (`ess-conformance-report/2`): the keys are
`completed_at`, `conformance_status`, `counts`, `coverage`, `execution_status`, `format`,
`implementation`, `outcomes`, `policy`, `producer_profile`, `spec_digest`, `specification`,
`suite`; `outcomes.failed` is `["demo.desk.AnswerTicket/outcome/no-such-note"]`.

`--format json`, check 15 of 20 of the failed scenario:

```json
{"about": "error demo.desk.NoSuchNote", "code": "ESS-CF-ERROR", "status": "failed",
 "diagnostic": {"code": "ESS-CF-ERROR",
  "expected": ["error = demo.desk.NoSuchNote", "error.note_id = \"00000000-0000-4000-8000-c0e2e0d903a1\""],
  "input": "demo.desk.AnswerTicket(cited_note = \"00000000-0000-4000-8000-76cfd14780f9\", ticket_id = \"00000000-0000-4000-8000-000000000006\")",
  "observed": ["error.note_id = \"00000000-0000-4000-8000-76cfd14780f9\""],
  "scenario": "demo.desk.AnswerTicket/outcome/no-such-note",
  "source": [{"kind": "error", "name": "demo.desk.NoSuchNote"}]}}
```

So the answer exists today behind `--format json` or `--format yaml`; neither the text output nor
the command's help says so.

## What a design has to settle

- What the text output prints under a `failed` or `error` line: the first failed check's `about`,
  `expected`, `observed` and `input`, or every failed check, and how long a line may be.
- Whether the step index of the suite is part of it. The diagnostic names the check and the input,
  not the step; a scenario that sends one command several times is told apart by the input only.
- Whether the report file changes. `ess-conformance-report/2` is a published contract that another
  tool reads as evidence; a detail there is a new report version, not a new key. The text output is
  not a contract.
- Whether the generated Go and TypeScript runners print the same lines.

## Acceptance (sketch, to be fixed by the fit review)

- For a failed scenario the default text output names the failed check, what was expected, what
  was observed and the input sent, in the words of the `diagnostic` of `ess-conformance-run/2`; a
  test pins the lines for the model above.
- A passing run's text output keeps its bytes.
- `ess verify conform run --help` names `--format json|yaml` as the place of the per-check detail.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the acceptance is fixed there.
