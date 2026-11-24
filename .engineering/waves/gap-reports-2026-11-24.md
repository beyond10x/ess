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

Priority, as given: P0, then P1 (first report items 4 to 8a), then the second report, then a third
list of five findings from an application's own generator, then a fifth list of two findings (a page
lifted into a document), then a fourth report of four findings (a service contract), then P2.

- P0: first report items 1, 2, 3.
- P1: first report items 4, 5, 6, 7, 8, 8a; then the second report's G1, G2, G6, G7, G8, G10, G13,
  G11 (the semantic half), F18 and the three generator defects G3, G4, G5. G12 is the first report's
  item 3. G9 and the colour half of G11 are styling and not ESS's.
- Third list (an application's own generator): five findings on `ess` 0.98.0, items 1 to 5 below.
- Fifth list (a page lifted into a document): two findings on `ess` 0.98.0.
- Fourth report (a service contract): four findings, below.
- P2: first report items 9 to 17.

## First group to build

Built side by side within a group, one group after another.

1. `story:a-per-value-channel-script-that-names-no-session-is-reported` (item 7, first),
   `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer`, `story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies`.
2. `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer`, `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order`.
3. `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer`, `story:a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with`.

## Map

One line per report item, in priority order: the need, then the stories that carry it with their
size. "Existing" marks a story that was in the store before. A group not filed yet says so.

### P0 (filed)

- Item 1: the actions available in a UI state, and a state that can be cloned. Five units:
  `story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state` (S, prose only), `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in` (M, about 300), `story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test`
  (L, about 550), `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the` (L, about 560), `story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers` (M, about 350).
- Item 2: a canonical byte form and digest of a UI document and of a UI state.
  `story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling` (S to M, about 260) and `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`.
- Item 3: the loader prints the resolved UI document with every default filled in. Ten units:
  `story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from` (M, about 480), `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format` (M, about 420),
  `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and` (S, about 180), `story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema` (M, about 480),
  `story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test` (S to M, about 220), `story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once` (L, about 520),
  `story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows` (M, about 280), `story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that` (M, about 260),
  `story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps` (M, about 250), `story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own` (L, about 550).

### P1, first report (filed)

- Item 4: a traced UI test run records, for every step, the page, the nodes drawn and the cells
  each occupies, and the state reached. `story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies` (M, about 350 lines),
  `story:each-step-of-a-traced-ess-ui-test-run-names-the-ui-state-it-reached-by-the-digest-of-that-state-s-serialised` (S, about 150).
- Item 5: one section, overlay or widget drawn alone at a given width, in the terminal and in the
  generated browser project. `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` (L, about 550),
  `story:the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec` (L, about 500).
- Item 6: a chart draws its rows in a declared order and a live insert keeps it, and a test asserts
  that order. `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order` (M, about 400), `story:a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with` (M, about 350; the chart's
  rows made addressable in both renderers and `expect.rows` at a chart; after the chart-sort
  story, the active terminal-addresses story and the item-7 story).
- Item 7 (defect): a fixture script for a per-value channel that names no session is reported, and
  both renderers give one verdict. `story:a-per-value-channel-script-that-names-no-session-is-reported`
  (existing, revision 3; M, about 350, hotfix material once reproduced).
- Item 8: a view filter reads the row a parameter names, one hop, and walks that row's list fields.
  `story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields` (L, about 600), `story:synthesis-witnesses-a-view-whose-filter-reads-the-row-a-parameter-names-against-a-decoy-row-and-a-parameter` (L, about 500),
  `story:generated-rust-and-go-keep-a-view-that-reads-a-parameter-s-row-as-a-named-obligation-entity-runtime-refuses-it` (M, about 250).
- Item 8a: the keyboard focus order of a UI state is declared by the document and read from a
  traced run. `story:the-keyboard-focus-order-of-a-ui-state-is-declared-by-the-document-and-read-from-a-traced-test-run-the` (L, about 550), `story:the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in` (L, about 500),
  `story:the-generated-react-project-s-tab-order-is-the-declared-focus-order-and-an-open-overlay-keeps-focus-inside-it` (L, about 450).

### P1, second report (filed)

- G1: a view route answers its rows under a named member, and views of one component may share a
  route. `story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their` (M, about 480), `story:the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows` (M, about 420),
  `story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares` (S, about 220).
- G2: a decimal literal sets or publishes a binary64 field. `story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share` (M, about 450).
- G3 (a gap, not a defect): a number with no declared format is drawn as written in every
  renderer; the story removes a documented difference between the renderers.
  `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer` (S, about 180).
- G4: a list field names the text that joins its items. `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer` (S, about 220); the
  renderers agree today and the reference is silent, so it is not a defect.
- G5: a structured value reads as one line (defect), a tagged union declares a flat layout, the
  generated codecs follow it, and a bound document reads the union as its variant and content in
  the layout its model declares. `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer` (S, about 150),
  `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object` (M, about 480; after `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`),
  `story:generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares` (M, about 560), `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model` (M, about 450; its title is the
  revised one, its artifact name is the earlier one).
- G6: a time, date or timestamp field declares UTC or the reader's local zone.
  `story:a-time-date-or-timestamp-field-declares-whether-it-is-drawn-in-utc-or-the-reader-s-local-zone` (M, about 420).
- G7: one host component stands in many slots. `story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path` (M, about 330); it follows
  the existing `story:a-host-slot-in-a-reused-template-is-one-registration` (active).
- G8: one search reads several fields. `story:one-search-reads-several-fields-each-word-found-in-any-of-them-documented-and-pinned-in-every-renderer` (S, about 160); expressible today by
  reading, made a contract.
- G9: declined as styling (decision SL-D9); no story.
- G10: a duration field shows the time between two instants, the clock standing in for an absent
  end. `story:a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end` (M, about 520).
- G11: the tone half, a page header's badge takes tone and words from its value.
  `story:a-page-header-s-badge-takes-its-tone-and-words-from-its-value-as-a-badge-does` (S, about 200). The colour half is declined as styling (SL-D10).
- G12: the first report's item 3, `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format` and its units; no story of its own.
- G13: an entity whose rows the deployment provides declares lifecycle moves the deployment makes.
  `story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command` (M, about 330), `story:synthesis-reaches-every-state-of-a-provided-entity-through-its-provider-and-never-drives-a-move-the-deployment` (M, about 400).
- F18: an event stream that no command opens. `story:an-http-route-streams-a-component-s-events-with-no-command-opening-it-and-frames-named-by-a-member-of-their` (M, about 480),
  `story:a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames` (M, about 320); it extends the existing
  `story:an-event-channel-may-be-bound-to-server-sent-events` (active).

### P1, third list (an application's own generator) (filed)

`sp1` is being fixed as patch release 0.98.1; `sp3` may ride the same patch; `sp3` and `sp4` are
defects of the React runtime.

- Item 1 (defect): a live insert of a key the section already holds shows one row, in both
  renderers. `story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers` (S, about 150; hotfix 0.98.1).
- Item 2: `ess ui check --model` warns when a live event lacks a field the section it feeds shows,
  or the field that matches its row. `story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches` (M, about 300). Beside
  `story:a-per-value-channel-script-that-names-no-session-is-reported` (`informed_by`, no order).
- Item 3 (defect): a looping fixture script repeats at its last entry's offset in the browser too.
  `story:a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test` (S, about 100; hotfix-sized).
- Item 4 (defect): in the browser, a live event that arrives before the section's read has
  answered is applied once it answers. `story:in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as` (S to M, about 150; after item 1).
- Item 5: a terminal test checks what a host slot is handed. `story:a-terminal-test-checks-what-a-host-slot-is-handed-its-resolved-reads-its-commands-and-the-page-s-parameters` (M, about
  350); after `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` and `story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path`.

### P1, fifth list (a page lifted into a document) (filed)

Two findings from lifting an existing page into `ess-ui/1`. No item of the first report is covered.

- Item 1: the reference shows how a section shows a paragraph or a field's value with no label, and
  both renderers are pinned to it. `story:the-reference-shows-how-a-section-shows-a-paragraph-or-a-field-s-value-with-no-label-and-both-renderers-are` (S, about 200; docs and tests, no
  key). Related, separate: `story:a-node-draws-text-written-in-markdown` (existing, draft).
- Item 2: an action or a navigation menu entry leads to an address outside the document. Four units:
  `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` (M, about 280; one rule for which external addresses a document may name,
  applied alike by `ess ui check` and both renderers), `story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub` (M, about 450; `href` as
  an action effect), `story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a` (M, about 300; a test asserts an action opened an external
  address), `story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub` (M, about 500; `links` beside `pages` in a navigation section).
  `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` answers the link question of
  `story:a-node-draws-text-written-in-markdown`.

### Fourth report (a service contract) (filed)

Four findings. Finding 1 is a defect being reproduced for a patch release after 0.98.1.

- Finding 1 (defect): a second caller's run expects the identity it sent, also in a nested payload
  member. `story:a-second-caller-s-run-expects-the-identity-it-sent-also-in-a-nested-payload-member` (S, about 240 with tests).
- Finding 2, stream half: existing `story:an-event-channel-may-be-bound-to-server-sent-events`
  (existing, active; framing and `keepalive` covered), blocked on the existing
  `story:an-exchange-is-declared-in-the-specification` and
  `story:asyncapi-and-openapi-project-an-exchange` (both existing, active, unbuilt). Authentication
  by header and by query token: existing `story:an-http-binding-declares-how-a-caller-authenticates`
  (existing, active), plus one amendment there (`bearer` with `in: query`, and the stream route takes
  `authentication`). A route-less feed: decision AC-D1, which the second report's F18 shares; no
  second stream design.
- Finding 2, bus request/reply half: `story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply` (M and S, about 700 over two
  units); beside the existing `story:a-broker-binding-carries-an-exchange` (existing, draft).
- Finding 3, header-borne inputs: `story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header` (three units, about 1,000 lines; credential
  headers refused, AC-D3). It gives the stream story's `Last-Event-ID` its request-side key.
- Finding 3, the `401` challenge: `story:a-401-answer-may-state-that-the-service-sends-no-www-authenticate-challenge` (S, about 250).
- Finding 4, a `2xx` on an unreadable request: `story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so` (S, about 250).
- Finding 4, plain-text error bodies: `story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text` (two units, about 750).
- Finding 4, unknown JSON fields ignored: existing
  `story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields` and
  `story:generated-servers-ignore-undeclared-json-members-and-check-echoed-path-fields` (both
  existing, active); no new story.
- Finding 4, crossed routes (`ESS-HTTP-011`): `story:two-crossed-route-templates-may-be-served-in-an-order-the-binding-declares` (M, about 400).
- Finding 4, the OpenAPI document lists outbound commands: existing
  `story:an-http-binding-may-declare-that-an-accepted-command-or-a-served-view-has-no-route`
  (existing, draft); this report is a second need for it.

### Store issues found

Found while assessing the fourth report; no existing story was changed for them.

- `story:an-http-binding-declares-how-a-caller-authenticates` names its new refusal code
  `ESS-HTTP-034`. That code is taken by `limits` (the request-body cap). It needs the next free
  code, and so do `story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header` and any other new code: each is numbered when it lands.
- `story:an-exchange-is-declared-in-the-specification` says `ess/37`. `ess/37` shipped in 0.93.0
  with no construct. The exchange layer would join `ess/38` (opened by the next wave, unreleased)
  or take `ess/39`.

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
`story:a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with` (after `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order`, the active terminal-addresses story and item 7); `story:the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec`; `story:the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in`; `story:the-generated-react-project-s-tab-order-is-the-declared-focus-order-and-an-open-overlay-keeps-focus-inside-it`;
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

P0: `story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state`, `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in` and `story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from` can start at once; then
`story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test` and `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`; `story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling` after the resolved document;
`story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`, `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and` (after the digest, same output function),
`story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows`, `story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema`; then `story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test`,
`story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once` (after `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in`, same file), `story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that`
(after `story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test`, same file), `story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps`; `story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers` after the
state document; `story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own` last.

Second report, six lanes built side by side, one after another within a lane:

- Renderer lane: `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer` (defect), `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer` (defect),
  `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer`, `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model`, `story:a-time-date-or-timestamp-field-declares-whether-it-is-drawn-in-utc-or-the-reader-s-local-zone`, `story:a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end`;
  `story:a-page-header-s-badge-takes-its-tone-and-words-from-its-value-as-a-badge-does` slots in after `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer`. `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer` also
  lands before `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order` and before `story:a-chart-names-its-axes-and-formats-its-ticks`
  (same function).
- Filter and host lane: `story:one-search-reads-several-fields-each-word-found-in-any-of-them-documented-and-pinned-in-every-renderer`, then `story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path` (after the active
  reused-template story).
- HTTP lane: `story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their`, then `story:the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows` beside `story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares`, then
  `story:an-http-route-streams-a-component-s-events-with-no-command-opening-it-and-frames-named-by-a-member-of-their` (after the active SSE story), then `story:a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames`;
  `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model` shares `binding.rs` with `story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares` and merges after it.
- Literal lane: `story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share`.
- Lifecycle lane: `story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command` (after `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`), then `story:synthesis-reaches-every-state-of-a-provided-entity-through-its-provider-and-never-drives-a-move-the-deployment`.
- Union wire lane: `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object` (after `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`), then `story:generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares`;
  `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model` waits for `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object` under SL-D13 A.

Third list: `story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers` (patch 0.98.1, in progress); then `story:a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test` and
`story:in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as` as one patch release 0.98.2 in two commits (`sp4` needs 0.98.1: a held
event re-applied after the answer must not duplicate a row; both edit `runtime/live.ts.tmpl`);
`story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches` after or beside the item-7 story (same check list); `story:a-terminal-test-checks-what-a-host-slot-is-handed-its-resolved-reads-its-commands-and-the-page-s-parameters`
after SP-D5 and after `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` (same `spec.rs` and parity table).

Fifth list: `story:the-reference-shows-how-a-section-shows-a-paragraph-or-a-field-s-value-with-no-label-and-both-renderers-are` any time (no key, independent of P0); `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` after
`story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from` and `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`; `story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub` after `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` and
the active `story:an-action-names-its-icon`; `story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a` after `story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub`; `story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub` after `story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a` (its tests use
`expect_visit`).

Fourth report: `story:a-second-caller-s-run-expects-the-identity-it-sent-also-in-a-nested-payload-member` first (a defect, patch release after
0.98.1); `story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so` before `story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text` (shared body checks in
`lib.rs`); `story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header` after the authentication story; `story:a-401-answer-may-state-that-the-service-sends-no-www-authenticate-challenge` and
`story:two-crossed-route-templates-may-be-served-in-an-order-the-binding-declares` have no dependency; `story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply` and
`story:a-broker-binding-carries-an-exchange` both open `ess-transport/3`, so one after the other.
The stream finding waits on the existing exchange stories (`story:an-exchange-is-declared-in-the-specification`,
`story:asyncapi-and-openapi-project-an-exchange`, both active and unbuilt).

## Format

Stories that name `ess/38` or `ess/39`: `story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields`, `story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row`,
`story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected`, `story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and`, `story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its`,
`story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`, `story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted`, `story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by`, `story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command`, `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object`,
and, under SL-D8 option B, `story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share`. `ess-diff/24` carries the diff kinds of
`story:ess-verify-diff-reports-an-entity-s-readers-declared-removed-widened-or-narrowed-at-ess-diff-24`. For all of them: format: decided before acceptance (one source format per
design decision, `docs/design/generic-language-features.md` § Policy, 2).

New published formats that are not specification formats: `ess-ui-state/1` (`story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`)
and `ess-ui-resolved/1` (`story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`); P1-D1 option B would add `ess-ui-test-trace/1`.
The other stories add no format version.

## Open decisions

Each has a default. Without an answer the default is taken and said so in the next report; every
option is reversible until its story's code is merged.

P0 (items 1 to 3):

- P0-D1 (item 1): where the exploration surface lives. A: `ess_ui_test::Session`; B: a new crate; C:
  in `ess-ui`; D: decline. Default A.
- P0-D2 (item 1): how a state is reached without replay. A: fork in process, `ess-ui-state/1`
  describes and restores nothing; B: restore from the document; C: the document carries its step
  path. Default A.
- P0-D3 (item 1): a command-line verb. A: `ess ui state`, last in order; B: library only. Default A.
- P0-D4 (item 1): steps with a free value. A: offered by target, the caller supplies the value; B:
  ESS picks values; C: left out. Default A.
- P0-D5 (item 1): the clock in a state's identity. A: absolute virtual time plus script positions;
  B: durations until each pending event. Default A.
- P0-D6 (item 2): the canonical rule. A: the published InfraIR scheme (compact key-sorted JSON,
  bare SHA-256); B: RFC 8785 with a prefix; C: the EssIr struct-order style. Default A.
- P0-D7 (item 2): does the digest cover author text and traceability keys. A: yes; B: no; C: both
  as two digests. Default A.
- P0-D8 (item 3): is `ess ui load --format json` versioned. A: yes, `ess-ui-resolved/1`; B: no; C:
  decline. Default A.
- P0-D9 (item 3): the document part. A: the loader's filled tree in `ess-ui/1` keys; B: `Serialize`
  on `Document`; C: a new shape. Default A.
- P0-D10 (item 3): does the React generator read the printed JSON. A: no, a round-trip test; B:
  yes. Default A.
- P0-D11 (item 3): what `--model` adds. A: the existing `Binding`; B: enums only; C: binding plus
  row field types. Default A.
- P0-D12 (item 3): `Field.label` default. A: the field name with `_` as a space, schema corrected;
  B: field labels in the specification language; C: leave unfilled. Default A.
- P0-D13 (item 3): where `Field.tone: neutral` is filled. A: only beside `tone_by`; B:
  everywhere; C: nowhere. Default A.
- P0-D14 (item 3): React's server-state key once `scope` is always filled. A: bare key at the
  store's default scope, `scope:key` otherwise; B: always `scope:key`; C: do not fill `scope`.
  Default A.
- P0-D15 (item 3): `form.fields`. A: from the model when `--model` is given; B: each renderer
  derives it; C: `--format json` requires `--model`. Default A.
- P0-D16 (item 3): `Reads.key` and `Live.match`. A: each from the other, else `id`, except on a
  choice's read; B: `id` everywhere; C: only from each other. Default A.
- P0-D17 (item 3): where both renderers agree against the schema's default. A: the schema, as
  ordered; B: correct the schema to what both do; C: per key. Default A.

P1 (first report):

- P1-D1 (item 4): where the per-step trace goes. A: optional `steps` in `ess-ui-test-report/1`
  with `--trace`; B: a new `ess-ui-test-trace/1` file; C: `ess-ui-test-report/2`. Default A.
- P1-D2 (item 4): what "the state reached" holds. A: the digest of the UI state form of items 1
  and 2; B: the full state; C: nothing. Default A.
- P1-D3 (item 5): how a test names the width. A: `width: {cells, px}`; B: cells only, fixed px per
  cell; C: cells only, no browser. Default A.
- P1-D4 (item 5): which nodes open alone. A: section, overlay, widget declaration; B: also nested
  composites; C: sections only. Default A.
- P1-D5 (item 7): a script without a session. A: plays into no instance in both renderers, `ess ui
  check` warns, and a `play` whose event comes only from such a script, or from a script whose
  session matches no reader on the page last opened, fails in the terminal and is `test.fixme` in
  the browser (every other `play` unchanged); B: plays into every instance; C: the loader refuses.
  Default A.
- P1-D6 (item 8): how a view reaches the row a parameter names. A: `param.<p>.<field>`, one hop, at
  `ess/39`; B: the rows quantifier in view filters; C: decline. Default A.
- P1-D7 (item 8a): who defines the focus order. A: ESS declares it from document and state; B: each
  renderer; C: decline. Default A.
- P1-D8 (item 8a): does ESS check reachability and traps. A: expose only; B: also check; C: both
  now. Default A.
- P1-D9 (item 8a): focus order between sections of a wide layout. A: section declaration order,
  with a check warning; B: the wide layout's reading order; C: per renderer. Default A.
- P1-D10 to P1-D12 (item 6, chart-sort story): where a chart's order lives (A: chart `sort`),
  which parts of `sort` (A: `by`, `dir`), the default order (A: none, rows as read). Default A each.
- P1-D13 (item 6, moved to the chart-rows story): how a test asserts order. A: `expect.rows` at a
  chart; B: a new `expect` key; C: no step. Default A.

P1 (second report):

- SL-D1 (G3, a gap: a documented renderer difference): where React keeps the locale's digit
  grouping. A: where a number format is declared or is the schema default (a metric, and the header
  total), and every other number is drawn as written; the result is the same before and after the
  loader story writes the defaults; B: a metric's schema default becomes "as written" too, and the
  loader writes no `metric.format`; C: as A, and a chart tick keeps React's grouping whatever its
  axis declares. Default A.
- SL-D2 (G5): what follows a union's variant label. A: the content's values in declaration order,
  space-joined; B: `key: value` pairs. Default A.
- SL-D3 (G8): how `contains` matches a word. A: against each item separately; B: against the items
  joined with a comma, as today. Default A.
- SL-D4 (G7): how "one host, many slots" meets the active reused-template story. A: a follow-up
  story after it; B: widen that story now. Default A.
- SL-D5 (G6): where the time zone is chosen. A: per field, `zone: utc | local`; B: also a document
  default; C: also IANA names. Default A.
- SL-D6 (G10): how a duration from two instants and the clock is spelled. A: `since: <field>` on an
  `as: duration` field; B: `now()` arithmetic in every expression; C: decline. Default A.
- SL-D7 (G1): how much `ess-http/1` takes for rows inside a larger answer. A: `body: {<member>:
  rows}` plus one route shared by views, three units; B: the member body only; C: decline. Default
  A.
- SL-D8 (G2): which formats admit a `Binary64` literal. A: every format that has `Binary64`, by the
  `Decimal` precedent; B: only the format `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38` opens. Default A.
- SL-D9 (G9): settled, declined as styling. Open half: A: decline a truncation contract too; B: a
  `wrap: false` contract on a column or record field. Default A.
- SL-D10 (G11 colour half): settled, declined as styling. No open question.
- SL-D11 (G13): how lifecycle moves only the deployment makes are declared. A: through `provided:
  true`, from `ess/39`, two units; B: decline, with the system-actor idiom; C: a `recorded_by`
  spelling. Default A.
- SL-D12 (F18): how `ess-http/1` takes an event stream no command opens. A: widen the active SSE
  story with a commandless `stream:` and a UI unit; B: wait for the exchange layer; C: decline.
  Default A.
- SL-D13 (G5): where the flat union layout is declared. A: on the union in the specification,
  `layout: flat` from `ess/39`, every target reads it and the UI binding carries it; B: a key in
  the `ess-http/1` binding; C: the UI reads any object with the tag member and no `value` member
  as flat; D: decline. Default A.
- SL-D14 (G5): how the flat layout is spelled. A: `layout: adjacent | flat`, `adjacent` the
  default; B: `tag_inline: true`; C: `wire: flat`, refused because `wire:` names an enum variant's
  spelling. Default A.

P1 (third list):

- SP-D1 (item 1): should a looping script that re-inserts held keys be reported. A: no report, a
  doc sentence only; B: a check warning on every such script; C: a note line in the text report of
  `ess ui test`. Default A.
- SP-D2 (item 2): severity and reach. A: a warning with `--model`, exit status unchanged; B: an
  error; C: A plus fixture-script payloads without `--model`. Default A.
- SP-D3 (item 3): a looping script's period. A: the last entry's `at`, everywhere; B: a fixed 1 s
  gap after the last entry; C: A plus a later `period:` key. Default A.
- SP-D4 (item 4): a live event during an outstanding read. A: hold and apply it to the answer, for
  every outstanding read; B: only during the first read; C: the terminal drops it too. Default A.
- SP-D5 (item 5): what of a host slot a test can check. A: what ESS hands the slot, `handed:`, in
  the terminal, `test.fixme` in the browser; B: a stand-in declared in the test file; C: decline.
  Default A.

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

Fifth list (a page lifted into a document):

- LP-D1 (item 1): does a text-only section need an empty record as its body. A: yes, document the
  record with `children` as the idiom; B: also let a primitive stand as a section's body; C: a
  `text` composite kind. Default A.
- LP-D2 (item 1): does the document tell a renderer a line is a kicker or a lede. A: decline, role
  and look are the theme's; B: a closed role enum the renderers may ignore; C: decline and keep the
  application's labelled fields. Default A.
- LP-D3 (item 2): how an action names an address outside the document. A: `href: <address>` as an
  action effect beside `navigate`; B: `navigate: {href}`; C: decline. Default A.
- LP-D4 (item 2): which external addresses a document may name. A: `http`, `https`, `mailto` and a
  relative reference, the set React admits today; B: `https` and `mailto` only; C: every scheme but
  a denylist. Default A.
- LP-D5 (item 2): may an action's address come from a row or state. A: no, a constant that `ess ui
  check` judges; B: yes, judged at run time; C: an expression limited to listed hosts. Default A.
- LP-D6 (item 2): how React opens an external address. A: a new tab with `rel="noopener
  noreferrer"` at every site, the `link` primitive included; B: the same tab with
  `rel="noreferrer"`; C: a key per link. Default A.
- LP-D7 (item 2): does the terminal open an external address. A: never, it shows the address,
  records the visit and shows a notice; B: opens it interactively. Default A.
- LP-D8 (item 2): how a navigation menu entry leads to an outside address (the requester confirmed
  such links sit in the menu). A: `NavSection` takes `links: [{name, label, href, icon?,
  synonyms?}]` beside `pages`, drawn after the pages, as `story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub`; B: a record
  `{name, label, href}` inside `pages`; C: out, the application keeps a stub page per entry.
  Default A (in).

Fourth report (a service contract):

- AC-D1 (finding 2, also F18): may an event stream follow no row. A: amend the exchange story so
  `follows` is optional; B: SSE binds only row-followed exchanges; C: an SSE route binds a
  component's `publishes:` events directly. Default A.
- AC-D2 (finding 2): is bus request/reply designed now. A: design it, one command bound to a
  broker subject in `ess-transport/3` with a requester timeout; B: decline; C: only the SSE half
  now. Default A.
- AC-D3 (finding 3): may a credential header be bound to a command input. A: no, `Authorization`,
  `Proxy-Authorization`, `Cookie` and a scheme's `api_key` header are refused; B: admit it; C: admit
  a value derived from it. Default A (a security boundary).
- AC-D4 (finding 3): may a declared `401` omit the `WWW-Authenticate` challenge. A: admit
  `headers: {WWW-Authenticate: none}`, a bare `401` stays refused; B: decline; C: only on
  `api_key` or `mutual_tls` routes. Default A (a security boundary).
- AC-D5 (finding 4): may an error answer use a `2xx` status. A: any error outcome and `malformed`;
  B: decline; C: a `2xx` with no body on `malformed` only. Default C (a security boundary).
- AC-D6 (finding 4): may two crossed route templates both be bound. A: a route declares
  `precedes:` and both servers dispatch in that order; B: decline; C: admit a pair whose variable
  types cannot hold each other's literal. Default A.

## Accepted

Accepted 2026-11-24 by the coordinator under the operator's delegation, each with the defaults of
its decisions as written; moved draft, proposed, active:

- `story:a-per-value-channel-script-that-names-no-session-is-reported`
- `story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies`
- `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order`
- `story:a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with`
- `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer`
- `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer`
- `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer`

## Not accepted yet

Every other story here is a draft until its critics and the coordinator accept it. Item 7 is a
defect: it is reproduced before it is fixed. No story here is dispatched in a wave before its fit review is
accepted.
