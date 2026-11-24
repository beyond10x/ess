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
- Eighth list (the fourth report's application, contract gaps): twelve items, ranked with the fourth
  report, below.
- Seventh report (a command that only reads one row): three attempts, ranked with the fourth
  report, below.
- Sixth list (items G12 to G15): G14 (P0, validation and synthesis time, three stories), G15 (P1),
  G12 and G13 (P2), below.
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

### P0 (filed, revised after four critics and one recheck, accepted)

Thirteen units in eight rounds, plus three follow-up drafts that are not in P0. Each unit's text
answers the critics' 76 findings and the recheck's 8.

- Item 1: the actions available in a UI state, and a state that can be cloned. Five units:
  `story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state` (S, about 290, prose only), `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in` (M, about 400),
  `story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test` (L, about 570), `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the` (L, about 590),
  `story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers` (M, about 380).
- Item 2: a canonical byte form and digest of a UI document and of a UI state.
  `story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling` (S to M, about 260) and `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`.
- Item 3: the loader prints the resolved UI document with every default filled in. Seven units:
  `story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from` (M, about 570), `story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test` (M, about 260),
  `story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows` (M, about 300), `story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once` (M, about 450),
  `story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema` (L, about 560), `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format` (M, about 450),
  `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and` (S, about 220).
- Rounds, at most three units at once and no two units of one round change one file:

  | round | units | wave |
  |---|---|---|
  | R1 | `story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state`, `story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from`, `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in` | 1 |
  | R2 | `story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test`, `story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows` | 1 |
  | R3 | `story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test`, `story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once` | 2 |
  | R4 | `story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema` | 2 |
  | R5 | `story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling` | 3 |
  | R6 | `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format` | 3 |
  | R7 | `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`, `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and` | 3 |
  | R8 | `story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers` | 3 |

  R5 to R8 are one release: it publishes `ess-ui-resolved/1` and `ess-ui-state/1` once, complete,
  after every fill unit has landed.
- Follow-up, not in P0 (tag `gap-p0` stays; no P0 unit waits on them, and no P0 round runs them):
  `story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own` (L, about 550; the filled `Option` fields become plain fields and
  the renderers' dead fallbacks go; it depends on every P0 unit),
  `story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that` (M, about 340; a terminal defect story) and
  `story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps` (M, about 200; a React defect story).

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
  route. The member body is the existing `story:an-answer-may-be-wrapped-in-one-named-member` and
  `story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member` (active,
  beyond10x/ess#755), spelled `{<member>: array}`. The shared route, each unit derived from the
  active story it extends: `story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their` (M, about 300; its title now names the shared
  route), `story:the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows` (M, about 300), `story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares` (S, about 100; its title
  now names the shared route).
- G2: a decimal literal sets or publishes a binary64 field. `story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share` (M, about 450).
- G3 (a gap, not a defect): a number with no declared format is drawn as written in every
  renderer; the story removes a documented difference between the renderers.
  `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer` (S, about 180).
- G4: a list field names the text that joins its items. `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer` (S, about 220); the
  renderers agree today and the reference is silent, so it is not a defect.
- G5: a structured value reads as one line (defect), a tagged union declares a flat layout, the
  generated codecs follow it, and a bound document reads the union as its variant and content in
  the layout its model declares. `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer` (S, about 150),
  `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object` (M, about 400) and `story:generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares` (M, about 450): the flat
  value of the union's encoding selector, derived from the existing
  `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`,
  `story:projections-use-a-unions-declared-content-key` and
  `story:generated-codecs-and-the-suite-use-a-unions-declared-content-key` (active,
  beyond10x/ess#637), one key family `layout: adjacent | flat` with `content:`, in one format.
  `story:union-tag-inline-with-fields` (draft) is archived as covered. `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model` (M, about 450; its title is the
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
- F18: an event stream that no command opens. After the owner's note of 2026-11-24 (section
  below) the existing `story:an-event-channel-may-be-bound-to-server-sent-events` (revised) binds
  the events one component publishes, and carries it (PR-D4). `story:an-http-route-streams-a-component-s-events-with-no-command-opening-it-and-frames-named-by-a-member-of-their` (draft) is
  merged into that story and archived as fully covered.
  `story:a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames` (M, about 320) depends on the revised SSE story and reads frame naming
  from the compiled binding.

### P1, third list (an application's own generator) (filed)

`sp1` and `sp3` ride patch release 0.98.1; `sp4` takes the next free patch after 0.98.2; `sp3`
and `sp4` are defects of the React runtime.

- Item 1 (defect): a live insert of a key the section already holds shows one row, in both
  renderers. `story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers` (S, about 150; hotfix 0.98.1).
- Item 2: `ess ui check --model` warns when a live event lacks a field the section it feeds shows,
  or the field that matches its row. `story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches` (M, about 300). Beside
  `story:a-per-value-channel-script-that-names-no-session-is-reported` (`informed_by`, no order).
- Item 3 (defect): a looping fixture script repeats at its last entry's offset in the browser too.
  `story:a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test` (S, about 100; rides patch 0.98.1, its terminal pin case still open).
- Item 4 (defect): in the browser, a live event that arrives before the section's read has
  answered is applied once it answers. `story:in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as` (S to M, about 150; after item 1, in the next free patch after 0.98.2).
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
  address), `story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub` (M, about 560; an entry of a navigation section's `pages` may be an outside
  address).
  `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` answers the link question of
  `story:a-node-draws-text-written-in-markdown`.

### Fourth report (a service contract) (filed)

Four findings. Finding 1 is a defect being reproduced for a patch release after 0.98.1.

- Finding 1 (defect): a second caller's run expects the identity it sent, also in a nested payload
  member. `story:a-second-caller-s-run-expects-the-identity-it-sent-also-in-a-nested-payload-member` (S, about 240 with tests).
- Finding 2, stream half: existing `story:an-event-channel-may-be-bound-to-server-sent-events`
  (existing, active, revised after the owner's note of 2026-11-24: it binds published events, with
  frames, names and `keepalive` in the protocol document, and no longer waits for
  `story:an-exchange-is-declared-in-the-specification` or
  `story:asyncapi-and-openapi-project-an-exchange`). Authentication
  by header and by query token: existing `story:an-http-binding-declares-how-a-caller-authenticates`
  (existing, active), plus one amendment there (`bearer` with `in: query`, and the stream route takes
  `authentication`). A route-less feed: PR-D4 A, which supersedes AC-D1 and SL-D12 (the second
  report's F18 shares it); no second stream design.
- Finding 2, bus request/reply half: `story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply` (M and S, about 700 over two
  units); beside the existing `story:a-broker-binding-carries-an-exchange` (existing, draft).
- Finding 3, header-borne inputs: `story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header` (three units, about 1,000 lines; credential
  headers refused, AC-D3). When a resume is designed, the stream story's `Last-Event-ID` is a
  header of the stream's request message.
- Finding 3, the `401` challenge: `story:a-401-answer-may-state-that-the-service-sends-no-www-authenticate-challenge` (S, about 250; derived from and after
  `story:an-http-binding-declares-how-a-caller-authenticates`, whose `refusals.unauthenticated` it
  extends).
- Finding 4, a `2xx` on an unreadable request: `story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so` (S, about 250).
- Finding 4, plain-text error bodies: `story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text` (two units, about 750).
- Finding 4, unknown JSON fields ignored: existing
  `story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields` and
  `story:generated-servers-ignore-undeclared-json-members-and-check-echoed-path-fields` (both
  existing, active); no new story.
- Finding 4, crossed routes (`ESS-HTTP-011`): `story:two-crossed-route-templates-may-be-served-in-an-order-the-binding-declares` (M, about 400). Archived
  on 2026-11-24 under AE-D1 A: no finding stands behind it (the eighth list's several operations on
  one path meet `ESS-HTTP-010`, not `ESS-HTTP-011`).
- Finding 4, the OpenAPI document lists outbound commands: existing
  `story:an-http-binding-may-declare-that-an-accepted-command-or-a-served-view-has-no-route`
  (existing, draft); this report is a second need for it.

### Sixth list (items G12 to G15) (filed)

Four items from one downstream application on 0.98.0. The letters G12 to G15 are this list's own;
they are not the second report's G-items and not the first report's items 12 to 15.

- G14, P0: validation and synthesis time. The number of tries grows with the number of row-set
  guards of one command, and each try builds every row again. Three stories, in this order:
  `story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per` (S, about 200; the arrangement of a selector's rows is built once
  per distinct prefix of counts, with an output-identical result and a work counter),
  `story:a-row-set-branch-s-search-does-not-arrange-a-combination-of-counts-that-provably-cannot-take-the-branch-and` (L; a combination of counts that provably cannot take the branch
  is not arranged, after a design page, with an exact fallback; after `story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per`;
  tag `gap-p2`, a draft, not scheduled: it does not shorten the reported case (at most 6 of its 37
  refusals qualify, none on the command that holds 94 % of the time); gains only with a
  linked-selector allowance, open as PF-D4)
  and `story:a-branch-the-row-set-strategy-prepares-is-prepared-once-per-model-branch-actors-and-arrangement-context-and` (M; a branch the row-set strategy prepares is prepared once per model,
  branch, actors and arrangement context; after `story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per`, accepted).
- G15, P1: fixtures are not typed against the model. `story:with-model-ess-ui-check-refuses-a-fixture-row-or-a-script-payload-whose-value-contradicts-the-type-the-model` (L, about 560;
  `ess ui check --model` finds a fixture row or a script payload whose value contradicts the
  declared type, SX-D1 to SX-D2) and `story:ess-ui-test-model-refuses-to-start-on-mistyped-document-fixtures-and-fails-a-test-whose-own-fixtures` (M, about 350; `ess ui test
  --model` refuses a mistyped document fixture and fails a test with mistyped own fixtures,
  SX-D3). No format version. Siblings under one feature, kept apart: `story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches`
  and `story:a-per-value-channel-script-that-names-no-session-is-reported` (item 7). The draft
  `story:board-widget-kinds-reads-every-fixture-answer` has its acceptance 1 covered by the single
  fixture reader.
- G12, P2: no expression reads a product of list counts. Three units, all `ess/39`, which is
  format: decided before acceptance: `story:at-ess-39-a-command-guard-compares-a-fact-with-an-expression-of-several-facts-and-every-expression-reads-the` (L, about 580; arithmetic over
  several facts as a command guard operand and `xs.count` in every expression; after
  `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`), `story:synthesis-witnesses-a-command-guard-that-compares-a-fact-with-an-expression-of-several-facts-on-both-sides-of` (L, about 450; synthesis witnesses both sides
  of the comparison) and `story:every-target-names-what-it-does-with-a-guard-over-an-expression-of-several-facts-generated-rust-and-go` (M, about 380; each target keeps the command
  an obligation named by it or evaluates it). SX-D4 to SX-D6. The count-product idiom of the first
  report's "coverage over the product of two stored lists" is covered; the quantifier form stays
  with the quantifier design of first report item 9 (SX-D4 B). `story:a-guard-compares-two-keyed-lists-element-by-element-and-the-reference-shows-a-dominance-refusal-that-the` shares
  the guard positions and is not covered.
- G13, P2: a system actor cannot carry the attribute a caller-recorded field needs.
  `story:a-command-that-records-its-caller-and-is-granted-to-a-system-actor-is-refused-naming-that-actor-as-the-system` (S, about 120; the refusal names the system actor and the idiom, the
  reference states who is a system actor), SX-D7. It is derived from
  `story:a-system-issued-creating-command-is-witnessed` (active), which covers the report's second
  sentence (`ESS-SYNTH-023` on a creating command).

### Seventh report (a command that only reads one row) (filed)

One application, three attempts to make a read-only command (404 when the row is absent, otherwise
the row, changing nothing) synthesize on `ess` 0.98.0. The base shape is not a defect: nothing
declares that the input names a row. The other four commands of the same shape were inferred by
the requester and not run. Ranked with the fourth report: after the first application's P1 lists,
before P2. No format version.

- Attempt 3 (`preserves:` with `instance:` needs a full `read_your_writes` view): not a new story.
  It amends the existing draft `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`
  (decided 2026-10-22, never shipped): its Need, Acceptance, Scope and line references gain the
  read-only case, its title names it, and it gains the tags `gap-reports-2026-11-24` and `gap-p1`
  (M, about 300; synthesis only, about 30 lines of source). RD-D1.
- Attempt 1 (`when_related` with `via`, `ESS-COMMAND-004`, an identity type shared by two
  entities): `story:the-refusal-of-an-input-whose-identity-type-several-entities-share-on-a-branch-with-no-subject-names-the-forms` (S, about 120; the refusal names the forms that say which
  entity, a refusal-only change). RD-D2.
- Attempt 2 (a row-set `exists: false` whose passing branch is not arranged, cause not
  established): `story:a-row-set-guard-that-selects-its-entity-s-row-by-identity-from-the-input-is-witnessed-on-both-sides-the` (M, about 450, a guess until the first acceptance; the
  passing branch is witnessed on a row created under that identity, a side it cannot arrange is
  refused naming why). RD-D3.
- Touch points, not covered: first report items 10 to 13; second report G13 (`sl13-provided-*`,
  where the read row has no creating command); sixth list G14 (none of the three adds validation
  cost). `story:an-outcome-that-keeps-its-subject-may-emit` builds on the amended story.
- Idiom that works today: give the other entity a distinct identity type.

### The owner's note on transport bindings (2026-11-24)

The note, in three generic lines:

- Request/reply is a command and its response; the broker is only the transport.
- A server-sent-events stream is an event stream.
- A header and a frame are protocol concepts, carried by the protocol specification kind
  (`ess specify protocol`, `ess-protospec/1`); a binding names the protocol document and maps each
  ESS message to one protocol message, and keeps the addresses (path, subject), methods, statuses
  and authentication.

Three existing stories get new bodies and titles, and four new units give the protocol kind what
they need. The units are `gap-p1` drafts. No format version: every addition is optional within
`ess-protospec/1` (PR-D3 A).

- Revised, existing: `story:an-event-channel-may-be-bound-to-server-sent-events` (M 450 + S 200;
  active; binds published events, frames in the protocol document, no longer waits for the exchange
  layer; it also gains the tags `gap-reports-2026-11-24` and `gap-p1`; its status is untouched and
  its new design is not yet reviewed),
  `story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply` (M 520 + M 320 +
  S 260; timeout, no-responders, correlation and reply members move to the protocol document, the
  subject stays in the binding), and
  `story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header` (M 480 +
  M 400 + S 150; the header is declared in the protocol document and the route only maps it).
  The UI live-channel draft `story:a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames`
  has a new body (title unchanged, M about 320) and depends on the revised SSE story.
- New units, in landing order: `story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body` (M, about 450; header fields,
  `required: false`, fixed `value:` and an `application` slot), `story:an-ess-protospec-1-reply-answers-on-the-exchange-of-the-message-it-received-and-a-credential-header-is-named`
  (S, about 260; value source `{kind: exchange}`, credential headers named and never valued),
  `story:an-ess-protospec-1-participant-reads-named-members-of-one-message-in-any-case-and-may-ignore-the-others-and-a` (M, about 420; `participants[].reads`, any-case reading of named
  members) and `story:an-ess-http-1-binding-names-the-protocol-document-its-routes-read-by-its-compiled-digest` (S, about 300; `ess-http/1` top-level
  `protocol: sha256:` plus `--protocol`).
- Left as it is: the commandless-stream draft (`story:an-http-route-streams-a-component-s-events-with-no-command-opening-it-and-frames-named-by-a-member-of-their`), merged into the revised SSE
  story and archived as fully covered.
  `story:a-response-stream-follows-one-row-until-a-terminal-state` has one sentence that went stale
  (the SSE story binds published events only); a line added to its Need says so, and one to
  `story:a-view-answer-may-be-followed-as-it-changes` says the same of its route keys. The exchange stories, the
  authentication story and the protocol-timer story are unchanged.
- Not written: a receiver-side close trigger in `ess-protospec/1`, and the exchange-opened SSE
  stream (written when the exchange layer lands).

### The eighth list (a service contract, second pass) (filed)

The application of the fourth report, twelve items on `ess` 0.98.0, read at the integration tree and
not run ("read, not run" marks an idiom checked by reading the validator only). It ranks with the
fourth report: P1, after the first application's lists, before P2. Fifteen new drafts, and one
amended draft.

- Item 1: several operations on one method and path, told apart by the body (`ESS-HTTP-010`).
  Gap. Six units: `story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected` (M, about 420),
  `story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body` (M, about 550),
  `story:the-openapi-projection-writes-a-route-shared-by-body-selection-as-one-operation-whose-request-body-is-one-of` (M, about 350),
  `story:the-generated-rust-and-go-servers-read-an-input-field-from-the-nested-body-member-its-binding-names` (M, about 350),
  `story:the-generated-rust-and-go-servers-send-a-request-on-a-shared-route-to-the-command-its-body-members-select-and` (M, about 500) and
  `story:the-generated-react-and-terminal-apps-send-the-select-members-and-nested-members-a-bound-route-declares-or` (S, about 250). AE-D1 to AE-D4.
- Item 2: reply member names read in any letter case. Covered by the protocol revision
  (`story:an-ess-protospec-1-participant-reads-named-members-of-one-message-in-any-case-and-may-ignore-the-others-and-a`). No story here.
- Item 3: which origins may read an answer (CORS). Gap. `story:an-http-binding-declares-which-origins-may-read-its-answers-and-the-openapi-projection-states-the-preflight`
  (M, about 480) and `story:the-generated-rust-and-go-servers-answer-the-cors-preflight-and-let-only-the-origins-their-binding-declares`
  (M, about 420). AE-D5.
- Item 4: a top-level JSON error body. Two of three shapes are expressible today (an error whose
  wire name is the code, `{<member>: error}`, literal payload fields; read, not run). The third, an
  error object with no member naming it, is the gap: `story:an-error-outcome-may-answer-its-error-s-fields-as-the-whole-json-body-with-no-member-naming-the-error`
  (M, about 400; lands after `story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text`).
  AE-D6. One route's `500` for an unreadable body amends
  `story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so`
  (AE-D7); no amendment text was filed with this list, so that story is not changed here.
- Item 5: one route on any method. Declined, with the idiom: route the method callers send (AE-D8).
  No story.
- Item 6: a second port for admin routes. Expressible today (read, not run): a second component owns
  those commands' domain and has its own `servers:` entry (AE-D9). No story.
- Item 7: a URL taken from a setting. Expressible today: the callee's path stays in the binding, the
  caller's address is a `settings:` entry, and an `ess-runtime` endpoint slot carries it per
  environment (AE-D12). No story.
- Item 8: callers without a route. Expressible today (an actor with `may:` grants); the refusal's
  hint misleads: `story:the-refusal-of-a-component-that-owns-accepts-and-publishes-nothing-names-the-actor-as-the-home-of-a-caller` (S, about 40). AE-D10.
- Items 9 and 10: one event type emitted several times from one outcome, and an event only for a
  value that changed. One feature, gap: `story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an`
  (L, about 580), `story:synthesis-and-the-interpreter-witness-every-emission-of-an-outcome-in-order-and-both-sides-of-a-conditional` (L, about
  550) and `story:generated-rust-and-go-behaviour-emits-repeated-and-conditional-emissions-as-the-model-declares-generated-docs` (M, about 450). AE-D11. Touch
  point, not covered: `story:an-outcome-that-keeps-its-subject-may-emit`.
- Item 11: milliseconds since the epoch read as a `Timestamp`. Covered by the active stories of
  beyond10x/ess#758 (`story:a-timestamp-newtype-declares-its-wire-encoding` and its codec,
  projection and synthesis stories). One amendment under AE-D16 A (the copy between encodings is
  admitted and converted); no amendment text was filed with this list, so those stories are not
  changed here. Not built yet.
- Item 12: from the protocol models. A filter list as comma-joined text is the HTTP binding's:
  `story:a-query-form-or-view-parameter-list-may-travel-as-one-comma-joined-value-validated-compiled-and-projected` (M, about 350) and
  `story:the-generated-servers-split-and-the-generated-apps-join-a-list-the-binding-declares-comma-joined` (M, about 300), AE-D13. A configurable
  bound revises the existing draft `story:a-protocol-timer-takes-its-duration-from-a-message-field`
  (title and body replaced; M, about 380; it also gains the tags `gap-reports-2026-11-24` and
  `gap-p1` and edges to the three `pr0` units that edit `protocol.rs`), AE-D14, format per PR-D3. A
  counter not kept across a restart is expressible today (a local `restart` input), AE-D15; no
  story.
- Archived: `story:two-crossed-route-templates-may-be-served-in-an-order-the-binding-declares`
  (AE-D1 A; no finding stands behind it; the lifecycle makes `archived` final).
- The README-style "open gaps" of the same application that are not in this list are covered by
  existing stories: `story:an-identity-may-be-a-composite-of-fields` (active) and
  `story:create-beside-a-move-on-the-same-key` (draft).

### Open issues, 2026-11-24 (filed)

Hotfix wave on the released line: #796, #793, #798, #809, #794 (one tree, serial). Every other open
issue: one pull request, serial, after a relevance triage. The one pull request has one new source
format, `ess/38`, opened by the unit already in work for #640 (D77).

Fourteen stories (tag `issues-2026-11-24`, each with its issue as a `github` ref) and two
amendments. After the plan check of the same day the texts were revised and the units below
accepted. Issue, story, size, status:

- #796, `story:a-subject-read-on-a-deletes-outcome-is-refused-with-a-message-that-names-deletes-and-the-related-value-read`, S, accepted
  (active). Message text only; no committed byte moves. Hotfix wave.
- #793, `story:the-generated-rust-port-of-a-component-that-accepts-no-command-and-declares-no-view-builds-with-warnings`, S,
  accepted (active). Hotfix wave.
- #798, an amendment of the active `story:obligation-contracts-state-what-the-suite-checks` (S
  added; its body holds the revised amendment, no status move). Hotfix wave.
- #794, `story:synthesis-sends-the-steps-that-arrange-a-held-state-and-the-refusal-step-itself-as-a-caller-whose-attribute`, M, accepted
  (active). Hotfix wave by the owner's order, not by content.
- #795, `story:a-creating-command-beside-an-exists-false-related-row-refusal-puts-its-second-row-under-the-subject-s-existing`, M, accepted
  (active). The one pull request.
- #809, #804 and #816 item 4: the feature
  `story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once` (umbrella, draft, not built
  itself, not accepted), with two units derived from it:
  `story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths` (#809, S-M, accepted,
  active; hotfix wave) and
  `story:an-exists-false-branch-beneath-several-input-refusals-is-sent-an-input-that-refutes-every-one-of-them-at-once` (#804, S, accepted,
  active; the one pull request). #816 item 4 is now an amendment of the active
  `story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes` (S added, retitled; one
  builder for `witness::fields`, after the #809 unit); the draft
  `story:the-interpreted-target-mints-a-generated-true-value-of-a-text-newtype-whose-invariant-fixes-its-length-so` is archived as covered by
  that amendment. #734 and #726 are active stories, unchanged; the feature records what they
  cover.
- #799, `story:a-view-member-an-outcome-clears-is-claimed-by-the-suite-and-published-by-the-interpreted-target-in-the-member` (suite and
  interpreter, M, accepted, active; breaking for stored suites that claim `null`) and
  `story:the-generated-rust-and-go-servers-write-null-for-an-absent-member-declared-null-when-absent-in-view-rows-and` (servers, S-M,
  accepted, active); independent of each other, both in the one pull request unless a hotfix is
  ordered for the servers.
- #822, in two parts, both draft and accepted with the format batch:
  `story:a-related-value-source-that-selects-its-row-with-where-may-read-the-row-s-lifecycle-state-with-field-state-as` (part 1: validation, compiler, IR
  and an arm in every consumer; L-; format `ess/38`, DF-D5 A and D77; parent
  `epic:generic-row-queries`; it starts after the unit that opens `ess/38`, a relation owed to that
  unit's id) and
  `story:the-interpreted-target-reads-and-synthesized-suites-expect-the-lifecycle-state-of-a-row-a-related-value-source` (part 2, M, after part
  1).
- #819, in two parts, both draft and accepted with the format batch:
  `story:a-conformance-suite-s-view-read-may-hold-the-target-s-clock-at-an-instant-set-from-an-observed-value-and-every` (part 1: suite step, runners, interpreter; M; a
  suite format, DF-D6 A and D77, at the suite pair the one pull request opens once) and
  `story:synthesis-reads-a-view-whose-filter-orders-a-minted-timestamp-against-now-by-observing-the-minted-value-and` (part 2: synthesis; M; after part
  1 and after the active `story:state-param-binds-from-post-command-state` and
  `story:synthesis-arranges-the-related-rows-a-guard-or-read-needs`).

Build order of the one pull request: d795, d804, the two d799 units, the #816 amendment with its
story, d822 part 1, d822 part 2, d819 part 1, d819 part 2. Pairs on one file: `synthesize.rs`
(d794 first, then d795, d799, d822 part 2, d819 part 2), `related_guard.rs` (d795 then d804; the
active related-row story rewrites the same `unclaimed` and the second to land rebases),
`witness.rs` (d809, then the #816 amendment). Every unit that edits a source of the synthesis suite
re-stamps `synthesis-sources.digest` in its own change, and the second to land on a tree re-runs
the stamp command after rebasing.

The `ess-ui-tui` navigation accessor the exploration design page names is now a line in the scope of
`story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test`.

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
  `story:navigation-reaches-any-depth-in-guards` (existing, active), an umbrella built only by
  these two units; neither unit depends on it, and it no longer depends on the view-navigation
  story (P2A-D7 A). `story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its` (L,
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

P0, 13 units in 8 rounds (the table under P0 above): R1 `story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state`,
`story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from`, `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in`; R2 `story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test`, `story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows`; R3
`story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test`, `story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once`; R4 `story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema`; R5
`story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling`; R6 `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`; R7 `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`, `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and`; R8
`story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers`. R5 to R8 are one release. The follow-ups `story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that`,
`story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps` and `story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own` run after P0, in no P0 round.

Second report, six lanes built side by side, one after another within a lane:

- Renderer lane: `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer` (defect), `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer` (defect),
  `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer`, `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model`, `story:a-time-date-or-timestamp-field-declares-whether-it-is-drawn-in-utc-or-the-reader-s-local-zone`, `story:a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end`;
  `story:a-page-header-s-badge-takes-its-tone-and-words-from-its-value-as-a-badge-does` slots in after `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer`. `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer` also
  lands before `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order` and before `story:a-chart-names-its-axes-and-formats-its-ticks`
  (same function).
- Filter and host lane: `story:one-search-reads-several-fields-each-word-found-in-any-of-them-documented-and-pinned-in-every-renderer`, then `story:one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path` (after the active
  reused-template story).
- HTTP lane: `story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their` (after the active
  `story:an-answer-may-be-wrapped-in-one-named-member`), then `story:the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows` (after the
  active `story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member`) beside
  `story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares` (after the active `story:a-generated-ui-reads-the-answers-its-http-binding-declares`),
  then `story:a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames` (after the revised SSE story);
  `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model` shares `binding.rs` with the active
  `story:a-generated-ui-reads-the-answers-its-http-binding-declares` and merges after it.
- Literal lane: `story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share`.
- Lifecycle lane: `story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command` (after `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`), then `story:synthesis-reaches-every-state-of-a-provided-entity-through-its-provider-and-never-drives-a-move-the-deployment`.
- Union wire lane: `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object` (after the active
  `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` and
  `story:projections-use-a-unions-declared-content-key`, and after `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`; the format is
  decided in the format triage, D57, before acceptance), then `story:generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares` (after the
  active `story:generated-codecs-and-the-suite-use-a-unions-declared-content-key`);
  `story:a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model` waits for `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object`.

Third list: `story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers` (patch 0.98.1, in progress); `story:a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test` rides the same
patch 0.98.1 (its change is in the `hotfix/0.98.1` tree); and
`story:in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as` follows in the next free patch after 0.98.2, which carries the fourth
report's finding 1 (`sp4` needs 0.98.1: a held event re-applied after the answer must not duplicate
a row; it edits `runtime/live.ts.tmpl` after `sp3`);
`story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches` after or beside the item-7 story (same check list); `story:a-terminal-test-checks-what-a-host-slot-is-handed-its-resolved-reads-its-commands-and-the-page-s-parameters`
after SP-D5 and after `story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal` (same `spec.rs` and parity table).

Fifth list: `story:the-reference-shows-how-a-section-shows-a-paragraph-or-a-field-s-value-with-no-label-and-both-renderers-are` any time (no key, independent of P0); `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` after
`story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from` and `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`; `story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub` after `story:one-rule-decides-which-external-addresses-a-document-may-name-and-ess-ui-check-and-both-renderers-apply-it` and
the active `story:an-action-names-its-icon`; `story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a` after `story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub`; `story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub` after `story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a` (its tests use
`expect_visit`).

Fourth report: `story:a-second-caller-s-run-expects-the-identity-it-sent-also-in-a-nested-payload-member` first (a defect, patch release after
0.98.1); `story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so` before `story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text` (shared body checks in
`lib.rs`); `story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header` after the authentication story; `story:a-401-answer-may-state-that-the-service-sends-no-www-authenticate-challenge` after the
authentication story (one `ESS-HTTP-027` check); `story:two-crossed-route-templates-may-be-served-in-an-order-the-binding-declares` is archived;
`story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply` and
`story:a-broker-binding-carries-an-exchange` both open `ess-transport/3`, so one after the other.
The stream finding is carried by the revised SSE story, which no longer waits on the exchange
stories (PR-D4).

Sixth list: G14 first, as P0: `story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per`, then `story:a-branch-the-row-set-strategy-prepares-is-prepared-once-per-model-branch-actors-and-arrangement-context-and` (one file, one
after another); `story:a-row-set-branch-s-search-does-not-arrange-a-combination-of-counts-that-provably-cannot-take-the-branch-and` is not scheduled (PF-D4). `story:with-model-ess-ui-check-refuses-a-fixture-row-or-a-script-payload-whose-value-contradicts-the-type-the-model` after `g07-session-script` and `story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches` (same
`CHECKS` list and schema check list; not a hard dependency, the later story rebases), then
`story:ess-ui-test-model-refuses-to-start-on-mistyped-document-fixtures-and-fails-a-test-whose-own-fixtures`; `story:a-command-that-records-its-caller-and-is-granted-to-a-system-actor-is-refused-naming-that-actor-as-the-system` any time; `story:at-ess-39-a-command-guard-compares-a-fact-with-an-expression-of-several-facts-and-every-expression-reads-the` once
`ess/39` is open (`story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`, one opener only), then `story:synthesis-witnesses-a-command-guard-that-compares-a-fact-with-an-expression-of-several-facts-on-both-sides-of` and
`story:every-target-names-what-it-does-with-a-guard-over-an-expression-of-several-facts-generated-rust-and-go` side by side.

Seventh report: the amended `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`
first (it runs beside the units of the wave of 2026-11-24: s3 and s4 edit other functions of
`synthesize.rs`), then `story:the-refusal-of-an-input-whose-identity-type-several-entities-share-on-a-branch-with-no-subject-names-the-forms` (after s1 of that wave, which edits the same
function, and after the amended story, so its hint names no form synthesis refuses), then
`story:a-row-set-guard-that-selects-its-entity-s-row-by-identity-from-the-input-is-witnessed-on-both-sides-the` (after s4, or rebased on it; if it slips, `story:the-refusal-of-an-input-whose-identity-type-several-entities-share-on-a-branch-with-no-subject-names-the-forms` names
only the `preserves:` form and `story:a-row-set-guard-that-selects-its-entity-s-row-by-identity-from-the-input-is-witnessed-on-both-sides-the` adds the second).

Transport bindings (the owner's note): `story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body` first; then
`story:an-ess-protospec-1-reply-answers-on-the-exchange-of-the-message-it-received-and-a-credential-header-is-named`, `story:an-ess-protospec-1-participant-reads-named-members-of-one-message-in-any-case-and-may-ignore-the-others-and-a` and
`story:an-ess-http-1-binding-names-the-protocol-document-its-routes-read-by-its-compiled-digest`, merged one after another (they share `protocol.rs` in three
crates). After those, unit a of the SSE story, of the broker request story and of the header-input
story; the SSE and header-input stories share `ess-http/src/lib.rs`, so they go in series. Every
`pr0` unit and the UI live-channel link wait for PR-D1, PR-D2 and PR-D3 (architecture); the SSE
story also for PR-D4, the header-input story for PR-D5.

Eighth list (every lane is serial within itself; lanes run side by side except where a file is
named):

- HTTP binding lane (`crates/specify/ess-http/src/lib.rs`, one at a time): `story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected`,
  `story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body`,
  `story:an-http-binding-declares-which-origins-may-read-its-answers-and-the-openapi-projection-states-the-preflight`, `story:a-query-form-or-view-parameter-list-may-travel-as-one-comma-joined-value-validated-compiled-and-projected`;
  then `story:an-error-outcome-may-answer-its-error-s-fields-as-the-whole-json-body-with-no-member-naming-the-error` after the 2xx-refusal story and unit a of
  the plain-text story, which are already ordered that way.
- OpenAPI (`crates/generate/ess-gen/src/openapi.rs`): `story:the-openapi-projection-writes-a-route-shared-by-body-selection-as-one-operation-whose-request-body-is-one-of`
  after `ae1a`; `ae3a`, `ae4` and `ae12a` also edit it, in the binding lane's order.
- Server lane (`rust/http/bound.rs`, `go/http/bound.rs`): `story:the-generated-rust-and-go-servers-read-an-input-field-from-the-nested-body-member-its-binding-names`,
  `story:the-generated-rust-and-go-servers-send-a-request-on-a-shared-route-to-the-command-its-body-members-select-and`,
  `story:the-generated-rust-and-go-servers-answer-the-cors-preflight-and-let-only-the-origins-their-binding-declares`,
  `story:the-generated-servers-split-and-the-generated-apps-join-a-list-the-binding-declares-comma-joined`; the server half of `ae4` joins where it lands.
- App lane (`crates/ui/`): `story:the-generated-react-and-terminal-apps-send-the-select-members-and-nested-members-a-bound-route-declares-or` after `ae1a` and
  `ae1d`; `story:the-generated-servers-split-and-the-generated-apps-join-a-list-the-binding-declares-comma-joined` touches the same two app files and
  goes after `ae1f`.
- Language lane: `story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an` once its source format
  is open, then `ae9b` and `ae9c` side by side (different crates).
- Protocol lane: the revised timer draft after the `pr0` units that edit `protocol.rs`, once PR-D3
  is decided.
- Any time: `story:the-refusal-of-a-component-that-owns-accepts-and-publishes-nothing-names-the-actor-as-the-home-of-a-caller`.

## Format

Eighth list: `story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an` needs a new source-format
version (the `emits` row of the effect by target grid), and `story:synthesis-and-the-interpreter-witness-every-emission-of-an-outcome-in-order-and-both-sides-of-a-conditional`
possibly a suite-format pair: format: decided before acceptance. The revised timer draft follows
PR-D3 (additive within `ess-protospec/1` under A, `/2` under B). Every `ess-http/1` story of the
list (`ae1a`, `ae1d`, `ae3a`, `ae4`, `ae12a`) adds optional keys or words, so absent compiles to the
same bytes and an older reader refuses the key by name; `ae3a` adds one refusal code (the next
free number when it lands). The other stories add no version.

Stories that name `ess/38` or `ess/39`: `story:at-ess-39-a-command-guard-compares-a-fact-with-an-expression-of-several-facts-and-every-expression-reads-the`, `story:synthesis-witnesses-a-command-guard-that-compares-a-fact-with-an-expression-of-several-facts-on-both-sides-of`,
`story:every-target-names-what-it-does-with-a-guard-over-an-expression-of-several-facts-generated-rust-and-go`,`story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields`, `story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row`,
`story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected`, `story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and`, `story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its`,
`story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38`, `story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted`, `story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by`, `story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command`, `story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object`
together with the active `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`,
`story:projections-use-a-unions-declared-content-key` and
`story:generated-codecs-and-the-suite-use-a-unions-declared-content-key` (written for `ess/37`,
which shipped without them; one format for the union's wire layout, decided in the format triage,
D57, before acceptance),
and, under SL-D8 option B, `story:a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share`. `ess-diff/24` carries the diff kinds of
`story:ess-verify-diff-reports-an-entity-s-readers-declared-removed-widened-or-narrowed-at-ess-diff-24`. For all of them: format: decided before acceptance (one source format per
design decision, `docs/design/generic-language-features.md` § Policy, 2).

New published formats that are not specification formats: `ess-ui-state/1` (`story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`)
and `ess-ui-resolved/1` (`story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`); P1-D1 option B would add `ess-ui-test-trace/1`.
The other stories add no format version.

## Open decisions

Each has a default. Without an answer the default is taken and said so in the next report; every
option is reversible until its story's code is merged.

P0 (items 1 to 3). P0-D14 (B) and P0-D17 (B) are settled; P0-D18 (A) is settled by the coordinator,
2026-11-24; the others are taken on their defaults:

- P0-D1 (item 1): where the exploration surface lives. A: `ess_ui_test::Session`; B: a new crate; C:
  in `ess-ui`; D: decline. Taken: A.
- P0-D2 (item 1): how a state is reached without replay. A: fork in process, `ess-ui-state/1`
  describes the state and its digest and restores nothing; B: restore from the document; C: the
  document carries its step path. Taken: A.
- P0-D3 (item 1): a command-line verb. A: `ess ui state`, last in order; B: library only. Taken: A.
- P0-D4 (item 1): steps with a free value. A: offered by target, the caller supplies the value; B:
  ESS picks values; C: left out. Taken: A.
- P0-D5 (item 1): the clock in a state's identity. A: absolute virtual time plus script positions;
  B: durations until each pending event. Taken: A.
- P0-D6 (item 2): the canonical rule. A: the `InfraIR` scheme with the ECMAScript number rule
  (compact key-sorted JSON, bare SHA-256, no trailing newline); B: RFC 8785 with a prefix; C: the
  `EssIr` struct-order style. Taken: A.
- P0-D7 (item 2): does the digest cover author text and traceability keys. A: yes, one digest is
  one document; B: no; C: both as two digests. Taken: A.
- P0-D8 (item 3): is `ess ui load --format json` versioned. A: yes, `ess-ui-resolved/1`, published
  once and complete; B: no; C: decline. Taken: A.
- P0-D9 (item 3): the document part. A: the loader's filled tree in `ess-ui/1` keys; B: `Serialize`
  on `Document`; C: a new shape. Taken: A.
- P0-D10 (item 3): does the React generator read the printed JSON. A: no, a round-trip test; B:
  yes. Taken: A.
- P0-D11 (item 3): what `--model` adds. A: the existing `Binding`, only under `--format json`; B:
  enums only; C: binding plus row field types. Taken: A.
- P0-D12 (item 3): `Field.label` default. A: the field name with `_` as a space, schema corrected;
  B: field labels in the specification language; C: leave unfilled. Taken: A.
- P0-D13 (item 3): where `Field.tone: neutral` is filled. A: only beside `tone_by`, schema note
  corrected; B: everywhere; C: nowhere. Taken: A.
- P0-D14 (item 3): how React keys a server state once `scope` is always filled. A: the loader
  records the filled keys; B: a bare key wherever the scope equals its store's default, written or
  not (the partner portal's `layout` moves once; a CHANGELOG line under Changed, breaking for a
  generated React project); C: do not fill `scope`. Settled: B. No record of filled keys exists.
- P0-D15 (item 3): `form.fields`. A: from the model when `--model` is given; B: each renderer
  derives it; C: `--format json` requires `--model`. Taken: A.
- P0-D16 (item 3): `Reads.key` and `Live.match`. A: each from the other, else `id`, except on a
  choice's read; B: `id` everywhere; C: only from each other. Taken: A.
- P0-D17 (item 3): where both renderers agree with each other against the schema's default
  (`confirm_label`, `rich_text.syntax`, `DynamicNavEntries.label`). A: the schema; B: correct the
  schema to what both do. Settled: B, by the operator, per key. `Document.actor` was taken out.
- P0-D18 (item 3): which actor rule both renderers follow. A: the reference's row source with
  React's sign-in condition (the first row of the first view the home page's shell preloads,
  signed in under `from_session` only when that row exists; the terminal stops signing in without a
  row); B: React's rule in both; C: the terminal's rule in both. Settled: A.

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
- SL-D7 (G1, reworded 2026-11-24): may views of one component share one route by their members? A:
  yes, the three shared-route units; C: decline. The member body itself, `{<member>: array}`, is
  the active `story:an-answer-may-be-wrapped-in-one-named-member` (the former B). Default A.
- DD-X3 (G1, the shared route; SECURITY: waits for the owner): what a caller granted only some
  views of a shared route gets. A: the route's not-granted refusal; B: the object with only the
  granted members; C: refuse at validation a shared route whose views have different grants.
  Default A, written into `story:the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows`.
- SL-D8 (G2): which formats admit a `Binary64` literal. A: every format that has `Binary64`, by the
  `Decimal` precedent; B: only the format `story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38` opens. Default A.
- SL-D9 (G9): settled, declined as styling. Open half: A: decline a truncation contract too; B: a
  `wrap: false` contract on a column or record field. Default A.
- SL-D10 (G11 colour half): settled, declined as styling. No open question.
- SL-D11 (G13): how lifecycle moves only the deployment makes are declared. A: through `provided:
  true`, from `ess/39`, two units; B: decline, with the system-actor idiom; C: a `recorded_by`
  spelling. Default A.
- SL-D12 (F18; superseded by PR-D4): how `ess-http/1` takes an event stream no command opens. A: widen the active SSE
  story with a commandless `stream:` and a UI unit; B: wait for the exchange layer; C: decline.
  Default A.
- SL-D13 and SL-D14 (G5): settled by reading, 2026-11-24. The layout is declared on the union, as
  the active `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` places the
  encoding selector, and spelled `layout: adjacent | flat`, because the selector also carries the
  keyed-by-variant encoding of `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`.
  The format is decided in the format triage (D57) before acceptance.

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
  Applied on its default 2026-11-24: the units do not depend on the parent, and the parent does
  not depend on the view-navigation story.

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
  such links sit between page entries). A: an entry of a fixed `pages` list may be `{name, label,
  href, icon?, synonyms?}` instead of a page name, in its place, as `story:an-entry-of-a-navigation-section-is-a-page-or-an-outside-address-in-one-ordered-list-so-a-menu-needs-no-stub` (the Rust
  type `NavPages::Fixed` changes; an older `ess` refuses it); B: a new ordered key `entries` that
  replaces `pages` when present; C: `links` beside `pages`, drawn after them (no outside entry
  between pages). Default A (in).

Fourth report (a service contract):

- AC-D1 (finding 2, also F18; superseded by PR-D4): may an event stream follow no row. A: amend the exchange story so
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
- AC-D6 (finding 4; moot under AE-D1 A, the story is archived): may two crossed route templates both be bound. A: a route declares
  `precedes:` and both servers dispatch in that order; B: decline; C: admit a pair whose variable
  types cannot hold each other's literal. Default A.

Sixth list (items G12, G13, G15):

- SX-D1 (G15): how severe is a fixture value that contradicts the model. A: every finding an
  error; B: every finding a warning; C: shape findings (kind, unknown or missing member, enum
  value, union tag) errors, an identity's `prefix` or `alphabet` a warning. Default C.
- SX-D2 (G15): where the types come from. A: the compiled IR's field types as plain data
  (`RowTypes`) walked by one `ess-ui` function; B: the served route's response schema from the
  bound OpenAPI document, validated with `schema-contract`. Default A.
- SX-D3 (G15): what `ess ui test --model` does with a mistyped fixture. A: a document fixture
  error refuses the run, a test's own fixture error fails that test at setup (`test.fixme` in the
  Playwright spec), warnings print; B: every error fails each test it reaches at setup; C: print
  and run everything. Default A.
- SX-D4 (G12): what the item opens. A: arithmetic over several facts as a command guard operand
  and `xs.count` in every expression (three units, about 1,400 lines, `ess/39`); B: the coverage
  quantifier "for every pair of two lists a row exists", with the quantifier work of item 9; C:
  A now, B later; D: decline, the application keeps the `external:` cause. Default A.
- SX-D5 (G12): in which guard positions. A: `when:`, `when_subject:` and the `when_related:`
  predicate, other positions refused by name; B: every predicate position; C: `when:` only.
  Default A.
- SX-D6 (G12): how an expression spells a list's length. A: `xs.count`, and the design page's
  `count(xs)` is amended; B: `count(xs)`, with `.count` guard-only; C: both. Default A.
- SX-D7 (G13): may a command record a caller attribute when a system actor is granted it. A: a
  system actor declares constant attributes (`ess/39`); B: the caller read names a fallback value
  (`ess/39`); C: decline, with the documented idiom (an automated actor with an identity is a
  caller with `attributes:`, not `system: true`) and a refusal that names both. Default C. A and
  B touch who is recorded as having done something, an audit and authority boundary.

Seventh report (a command that only reads one row):

- RD-D1: how the amended preserve-without-view story is delivered. A: a unit beside the wave of
  2026-11-24, released with it; B: a patch release from 0.98.0 on the hotfix route, ahead of the
  wave (a tag is an operator decision; its fit review classes it a gap); C: after the wave.
  Default A.
- RD-D2 (attempt 1): how an author says which entity an input names when its identity type is
  shared and the branch acts on no subject. A: no new surface, the refusal's hint names
  `preserves:` with `instance:` and the row-set selector by identity; B: a `references: <entity>`
  key on an input field (`ess/39`, every target); C: the requester's `entity:` beside `via:` in
  `when_related` (`ess/39`, a second spelling against `docs/design/filtered-related-reads.md:39`).
  Default A.
- RD-D3 (attempt 2): a row-set selector that pins an identity the target mints. A: synthesis sends
  the captured instance as the input, as `instance:` does, and witnesses both sides; B: refuse that
  branch by name and point to `preserves:` with `instance:`; C: change nothing. Default A.

The owner's note on transport bindings. PR-D1 to PR-D5 are marked "architecture: waits for the
owner": they do not take the 15-minute default. Defaults are what the stories are written to.

- PR-D1 (architecture: waits for the owner): where header, frame, correlation and stream-end detail
  are written. A: in an `ess-protospec/1` document that a binding names; B: in each binding; C: a
  third document kind, a codec binding. Default A.
- PR-D2 (architecture: waits for the owner): which way the link between a binding and a protocol
  document points. A: binding to protocol, a top-level `protocol: sha256:<hex>` and `--protocol`
  beside `--http` or `--transport`; B: protocol to specification (`ess-protospec/2`); C: a third
  document that pairs them. Default A.
- PR-D3 (architecture: waits for the owner): the format consequence of the protocol-kind additions.
  A: additive within `ess-protospec/1`, each key skipped when empty, an old-reader test per unit;
  B: `ess-protospec/2`. Default A.
- PR-D4 (architecture: waits for the owner): what an SSE route binds. A: the events one component
  publishes (supersedes AC-D1 and SL-D12); B: only a named exchange; C: AC-D1 A, `follows`
  optional. Default A.
- PR-D5 (architecture: waits for the owner): where a request header is declared for an HTTP route.
  A: in the protocol document, the route names `request.message` and maps `request.headers`; B: in
  `ess-http/1`; C: as A, and response headers move too. Default A.
- PR-D6: what happens to the response headers `ess-http/1` declares. A: they stay, released and
  unchanged; B: plan their move in `ess-http/2`. Default A.
- PR-D7: how conformance runs the scenarios over the bound transport when no wire target exists. A:
  each binding story proves the suite is byte-identical with and without the binding, and the wire
  run is a `--target process` adapter; B: a loopback per story; C: a real wire target per story.
  Default A.
- PR-D8: who names a broker request's subjects. A: the binding (`requests[].subject` in
  `ess-transport/3`), the reply subject is drawn by the requester; B: the protocol document; C: a
  template in the protocol document filled by the binding. Default A.
- PR-D9: do generators serve or call a broker request. A: no, `ess-publisher` and every generated
  server refuse a transport document with `requests:` by name; B: `ess-publisher` skips it and
  says so. Default A.

The eighth list (a service contract, second pass). A decision marked SECURITY (what a generated
server answers or parses, who may call a route, which origin may read an answer, which port serves a
route, where a URL comes from) or ARCHITECTURE (a new link between formats, a new kind of binding, a
new protocol-format version) waits for the owner and does not take the 15-minute default; its
default is fail-closed. AC-D3, AC-D4, AC-D5 and PR-D1 to PR-D9 stay open.

- AE-D1 (item 1): what happens to the crossed-templates story. A: archive it, "no finding: the
  reported need is several operations on one path"; B: keep it as a draft, deferred; C: revise it
  into the body-selection story. Default A. Taken: A, the story is archived.
- AE-D2 (item 1; SECURITY: waits for the owner): how one route tells several operations apart. A:
  `select:` per route, member-path equality with `String`, `Boolean` or `Integer` literals; a route
  without `select` is the catch-all, and with none an unmatched body is `malformed`; B: one
  `discriminator: <member>` per shared path; C: decline. Default A.
- AE-D3 (item 1): which `operationId` OpenAPI gives a shared method and path. A: the first sharing
  route's, all listed in `x-ess-operations`; B: a `group_id` key; C: refuse the projection. Default
  A.
- AE-D4 (item 1; SECURITY: waits for the owner): may a JSON request read an input field from a
  nested member. A: `request.members: {<field>: <member.path>}`, the model's input stays flat, a
  missing required field at any level is `malformed`; B: a struct input holding the envelope; C:
  decline. Default A.
- AE-D5 (item 3; SECURITY: waits for the owner): may a binding state which origins read its answers.
  A: a `cors:` policy in `ess-http/1` (listed origins or `any`, `credentials`, headers, `max_age`;
  `any` with credentials refused; absent means no cross-origin reads); B: authored literal headers
  and an `OPTIONS` route; C: decline, it is gateway policy. Default A.
- AE-D6 (item 4; SECURITY: waits for the owner): may an error outcome answer its error's fields as
  the whole body. A: `body: error`, the error object bare; B: `{<member>: error}` with `member:
  none`; C: decline. Default A.
- AE-D7 (item 4; SECURITY: waits for the owner): may `malformed` answer a `5xx`. A: amend the
  2xx-refusal story so it may, with `body: none` or a literal (`not_granted` and `too_large` keep
  `4xx`); B: decline; C: split `malformed` in two. Default A.
- AE-D8 (item 5; SECURITY: waits for the owner): may one route answer any method. A: decline, with
  the idiom; B: `method:` may be a list; C: `method: any`. Default A. A story only under B or C.
- AE-D9 (item 6; SECURITY: waits for the owner): may some routes of a component be served on a
  second port. A: the idiom, a second component with its own `servers:` entry; B: per-route
  `servers:`; C: OpenAPI-only per-operation servers. Default A.
- AE-D10 (item 8): is a caller that serves nothing a component. A: no, it is an actor with `may:`
  grants and the refusal's hint says so; B: change nothing; C: a component kind `caller`. Default A.
- AE-D11 (items 9 and 10): may an outcome emit one event several times, and an emission hold only
  under a predicate. A: an `emits:` entry may be an occurrence `{event, payload, when}` at a new
  source format (format: decided before acceptance); B: a chain of moves and the branch product
  idiom; C: only `when:`. Default A.
- AE-D12 (item 7; SECURITY: waits for the owner): where a URL taken from a setting lives. A: the
  idiom, a `settings:` entry on the caller and an `ess-runtime` endpoint slot, no link to the
  route; B: `servers[].url` from a component setting; C: a declared link from setting to operation,
  ARCHITECTURE. Default A.
- AE-D13 (item 12; SECURITY: waits for the owner): may a list travel comma-joined. A: `lists:
  {<name>: comma}` on a route, written strictly; B: A plus declared leniencies; C: decline. Default
  A.
- AE-D14 (item 12; ARCHITECTURE under B, or if PR-D3 B makes a new protocol format: waits for the
  owner then): where a protocol timer's non-literal duration comes from. A: revise the timer draft
  so a duration reads an `Integer` field of the trigger or a local field below a ceiling; B:
  top-level `parameters:` swept by exploration; C: decline. Default A.
- AE-D15 (item 12; ARCHITECTURE under B: waits for the owner then): does the protocol kind get a
  declared restart. A: no, a restart is a local input resetting volatile fields and timers; B: a
  declared fault; C: A now, B when a model needs durable timers. Default A.
- AE-D16 (item 11): may an encoded `Timestamp` newtype be copied into a plain `Timestamp` field. A:
  yes, amend the encoding story and its codec story to convert; B: no. Default A.

- PF-D4 (G14): may the skip rule of the second row-set story allow a selector linked to the input
  row. A: not now, the story stays a draft and is not scheduled; B: allow it, after a design page.
  Default A.

Open issues (defects). DF-D1 to DF-D4 are taken on their defaults, 2026-11-24; DF-D5 and DF-D6
are taken as A by the format ruling D77, 2026-11-24.

- DF-D1 (#796): is `{subject: ...}` admitted on a `deletes:` outcome. A: not in this unit; the
  refusal names `deletes:` and its hint points at `{related: {via: input.<id>, field: ...}}`; B:
  admit it from the next source format, in a story of its own; C: admit it at every format.
  Default A. Taken: A.
- DF-D2 (#804): how far the "every input refusal refuted" goal reaches. A: only in `unclaimed`,
  after its existing goals; B: in `Directed::goals` for every directed search. Default A. Taken: A.
- DF-D3 (#799): what no `presence:` means for a view member. A: the member is left out when
  absent; B: `null`; C: either spelling. Default A. Taken: A.
- DF-D4 (#799): does the suite claim that an `omitted_when_absent` member is left out. A: no, the
  suite drops the `null` claim and adds no claim of absence; B: a new suite major with a per-field
  presence. Default A. Taken: A.
- DF-D5 (#822): which source format admits `field: state` in a `where:`-selected related value.
  A: the open format `ess/38`; B: the next format after it; C: no format. Taken: A (D77: the one
  pull request has one new source format, `ess/38`, opened by the unit already in work for #640).
- DF-D6 (#819): how a view ordering a minted `Timestamp` against `now` is read. A: suite-side, a
  held read clock around an observed minted value, a new suite version and no source format; B: a
  source spelling; C: keep the refusal. Taken: A (D77: at the suite pair the one pull request opens
  once).

## Accepted

Accepted 2026-11-24 for the hotfix wave (decision D76 of the coordinator, under the owner's
delegation, on the owner's order of 03:10 CET to fix the top five issues as one hotfix wave):
`story:a-subject-read-on-a-deletes-outcome-is-refused-with-a-message-that-names-deletes-and-the-related-value-read`,
`story:the-generated-rust-port-of-a-component-that-accepts-no-command-and-declares-no-view-builds-with-warnings` and
`story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths`; #798 rides its already
active story.

Accepted 2026-11-24 after the plan check and the revision of the texts (decision D78 of the
coordinator, under the owner's delegation, on the owner's order of 03:10 CET that every open issue is
delivered in one pull request): `story:synthesis-sends-the-steps-that-arrange-a-held-state-and-the-refusal-step-itself-as-a-caller-whose-attribute`,
`story:a-creating-command-beside-an-exists-false-related-row-refusal-puts-its-second-row-under-the-subject-s-existing`,
`story:an-exists-false-branch-beneath-several-input-refusals-is-sent-an-input-that-refutes-every-one-of-them-at-once`,
`story:the-generated-rust-and-go-servers-write-null-for-an-absent-member-declared-null-when-absent-in-view-rows-and` and
`story:a-view-member-an-outcome-clears-is-claimed-by-the-suite-and-published-by-the-interpreted-target-in-the-member`. The two parts of
#822 and the two parts of #819 are accepted with the format batch.

Accepted 2026-11-24 by the coordinator under the operator's delegation, each with the defaults of
its decisions as written; moved draft, proposed, active:

- `story:a-per-value-channel-script-that-names-no-session-is-reported`
- `story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies`
- `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order`
- `story:a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with`
- `story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer`
- `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer`
- `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer`

Accepted 2026-11-24 by the coordinator under the owner's delegation after a panel of four critics
(76 findings) and one recheck (8 findings), all answered in the texts; moved draft, proposed,
active. The thirteen P0 units, by round:

- R1: `story:a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state`, `story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from`, `story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in`
- R2: `story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test`, `story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows`
- R3: `story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test`, `story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once`
- R4: `story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema`
- R5: `story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling`
- R6: `story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format`
- R7: `story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the`, `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and`
- R8: `story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers`

The three P0 follow-ups (`story:the-model-holds-every-filled-default-as-a-plain-value-and-no-renderer-or-checker-keeps-a-fallback-of-its-own`, `story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that`,
`story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps`) stay drafts.

Accepted 2026-11-24 by the coordinator under the owner's delegation: `story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per`,
an output-identical change with a measured work counter. `story:a-branch-the-row-set-strategy-prepares-is-prepared-once-per-model-branch-actors-and-arrangement-context-and` is accepted after
its check (decision D72 of the coordinator, under the owner's delegation).
`story:a-row-set-branch-s-search-does-not-arrange-a-combination-of-counts-that-provably-cannot-take-the-branch-and` stays a draft and leaves the P0 group (tag `gap-p2`): it does not
shorten the reported case (at most 6 of its 37 refusals qualify, none on the command that holds
94 % of the time); gains only with a linked-selector allowance, open as PF-D4; not scheduled.

## Not accepted yet

Every other story here is a draft until its critics and the coordinator accept it. Item 7 is a
defect: it is reproduced before it is fixed. No story here is dispatched in a wave before its fit review is
accepted.
