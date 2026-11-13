---
format: aep.planning-md/3
id: story:ui-check-reports-an-expression-that-reads-a-path-and-is-no-expression
kind: story
status: active
title: ui check reports an expression that reads a path and is no expression of the grammar
summary: 'copy: "shell.base_address + ''/link/'' + row.id" checks with 0 errors and the terminal copies an empty value; joined paths, braces and unknown function forms pass alike (a consumer specification''s UI list, item 4)'
tags:
- backlog
- defect
- ess-ui
- external-request
- from-consumer-report
- severity-medium
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:ui-check-resolves-the-root-and-section-of-every-binding-path
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-13T07:28:48Z", actor: "human:timo", revision: 2, executor: "agent:b10x", correlation: "operator-2026-11-13-check-refuses"}
- {from: "proposed", to: "active", at: "2026-11-13T07:28:49Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-13-check-refuses"}
---
## Defect

`ess ui check` accepts, with no finding, an expression that reads a path and that no renderer can
evaluate: two paths or a path and a text joined by `+`, a path inside braces, a function form the
grammar does not have. The terminal renderer then acts on an empty value or on the expression's
source text. For a `copy` action the user's clipboard holds nothing or the source text, and the
author learns it from the clipboard, not from the check.

The reference (`website/docs/reference/ess-ui.md`, Expressions) says "a renderer evaluates them, a
validator resolves every path", and gives the operators `==`, `!=`, `in [..]`, `not`, `and`, `or`.
It makes a string literal text when it "does not parse, or … joins words with an operator but
reads no path or function form". A string that joins with an operator and does read a path is
neither: it is an expression the grammar has no operator for.

Evidence: a consumer specification's UI document (2026-11-13, ess 0.94.0), item 4 of its third collected list.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`. Each reproduction adds one row action
`{name: link, copy: "<expression>", label: Copy the link}` to the section `list`.

| `copy:` | `ess ui check --model` | the terminal's notice after `act` on row `r1` |
|---|---|---|
| `row.id` | 0 errors, 0 warnings | `copied r1` |
| `shell.base_address + '/link/' + row.id` | 0 errors, 0 warnings | `copied` (empty) |
| `row.title + row.id` | 0 errors, 0 warnings | `copied` (empty) |
| `'request ' + row.id` | 0 errors, 0 warnings | `copied 'request ' + row.id` (the source text) |
| `{row.title}/{row.id}` | 0 errors, 0 warnings | `copied {row.title}/{row.id}` (the source text) |
| `concat(row.title, row.id)` | 0 errors, 0 warnings | `copied` (empty) |
| `page_address(requests.detail, {id: row.id})` | 0 errors, 0 warnings | `copied` (empty) |
| `shell.base_address` (no such shell state) | 0 errors, 0 warnings | `copied` (empty) |
| `url.origin` | 0 errors, 0 warnings | `copied` (empty) |
| `config.base_address` | 0 errors, 0 warnings | `copied config.base_address` (the source text) |
| `env.BASE_ADDRESS` | 0 errors, 0 warnings | `copied env.BASE_ADDRESS` (the source text) |

The terminal's `copy` effect evaluates the expression and copies the empty text when the
evaluation gives nothing (`crates/ui/ess-ui-tui/src/app.rs`, the `action.copy` arm:
`self.eval(…).map(display).unwrap_or_default()`, then `copied {value}`). The React renderer was not
run for this draft.

Red case, three lines of a test on the second row of the table:

```yaml
- open: requests.inbox
- act: pages/requests.inbox/sections/list/rows/r1/row_actions/link
- expect: {at: pages/requests.inbox, text: /link/r1}
```

```
failed  a copy of a joined expression copies the joined value (step 3)
        expect at pages/requests.inbox: expected "/link/r1" at pages/requests.inbox; it shows: … copied …
```

## What the open stories cover and what they do not

`story:ui-check-resolves-the-root-and-section-of-every-binding-path` (active) resolves the roots
at every expression site, `copy` among them: after it, `shell.base_address` without such a shell
state is an error, and `config.base_address` and `env.BASE_ADDRESS` get the warning for a dotted
word whose first segment is no root. It does not cover the rows whose paths all resolve:
`row.title + row.id`, `'request ' + row.id`, `{row.title}/{row.id}`, `concat(…)`,
`page_address(…)`, and `shell.<declared> + '/link/' + row.id`. Those are this story.

Whether an expression may join texts at all is
`story:a-ui-expression-joins-texts` (draft). This story holds whatever that one decides: what the
renderers cannot evaluate, the check refuses.

## Acceptance (sketch, to be fixed by the fit review)

- A string in an `expr` position that reads a path form (a listed root followed by a member) and
  is not an expression of the grammar is an error at the node path of the expression. The message
  names the part that is no expression (`+`, `{`, the unknown function form) and says that the
  grammar joins no texts, or names the form that does once one exists.
- A string that reads no path form stays literal text and gets no finding (`Limit in cents`,
  `not yet sent`, the reference's examples).
- Both renderers and the check read one string alike: no renderer copies, shows or sends the
  source text of a string the check reads as an expression, and none evaluates to empty a string
  the check passes.
- Tests that fail before the change: rows 2 to 7 of the table are errors; row 1 has no finding;
  the three-line test above either passes (a join exists by then) or its document is refused by
  the check.
- Every document under `examples/` keeps its findings.

## Decisions

**Accepted (2026-11-13) on the operator's behalf.** The operator's delegate accepts, as they are
filed, drafts that make the check refuse what a renderer cannot render; this is one. Reason: a
defect against the reference's own sentence, reproduced on the 0.94.0 release binary. It is
scheduled first after the stories in work on that day. The two transitions carry the correlation
`operator-2026-11-13-check-refuses` and the executor `agent:b10x`. No review-result exists for this
draft, so no `review_outcome` record was written.

## Fit review

Owed before dispatch. The acceptance above is a sketch; the seven questions of
`.agents/skills/assessing-external-requests/SKILL.md` fix it, and the first of them to settle is
whether the finding is an error or, as for an unknown root, a warning.
