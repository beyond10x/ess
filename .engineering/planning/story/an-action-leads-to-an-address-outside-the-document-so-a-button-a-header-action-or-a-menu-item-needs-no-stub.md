---
format: aep.planning-md/3
id: story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub
kind: story
status: draft
title: An action leads to an address outside the document, so a button, a header action or a menu item needs no stub page
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it
- depends_on: story:an-action-names-its-icon
revision: 3
---
## Need

A call-to-action button, a header action, a row action, an empty state's action or an account-menu
entry leads to a page outside the application: a source repository, a status page, a public
handbook, a provider's record of an order. In `ess-ui/1` none of them can. A downstream application
lifting an existing page on 0.98.0 wrote an invented stub page for every external link (fifth
list, item 2).

- An `Action` has exactly one effect: `does`, `opens`, `navigate`, `export`, `upload`, `copy` or
  `sets` (`schemas/ui/ess-ui.schema.yaml:1296-1334`, `crates/ui/ess-ui/src/path.rs:602-621`).
- `navigate.to` is a page, `{ref: page}` (schema `:1315`, `model.rs:3084-3092`). `ess ui check`
  refuses any other value under `page_refs` with ``<name>` names no page`
  (`crates/ui/ess-ui-check/src/rules.rs:594-598`, `:1272-1273`; schema `:1759`). The design
  sentence is "Navigation lists every page exactly once" (schema `:325`). A page is a route of the
  app, so a target outside the app is no page, and the app has no stub page to stand in for it.
- The only node that leads outside today is the `link` primitive, `href` beside `to`
  (schema `:1158-1168`: "Prefer `to` (a page of this app); `href` is for external addresses"). It
  stands only where a node stands: in `children`, `item` or a widget body. A header's `actions`, a
  collection's `row_actions`, a `button` primitive's `action`, `empty.action`, a form's `actions`
  and an account menu's `actions` take an `Action`, and an `Action` has no `href`.
- The account menu admits the keys `name`, `label`, `opens`, `does`, `sets` and `navigate` only
  (`rules.rs:188`).
- React runs `navigate` through its router (`crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl:197-202`).
  The terminal runs it with `go` (`crates/ui/ess-ui-tui/src/app.rs:4163-4172`).

Minimal reproduction, written fresh and not run:

```yaml
header:
  actions:
    - {name: source, href: "https://example.org/source", label: Source}
sections:
  - name: hero
    component: record
    children:
      - {name: repo, primitive: button, label: Repository, action: {name: repo, href: "https://example.org/repo"}}
```

`ess ui load` refuses `href` as an unknown field of the action (`model.rs:3013-3014`,
`deny_unknown_fields`). Without it, the author writes a page `repo` that the address cannot reach.

The requester's proposal, in their words: "an action or link to an address outside the document".

## Outcome

An `Action` may have `href: <address>` as its one effect. It reuses the word the `link` primitive
already uses for the same meaning. Running the action opens the address outside the application.

- `href` joins the list of effects, so exactly one of `does`, `opens`, `navigate`, `export`,
  `upload`, `copy`, `sets` and `href` is present.
- An `href` action has a written `name`, as a `copy` action does, because no name can be derived
  from an address (`crates/ui/ess-ui/src/expand.rs:846-853`).
- Under LP-D5 (A), the address is a constant: an `href` that reads a path is refused by
  `ess ui check` as `external_address`. The constant is judged by the rule of `lp2-address-rule`.
- Keys that mean nothing beside `href` are refused by name: `bind`, `optimistic`, `bulk: true`,
  `checked`, `choice`, and `as: toggle` or `as: choice`. `confirm` is kept: a confirm before leaving
  the application is meaningful.
- A toggle primitive's `action` refuses `href`. An on/off switch does not leave the application.
- The account menu admits `href` (`rules.rs:188`), and both renderers draw it there.
- React opens the address as LP-D6 decides: by default `window.open(href, "_blank",
  "noopener,noreferrer")`, and an `<a href target rel>` for `as: link` and `as: menu_item`.
- Under LP-D7 (A), the terminal opens nothing. It draws the action's label with `↗`. Running the
  action records the visit as `App::visits()`, the way `App::exports()` records an export
  (`app.rs:381-388`, `:756`, `:4216-4217`). It shows a notice naming the address.
- A document without `href` means and generates exactly what it did.

## Acceptance

1. The reproduction loads, and `ess ui check` reports no error at `…/header/actions/source` or at
   `…/children/repo/action`. Red today: `ess ui load` refuses `href`.
2. Refusals, each at its node path:
   - an action with both `href` and `navigate` (the exactly-one message names `href`);
   - an `href` action without `name`;
   - `href` beside `bind`, `optimistic`, `bulk: true`, `checked` or `choice`, and with `as: toggle`
     or `as: choice`;
   - a toggle primitive whose `action` has `href`;
   - under LP-D5 (A), `href: row.website`, as `external_address`;
   - `href: "javascript:run()"`, as `external_address`, by the rule of `lp2-address-rule`.
3. React: a test of the generated runtime runs the header action and the button. Each calls
   `window.open` once, with the address, `_blank` and `noopener,noreferrer`, and the router's
   location does not change. An `as: link` action is an anchor with that `href`, `target` and
   `rel`.
4. Terminal: running `…/header/actions/source` leaves the page as it was, adds `{path:
   pages/home/header/actions/source, href: https://example.org/source}` to `App::visits()`, and
   shows a notice naming the address. No process is started. The test runs with an empty `PATH`,
   so a started opener would fail it.
5. An account-menu entry `{name: status, href: "https://example.org/status", label: Status}` is
   drawn and run in both renderers, and `account_menu_keys` reports nothing for it.
6. Old reader. Every committed document under `examples/` and every corpus document of
   `crates/ui/*/tests/` loads to the same `Document`, checks with the same findings, and generates
   the same React and terminal output as before. With P0 landed, `ess ui load --format json`
   prints `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json` unchanged
   (`g03-resolved-json` acceptance 6 and 7).
7. The schema's `Action` lists `href` in `fields` and in `exactly_one_of`, with a `name` note
   saying it must be written. The reference is regenerated by `ess ui docs`. `CHANGELOG.md` names
   the key and the release.
8. Each new test runs in under 5 s, none may run over 60 s, and each test step stays within 120 s.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Received 2026-11-24 from
a downstream application, fifth list, item 2.

1. **Need.** Something a user triggers leads to an address outside the application. See the
   reproduction. The requester's proposal is quoted under Need.
2. **Class.** *Gap*. The `link` primitive leads outside, but no `Action` can, and every place that
   takes an action instead of a node (header, rows, button, empty state, form, account menu) is
   closed to an external address. A stub page is no answer: it moves the figure the reader sees
   (where the button leads) out of the document.
3. **Already expressible?** Partly. A `link` primitive in a section's `children` is expressible
   today. A button, a header action and a menu entry are not. See Need.
4. **Fit.**
   - The word `href` already means "an external address" on the `link` primitive. The action takes
     the same word for the same meaning.
   - `navigate` keeps meaning "a page of this app", so `page_refs` and "Navigation lists every page
     exactly once" are unchanged.
   - It composes with `confirm`, `visible`, `label`, `as` and actor grants (an `href` action needs
     no grant, because it sends no command). It is refused where it means nothing.
   - Siblings: the `link` primitive already has `href`. A navigation menu entry is the one
     remaining place. Under LP-D8 (A) it gets `href` too, in its own story `lp2-nav-href`.
   - Every target handles it: React, the terminal, `ess ui check`, `ess ui test` (`lp2-visit-expect`)
     and the docs. No ESS model target reads a UI action.
5. **Second adopter.** A logistics console's row action opens the carrier's tracking page for a
   shipment. Under LP-D5 (A) that address comes from a row, so it is a `link` primitive in the
   row's `item` or a column `as: link`. The action form serves the constant cases: a header action
   to the carrier's portal, and a help button to the handbook.
6. **Cost.**
   - One optional key in `ess-ui/1`. A document without it keeps its meaning and its generated
     bytes, so no new version is needed under "a new format version is required when meaning …
     changes" (`AGENTS.md` § Determinism and formats).
   - An older `ess` refuses a document that uses `href`, as it refuses every key added within
     `ess-ui/1` (precedent: `story:a-chart-names-its-axes-and-formats-its-ticks`).
   - The public Rust struct `ess_ui::Action` gains a field. That is an API addition for a crate
     user that builds an `Action` by hand. The workspace is pre-1.0.
   - No migration and no new diagnostic code beyond `external_address`, which `lp2-address-rule`
     adds.
7. **Considered.**
   - (a) Change nothing. Rejected: the application keeps its stub pages (question 2).
   - (b) `navigate: {href}`, with `to` made optional. Rejected as the default (LP-D3 B):
     `Navigate` is also the `link` primitive's `to` (`model.rs:2447`), where `to: {href}` would be a
     second spelling of the link's own `href`, and `navigate` would no longer always be a page.
   - (c) A `visit` effect. Rejected: a new word for what `href` already names.
   - (d) Chosen: `href` as an action effect, constant by default.

## Decisions

**Accept, redesigned.** Not a link of its own, which exists, but an action effect with the link's
word. Open questions:

- **LP-D3. How does an action name an external address?** (A) `href: <address>` as an action
  effect beside `navigate`. (B) `navigate: {href}`, `Navigate.to` made optional. (C) Decline: only
  the `link` primitive leads outside, and the application keeps its stub pages.
  **Recommended: (A).**
- **LP-D5. May an action's address come from a row or state?** (A) No: a constant, judged by
  `ess ui check`. A row-given address stays a `link` primitive or a field `as: link`, which already
  read rows. (B) Yes, any expression, judged at run time by the rule of `lp2-address-rule`, and the
  action is not offered when the rule refuses the value. (C) An expression, limited to hosts the
  document lists. **Recommended: (A).** It is the safe default: a button whose destination a row
  can set is an open-redirect surface, and the check can judge only a constant. (B) can follow
  without a version, since it widens what is accepted.
- **LP-D7. Does the interactive terminal open the address?** (A) Never: it shows the address and
  records the visit, in a run and in a test alike. (B) It opens it with the host's opener when run
  interactively, and records it in a headless run. **Recommended: (A).** Starting a program is the
  host's business, and a terminal that sometimes starts one behaves differently in a test.
- **LP-D8. Does a navigation menu section list external entries?** It does: the requester said
  external links sit in the navigation menu, so this decision is now about how, and it is carried
  by `lp2-nav-href`, not by this story. The requester also said the outside entry sits between
  page entries. (A) An entry of `pages` may be `{name, label, href}`, in one ordered list. (B) A
  new ordered key replacing `pages`. (C) `links` beside `pages`, drawn after them.
  **Recommended: (A).** Navigation entries stay out of this story, which keeps it at about
  450 lines.

## Scope

- cited: `crates/ui/ess-ui/src/model.rs` (`Action` 3013-, `Navigate` 3084-3092),
  `crates/ui/ess-ui/src/path.rs` (`check_action` 602-621), `crates/ui/ess-ui/src/expand.rs` (name
  derivation 846-853)
- cited: `schemas/ui/ess-ui.schema.yaml` (`Action` 1296-1334)
- cited: `crates/ui/ess-ui-check/src/rules.rs` (`ACCOUNT_MENU_KEYS` 188; action walk near
  1272; toggle 1352-1360)
- cited: `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl` (run 180-245, `as: link`
  334-), `crates/ui/ess-ui-react/src/emit.rs` (action props),
  `templates/runtime/shell.tsx.tmpl` (`AccountMenu` 142-)
- cited: `crates/ui/ess-ui-tui/src/app.rs` (action run 4163-, `Exported` 381-388, `exports` 756,
  `export` 4216-), `crates/ui/ess-ui-tui/src/view.rs` (action hints)
- inferred: tests in `ess-ui`, `ess-ui-check`, `ess-ui-react` and `ess-ui-tui`
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- Size: M, about 450 changed lines.

## Depends on

- `lp2-address-rule`, for the rule and the check id.
- P0's `g03-loader-fills-defaults` and `g03-resolved-json` land first. `href` has no default, so
  P0's `every_schema_default_is_named_in_the_table` is unaffected. The printed resolved tree carries
  `href` where it is written, and the round-trip test reads it back once a corpus document uses it.
  The partner-portal golden file does not move.
- `story:an-action-names-its-icon` (active) adds keys to the same `Action` in the same files.
  One lands after the other.
- Unblocks `lp2-visit-expect` and `lp2-nav-href`, which reuses `App::visits()` and the notice.

## Source

A downstream application, 2026-11-24, fifth list, item 2.
