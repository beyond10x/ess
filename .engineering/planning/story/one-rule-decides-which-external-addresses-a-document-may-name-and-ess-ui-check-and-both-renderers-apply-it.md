---
format: aep.planning-md/3
id: story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it
kind: story
status: draft
title: One rule decides which external addresses a document may name, and ess ui check and both renderers apply it alike
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
revision: 1
---
## Need

An `ess-ui/1` document already names addresses outside itself in three places:

- the `link` primitive's `href`, an `expr` (`schemas/ui/ess-ui.schema.yaml:1158-1168`,
  `crates/ui/ess-ui/src/model.rs:2440-2450`);
- `help.link`, a string, on the header, a field and a form group (schema `:741`, `:975`, `:1005`);
- a field drawn `as: link`, whose address is the row's value (schema `:962`).

Only React judges such an address, and only at run time. `safeUrl` admits `http:`, `https:` and
`mailto:` and a relative reference, parsed the way the browser parses it
(`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:873-884`). Any other address is drawn as
text: a script address, `data:`, `file:`, or a scheme hidden behind leading spaces or tabs
(`templates/runtime/primitives/link.tsx.tmpl:28-43`, `fields.tsx.tmpl:55-68`, `:172-183`,
`core.tsx.tmpl:758-764`). Three things are missing:

- `ess ui check` says nothing about a constant address. `page_refs` judges a page target only
  (`crates/ui/ess-ui-check/src/rules.rs:594-598`, `:1340-1350`; schema `:1759`). A document with
  `href: "javascript:run()"` checks clean, and React silently draws it as text.
- The terminal shows a link primitive's text and `↗`, never its address
  (`crates/ui/ess-ui-tui/src/view.rs:2717-2723`). It prints a help link as given, whatever its
  scheme (`view.rs:591-593`, `help_line` in `view.rs`).
- The React targets disagree. A field link and a help link open a new tab with `rel="noreferrer"`
  (`fields.tsx.tmpl:65`, `:179`, `core.tsx.tmpl:761`). The link primitive opens in the same tab with
  no `rel` (`link.tsx.tmpl:40`).

Report item 2 of the fifth list (an action that leads outside the document, `lp2-action-href`) adds
a fourth site. Before that, the rule must exist once, so that the new site reuses it rather than
copying `safeUrl` a fourth time. The structured-text draft asks the same question for a link inside
Markdown (`story:a-node-draws-text-written-in-markdown`, "where a link in the text may lead").

Minimal reproduction, written fresh and not run:

```yaml
children:
  - {name: bad, primitive: link, text: Run, href: "javascript:run()"}
  - {name: hidden, primitive: link, text: Run, href: " \tjavascript:run()"}
  - {name: ok, primitive: link, text: Source, href: "https://example.org/source"}
```

`ess ui check` reports `0 error(s)`. React draws `Run` twice as text. The terminal draws
`Run ↗` twice and `Source ↗`, with no address.

## Outcome

`ess_ui::address::admits(text) -> Result<(), AddressRefusal>` is the one rule. It decides by the
scheme, found as a browser finds it: tabs and newlines are removed, leading C0 controls and spaces
are stripped, then a scheme is `[A-Za-z][A-Za-z0-9+.-]*:`, compared without case. It admits the
LP-D4 set: by default `http`, `https`, `mailto` and a reference with no scheme, the set React
admits today. It reads no network and resolves nothing.

- `ess ui check` applies it to every constant address: an `href` or `help.link` whose value reads
  no path. A refused constant is an error under a new check id `external_address`, at the
  address's node path, naming the scheme it found and the admitted set.
- Both renderers apply it at run time to an address an expression or a row gives. An address it
  refuses is drawn as its text with no link, as React does today. The terminal draws an admitted
  address after the link's text, `Source ↗ https://example.org/source`, so a reader of a terminal
  sees where a link leads.
- React's `safeUrl` stays the browser's copy of the rule. A test feeds both copies one table of
  addresses and requires the same answer for each.
- Under LP-D6, every external address React draws opens the same way. By default that is a new tab
  with `rel="noopener noreferrer"`: the link primitive, a field link and a help link.

## Acceptance

1. `ess ui check` on the reproduction reports `error external_address
   …/children/bad/href: "javascript:" is not an address a document may name; it names http, https,
   mailto or a relative reference`. It reports the same at `…/children/hidden/href`, and nothing at
   `…/children/ok`. Red today, with `0 error(s)`.
2. `help: {link: "data:text/html,x"}` on a header, a field and a form group is reported at
   `…/help/link`. `help: {link: guide/start}` is not reported.
3. An `href` that reads a path (`row.website`, `args.url`) is not judged by the check. It is judged
   at run time (acceptance 5).
4. One vector file, `crates/ui/ess-ui/tests/vectors/addresses.json`, lists at least 20 addresses
   with their verdicts. It covers each admitted scheme in mixed case, a relative path, `//host`,
   `javascript:`, `JaVaScRiPt:`, a leading space or tab or newline before a scheme, a tab inside
   the scheme, `data:`, `file:`, `vbscript:`, the empty string and spaces only. The Rust rule
   passes it. A React test runs the generated `safeUrl` over the same file and gets the same
   verdicts.
5. Terminal: a link primitive whose `href` reads a row field draws `<text> ↗ <address>` for an
   admitted value and `<text>` alone for a refused one. A help link that the rule refuses is drawn
   as its text, as React draws it.
6. React: under LP-D6 (A), the link primitive's anchor carries `target="_blank"` and
   `rel="noopener noreferrer"`, and so do the field link and the help link. A page link (`to`) is
   unchanged.
7. The schema's `checks.list` and `crates/ui/ess-ui-check/src/lib.rs` `CHECKS` (`:109-`) name
   `external_address`. The reference's `link` section and the Checks section say which addresses a
   document may name and where each is judged. `CHANGELOG.md` names the check and the terminal's
   address.
8. Every committed example and test document checks with the findings it had before, plus
   `external_address` only where a constant is refused. Each new test runs in under 5 s, none may
   run over 60 s, and each test step stays within 120 s.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Received 2026-11-24 from
a downstream application, fifth list, item 2. This unit is the part of the feature that adds no
key.

1. **Need.** A document that names an address outside itself gets one verdict on that address:
   from the checker for a constant, and the same verdict from every renderer at run time.
2. **Class.** *Gap*. No check judges an address. The renderers disagree on what they draw for it,
   and the React targets disagree on how it opens.
3. **Already expressible?** The addresses can be named today. Judging them cannot be done (Need).
4. **Fit.** It reuses the set React already ships. It adds one check id, of the same kind as
   `page_refs`, for the other kind of target. It applies to every external address site, so no
   sibling is left out. The structured-text draft reuses it.
5. **Second adopter.** A support console links each ticket to the customer's own site from a row
   field, and links its help texts to a public handbook. Both are judged by the same rule.
6. **Cost.**
   - No key and no format version.
   - One check id. A constant address that React already refused to draw as a link becomes an
     error finding, a stricter check that leaves every document's meaning as it was.
   - The terminal output of an `href` link gains the address.
   - Under LP-D6 (A), the link primitive opens in a new tab. That is renderer behaviour the format
     never fixed. The `CHANGELOG.md` entry names it.
7. **Considered.**
   - (a) Change nothing. Rejected: a fourth site would copy `safeUrl` again.
   - (b) A denylist of dangerous schemes (LP-D4 C). Rejected as the default: a new scheme is
     admitted until someone lists it.
   - (c) https only (LP-D4 B). Rejected as the default: it refuses `http` and relative links that
     documents use today.
   - Chosen: the present allowlist, written once in Rust and held equal to React's copy.

## Decisions

**Accept, redesigned.** The request asks for an action, which is `lp2-action-href`. This unit
writes the rule that action reuses. Open questions:

- **LP-D4. Which addresses may a document name?** (A) `http`, `https`, `mailto` and a relative
  reference, the set React admits today, for every site. (B) `https` and `mailto` only, absolute.
  (C) Every scheme except a denylist (`javascript`, `data`, `vbscript`, `file`).
  **Recommended: (A).** (B) refuses links documents use today. (C) admits any scheme nobody listed.
- **LP-D6. How does React open an external address?** (A) In a new tab with
  `rel="noopener noreferrer"`, at every site, the link primitive included. (B) In the same tab with
  `rel="noreferrer"`, at every site. (C) A key per link or action that chooses.
  **Recommended: (A).** It is what two of the three sites do today. (C) adds a key for a choice no
  finding asks for.

## Scope

- inferred: new `crates/ui/ess-ui/src/address.rs`, exported from `crates/ui/ess-ui/src/lib.rs`
- cited: `crates/ui/ess-ui-check/src/rules.rs` (`primitive` 1320-1350; the header, field and group
  `help` walk), `crates/ui/ess-ui-check/src/lib.rs` (`CHECKS` 109-)
- cited: `schemas/ui/ess-ui.schema.yaml` (`link` 1158-1168 doc; `checks.list` near 1759)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`primitive_spans` 2717-2723, `header_help_line`
  591-593, `help_line`)
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (`safeUrl` 873-884, header help
  758-764), `templates/runtime/primitives/link.tsx.tmpl`, `templates/runtime/fields.tsx.tmpl`
  (55-68, 172-183)
- inferred: `crates/ui/ess-ui/tests/vectors/addresses.json`, tests in `ess-ui`, `ess-ui-check`,
  `ess-ui-tui` and `ess-ui-react`
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- Size: M, about 280 changed lines.

## Depends on

- Lands after P0's `g03-loader-fills-defaults` and `g03-resolved-json`. Those units touch
  `crates/ui/ess-ui/src/lib.rs` and the schema, and this unit adds a module to the same crate. It
  adds no key, so no fill row and no golden file moves.
- Shares `link.tsx.tmpl` with `story:a-generated-react-app-runs-under-a-base-path` (active), which
  prefixes page links. One lands after the other.
- Unblocks `lp2-action-href`.
- Informs `story:a-node-draws-text-written-in-markdown` (draft): a link in Markdown uses this rule.

## Source

A downstream application, 2026-11-24, fifth list, item 2.
