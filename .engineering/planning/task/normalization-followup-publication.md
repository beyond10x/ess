---
format: aep.planning-md/1
id: task:normalization-followup-publication
kind: task
status: archived
title: Publish shared Binary64 codec and positional normalization guidance
relations:
- derived_from: story:binary64-structural-codecs
- derived_from: story:normalize-positional-array-input
- serves: vision:O2
revision: 3
---
## Outcome

Publish accurate shared capability and compatibility documentation after the
standalone Binary64 codecs and positional normalization units integrate and pass
their gates. This is coordinator-owned integration work, so those implementation
units never compete to edit the same public summary or planning journal.

## Scope and ownership

- CHANGELOG.md: the Unreleased Binary64 capability paragraph and a separate
  positional normalization/6 entry, based on actual accepted behavior.
- website/docs/reference/formats.md: preserve the structural report family;
  describe normalization/6 and its retained normalization-target/3 envelope.
- website/docs/guides/generate-artifacts.md: standalone finite codec API and
  source-deserializer limits; explicit fixed-string-array policy and checked
  position reads; old format and whole-system/conformance boundaries.

The concrete sections and their read dependencies were inspected in frozen
Binary64 unit bf16e504ccad68b2ee67607ba39606aadf07f627. Refresh them after that
unit's review/integration. Implementation-specific bindings remain in each
story's disjoint write scope. Only the coordinator changes the three common
files, planning, package versions or release metadata.

## Acceptance

The public descriptions match the two final implementation reports and executed
tests, with no premature release or unsupported-consumer claim. The integration
workspace and site gates pass, complete old normalization output maps remain
qualified, and the resulting commits are published through the organization bot.
No new execution format or primitive is introduced by this documentation task.
The inherited report version is justified by unchanged report fields, digest
inputs and prepared-stage root meanings; recipe format 6 owns the new semantics.

## Scheduling

The companion implementation stories can be scheduled together only after their
Binary64 dependency is implemented and their final source scopes have been
refreshed. The wave page records this serial coordinator-owned publication step.
It is not an omitted collision or authorization for concurrent common-file edits.

## Disposition

The CLI refused a typed scope on this task because only stories own write
surfaces; tasks inherit their parent story surface. The coordinator therefore
records this bounded integration work in story:normalization-followup-publication,
which owns the three shared files and depends on both implementation stories.
This preliminary task is archived as a replaced planning draft. No implementation
or completed gate is claimed by this disposition.
