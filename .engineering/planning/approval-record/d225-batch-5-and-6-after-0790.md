---
format: aep.planning-md/3
id: approval-record:d225-batch-5-and-6-after-0790
kind: approval-record
status: approved
title: Batch 5 takes the React look-and-feel gaps; batch 6 the minor findings (D225)
summary: 'Batch 5 (0.80.0): tokens and themes, badge tone and account menu, row confirm key, Uuid path segment; batch 6: diagnostics, rustfmt/clippy, tone contrast, wide tables'
relations:
- decides: story:the-react-renderer-draws-the-declared-tokens
- decides: story:light-and-dark-themes-render-differently
- decides: story:a-badge-field-takes-a-tone
- decides: story:account-menu-entries-keep-sets-and-navigate
- decides: story:a-row-actions-confirm-is-addressed-by-the-views-key
- decides: story:a-uuid-newtype-identity-binds-a-path-segment
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T21:00:10Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-5"}
---
# Batch 5 takes the React look-and-feel gaps; batch 6 the minor findings (D225)

## Decision

On 2026-10-29 the orchestrator approved D225 under the operator's grant. It revises the batches after
`0.78.0`. Batch 4 (`0.79.0`) is unchanged from D220.

Batch 5 (`0.80.0`), four units:

- Tokens and themes: `story:the-react-renderer-draws-the-declared-tokens` and
  `story:light-and-dark-themes-render-differently`.
- Tone on a badge, plus account menu `sets` and `navigate`, with a check refusal for any key a renderer drops:
  `story:a-badge-field-takes-a-tone` and `story:account-menu-entries-keep-sets-and-navigate`.
- `story:a-row-actions-confirm-is-addressed-by-the-views-key`.
- `story:a-uuid-newtype-identity-binds-a-path-segment`.

Batch 6:

- `story:ui-component-diagnostics-cite-their-own-view-and-reader`
- `story:the-rust-crate-layout-is-rustfmt-and-clippy-clean`
- `story:the-built-in-tones-meet-wcag-aa-for-badge-text`
- `story:a-wide-table-scrolls-inside-its-column`

The batch 5 units block a downstream adopter's story; the batch 6 stories do not.

Decider: operator
