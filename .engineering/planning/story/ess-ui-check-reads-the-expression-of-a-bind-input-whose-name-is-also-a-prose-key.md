---
format: aep.planning-md/3
id: story:ess-ui-check-reads-the-expression-of-a-bind-input-whose-name-is-also-a-prose-key
kind: story
status: draft
title: ess ui check reads the expression of a bind input whose name is also a prose key
summary: raw.rs:131 returns before checking a string under a PROSE key, so a bind input called label or title has its expression skipped
tags:
- defect
- ess-ui
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:both-renderers-read-one-binding-expression-grammar
revision: 1
---
## Summary

`ess ui check` does not read the expression of a `bind` input whose name is also one of its prose
keys. `crates/ui/ess-ui-check/src/raw.rs` lists keys whose string values are prose for a reader,
never an expression (`PROSE`, line 22: `alt`, `body`, `confirm_label`, `consequences`, `doc`,
`endpoint`, `derived`, `label`, `message`, `note`, `placeholder`, `purpose`, `reason`, `source`,
`summary`, `synonyms`, `title`, `transport_today`), and the walker returns before it checks a string
under one of them (`raw.rs:131`). A `bind` is a map from an input name to an expression, so an
input called `label` or `title` has its expression skipped: a `channel.<name>` that names no
channel, or a `section.<name>.row` that names no section, passes `ess ui check` there and is
refused or fails only at run time.

## Evidence

By reading, found while writing the tests of `loads.<field>` (the check of a bound input in a
section row): a test input named `label` was not walked, and the test renamed it to `text`. Not
yet reproduced by a test of its own; the first step is the red test below. The skip is older than
that work and the branch changed no line of it.

## Red test

In `crates/ui/ess-ui-check/tests/`: a document with an action whose `bind` carries an input named
`label` with an expression that names no channel (`channel.nosuch`), beside the same expression
under an input named `text`. Expected: the `channel_refs` finding at both. Red today: the finding
appears for `text` only. A second case does the same for a `section.<name>.row` expression
(`section_row`).

## Acceptance

- The red test is green: the expression of a `bind` input is checked whatever the input is called;
  a prose key still skips the prose it names (a `label:` string of a widget stays unchecked).
- The check tells a bind input from a prose key by where it stands (the value of `bind`), not by
  its name; no existing document's findings change except documents that hid an unresolved
  expression under such an input name, which now report it with the same finding code.
- `ess-ui-check`'s tests and the reference page `website/docs/reference/ess-ui.md` stay in step
  (render it again if the text of a check changes).

## Fit review

1. **Class.** A defect of a check: a false negative, not a false positive; no document that passes
   today stops passing unless it holds an unresolved expression in such an input.
2. **Cost.** No format, schema or diagnostic code; a change of one walker in one crate.
3. **Border.** `story:both-renderers-read-one-binding-expression-grammar` (where `loads.<field>` and
   the check of a bound input landed); this is outside its acceptance.

## Decisions

- None open.
