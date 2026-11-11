---
format: aep.planning-md/3
id: story:a-view-field-reads-a-setting-whose-value-differs-per-environment
kind: story
status: draft
title: A computed view field may read a setting whose value differs per environment
summary: setting.<name> in a view field is ESS-VIEW-001 on 0.93.0, and the first cut of ess#674 admits only a fixed value (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 1
---
## Need

A downstream adopter's service adds a link to every row it answers: the deployment's base URL, a
fixed path and the row's identifier. The base URL is the one value that differs per environment.
On 0.93.0 the row part is expressible and the base is not (scratch model `demo.tags`, `ess/34` and
`ess/37`):

```text
# {name: _url, type: String, expr: "\"https://example.test/api/v2/calls/\" + id"}
demo v1 — 3 file(s), valid

# {name: url, type: String, expr: "setting.app_base"}   (component setting app-base)
error[ESS-VIEW-001]: `setting.app_base` is no field of `demo.tags.Call`, the row `demo.tags.Calls` computes `url` from

# {name: url, type: String, expr: "concat(\"…\", from)"}
error[ESS-SPEC-012]: `concat` is no function of the expression language
```

`story:a-view-may-answer-values-no-row-holds` (beyond10x/ess#674) opens `setting.<name>` in a view
field, but its first cut admits only a setting with a fixed `value:`. A base URL has no fixed
value, so that cut does not settle this need.

## Acceptance

- Under the format that opens `setting.` in a view field, a computed field may read a setting that
  declares no fixed `value:` (`{name: app-base, type: String, required: true}`), joined with row
  fields: `{name: _url, type: String, expr: "setting.app_base + \"/api/v2/calls/\" + id"}` validates
  and compiles, and the IR records the setting read.
- Synthesis and the interpreted target take the setting's value from the run's environment
  (`ess-environment/1`, or a value the runner is given). A view check expects the joined string: a
  target that answers another host or drops the identifier fails. Without a value for the setting
  the scenario is refused by name.
- A setting marked `secret: true` stays refused (`conflicting_declaration`), as in #674.
- The value-expression table in `website/docs/guides/specify/values-and-views.md` lists `a + b`
  over two Strings, which the evaluator implements and the table does not name.
- A model without the read keeps its IR bytes and its suite bytes.

## Evidence

The three runs above, observed by the reporting session on the 0.93.0 release binary on
2026-11-11. Not rerun in this repository; the scoper reruns them before the story is proposed.
