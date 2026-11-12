---
format: aep.planning-md/3
id: story:a-related-guarded-command-arranges-its-other-referenced-rows
kind: story
status: draft
title: A command a related guard reads arranges its other referenced rows
summary: 'Adversary finding on story:accepted-input-arranges-referenced-list-rows: a command read by a when_related, list-of-references or row-set guard is skipped whole, so a second references field it copies stays dangling (turn-3 split, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:accepted-input-arranges-referenced-list-rows` arranges the rows an input names when the
branch copies that input into a field carrying a `references` relation, but skips the whole
command when a `when_related:` guard, a list-of-references guard or a row set reads anything
(`crates/verify/ess-conformance/src/synthesize.rs`, the hook in `run_as` guarded by
`related_guard::uses`, `related_list::uses` and `row_set::uses`). Those commands arrange the rows
their own guard reads, through their own families, and a field they copy that no guard reads is
left to the witness search. It holds a fresh identity, so a target that resolves the references it
stores refuses the accepted scenario.

The skip exists because applying the arrangement to such a command moved repository suites in the
wrong direction: it overrode the list rows of `list-of-references.yaml` and the
`list-identity-guard-beside-*` fixtures, and it arranged the row a create-or-update creating half
deliberately leaves unarranged (`create-or-update-beside-related-refusal.yaml`, beyond10x/ess#641).

## Problem

Reproduction: the `demo.reg` model of the accepted-input story with a second reference on `Server`,
`primary` (`cardinality: one`), written by `Register` from `input.primary` beside `sources`, and a
refusal `when_related: {via: input.primary, exists: false}` on `Register`. The accepted scenario
`demo.reg.Register/outcome/accepted` arranges `primary` through the related-guard family, and sends
`sources` as a literal identity. No `AddSource` step precedes it.

## Design rule

On any scenario that requires an accepting branch, every input written into a field carrying a
`references` relation names rows the scenario arranged, whichever family arranged the rest of the
scenario. Where a guard already arranges the rows a field names (the field the guard reads), that
arrangement stands; every other referenced field is arranged as `referenced_input::arrange` does.
The creating half of create-or-update, which validates beside a related guard only by leaving its
row unarranged, keeps doing so.

## Acceptance

1. `cargo test -p ess-conformance --locked --test referenced_list_input
   adversary_a_related_guarded_command_still_sends_dangling_sources` passes (the case is
   `#[ignore = "backlog: guarded-command"]` today): the `Register` step of the accepted scenario
   sends a non-empty `sources` whose every identity is the `id` of an earlier
   `execute_command demo.reg.AddSource` step, and `primary` still names the row the related-guard
   family arranged.
2. `list-of-references`, `list-identity-guard-beside-creates`, `list-identity-guard-beside-updates`
   and `create-or-update-beside-related-refusal` keep their rows of
   `synthesis-suite-bytes-table.tsv`.
3. Each pinned-table row that moves has a model whose guarded command writes a second `references`
   field from its input, named with its cause.

## The case that shows it

`adversary_a_related_guarded_command_still_sends_dangling_sources` in
`crates/verify/ess-conformance/tests/referenced_list_input.rs`.

## Notes for whoever picks it up

- The skip is one condition at the hook; the work is deciding, per family, which fields the
  family already arranged (`related_guard`, `related_list`, `row_set` each know the field their
  guard reads) so the arrangement leaves exactly those alone.
- The four fixtures named in acceptance 2 are the regression set.
