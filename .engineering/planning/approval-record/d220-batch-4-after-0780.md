---
format: aep.planning-md/3
id: approval-record:d220-batch-4-after-0780
kind: approval-record
status: approved
title: Batch 4 takes the CLI-over-service gaps, the shared binary and two UI defects (D220)
summary: U16 service_forward and CLI primitives, U17 clap shared binary, U18 required form inputs, U19 record action row scope; after 0.78.0
relations:
- decides: story:a-service-forward-callable-names-a-command-or-a-view
- decides: story:the-cli-projection-carries-timestamp-decimal-and-uuid
- decides: story:components-sharing-a-cli-binary-generate-one-tree-with-every-group
- decides: story:a-form-with-an-empty-required-input-sends-no-command
- decides: story:a-record-action-in-the-terminal-renderer-sees-its-row
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T20:09:41Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-4"}
---
# Batch 4 takes the CLI-over-service gaps, the shared binary and two UI defects (D220)

## Decision

On 2026-10-29 the orchestrator approved D220 under the operator's grant. Batch 4 is the next batch
after `0.78.0`, inside the four-unit cap. It starts once `0.78.0` is released and follows the
batch 2 and batch 3 rules: one local gate, one pull request, one release, and an adopter replay
before the release.

- U16, CLI over the service: `story:a-service-forward-callable-names-a-command-or-a-view` and
  `story:the-cli-projection-carries-timestamp-decimal-and-uuid`, together, because the reporting
  consumer is blocked on both.
- U17, clap shared binary: `story:components-sharing-a-cli-binary-generate-one-tree-with-every-group`.
- U18, required form inputs: `story:a-form-with-an-empty-required-input-sends-no-command`.
- U19, record action row scope: `story:a-record-action-in-the-terminal-renderer-sees-its-row`.

The rest goes to batch 5:

- `story:a-row-actions-confirm-is-addressed-by-the-views-key`
- `story:ui-component-diagnostics-cite-their-own-view-and-reader`
- `story:a-uuid-newtype-identity-binds-a-path-segment`
- `story:the-rust-crate-layout-is-rustfmt-and-clippy-clean`

A unit whose fit review finds a new source format, an ADR-level change, or a breaking change with no
opt-in form leaves the batch with a decision-blocker (D135).

Decider: operator
