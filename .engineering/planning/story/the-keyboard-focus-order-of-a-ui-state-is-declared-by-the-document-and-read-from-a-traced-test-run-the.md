---
format: aep.planning-md/3
id: story:the-keyboard-focus-order-of-a-ui-state-is-declared-by-the-document-and-read-from-a-traced-test-run-the
kind: story
status: draft
title: 'The keyboard focus order of a UI state is declared by the document and read from a traced test run: the focusable nodes in order and the one focused'
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies
revision: 1
---
## Need

No ESS surface says which nodes of a UI state take keyboard focus, in which order, or which one has
it. Focus reachability, focus traps and focus order against reading order (WCAG 2.4.3, 2.1.2,
2.4.11) cannot be checked from ESS today, though on a declared document they are exact graph
checks.

How focus is decided today, by renderer:

- Terminal. `Focus` is `Nav` or `Section(index)` (`crates/ui/ess-ui-tui/src/app.rs:311-314`).
  Tab and Shift-Tab cycle the navigation (when the shell has a navigation region) and the visible
  sections, in the order of the page's `sections` list (`cycle_focus`, `app.rs:2856-2880`, over
  `visible_sections`, `:978-994`): document order, not layout order. Inside a section, rows,
  actions and fields are reached by letter keys and a row cursor (`j`, `k`); in a form being
  edited, Tab and Shift-Tab cycle its fields (`app.rs:3668-3677`). The focus is private state; no
  public method returns it or an order.
- React. Focus is the browser's: DOM order of native controls (buttons, inputs, links) and a few
  `tabIndex={0}` elements such as the drag handle (`templates/runtime/composites/collection.tsx.tmpl:299-303`).
  The overlay frame sets `role="dialog"` and `aria-modal="true"` but has no focus containment and no
  initial focus that this reading found (`templates/runtime/overlays.tsx.tmpl:64-96`). Which row is
  "focused" is a class, `ui-focused` (`collection.tsx.tmpl:366`, `:409`), not DOM focus.
- `ess-ui-test/1`. No step presses a key (the 14 steps, `crates/ui/ess-ui-test/src/lib.rs:43-58`);
  the runner moves focus by calling `App::focus_section` directly (`runner.rs:276`, `:299`, …); no
  check reads focus (`Check`, `spec.rs:195-208`).
- The reference says nothing about focus order. An open draft,
  `story:areas-that-interleave-sections-stack-as-the-terminal-does`, asks in its acceptance 2 that
  "the focus order above `48rem` is stated … grid reading order or section order".

The requester's words: "The terminal renderer exposes no keyboard focus order of a UI state.
Without it focus reachability, focus traps and focus order against reading order … cannot be
checked; … on a declared model they are exact graph checks."

## Outcome

A design page and the `ess-ui/1` reference define the declared focus order of a UI state: the
ordered list of focusable node paths of the page (or open overlay) shown, derived from the document
and the state alone (which page, which overlay, which nodes are visible, which rows are shown). The
terminal computes it by that rule and exposes it with the focused node; a traced `ess ui test`
step record (`g04-step-trace`) carries `focus: {order: [<path>…], at: <path>}`. ESS exposes the
order; reachability, trap and order-against-reading-order checks are the application's (P1-D8).
The renderers' own Tab behaviour is brought to the declared order in `g08a-focus-order-steps`
(terminal and tests) and `g08a-focus-order-react`.

## Acceptance

- First: a traced run of a test over `examples/partner-portal/ui.yaml` that opens a list page and
  then an overlay records, for the page, an order that starts with the navigation and lists each
  visible section's focusable nodes in the rule's order, and, with the overlay open, an order of the
  overlay's nodes only. It fails today: the record has no `focus`.
- `docs/design/ui-focus-order.md` (new) states the rule: which node kinds are focusable (actions,
  form fields, a collection's row set as one stop, links, choices, the navigation entries), their
  order (P1-D9), what an open overlay does to the order (only its nodes, plus its close), what a
  hidden (`visible: false`), collapsed or not-loaded node does (absent from the order), and that a
  host slot is one stop whose inside the host owns.
- The `ess-ui/1` reference (`ess-ui.md`, generated from the schema's docs) gains a "Focus order"
  section stating the rule in reader's words; no key is added to `ess-ui/1`.
- `ess_ui_tui::App` gains `focus_order()` (node paths) and `focused()` (a path); a test pins both on
  three states: a page, a page with a section hidden by `visible`, an open overlay.
- `focused()` maps the terminal's present focus onto the order: the navigation entry, the section's
  first focusable node, or the edited form field. Where the terminal's Tab today skips nodes the
  order lists, the record still lists them; the design page says the terminal follows the rule
  only after `g08a-focus-order-steps`.
- Without `--trace` nothing changes; `formats.md:221` names the optional `focus` key.

## Fit review

Tree `bcfc51555`, read-only. Report item 8a of a downstream application's report of 2026-11-24.

1. **Need.** Read, for a UI state, which nodes take focus and in which order, so focus checks run
   on the declared document rather than on a browser. Requester's wording quoted under Need.
2. **Class.** *Gap*: no surface states or exposes a focus order (Need). Not a defect: no page
   promises one; the open draft above asks for one statement of it.
3. **Already expressible?** No. The document's node order is readable, but the renderers' focus
   differs from it (terminal: sections only; React: DOM controls), so node order alone is not the
   focus order of any renderer.
4. **Fit.**
   - The state the order is read from is the UI state of report items 1 and 2 (P0 group). This
     story designs no second state form: it computes from the terminal's state now and reads the
     P0 state when it exists; the trace key sits beside the state digest of
     `g04-step-trace-state`.
   - Nodes are named by canonical node paths, as every UI tool names them.
   - No `ess-ui/1` key: the order is derived, not authored. An authored `tab_index` would be a
     second way to say what the document order says.
   - Every renderer: the rule is renderer-neutral; terminal exposure here, terminal Tab and tests
     in `g08a-focus-order-steps`, React in `g08a-focus-order-react`.
   - `ess-ui` keeps its one dependency (`ess-yaml`): the rule needs the document and the state only.
5. **Second adopter.** An accessibility audit of a generated admin console wants WCAG 2.4.3
   evidence per page without a browser; a public-sector application must show that every overlay
   can be left by keyboard.
6. **Cost.** No `ess-ui/1` or `ess-ui-test/1` key; one optional key in traced reports; one design
   page and a reference section; two public `App` methods. Renderer behaviour changes come in the
   follow-up stories.
7. **Considered.**
   - (a) Change nothing: refused (question 3).
   - (b) Each renderer exposes its own actual order, stated per renderer (P1-D7 B): cheaper, but
     the application's graph checks would then hold for one renderer and say nothing of the other.
   - (c) ESS checks reachability and traps itself in `ess ui check` (P1-D8 B): useful, but a policy
     choice (what counts as a trap with a close action) that the application already owns; can
     follow once the order is exposed.
   - (d) Chosen: a declared, renderer-neutral order, exposed first.

## Decisions

**Accept, redesigned.** What changed from the request: the order is ESS's declared rule, not the
terminal's present Tab behaviour, and ESS exposes it without judging it.

- **P1-D7. Who defines the focus order.** (A) ESS declares it from document and state, renderers
  are brought to it. (B) Each renderer exposes its own order, stated per renderer. (C) Decline; the
  application derives an order from the document. **Recommended: (A).**
- **P1-D8. Does ESS check it.** (A) Expose only; the application runs reachability, trap and
  reading-order checks. (B) `ess ui check` also reports unreachable nodes and overlays without a
  keyboard exit. (C) Both now. **Recommended: (A), expose first.**
- **P1-D9. Order between sections of a page with `layout: columns` or `areas`.** (A) Section
  declaration order, the same in every renderer, with an `ess ui check` warning where the wide
  layout's reading order differs from it. (B) The wide layout's reading order (columns left to
  right, areas row by row). (C) Per renderer. **Recommended: (A).** It answers acceptance 2 of
  `story:areas-that-interleave-sections-stack-as-the-terminal-does`.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (`Focus`, `cycle_focus`, `visible_sections`, public
  `focus_order`, `focused`)
- cited: `crates/ui/ess-ui-test/src/lib.rs`, `crates/ui/ess-ui-test/src/runner.rs` (trace key)
- inferred: `docs/design/ui-focus-order.md` (new)
- cited: `schemas/ui/ess-ui.schema.yaml` (a doc block for the reference), `website/docs/reference/ess-ui.md`
  (regenerated), `website/docs/reference/formats.md`, `CHANGELOG.md`
- inferred: a test module in `crates/ui/ess-ui-tui/tests/` and one in `crates/ui/ess-ui-test/tests/`
- Size: L (about 550 changed lines).

## Depends on

- `g04-step-trace` (the record the key goes into).
- Reads the UI state of report items 1 and 2 (P0 group) once it exists; does not wait for it.
