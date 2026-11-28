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
revision: 2
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

## A second case of the same walker: a name that begins `channel.` (downstream report, finding G1)

The walker of `raw.rs` decides what is an expression by the key a string stands under: every string
whose key is not in `PROSE` is scanned for `channel.<name>` (`string`, `raw.rs:121`;
`unknown_channels`, `raw.rs:271-301`). This story's first case is a key name that is wrongly on that
list. The second is the same cause from the other side: a string under a key that holds a name, not an
expression, is read as an expression.

Reported by a downstream specification of reusable building blocks (ess 0.99.2, finding G1 of one
collected report): in a widget, a `component` written as a domain-relative view name
(`channel.Messages`) is read as a channel expression, `channel_refs … names no channel (in
component)`; the full name `<system>.channel.Messages` works. It reproduces on 0.99.9, and wider than
reported, with a neutral model whose domain is `shop.channel`
(`.engineering/repro/ui-check-name-positions-are-not-expressions/`, `run.sh`, `transcript.txt`):

| name written | where | 0.99.9 |
|---|---|---|
| widget `channel.Items`, used as `component: channel.Items` | `component` | `channel_refs` error |
| `reads: {view: channel.Items}` | `reads/view` | `channel_refs` error |
| `does: channel.AddItem` | `does` | `channel_refs` error |
| `navigate: {to: channel.list}`, `navigation … pages: [channel.list]` | `navigate/to`, `pages` | `channel_refs` error |
| the same names written in full (`shop.channel.Items`) | | 0 findings |

So a domain named `channel` cannot spell any of its views, commands, widgets or pages in the
documented relative form (`examples/partner-portal/ui.yaml` writes `activity.Page`,
`partners.list`). One more finding of the same reading, from the same run: `fixture_per_view` compares
the spelling as written, so a view read as `channel.Items` with a fixture keyed `shop.channel.Items`
warns "no fixture answers view `channel.Items`", and the reverse spelling warns the same.

## Acceptance (downstream report G1)

- **G1.** On the repro's four documents, `ess ui check --model` reports no `channel_refs` for a name
  written where a name stands (`component`, `reads/view`, `does`, `navigate/to`, `pages`), while
  `visible: channel.nosuch` and a bind input's `channel.nosuch` still report it. Red today: three of the
  four documents exit 1.
- **Same cause.** A fixture keyed by the full name answers a read written relative, and the reverse;
  `fixture_per_view` warns for neither.
- The check tells a name from an expression by where the string stands (the schema's typing of the
  position, as the first case asks for `bind`), never by its first characters. The rows of the two
  cases are tested in one target.
- A document with a channel named like a view's domain is unchanged for a real channel expression:
  `live: {channel: …}` and `channel.<name>.…` in an `expr` position are resolved as before.

## Fit review (downstream report G1)

Written with `.agents/skills/assessing-external-requests/SKILL.md`. The consumer is a downstream
specification of reusable building blocks.

1. **Need, apart from the syntax.** A name written where the language expects a name is read as a
   name. Reproduction: the table above. The reporter proposed no syntax.
2. **Class.** Defect: `ess-ui.md` says `component` is "name of a Composite | name of a Widget",
   `reads.view` a view and `does` a command, and that names follow navigation (§ Document); a validator
   that reads them as expressions contradicts it.
3. **Already expressible?** Partly: the full name works (`transcript.txt`, third command). It is
   the form the reference's own example avoids, and it ties every widget name to the system's name; the
   check still holds the figure, so this is a workaround, not a reason to decline.
4. **Fit.** No new key. One walker, one rule shared with this story's first case: position, not
   spelling. Grid R: expression positions are the schema's `expr`-typed fields, `bind` values and
   `args` values (an argument is "an expression, or a literal"); everything else is a name or prose.
   Targets: only `ess ui check` reads the walker; the loader takes the repro's documents (`ess ui
   load` passes them), and I do not know how each renderer reads such a name.
5. **Second adopter.** Any model with a domain named `channel` or a widget called `channel.list`: a
   chat or a notification system.
6. **Cost.** No format, schema or diagnostic code. A document that was refused for such a name passes;
   one that hid an unresolved expression under a name-valued key (none known) is not reached by
   this change.
7. **Considered.** (a) Change nothing: write full names. (b) A list of name-valued keys beside `PROSE`:
   a second list that drifts with every new key. (c) Chosen: read by position, with the schema as the
   authority.

## Decisions (downstream report G1)

- **NP-D1, how the walker tells a name from an expression.** A: by position, from the schema's typing
  of the field. B: keep the scan and add a list of name-valued keys beside `PROSE`. C: change nothing,
  and refuse a domain called `channel` in the model. Recommended: A. It is what this story's first case
  already asks for `bind`, and one rule closes both. Fails closed for expressions: an `expr`-typed
  field is still scanned.
- **NP-D2, the spelling of a view in a fixture index.** A: `fixture_per_view` resolves both spellings
  to the one view. B: change nothing. Recommended: A, since the compiled model has one view and two
  spellings of it are one fact.
