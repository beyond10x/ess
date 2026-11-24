# Gap reports of 2026-11-24: every gap is a draft story

This page maps two reports of gaps in ESS 0.98.0, received 2026-11-24 from downstream
applications, to the stories that carry them. It stands beside
[wave-2026-11-24.md](wave-2026-11-24.md) and [zero-open-issues.md](zero-open-issues.md). Every
story here is tagged `gap-reports-2026-11-24` and `gap-p0`, `gap-p1` or `gap-p2` by its group.

## Order

The owner's order, 2026-11-24: every gap the reports name becomes an ESS story and is implemented
in the priority order below, ahead of other feature work.

- Styling is out of scope. Themes and templates belong to the application; `ess-ui` is not
  extended for styling.
- Findings are designed into generic features, never filed one per finding (`AGENTS.md`, "Where
  work is tracked"). One story may cover several items, and one item may need several stories.
- An item that conflicts with ESS's design is not dropped. It is put as a lettered decision with a
  default.

Priority, as given:

- P0: first report items 1, 2, 3.
- P1: first report items 4, 5, 6, 7, 8, 8a; then the second report's G1, G2, G6, G7, G8, G10, G13,
  G11 (the semantic half), F18 and the three generator defects G3, G4, G5. G12 is the first report's
  item 3. G9 and the colour half of G11 are styling and not ESS's.
- P2: first report items 9 to 17.

## Map

One line per report item, in priority order: the need, then the stories that carry it with their
size. "Existing" marks a story that was in the store before. A group not filed yet says so.

### P0 (stories not filed yet: assessment running)

- Item 1: the actions available in a UI state, and a state that can be cloned. Assessment running.
- Item 2: a canonical byte form and digest of a UI document and of a UI state. Assessment running.
- Item 3: the loader prints the resolved UI document with every default filled in. Assessment
  running.

### P1, first report (filed)

- Item 4: a traced UI test run records, for every step, the page, the nodes drawn and the cells
  each occupies, and the state reached. `story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies` (M, about 350 lines),
  `story:each-step-of-a-traced-ess-ui-test-run-names-the-ui-state-it-reached-by-the-digest-of-that-state-s-serialised` (S, about 150).
- Item 5: one section, overlay or widget drawn alone at a given width, in the terminal and in the
  generated browser project. `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` (L, about 550),
  `story:the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec` (L, about 500).
- Item 6: a chart draws its rows in a declared order and a live insert keeps it.
  `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order` (M).
- Item 7 (defect): a fixture script for a per-value channel that names no session is reported, and
  both renderers give one verdict. `story:a-per-value-channel-script-that-names-no-session-is-reported`
  (existing, revision 3; M, about 350, hotfix material once reproduced).
- Item 8: a view filter reads the row a parameter names, one hop, and walks that row's list fields.
  `story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields` (L, about 600), `story:synthesis-witnesses-a-view-whose-filter-reads-the-row-a-parameter-names-against-a-decoy-row-and-a-parameter` (L, about 500),
  `story:generated-rust-and-go-keep-a-view-that-reads-a-parameter-s-row-as-a-named-obligation-entity-runtime-refuses-it` (M, about 250).
- Item 8a: the keyboard focus order of a UI state is declared by the document and read from a
  traced run. `story:the-keyboard-focus-order-of-a-ui-state-is-declared-by-the-document-and-read-from-a-traced-test-run-the` (L, about 550), `story:the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in` (L, about 500),
  `story:the-generated-react-project-s-tab-order-is-the-declared-focus-order-and-an-open-overlay-keeps-focus-inside-it` (L, about 450).

### P1, second report (stories not filed yet: assessment running)

- G1, G2, G6, G7, G8, G10, G13, G11 (semantic half), F18, G3, G4, G5. Assessment running.

### P2, first report (filed)

- Item 9: every selected row has a related row that passes. `story:every-row-of-a-selection-has-a-related-row-that-passes-is-stated-with-a-row-set-guard-and-a-rows-quantifier` (S, about
  150; idiom plus pin, no format).
- Item 10: a guard over the rows a list of references names, beside other related guards.
  `story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row` (M, about 350), `story:synthesis-witnesses-a-list-of-references-guard-beside-other-related-guards-in-declaration-order-and-the` (M, about 450).
- Item 11: a related-row predicate reads the addressed row's fields and state.
  `story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected` (M to L, about 500), `story:synthesis-witnesses-a-related-predicate-that-reads-the-addressed-row-on-rows-that-separate-the-held-state-from` (M, about 400),
  `story:generated-rust-and-go-behaviour-reads-the-addressed-row-in-a-related-predicate-and-entity-runtime-refuses-it` (M, about 350).
- Item 12: a secondary effect beside a create on the same branch. Parent:
  `story:create-beside-a-move-on-the-same-key` (existing, draft). `story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and` (M,
  about 450), `story:synthesis-witnesses-a-creating-branch-s-affects-entries-with-no-selected-row-with-selected-rows-and-decoys-and` (M, about 450), `story:generated-rust-and-go-behaviour-applies-a-pinned-affects-entry-beside-a-creation-in-the-same-transaction-and` (M,
  about 400).
- Item 13: a guard that reaches a row several relations away. Parent:
  `story:navigation-reaches-any-depth-in-guards` (existing, active). `story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its` (L,
  about 600), `story:synthesis-witnesses-a-when-related-chain-with-a-decoy-at-each-step-and-a-missing-middle-row-and-generated-rust` (M, about 500).
- Item 14: a comparison across the elements of two keyed lists. `story:a-guard-compares-two-keyed-lists-element-by-element-and-the-reference-shows-a-dominance-refusal-that-the` (S, about
  250; idiom plus pin), `story:synthesis-witnesses-a-comparison-of-two-keyed-lists-so-a-target-that-matches-elements-by-position-drops-the` (L, about 550); it lands after the existing
  `story:a-nested-collection-quantifier-is-witnessed-apart` (draft) or absorbs its third shape.
- Item 15: a guard compares a value with the digest of several named inputs.
  `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38` (S, about 200), `story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted` (M, about 450),
  `story:synthesis-witnesses-a-digest-of-inputs-refusal-by-changing-each-named-input-alone-so-a-target-that-leaves-an` (M, about 350). The existing `story:an-identity-may-be-a-composite-of-fields`
  (active) lands anyway (option B of P2B-D2).
- Item 16: a claim that a command never reads an entity, checked at validation.
  `story:ess-specify-reads-prints-every-entity-each-command-s-decisions-read-by-kind-and-site-from-one-read-analysis` (M, about 450), `story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by` (M, about 500),
  `story:ess-verify-diff-reports-an-entity-s-readers-declared-removed-widened-or-narrowed-at-ess-diff-24` (S, about 150); also `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`.
- Item 17: each node of the resolved UI document says whether it is a host slot.
  `story:each-node-of-the-resolved-ui-document-says-whether-it-is-a-host-slot-so-a-precondition-over-a-document-is` (S, about 120).

## Landing order per group

P1, first report: item 7 first; then item 6 (after the active chart-axes story, which edits the
same files); `story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies`; `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal`; `story:the-keyboard-focus-order-of-a-ui-state-is-declared-by-the-document-and-read-from-a-traced-test-run-the`;
`story:the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec`; `story:the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in`; `story:the-generated-react-project-s-tab-order-is-the-declared-focus-order-and-an-open-overlay-keeps-focus-inside-it`;
`story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields` once `ess/39` is open; then `story:synthesis-witnesses-a-view-whose-filter-reads-the-row-a-parameter-names-against-a-decoy-row-and-a-parameter` and
`story:generated-rust-and-go-keep-a-view-that-reads-a-parameter-s-row-as-a-named-obligation-entity-runtime-refuses-it` side by side. `story:each-step-of-a-traced-ess-ui-test-run-names-the-ui-state-it-reached-by-the-digest-of-that-state-s-serialised` lands once the P0 state form
exists.

P2, items 9 to 13: `story:every-row-of-a-selection-has-a-related-row-that-passes-is-stated-with-a-row-set-guard-and-a-rows-quantifier` now, beside the wave (it shares no file with it);
after the wave's s2: `story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row`, `story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected`,
`story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and`, `story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its`; after s4: the four synthesis units one after
another; after s5: `story:generated-rust-and-go-behaviour-reads-the-addressed-row-in-a-related-predicate-and-entity-runtime-refuses-it`, then `story:generated-rust-and-go-behaviour-applies-a-pinned-affects-entry-beside-a-creation-in-the-same-transaction-and`.

P2, items 14 to 17: `story:a-guard-compares-two-keyed-lists-element-by-element-and-the-reference-shows-a-dominance-refusal-that-the`, then `story:synthesis-witnesses-a-comparison-of-two-keyed-lists-so-a-target-that-matches-elements-by-position-drops-the` after the nested
quantifier story; `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`, `story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted`, `story:synthesis-witnesses-a-digest-of-inputs-refusal-by-changing-each-named-input-alone-so-a-target-that-leaves-an`;
`story:ess-specify-reads-prints-every-entity-each-command-s-decisions-read-by-kind-and-site-from-one-read-analysis` (no dependency, may run beside item 14), `story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by`,
`story:ess-verify-diff-reports-an-entity-s-readers-declared-removed-widened-or-narrowed-at-ess-diff-24`; `story:each-node-of-the-resolved-ui-document-says-whether-it-is-a-host-slot-so-a-precondition-over-a-document-is` right after P0's story for item 3.

P0 and the second report: the landing order is written when their stories are filed.

## Format

Stories that name `ess/38` or `ess/39`: `story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields`, `story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row`,
`story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected`, `story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and`, `story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its`,
`story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`, `story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted` and `story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by`. `ess-diff/24` carries
the diff kinds of `story:ess-verify-diff-reports-an-entity-s-readers-declared-removed-widened-or-narrowed-at-ess-diff-24`. For all of them: format: decided before acceptance (one
source format per design decision, `docs/design/generic-language-features.md` § Policy, 2). The
other stories add no format version.

## Open decisions

Each has a default. Without an answer the default is taken and said so in the next report; every
option is reversible until its story's code is merged.

P1 (first report):

- P1-D1 (item 4): where the per-step trace goes. A: optional `steps` in `ess-ui-test-report/1`
  with `--trace`; B: a new `ess-ui-test-trace/1` file; C: `ess-ui-test-report/2`. Default A.
- P1-D2 (item 4): what "the state reached" holds. A: the digest of the UI state form of items 1
  and 2; B: the full state; C: nothing. Default A.
- P1-D3 (item 5): how a test names the width. A: `width: {cells, px}`; B: cells only, fixed px per
  cell; C: cells only, no browser. Default A.
- P1-D4 (item 5): which nodes open alone. A: section, overlay, widget declaration; B: also nested
  composites; C: sections only. Default A.
- P1-D5 (item 7): a script without a session. A: plays into no instance in both renderers, check
  warns, `play` fails at once; B: plays into every instance; C: the loader refuses. Default A.
- P1-D6 (item 8): how a view reaches the row a parameter names. A: `param.<p>.<field>`, one hop, at
  `ess/39`; B: the rows quantifier in view filters; C: decline. Default A.
- P1-D7 (item 8a): who defines the focus order. A: ESS declares it from document and state; B: each
  renderer; C: decline. Default A.
- P1-D8 (item 8a): does ESS check reachability and traps. A: expose only; B: also check; C: both
  now. Default A.
- P1-D9 (item 8a): focus order between sections of a wide layout. A: section declaration order,
  with a check warning; B: the wide layout's reading order; C: per renderer. Default A.
- P1-D10 to P1-D13 (item 6): where a chart's order lives (A: chart `sort`), which parts of `sort`
  (A: `by`, `dir`), the default order (A: none, rows as read), how a test asserts order (A:
  `expect.rows`). Default A each.

P2A (items 9 to 13):

- P2A-D1 (item 9): A: the existing row-set idiom, documented and pinned; B: a `forall` over rows in
  the predicate grammar; C: decline. Default A.
- P2A-D2 (item 9): over an empty selection. A: vacuously true, as today; B: a new non-empty test; C:
  change `forall:`. Default A.
- P2A-D3 (item 11): A: the predicate reads `subject.<field>` and `subject.state`; B: admit
  `when_subject*` beside `when_related`; C: decline. Default A.
- P2A-D4 (item 10): A: a list guard beside identity, row-set and list guards; B: beside identity
  guards only; C: decline. Default A.
- P2A-D5 (item 12): A: `affects:` beside `creates:`; B: also beside `preserves:`; C: decline.
  Default A.
- P2A-D6 (items 10 to 13): which format. A: each unit joins `ess/38` while it is unreleased, the
  first after its release opens `ess/39`; B: all in `ess/39`; C: one format per feature. Default A.
- P2A-D7 (item 13): A: re-target the existing story to that format, drop its dependency on the
  view-navigation story, land it as the two units; B: keep the dependency; C: decline. Default A.

P2B (items 14 to 17):

- P2B-D1 (item 14): A: no construct, nested quantifiers documented and witnessed; B: a named
  `dominates` construct; C: a computed relation; D: decline. Default A.
- P2B-D2 (item 15): A: extend the digest family with `tuple(...)` and `hex(...)`; B: no
  computation, composite identity only; C: decline. Default A.
- P2B-D3 (item 16): A: `readers:` on the entity, a closed list checked at validation; B:
  `never_reads:` on a command; C: only `ess specify reads`; D: decline. Default A.
- P2B-D4 (item 17): A: the application evaluates over the resolved document, whose nodes gain
  `host: true`; B: an `ess ui assert` verb; C: a language construct; D: decline. Default A.
- P2B-D5 (items 15, 16): A: their constructs join `ess/38`; B: they land in `ess/39`, opened by
  `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`. Default B.

P0 and the second report: their decisions are added when their stories are filed.

## Not accepted yet

Every story here is a draft until its critics and the coordinator accept it. Item 7 is a defect: it
is reproduced before it is fixed. No story here is dispatched in a wave before its fit review is
accepted.
