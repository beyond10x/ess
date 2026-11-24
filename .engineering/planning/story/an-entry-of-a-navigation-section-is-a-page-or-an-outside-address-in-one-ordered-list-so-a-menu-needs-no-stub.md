---
format: aep.planning-md/3
id: story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub
kind: story
status: draft
title: An entry of a navigation section is a page or an outside address, in one ordered list, so a menu needs no stub page
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it
- depends_on: story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub
- depends_on: story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a
revision: 1
---
## Need

The application's menu has an entry, on every page, that leads to another site: a handbook, a
source repository, a status page. A downstream application lifting existing pages on 0.98.0 wrote a
stub page for each such entry (fifth list, item 2). The requester confirmed that these external
links sit in the navigation menu, as a top-bar entry. Buttons that lead outside are the action
case, `lp2-action-href`.

How a menu entry names its target today:

- A `NavSection` has `name`, `label`, `icon` and `pages`, and `pages` is required. Its entries are a
  fixed list of page names, or `DynamicNavEntries` (`from_view`, `page`, `param`, `label`), whose
  entries all open one page (`schemas/ui/ess-ui.schema.yaml:344-368`,
  `crates/ui/ess-ui/src/model.rs:494-535`). Every entry is a page.
- `ess ui check` requires a page at every entry under `nav_resolves` ("`<name>` names no page"). It
  requires each page to be listed once under `nav_unique`, and every page to be listed or hidden
  under `page_reachable` (`crates/ui/ess-ui-check/src/rules.rs:600-657`; schema `:1754-1756`). The
  design sentence is "Navigation lists every page exactly once" (schema `:325`). An external
  address is no page, so nothing in the menu can name one.
- React draws each entry as a router `<Link to={hrefFor(page, params)}>` inside its section's
  `<ul>`. Only the section carries a `data-ui-path`
  (`crates/ui/ess-ui-react/templates/runtime/navigation.tsx.tmpl:99-150`,
  `crates/ui/ess-ui-react/src/emit.rs:3070-3110`).
- The terminal builds `NavEntry {label, page, params, synonyms}` per entry (`crates/ui/ess-ui-tui/src/app.rs:390-404`,
  `:1007-1035`). It draws each entry as `▸`/blank plus its label in the navigation pane
  (`crates/ui/ess-ui-tui/src/view.rs:315-351`), and Enter opens the page (`app.rs:2955-2960`).
- A test cannot reach a menu entry. The node walk stops at `navigation/sections/<s>`
  (`crates/ui/ess-ui/src/path.rs:129-135`). Neither renderer registers a region for an entry, and
  `ess-ui-test/1` has no step on the menu: `open` names a page.

Minimal reproduction, written fresh and not run:

```yaml
navigation:
  home: home
  sections:
    - name: main
      pages: [home, orders.list]
      links:
        - {name: handbook, label: Handbook, href: "https://example.org/handbook"}
```

`ess ui load` refuses `links` as an unknown field of the navigation section
(`model.rs:495`, `deny_unknown_fields`).

## Outcome

Under LP-D8 (A), a `NavSection` takes `links: [{name, label, href, icon?, synonyms?}]` beside
`pages`, and its entries are drawn after the section's pages. A section with `links` may leave out
`pages`.

- `href` is a string, so a constant by construction. `ess ui check` judges it with the rule of
  `lp2-address-rule` and reports a refused address as `external_address` at
  `navigation/sections/<s>/links/<name>/href`. `label` is required, because there is no page title
  to fall back to. `name` is unique among a section's links.
- `nav_resolves`, `nav_unique` and `page_reachable` are unchanged: a link is not a page. Under
  `visibility: by_actor_grants`, a link needs no grant and is shown whenever its section is shown.
  `restrictions` hide it with its section. The menu search matches its `label` and `synonyms` as it
  matches a page entry's.
- Each link is a node, `navigation/sections/<s>/links/<name>`.
- React draws it as `<li data-ui-path=…><a href target rel>` in the section's list, opened as LP-D6
  decides (by default a new tab with `rel="noopener noreferrer"`). It is never `aria-current`.
- The terminal draws it in the pane as `  <label> ↗`. The cursor reaches it like any entry. While
  the cursor rests on it, the pane's bottom border names the address. Enter opens nothing (LP-D7):
  it records the visit in `App::visits()`, with the link's node path, and shows a notice naming the
  address, as an `href` action does (`lp2-action-href`).
- In a test, `act` at the link's path runs it in both renderers. The terminal moves the navigation
  cursor to the entry and presses Enter. The Playwright spec clicks the anchor. `expect_visit`
  (`lp2-visit-expect`) then holds, and `expect: {at: <link>, text: Handbook}` reads its line. A
  page entry of the menu stays unaddressed. Acting on it, or on its section, is refused by one
  parity rule with one reason in both renderers.

## Acceptance

1. The reproduction loads, and `ess ui check` reports nothing at `navigation`. Red today:
   `ess ui load` refuses `links`.
2. Refusals, each at its path:
   - `href: "javascript:run()"` (`external_address`);
   - a link without `label`;
   - two links named `handbook` in one section;
   - a section with neither `pages` nor `links`.
3. React: the generated navigation draws `Handbook` as an anchor with `href`,
   `target="_blank"` and `rel="noopener noreferrer"` at `navigation/sections/main/links/handbook`,
   after `home` and `orders.list`. A click does not move the router's location.
4. Terminal: the pane draws `Handbook ↗` after the two page entries. With the cursor on it, the
   border names `https://example.org/handbook`. Enter leaves the page as it was, adds
   `{path: navigation/sections/main/links/handbook, href: https://example.org/handbook}` to
   `App::visits()`, and starts no process (the test runs with an empty `PATH`).
5. `ess ui test`, in the terminal and in the generated Playwright spec:
   `act: navigation/sections/main/links/handbook`, then
   `expect_visit: {href: "https://example.org/handbook"}`, holds. The spec makes no network request.
   `act: navigation/sections/main` is refused in both with the same reason.
6. A restriction that hides section `main` hides the link in both renderers. A menu search for
   `hand` keeps it in React.
7. Old reader. Every committed document under `examples/` and every corpus document loads to the
   same `Document`, checks with the same findings, and generates the same output.
   `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json` is unchanged once P0 has landed.
8. The schema's `NavSection` lists `links`, and `pages` is `required: {unless: links}`, the form
   `required: {unless: HostSlot}` already takes (schema `:419`). The reference is regenerated, and
   `ess-ui-test.md`'s `act` names a navigation link. `CHANGELOG.md` names the key.
9. Each new test runs in under 5 s, none may run over 60 s, and each test step stays within 120 s.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Received 2026-11-24 from
a downstream application, fifth list, item 2, with the requester's answer that the external links
sit in the navigation menu.

1. **Need.** A menu entry leads outside the application, on every page.
2. **Class.** *Gap*. Every menu entry is a page (`nav_resolves`), and a stub page moves the figure
   the reader sees out of the document.
3. **Already expressible?** No. An account-menu entry with `href` (`lp2-action-href`) is not the
   menu the reader sees as the navigation. A `link` primitive in a section stands on one page, not
   on every page.
4. **Fit.**
   - `href` means an external address, as on the `link` primitive and the action.
   - `pages` keeps meaning pages, so the three navigation checks and "Navigation lists every page
     exactly once" are unchanged.
   - It composes with `restrictions`, `visibility`, `search` and `icon`.
   - Siblings: the action and the `link` primitive take `href`. `DynamicNavEntries` gets no
     `href`, because a row-given address in a menu is the data-driven case LP-D5 keeps out.
5. **Second adopter.** An internal admin app with a "Status" entry to the provider's status page,
   and a "Handbook" entry to the team wiki, in its side menu.
6. **Cost.**
   - One optional key on `NavSection` in `ess-ui/1`, and `pages` optional beside it. A document
     without `links` keeps its meaning and bytes, so no new version is needed.
   - An older `ess` refuses `links` as unknown.
   - A public `ess_ui::NavSection` field. No new check id: it reuses `external_address`.
7. **Considered.**
   - (a) Account-menu entries only (the former default of LP-D8). Rejected on the requester's
     answer.
   - (b) A record inside `pages` (LP-D8 B). It keeps the order among page entries, but makes
     `pages` hold non-pages and changes `NavPages::Fixed(Vec<String>)` for every reader.
   - (c) Chosen: `links` beside `pages`, drawn after them.

## Decisions

**Accept, redesigned.** Not a page, but a link entry with the action's and the link primitive's
word. It is decided by LP-D8 (A, now the default), with LP-D4, LP-D6 and LP-D7 as for the action.
It is a story of its own, not part of `lp2-action-href`. Widening that story would take it from
about 450 to about 950 changed lines. The menu also touches files the action does not: the
navigation model, the three navigation checks, `navigation.tsx.tmpl`, the terminal's navigation
pane, and test addressing.

## Scope

- cited: `schemas/ui/ess-ui.schema.yaml` (`NavSection` 344-354)
- cited: `crates/ui/ess-ui/src/model.rs` (`NavSection` 494-506), `crates/ui/ess-ui/src/path.rs` (node
  walk 129-135)
- cited: `crates/ui/ess-ui-check/src/rules.rs` (`navigation` 600-657)
- cited: `crates/ui/ess-ui-react/src/emit.rs` (3070-3110), `templates/runtime/navigation.tsx.tmpl`
  (99-150)
- cited: `crates/ui/ess-ui-tui/src/app.rs` (`NavEntry` 390-404, `nav` 1007-1035, `nav_key` near 2955),
  `crates/ui/ess-ui-tui/src/view.rs` (`draw_nav` 315-351)
- inferred: `crates/ui/ess-ui-test/src/runner.rs`, `parity.rs`, `playwright.rs` (act at a navigation
  link)
- inferred: tests in `ess-ui`, `ess-ui-check`, `ess-ui-react`, `ess-ui-tui` and `ess-ui-test`
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `website/docs/reference/ess-ui-test.md`,
  `CHANGELOG.md`
- Size: M, about 500 changed lines.

## Depends on

- `lp2-address-rule` (rule, `external_address`), `lp2-action-href` (`App::visits()`, the notice),
  `lp2-visit-expect` (`expect_visit`). It lands after all three.
- P0's `g03-loader-fills-defaults` and `g03-resolved-json` land first. `links` has no default, so
  there is no fill row.
- `story:the-terminal-addresses-every-node-the-browser-addresses` (active) changes the same test
  addressing code. One lands after the other.

## Source

A downstream application, 2026-11-24, fifth list, item 2, and the requester's answer of the same
day.
