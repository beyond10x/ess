---
format: aep.planning-md/3
id: specification:ess-ui-reuse-and-composite-values-use-widgets-and-field-widgets
kind: specification
status: draft
title: A reused UI part is a widget, and a composite value is a field widget over its model type
summary: Declines the reusable-fragment and composite-value candidates of the UI adopter composition report (P3, P5) with the existing idioms
relations:
- serves: vision:O2
revision: 1
---
## Idiom

Answers two candidates of a UI adopter's collected composition report (2026-11-08), each counted at
3 gap lines: a reusable fragment and a composite value field. Both are declined with the idiom.

**A part shown on several pages is a widget.** `widgets:` declares an app-defined composite once,
with typed `params`, and every page uses it as `{component: <widget>, args: {…}}`
(`crates/ui/ess-ui/src/model.rs:2073-2088`, `2121-2131`). An overlay repeated on another page is
`same_as: <overlay>` (`schemas/ui/ess-ui.schema.yaml:115`). The copies the report counts (24
host-component entries) exist because a host slot inside a reused template is refused as declared
twice; that is `story:a-host-slot-in-a-reused-template-is-one-registration`, already drafted. A
group of fields repeated inside list entries or variants is answered by the list-of-records and
union-field stories, whose nested `fields:` are written once per element type.

**A value entered through several controls is a field widget over its model type.** A cron string
is `as: cron`; a JSON value is `as: json`; a list of keys is `as: list`; a date and a time are
`as: date` / `as: time` (`schemas/ui/ess-ui.schema.yaml:945`). A date and time entered in a named
zone and stored as an instant has no idiom; it is one line in the report, the reporter states that
no checklist line asks for it, and it is not filed until a second report names it.

## Evidence

ess 0.90.0, 2026-11-08: a brand-free document declaring widget `prompt_entry` (params `step`, a form
with two labelled `groups` and a field `{field: schedule, as: cron}`) used on two pages:
`ess ui check --path ui.yaml` → `ui.yaml: 0 error(s), 0 warning(s)`, exit 0.
