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
- informed_by: story:the-swapped-half-of-a-wrapped-value-scenario-at-a-view-selects-its-witness-row-by-the-identity-field-s-own-name
revision: 2
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

## Second report (2026-11-25, 0.99.4)

A downstream application's suite held two failed scenarios (see
`story:the-swapped-half-of-a-wrapped-value-scenario-at-a-view-selects-its-witness-row-by-the-identity-field-s-own-name`).
The text run printed their ids and nothing else, and the cause was found only through `--format json`.
The text run prints a check's diagnostic only for the statuses `error` and `unsupported`
(`crates/verify/ess-conformance/src/report.rs`, the loop that continues unless the status is
`Status::Error` or `Status::Unsupported`, lines 741 to 757), so a scenario with a
`failed` check prints its id alone. A report written with `--report-out` and `--report-format 2`
holds ids only (`outcomes.failed`). The reason is in `--format json` under
`checks[].diagnostic.observed`.

This is the same gap as the first report, met again on a second suite: the first report's example is
also a scenario with a `failed` check whose diagnostic the text run does not print.

## Outcome

A failed scenario's reason is printed by the text run and held by the written report.

## Acceptance (second report)

- A failed scenario's line in the text run names the failing step and what was observed, in the
  words of the check's `diagnostic`.
- The test is written first and is red on the 0.99.4 tree: a scenario with a failed check prints
  its id alone there.
- A passing run's text output keeps its bytes.

## Decisions

- RG-D1 (open, the coordinator's): whether the written report/2 gains the reason is a format
  question of `ess-conformance-report/2`; the text line is not.
