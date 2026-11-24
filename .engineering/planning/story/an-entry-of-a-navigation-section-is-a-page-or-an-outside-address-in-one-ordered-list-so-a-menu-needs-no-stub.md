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
revision: 2
---
## Need

The application's menu has an entry, on every page, that leads to another site. That entry sits
between page entries: page, page, outside link, page. A downstream application lifting existing
pages on 0.98.0 wrote a stub page for each such entry (fifth list, item 2). The requester confirmed
both points: the external links sit in the navigation menu, and between the page entries. Buttons
that lead outside are the action case, `lp2-action-href`.

How a menu entry names its target today:

- `NavSection.pages` is required. It is a fixed list of page names, or `DynamicNavEntries`
  (`from_view`, `page`, `param`, `label`, `filter`), whose entries all open one page
  (`schemas/ui/ess-ui.schema.yaml:344-368`). In Rust it is
  `NavPages::Fixed(Vec<String>) | NavPages::Dynamic(DynamicNavEntries)`, untagged
  (`crates/ui/ess-ui/src/model.rs:494-535`).
- The fixed list is read in five places:
  - `ess ui check`'s `navigation` rule (`crates/ui/ess-ui-check/src/rules.rs:600-657`):
    `nav_resolves` requires a page at every entry ("`<name>` names no page"), `nav_unique` requires
    each page once, and `page_reachable` requires every page to be listed or hidden (schema
    `:1754-1756`);
  - React's emitter, which writes one `{page, label, synonyms}` per entry
    (`crates/ui/ess-ui-react/src/emit.rs:3070-3104`). The template draws each as a router `<Link>`
    in one `<ul>`, and only the section carries a `data-ui-path`
    (`templates/runtime/navigation.tsx.tmpl:12-17`, `:99-150`);
  - the terminal's `nav`, which builds one `NavEntry {label, page, params, synonyms}` per entry
    (`crates/ui/ess-ui-tui/src/app.rs:390-404`, `:1007-1029`). The pane draws each as a label
    line (`crates/ui/ess-ui-tui/src/view.rs:315-351`), and Enter opens the page
    (`app.rs:2955-2960`);
  - a test of the example (`crates/ui/ess-ui/tests/example.rs:175`).
  Dynamic entries are read in `ess-ui-check/src/filter.rs:90`, `model.rs:1222` and
  `ess-ui-tui/src/app.rs:1575`, all untouched here.
- `Navigation.visibility` (`by_actor_grants`, `by_roles`) is checked as an enum only
  (`ess-ui-check/src/enums.rs:219`). Neither renderer reads it, so no grant decides an entry today.
- The design sentence is "Navigation lists every page exactly once" (schema `:325`). An outside
  address is no page, so nothing in the menu can name one.
- A test cannot reach a menu entry. The node walk stops at `navigation/sections/<s>`
  (`crates/ui/ess-ui/src/path.rs:129-135`), and no renderer registers a region for an entry.

Minimal reproduction, written fresh and not run:

```yaml
navigation:
  home: home
  sections:
    - name: main
      pages:
        - home
        - orders.list
        - {name: handbook, label: Handbook, href: "https://example.org/handbook"}
        - shipments.list
```

`ess ui load` refuses it today: the entry matches neither `NavPages` variant.

## Outcome

Under LP-D8 (A), an entry of a fixed `pages` list is a page name, as today, or an outside link
`{name, label, href, icon?, synonyms?}`. One list holds both, in the order written, and both
renderers draw them in that order.

- In Rust, `NavPages::Fixed(Vec<NavItem>)` with `NavItem::Page(String) | NavItem::Link(NavLink)`,
  untagged. A list of names reads exactly as before.
- `href` is a string, so a constant by construction. `ess ui check` judges it with the rule of
  `lp2-address-rule` and reports a refused address as `external_address` at
  `navigation/sections/<s>/pages/<name>/href`. `label` is required, because there is no page title
  to fall back to.
- The three navigation checks on a mixed list:
  - `nav_resolves` resolves the page entries and skips the links;
  - `nav_unique` holds pages to one listing each, as before. It also holds a link's `name` unique
    in its section and distinct from the names of the pages that section lists, so a path names
    one entry;
  - `page_reachable` counts page entries only, as before. A link lists no page.
- A `DynamicNavEntries` section takes no link. Its entries come from rows, and a row-given menu
  address is the data-driven case LP-D5 keeps out.
- A link needs no grant, and is shown whenever its section is shown. `restrictions` hide it with
  its section. The menu search matches its `label` and `synonyms` as it matches a page entry's.
- A link is a node, `navigation/sections/<s>/pages/<name>`.
- React draws it in its place in the section's list, as `<li data-ui-path=…><a href target rel>`,
  opened as LP-D6 decides (by default a new tab with `rel="noopener noreferrer"`). It is never
  `aria-current`.
- The terminal draws it in its place in the pane as `  <label> ↗`, and the cursor reaches it like
  any entry. While the cursor rests on it, the pane's bottom border names the address. Enter opens
  nothing (LP-D7): it records the visit in `App::visits()` with the link's path, and shows a notice
  naming the address, as an `href` action does (`lp2-action-href`).
- In a test, `act` at the link's path runs it in both renderers. The terminal moves the navigation
  cursor to it and presses Enter. The Playwright spec clicks the anchor. `expect_visit`
  (`lp2-visit-expect`) then holds, and `expect: {at: <link>, text: Handbook}` reads its line. A
  page entry stays unaddressed. Acting on it, or on its section, is refused by one parity rule
  with one reason in both renderers.

## Acceptance

1. The reproduction loads, and `ess ui check` reports nothing at `navigation`. Red today.
2. Order. React's navigation lists `home`, `orders.list`, `Handbook`, `shipments.list` in that
   order, and so does the terminal's pane.
3. Refusals, each at its path:
   - `href: "javascript:run()"` (`external_address`);
   - a link without `label`;
   - two links named `handbook` in one section;
   - a link named `orders.list` beside the page `orders.list` (`nav_unique`);
   - a link inside `DynamicNavEntries`.
   `nav_resolves` reports no finding for a link. `page_reachable` still reports a page that no
   section lists as a page entry, whatever links the sections hold.
4. React: `Handbook` is an anchor with `href`, `target="_blank"` and `rel="noopener noreferrer"` at
   `navigation/sections/main/pages/handbook`. A click does not move the router's location.
5. Terminal: with the cursor on `Handbook ↗`, the border names `https://example.org/handbook`.
   Enter leaves the page as it was, adds `{path: navigation/sections/main/pages/handbook, href:
   https://example.org/handbook}` to `App::visits()`, and starts no process (the test runs with an
   empty `PATH`).
6. `ess ui test`, in the terminal and in the generated Playwright spec:
   `act: navigation/sections/main/pages/handbook`, then
   `expect_visit: {href: "https://example.org/handbook"}`, holds. The spec makes no network request.
   `act: navigation/sections/main` is refused in both with the same reason.
7. A restriction that hides section `main` hides the link in both renderers. A menu search for
   `hand` keeps it in React.
8. Old reader:
   - Every committed document under `examples/` and every corpus document of `crates/ui/*/tests/`
     loads to a `Document` that, through the new types, lists the same pages in the same order.
   - Each checks with the same findings and generates the same React and terminal output, byte for
     byte.
   - With P0 landed, `ess ui load --format json` prints
     `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json` unchanged.
   - An `ess` before this change refuses the reproduction at the section's `pages`, and never draws
     a menu without the link. This is stated in the reference, not tested: the old binary is not
     built.
9. The schema's `NavSection.pages` is `{one_of: [{list: {one_of: [{ref: page}, NavLink]}},
   DynamicNavEntries]}`, with a `NavLink` construct. The reference is regenerated, and
   `ess-ui-test.md`'s `act` names a navigation link. `CHANGELOG.md` names the entry form, and the
   Rust change to `NavPages::Fixed` for a crate user.
10. Each new test runs in under 5 s, none may run over 60 s, and each test step stays within 120 s.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Received 2026-11-24 from
a downstream application, fifth list, item 2. The requester answered twice the same day: the
external links sit in the navigation menu, and between the page entries.

1. **Need.** A menu entry leads outside the application, in its place among the page entries, on
   every page.
2. **Class.** *Gap*. Every menu entry is a page (`nav_resolves`), and a stub page moves the figure
   the reader sees out of the document.
3. **Already expressible?** No. An account-menu entry with `href` (`lp2-action-href`) is not the
   navigation, and a `link` primitive stands on one page.
4. **Fit.**
   - `href` means an outside address, as on the `link` primitive and the action.
   - The order is the list's own order, so no position key is needed.
   - The checks keep their meaning for pages: every page is listed once, and every page is
     reachable.
   - It composes with `restrictions`, `search` and `icon`.
   - Siblings: the action and the `link` primitive take `href`. Dynamic entries do not (LP-D5).
5. **Second adopter.** An admin app's side menu: Orders, Shipments, then "Status" to the provider's
   status page, then Settings.
6. **Cost.**
   - One more entry form inside an existing key of `ess-ui/1`, with no version. A list of page
     names, which is every existing document, reads, checks and generates as before, so meaning
     and bytes are kept (`AGENTS.md` § Determinism and formats).
   - An older `ess` refuses a document that uses the form at `pages`. It does not drop the entry.
     Its message names the untagged enum, not the form, and that cannot be changed retroactively.
   - The public `ess_ui::NavPages::Fixed` changes from `Vec<String>` to `Vec<NavItem>`. That is a
     breaking change for a crate user that matches it, five places in this workspace. The
     workspace is pre-1.0, and `CHANGELOG.md` names it.
   - No new check id: it reuses `external_address` and extends `nav_unique`'s duty.
7. **Considered.**
   - (a) `links` beside `pages`, drawn after them. Rejected on the requester's second answer: the
     outside entry sits between pages.
   - (b) A new ordered key, say `entries`, that replaces `pages` when present. It keeps `pages`
     pure and `NavPages` unchanged. An older reader refuses `entries` by name, a clearer message.
     But it is a second key for "the entries of this section", and every reader and check then
     reads two keys (red flag: one concept, two spellings).
   - (c) Chosen: an entry of `pages` may be a link. One list, one order, no second key.

## Decisions

**Accept, redesigned.** The entry form is decided by LP-D8 (A), with LP-D4, LP-D6 and LP-D7 as for
the action. It stays a story of its own, not part of `lp2-action-href`. Together they would be about
1,000 changed lines, and the menu touches files the action does not: the navigation model, the
three navigation checks, `navigation.tsx.tmpl`, the terminal's pane, and test addressing.

## Scope

- cited: `schemas/ui/ess-ui.schema.yaml` (`NavSection` 344-354; checks 1754-1756)
- cited: `crates/ui/ess-ui/src/model.rs` (`NavSection`, `NavPages` 494-516),
  `crates/ui/ess-ui/src/path.rs` (node walk 129-135)
- cited: `crates/ui/ess-ui-check/src/rules.rs` (`navigation` 600-657)
- cited: `crates/ui/ess-ui-react/src/emit.rs` (3070-3104),
  `templates/runtime/navigation.tsx.tmpl` (12-17, 99-150)
- cited: `crates/ui/ess-ui-tui/src/app.rs` (`NavEntry` 390-404, `nav` 1007-1029, `nav_key` near
  2955), `crates/ui/ess-ui-tui/src/view.rs` (`draw_nav` 315-351)
- cited: `crates/ui/ess-ui/tests/example.rs` (175, the match on `NavPages`)
- inferred: `crates/ui/ess-ui-test/src/runner.rs`, `parity.rs`, `playwright.rs` (act at a navigation
  link)
- inferred: tests in `ess-ui`, `ess-ui-check`, `ess-ui-react`, `ess-ui-tui` and `ess-ui-test`
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `website/docs/reference/ess-ui-test.md`,
  `CHANGELOG.md`
- Size: M, about 560 changed lines: about 330 of code, 190 of tests and 40 of documentation. If
  the implementor's own estimate passes 600, the test-runner part (`act` at a navigation link in
  both renderers, about 120 lines) splits off as a follow-up unit, and acceptance 6 moves with it.

## Depends on

- `lp2-address-rule` (rule, `external_address`), `lp2-action-href` (`App::visits()`, the notice),
  and `lp2-visit-expect` (`expect_visit`). It lands after all three.
- P0's `g03-loader-fills-defaults` and `g03-resolved-json` land first. The new form has no
  default, so there is no fill row. P0's key walk follows the `one_of` inside the list.
- `story:the-terminal-addresses-every-node-the-browser-addresses` (active) changes the same test
  addressing code. One lands after the other.

## Source

A downstream application, 2026-11-24, fifth list, item 2, and the requester's two answers of the
same day.
