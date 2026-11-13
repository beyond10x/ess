---
format: aep.planning-md/3
id: story:a-ui-expression-reads-a-value-the-deployment-sets
kind: story
status: draft
title: A UI expression reads a value the deployment sets
summary: No root of an ess-ui/1 expression is a configured value such as the address the app is served at (a consumer specification's UI list, item 2)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:a-view-field-reads-a-setting-whose-value-differs-per-environment
revision: 1
---
## Need

Some values of a user interface differ per deployment and are no fact of the model: the address
the app is served at, the address of a help site, the name of the environment shown in a banner.
An `ess-ui/1` expression reads state, params, rows, the actor and the shell's state
(`website/docs/reference/ess-ui.md`, Expressions); none of its roots is a value the deployment
sets. `url.query.<name>` is readable in guards only, and no root gives the address the app was
opened at.

Evidence: a consumer specification's UI document (2026-11-13, ess 0.94.0), item 2 of its third collected list.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`. Each reproduction adds one row action
`{name: link, copy: "<expression>", label: Copy the link}` to the section `list`.

| `copy:` | `ess ui check --model` | the terminal's notice after `act` on row `r1` |
|---|---|---|
| `shell.base_address` (no such shell state) | 0 errors, 0 warnings | `copied` (empty) |
| `url.origin` | 0 errors, 0 warnings | `copied` (empty) |
| `config.base_address` | 0 errors, 0 warnings | `copied config.base_address` (the source text) |
| `env.BASE_ADDRESS` | 0 errors, 0 warnings | `copied env.BASE_ADDRESS` (the source text) |

The check passes all four; `story:ui-check-resolves-the-root-and-section-of-every-binding-path`
(active) makes the first an error and gives the dotted words with an unknown root a warning.

Related, not the same: `story:a-view-field-reads-a-setting-whose-value-differs-per-environment`
(draft) is the model's side, a `setting.<name>` in a view field.
`story:a-generated-react-app-runs-under-a-base-path` (active) is the path the generated project is
served under, known at build time.

## What a design has to settle

- Whether the document declares the values it reads (a name, a type, a default for fixtures and
  tests) and the deployment supplies them, as a shell's state is declared, or whether the answer is
  a view of the model that publishes a setting.
- Where the value comes from in each renderer: the generated React project at build or at start,
  the terminal from its launch, a test from the test file.
- Whether the address the app is opened at is such a value or a root of its own; a design for
  `story:an-action-copies-the-address-of-a-page` may make it unnecessary for links.
- What `ess ui check` resolves: an undeclared name is an error, as for `channel.<name>`.
- That no secret is such a value: everything a UI document reads reaches the browser.

## Acceptance (sketch, to be fixed by the fit review)

- Either the reference names the idiom (a model view publishing the setting) and shows it running,
  or a document declares a deployment value once, an expression reads it in every `expr` position,
  both renderers show the supplied value, a test sets it, and the check refuses an undeclared one.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
