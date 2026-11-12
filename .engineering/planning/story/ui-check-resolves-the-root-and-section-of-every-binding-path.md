---
format: aep.planning-md/3
id: story:ui-check-resolves-the-root-and-section-of-every-binding-path
kind: story
status: active
title: ui check resolves the root and the section of every binding path
summary: ess ui check accepts an unknown binding root and a section path naming no section or no listed member, in a navigate parameter and a read's params, on 0.94.0 (adopter report)
tags:
- backlog
- defect
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:16Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:17Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Defect

`ess ui check` accepts a binding path whose root is no root of the expression grammar, and a
`section.<name>` path that names no section of the page or a member the reference does not give,
in a navigate parameter and in a read's `params`. The reference
(`website/docs/reference/ess-ui.md`, Expressions) says "a renderer evaluates them, a validator
resolves every path". A renderer then evaluates the path to nothing: the read goes out without its
parameter and a link has no target.

Reported by an adopter on 0.90.0 (2026-11-12); reproduced on the 0.94.0 release binary.

On `examples/partner-portal/ui.yaml`, line 339, the read parameter
`deal_id: section.deals.selection.id` replaced, then `ess ui check --path ui.yaml`:

| `deal_id:` | result |
|---|---|
| `section.deals.selection.id` (as committed) | 0 errors, 1 warning (the placeholder) |
| `bogusroot.x` | 0 errors, 1 warning |
| `section.nosuch.selection.id` | 0 errors, 1 warning |
| `section.deals.junk.id` | 0 errors, 1 warning |
| `section.deals.row.id` | 0 errors, 1 warning |

The same four paths in a navigate parameter (`navigate: {to: q, params: {id: <path>}}` on a
record's action, checked with `--model`) add no finding to the document's baseline either.

Related: `story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands`
covers `row.` and `actor.` paths inside an action's `visible`. This story is the root and the
`section.` member, at every expression site.

## Acceptance

Redesigned by the fit review (2026-11-12).

1. `section.<name>` inside an expression whose `<name>` is no section of the page is an error
   under `section_refs`, at the node path of the expression, naming the page's sections, as
   `channel_refs` reports `channel.<name>`.
2. `section.<name>.<member>` whose member the reference does not list is an error naming the
   members. The members are `selection` and `row` (`story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither`)
   and are read from the schema's `expressions.forms`, the source of the reference's table.
3. `state.<name>`, `params.<name>` and `shell.<name>` inside an expression name a state, a
   parameter or a shell state in scope where the expression stands; otherwise an error names what
   is in scope.
4. An unquoted dotted word whose first segment is no listed root (`bogusroot.x`, `example.com`)
   stays literal text and gets a warning that names the roots and the quoted form. A quoted string
   (`"example.com"`), a number and a word without a dot get none.
5. Points 1 to 4 hold at every site the document model types as an expression (a read's `params`,
   a navigate's `params`, `bind`, `visible`, `when`, `args`, a field's value). The list of sites
   is derived from the document model's `Expr` fields, so a new expression field is covered
   without a new rule.
6. The check runs without `--model`.
7. Every document under `examples/` keeps its findings; `examples/partner-portal/ui.yaml` passes
   with its one placeholder warning.
8. Tests that fail before the change: the rows `section.nosuch.selection.id` and
   `section.deals.junk.id` of the table above (errors), `bogusroot.x` (warning), and
   `state.nosuch`, `params.nosuch`, `shell.nosuch` of the fit review's probe (errors), each in a
   read parameter and in a navigate parameter.

## Evidence

Probe script, documents and output: session scratch `ess/adopter9/` (`probe.sh`, `probe.txt`,
`pp-*`; not in the repository).

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

**Fit review (2026-11-12): accept, redesigned.** What changed from the draft and why:

- An unknown root is a warning, not an error. The reference makes such a string literal text
  (`ess-ui.md:55`), and both renderers and the check read it so; an error would refuse
  `example.com` in an `expr` position.
- `state.<name>`, `params.<name>` and `shell.<name>` are added. The probe in the fit review shows
  the check resolves `channel.<name>` and none of these three; they are the same walk.
- The members of `section.<name>` are `selection` and `row`: the sibling story's fit review chose
  form A. This story depends on it.

The requester is told in one message before the work starts.

## Fit review

Source read at main `582524ae95`; probes on the 0.94.0 release binary, 2026-11-12.

1. **Need.** A binding that names a section the page does not have, or a member of a section the
   reference does not list, resolves to nothing in a renderer: the read goes out without its
   parameter and a link has no target. The author learns it on screen, not from the check. The
   requester proposed no syntax. Minimal reproduction: the table in the Defect section.
   One row of that table is described wrongly there: `bogusroot.x` is not evaluated to nothing.
   By the reference it is literal text, and the read goes out with the text `bogusroot.x` as its
   parameter (point 2).
2. **Class.** Two classes.
   - `section.<name>` and its member: defect. The reference says "a validator resolves every
     path" (`website/docs/reference/ess-ui.md:55`), `section` is a listed root (`:65`), and the
     check resolves the sibling `channel.<name>` inside an expression (`channel_refs`,
     `schemas/ui/ess-ui.schema.yaml:1756`) and not `section.<name>` (`section_refs`, `:1757`, has
     no "inside expr" subject).
   - An unknown root: not a defect. The same paragraph of the reference says a string that "reads
     no path or function form" is literal text. The check, React and the terminal agree:
     `crates/ui/ess-ui-check/src/expr.rs:7-10`, `expr.ts.tmpl:387-399` and `:408-421`,
     `crates/ui/ess-ui-tui/src/expr.rs:68-76`. `example.com` in an `expr` position is text today
     and stays text. What is missing is a hint: a misspelt root becomes text without a word.
3. **Already expressible?** Nothing to express; the question is what the check reports. Text is
   already stated without doubt by quoting it: a lone quoted string is its value
   (`expr.ts.tmpl:404-406`).
4. **Fit.** The language grids (`docs/design/language-grids.md`) have no row for the binding
   expressions of a UI document; the triage is on the reference's table of forms
   (`ess-ui.md:57-70`) and the check's rule list.
   - Vocabulary: `section.<name>` inside an expression joins the subjects of `section_refs`, as
     `channel.<name>` inside an expression is a subject of `channel_refs`. No new rule id for it.
   - One source: the check reads the roots from the schema's `expressions.forms`
     (`crates/ui/ess-ui-check/src/schema.rs:165-176`, `expr.rs:36-58`), which the reference page
     is generated from. The members of `section.<name>` are read from the same list.
   - Siblings. A probe with one navigate parameter per root, `ess ui check --model`, 0.94.0:

     | navigate parameter | findings added to the baseline |
     |---|---|
     | `channel.nosuch.x` | error `channel_refs`: "`channel.nosuch` names no channel" |
     | `state.nosuch` | none |
     | `params.nosuch` | none |
     | `shell.nosuch` | none |
     | `row.nosuch`, `actor.nosuch` | none |
     | `example.com`, `bogusroot.x` | none |

     `state.<name>`, `params.<name>` and `shell.<name>` name declarations of the document, as
     `section.<name>` and `channel.<name>` do, and are resolved without the model. They are in
     this story: the same walk over the same sites, and the sentence of the reference covers
     them. `row.<field>` and `actor.<field>` need the model and are
     `story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands`.
     `selection`, `draft`, `args`, `widget` and `url.query` depend on where the expression stands;
     they are not in this story, and the implementor reports what the walk finds for them.
   - Targets: the check only. No renderer changes.
5. **Second adopter.** An order page whose customer list reads
   `customer_id: section.ordr.selection.customer_id`: the list goes out without the parameter and
   shows every customer or none, and the check is silent.
6. **Cost.** No format change and no new key. `section_refs` gains a subject; resolving
   `state.`, `params.` and `shell.` adds subjects to the rules that hold those names, or one rule
   where none does. One new warning for a dotted word that reads no path. A document with a wrong
   section, member, state, parameter or shell name now fails the check; such a binding was empty
   on screen. The changelog names the new errors.
7. **Alternatives.** (a) Change nothing: the check stays silent on a binding that is empty on
   screen. (b) The story as drafted, an unknown root is an error: refuses `example.com` and
   `v1.2` in an `expr` position against `ess-ui.md:55`; refused. (c) Chosen: errors for names
   under listed roots, a warning for a dotted word that reads no path. (d) An error for an unknown
   root only at sites that take a path and no text: I don't know which sites the model types that
   way; not taken.
