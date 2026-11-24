---
format: aep.planning-md/3
id: story:a-terminal-test-checks-what-a-host-slot-is-handed-its-resolved-reads-its-commands-and-the-page-s-parameters
kind: story
status: draft
title: 'A terminal test checks what a host slot is handed: its resolved reads, its commands and the page''s parameters'
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal
- depends_on: story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path
revision: 1
---
## Need

A page with a host slot cannot have the slot's part tested on the terminal renderer. The
terminal draws the slot as a box labelled `<host>: <reason>`. A test can only read that label,
and every step that acts inside the slot is refused. The application wants a terminal test to
tell it something about the slot beyond its reason.

Reported by a downstream application on 0.98.0 (third list, item 5).

What a host slot is, and what each renderer does:

- `ess-ui/1`: "A declared place where the host application supplies its own component … ESS
  does not draw it and never counts it as covered". It stands at one of five scopes (page,
  section, overlay, tab `form`, field), declares `host`, `reason`, `stands_in_for`, and the
  `reads` and `commands` the component uses, "and it reads and sends nothing else"
  (`schemas/ui/ess-ui.schema.yaml:1213-1246`; design `.engineering/planning/design/ui-host-slots.md`).
- Terminal: "The label of a host slot: `<host>: <reason>`. The terminal cannot draw the host's
  component, so it marks the slot's place with this" (`crates/ui/ess-ui-tui/src/view.rs:1806-1822`;
  a page slot `:379`; a field slot's cell `:1826-1830`). The design asks for exactly this: "the
  renderer draws a labelled placeholder region at the slot's path … `ess-ui-test/1` can address
  the region by that path. A test step that acts inside the region is refused by name"
  (`ui-host-slots.md:68-73`).
- React: `HostSlot` renders the component the application registers under `host`. It hands that
  component `{adapter, path, page, scope}`, with the adapter narrowed to the declared reads and
  commands (`crates/ui/ess-ui-react/templates/runtime/hosts.tsx.tmpl:30-40`, `:56-76`, `:83-107`).
- Test runner: `inside_host` refuses any step but `expect` at or inside a slot
  (`crates/ui/ess-ui-test/src/parity.rs:257-283`). `host_text` marks a `text` check at a slot
  `test.fixme` in the Playwright spec, because the browser shows the host's component there
  (`:286-300`). The reference lists both (`website/docs/reference/ess-ui-test.md:436`, `:440`).

So the content is the application's own component. The terminal cannot run it, and the browser
runs it only inside the application's build. "Styling is out of scope" bounds this further: the
content's look is the application's. What ESS owns at a slot is the contract: which reads, with
which parameters, which commands and which page parameters the slot is handed. Today no test can
check that contract in either renderer. A regression in what ESS hands a slot (a read parameter
bound to the wrong state, a page parameter missing) passes every terminal test.

The requester proposed, in their words: test "a host slot's content" on the terminal renderer.

## Outcome

Under SP-D5 (A): `expect` at a host slot may name `handed`, what ESS hands the slot at this
moment:

- `page`: the page's parameters;
- `reads`: each declared view with its parameters, evaluated where the slot stands;
- `commands`: the declared commands.

The terminal runner evaluates them from the document and the state it holds. The Playwright
spec marks such a check `test.fixme` with a reason from `ess-ui-test`'s parity rules, because the
browser hands the values to the application's component and draws no element for them. The
terminal keeps drawing the label, and the content stays the application's to test in its own
harness. `ess-ui-test.md` says both.

## Acceptance

- First: a test file under `crates/ui/ess-ui-test/tests/` with a detail page `orders.detail`
  (param `order_id`) holding a section slot
  `{name: route, host: RouteMap, reason: draws the route on a map, stands_in_for: chart, reads:
  {placeholder: orders.Route, fixture: route.yaml, params: {order_id: params.order_id}}}`, and the
  step `expect: {at: pages/orders.detail/sections/route, handed: {page: {order_id: o-1}, reads:
  [{view: orders.Route, params: {order_id: o-1}}]}}`. Today it is refused when the file is read:
  "unknown key `handed`" (`crates/ui/ess-ui-test/src/spec.rs:644`). After the change it passes for
  the page opened at `o-1`, and fails at `o-2` naming both values.
- A `handed` check at a path that is not a host slot is refused when the file is read, naming the
  slot paths of the page.
- A field slot inside a row (`<collection>/rows/<key>/columns/<c>`) evaluates `row.<field>` in its
  read parameters against that row.
- The emitted Playwright spec has `test.fixme` at that step, with the reason. The table "One
  verdict in both renderers" lists it.
- `text` at a slot still reads the label, and the existing host-slot tests
  (`crates/ui/ess-ui-test/tests/host_slots.rs`, `crates/ui/ess-ui-tui/tests/host_slots.rs`) stay
  green.
- `ess-ui-test.md` documents `handed`, and that a host component's content is tested in the
  application's own harness. `CHANGELOG.md` names it.

## Fit review

Verdict: **redesign (gap), behind SP-D5**. Tree `bcfc51555`, read-only.

1. Need: a test proves that a host slot is handed the right reads, commands and parameters.
   The requester's need, the component's own content, is the application's.
2. Class: gap for the contract; local to the application for the content.
3. Already expressible: `text` at the slot reads the label only (`ess-ui-test.md:440`).
4. Fit: one key on `expect`, evaluated with the terminal's existing expression and read
   resolution. No `ess-ui/1` change. The browser is refused by name through the parity rules,
   which the promise allows ("never … different verdicts without saying so", `:416`).
5. Second adopter: every document with host slots, for example any map, editor or vendor
   widget that reads a view by a page parameter.
6. Cost: a new key in `ess-ui-test/1`, accepted without a version. that story adds `width` and
   `open: {node}` the same way. An old runner refuses a file that uses it, as it does that story's.
7. Considered: see SP-D5.

Covers report item 5 of the third list. The generic feature is "the host slot contract is
testable".

Overlaps:

- `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` (P1 item 5) opens a section, overlay or widget
  alone. A section that is a host slot, opened alone, draws only its label. that story should say so,
  and `handed` works on it unchanged. Both add keys to `spec.rs` and rows to the parity table, so
  they land one after the other.
- `story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path` (second report, G7): one `host` in several slots, each
  identified by its path. `handed` is checked per path, so it composes; the slot-path lookup in
  `parity.rs` is shared.
- Draft `story:a-host-slot-in-a-collection-item-is-handed-its-row`: when slots take `args`,
  `handed` gains `args`.
- Active `story:a-form-host-slot-sets-its-value-and-can-hold-submit`: the host's output side.
  Option (B) below would let a terminal test play that output.

## Decisions

- **SP-D5. What of a host slot a test can check.** (A) What ESS hands the slot (`handed`: page
  parameters, resolved reads, commands) in the terminal; `test.fixme` in the browser. M, one new
  `ess-ui-test/1` key. (B) A declared stand-in: the test file gives `fixtures.hosts.<Host>` a
  text the terminal draws instead of the label, and the value a form slot sets. This tests the
  page around the slot and a form's submit, not the component; terminal only; also M.
  (C) Decline: a host component is the application's, tested in the application's own harness.
  The label stays and `ess-ui-test.md` says so (a sentence, XS). **Recommended: (A).** It is the
  part only ESS can check. (C) is the fallback if the operator wants no new test key.

## Scope

- cited: `crates/ui/ess-ui-test/src/spec.rs` (`expect` keys, `:644`), `crates/ui/ess-ui-test/src/runner.rs` (the check), `crates/ui/ess-ui-test/src/parity.rs` (`host_text`, a new reason), `crates/ui/ess-ui-test/src/playwright.rs`
- cited: `crates/ui/ess-ui-tui/src/app.rs` (a read-only accessor that evaluates a slot's read params where it stands; `section_request` is the model)
- cited: `website/docs/reference/ess-ui-test.md` (§ expect, the parity table), `CHANGELOG.md`
- inferred: test fixtures under `crates/ui/ess-ui-test/tests/fixtures/`
- Size: M (about 350 changed lines). Hotfix-sized: no (a feature).

## Depends on

- None hard. Land after `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` (shared `spec.rs` and parity table). `args`
  in `handed` waits for `story:a-host-slot-in-a-collection-item-is-handed-its-row`.
