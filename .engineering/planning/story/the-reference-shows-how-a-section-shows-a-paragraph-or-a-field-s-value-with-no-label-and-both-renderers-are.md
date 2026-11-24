---
format: aep.planning-md/3
id: story:the-reference-shows-how-a-section-shows-a-paragraph-or-a-field-s-value-with-no-label-and-both-renderers-are
kind: story
status: draft
title: The reference shows how a section shows a paragraph or a field's value with no label, and both renderers are pinned to it
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A page shows text that has no label: a lead paragraph under a heading, a short line above a
heading, a line of meta information. The text is a constant of the document or the value of a
field. A downstream application lifting an existing page into `ess-ui/1` on 0.98.0 found no way to
say this, and wrote each text as a labelled field with an invented label (fifth list, item 1).

The construct exists. It is not where a reader of the reference looks for it, and three rules a
paragraph meets are not written next to it.

- The `text` primitive shows a constant or an expression (`text`) or a row field (`field`), with
  no label (`schemas/ui/ess-ui.schema.yaml:1085-1097`, `crates/ui/ess-ui/src/model.rs:2290-2304`).
  The terminal draws only the value (`crates/ui/ess-ui-tui/src/view.rs:2677-2695`). React draws a
  `<span class="ui-text ui-text-<style>">` holding only the value
  (`crates/ui/ess-ui-react/templates/runtime/primitives/text.tsx.tmpl:15-22`).
- A primitive cannot be a section's body: `a primitive cannot stand here; name a component`
  (`model.rs:1266`). It stands in a section's `children`, a record's `item` or a widget's `body`
  (schema `:556`, `:641`, `:901`). The built-in `static_page` kind is a section `body` with
  `component: record` and no read, "fixed content without reads" (schema `:519-521`). A record
  without a read and without fields draws nothing of its own (`view.rs:1023-1044`), so its
  `children` are the section's content. The partner-portal example already writes a caption this
  way (`examples/partner-portal/ui.yaml:292-293`).
- A section's child is addressed and drawn on cells of its own in the terminal
  (`view.rs:805-817`), and the parity rule accepts it (`crates/ui/ess-ui-test/src/parity.rs:366`).
  `expect: {at: …/children/<name>, text: …}` reads it (`website/docs/reference/ess-ui-test.md:208`).
- A record's `item` draws with the record's row in scope, in the terminal (`view.rs:1067-1071`,
  `:1125-1127`) and in React (`templates/runtime/composites/record.tsx.tmpl:89`, `:105`). So
  `{primitive: text, field: summary}` there shows the row's `summary` with no label.

What a field with a label does instead. A label cannot be left out: an absent `label` falls back
to the field's name, in the terminal (`view.rs:2771-2773`) and in React
(`templates/runtime/fields.tsx.tmpl:117-119`). `label: ""` draws an empty `<dt>` in React
(`record.tsx.tmpl:83`) and 16 blank columns before the value in the terminal (`view.rs:2816-2820`).
A labelled field is the wrong construct for a paragraph, and the reference should say so.

Three rules a paragraph meets, which the `text` section of the reference
(`website/docs/reference/ess-ui.md:1742-1770`) does not state:

1. `text` is an `expr`. A constant is literal text only when it reads no path
   (`ess-ui.md:55`; React `templates/runtime/expr.ts.tmpl:404-423`). `true`, `false`, a number and
   `null` are literals, and `null` shows as nothing. A quoted string loses its quotes
   (`crates/ui/ess-ui-tui/tests/literal_text.rs:39`, `:69-72`).
2. A dotted word in a constant, such as a version number, an abbreviation or a host name, draws the
   `expr_roots` warning unless it is quoted (`ess-ui.md:2860`). Quoting the whole paragraph as one
   string literal shows it as written. The tokenizer has no escape inside a string
   (`expr.ts.tmpl:56-61`), so a paragraph holding one quote kind is quoted with the other.
3. A text's `style` is a typographic role (`body`, `caption`, `heading`, `mono`). What a role looks
   like is the theme's `tokens.type` (`ess-ui.md:2645`, `docs/design/ui-style-tokens.md:174-176`).

Minimal reproduction, written fresh and not run, because the assessment read the code and built
nothing:

```yaml
pages:
  home:
    kind: static_page
    title: Home
    sections:
      - name: body
        component: record
        children:
          - {name: kicker, primitive: text, text: "'Release 1.4'", style: caption}
          - {name: lead, primitive: text, text: "'Orders, lines and shipments in one place.'"}
      - name: summary
        component: record
        reads: {placeholder: shop.Summary, fixture: fixtures/summary.yaml}
        item:
          - {name: meta, primitive: text, field: updated_line, style: caption}
```

## Outcome

The reference answers "text with no label" where a reader looks for it, in the `text` primitive
and in the section's `children`. It names the idiom: a `text` primitive in a section's `children`,
a record's `item` or a widget body, and a `static_page`'s record for a section with nothing else.
It states the three rules above. Tests in both renderers and in `ess ui test` pin the idiom, so a
change that adds a label or loses the text turns them red. No key is added.

## Acceptance

1. A journey test of `ess ui test` runs the reproduction's document in the terminal. It holds
   `expect: {at: pages/home/sections/body/children/lead, text: "Orders, lines and shipments in one
   place."}`, the same at `…/children/kicker` for `Release 1.4`, and `expect: {at:
   pages/home/sections/body, not_text: lead}`, so no label or node name is drawn. This test is the
   pin. It is written first and must fail when the terminal draws a label before a text. Mutate
   `primitive_spans` to prefix the node name to check that.
2. The generated Playwright spec for the same test runs the same steps. The parity rule refuses
   none of them.
3. A React test renders the document and finds each text in its own `ui-text` element, with no
   `<dt>` or `<label>` around it.
4. A terminal test draws the `summary` section. The row's `updated_line` value is on a line of its
   own with no label before it.
5. A constant holding a dotted word, quoted as one string literal, draws no `expr_roots` warning
   in `ess ui check`. Written unquoted, it draws the warning. Both cases are pinned in
   `ess-ui-check`'s tests.
6. The schema's `text` doc and `Section.children` note say where a text with no label goes, and
   name the three rules. The reference is generated from the schema by `ess ui docs`. The
   reference's `text` section shows the reproduction as its second example. `CHANGELOG.md` names
   the documentation change.
7. Every test above runs in under 5 s. None may run over 60 s, and the test step stays within
   120 s.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Received 2026-11-24 from
a downstream application, fifth list, item 1.

1. **Need.** A section shows a constant paragraph, or a field's value, with no label. The
   requester's words: "a plain text component; a paragraph or a short line above a heading
   (kicker) without a label".
2. **Class.** *Convenience*. It can be expressed today (question 3). What is missing is a
   reference that says how.
3. **Already expressible?** Yes. Use a `text` primitive in `children`, `item` or a widget body,
   with `static_page`'s record when the section has nothing else. See Need for file and line. The
   suite still checks the figure the reader sees: `expect` reads a section's child on its own cells
   in both renderers (Need). So the idiom is a valid answer under the skill's condition.
4. **Fit.** No new vocabulary. A labelled field stays a labelled field. The primitive's `style` is
   the existing closed role set, and look stays the theme's. That is the owner's line: content is
   the document's, styling the application's.
5. **Second adopter.** An operations console shows a one-line explanation above a filter bar, and a
   record's free-text `notes` field as a paragraph under the record's fields. Both use the same
   idiom.
6. **Cost.** No key, no format version, no diagnostic. A schema `doc` and `note` text change. The
   reference is regenerated, plus tests.
7. **Considered.**
   - (a) Change nothing. Rejected: the application did not find the idiom, and the pitfalls of
     rules 1 and 2 are not written next to it.
   - (b) Let a primitive stand as a section's body, lifting `model.rs:1266`. Deferred to LP-D1 (B):
     it saves one line (`component: record`) and adds a second way to write the same section.
   - (c) A `text` composite kind. Rejected: it duplicates the primitive (red flag "names a concept
     ESS already spells").
   - (d) Kicker and lede roles. Put to LP-D2. Chosen: (a') the idiom, documented and pinned.

## Decisions

**Decline, with the idiom.** It is recorded as this story, which documents and pins the idiom,
because the owner's order makes every gap a story. Open questions:

- **LP-D1. Does a section with only text need an empty record?** (A) Yes. Document `static_page`'s
  record and `children` as the idiom. (B) Also let a primitive stand as a section's body. (C) A
  `text` composite kind. **Recommended: (A).** (B) is a convenience with a second spelling.
  (C) duplicates the primitive.
- **LP-D2. Does the document tell a renderer that a line is a kicker or a lede?** (A) Decline: role
  and look are the application's theme. A kicker is `style: caption`, a lede `style: body`, and the
  theme styles them through `tokens.type` and the node's path. (B) Add `kicker` and `lead` to the
  closed `style` enum, which the renderers may draw as `caption` and `body`. (C) Decline, and the
  application keeps its labelled fields. **Recommended: (A).**

## Scope

- cited: `schemas/ui/ess-ui.schema.yaml` (`text` 1085-1097 doc; `Section.children` 556 note;
  `static_page` 519-521)
- cited: `website/docs/reference/ess-ui.md` (regenerated; `text` 1742-1770)
- inferred: a journey test in `crates/ui/ess-ui-test/tests/` (a module of the crate's one test
  binary once `story:the-test-step-meets-the-test-budget` has merged them)
- inferred: a test in `crates/ui/ess-ui-tui/tests/` and one in `crates/ui/ess-ui-react/tests/`
  (beside `literal_text.rs`)
- inferred: a case in `crates/ui/ess-ui-check/tests/expression_paths.rs`
- inferred: `CHANGELOG.md`
- No change to `examples/partner-portal/ui.yaml`, so the golden file of P0 (`g03-resolved-json`)
  does not move.
- Size: S, about 200 changed lines, mostly tests.

## Depends on

- None to start. It can land before or after P0. The test documents join P0's round-trip corpus
  (`g03-resolved-json` acceptance 5) whenever both have landed.
- Related: `story:a-node-draws-text-written-in-markdown` (draft) is the structured sibling and
  stays separate. `story:a-bare-root-word-reads-alike-in-both-grammars` (active) may change
  rule 1 for a bare root word. If it lands first, the reference text follows it.

## Source

A downstream application, 2026-11-24, fifth list, item 1.
