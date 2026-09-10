---
format: aep.planning-md/1
id: story:release-gate-green-at-the-tag
kind: story
status: implemented
title: The release gate is green at the tag
owner: claude-release-0-22-1
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json
- confidence: cited
  path: fuzz/Cargo.lock
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 11
---
## Context

The `Release` workflow dispatched for 0.22.0 on 2026-09-10 (run 34483491460) failed its gate: 11
`ess-xtask` tests, reproduced locally at the tag. Nine of them were already red on the PR that
introduced `delivery: at_most_once` (CI run 34475658657), and the release commit carried
`[skip ci]`, so nothing ran at the tag. 0.21.0's `Release` run had failed the same way. Both tags
therefore have no archives, and agentplugins cannot pin either.

## Acceptance

`task check` exits 0 at the commit tag 0.22.1 points at, and that tag's `Release` run publishes
the four archives and `SHA256SUMS`.

## Notes

- `reviewed-schema-metadata.json`: the `wire:RawSpecFile#/definitions` shape digest is re-reviewed
  against the fresh provider; reason and decision page unchanged.
- `where-this-stands.md`: the support block embeds the workspace version, so the bump and the
  `cargo xtask support` re-render land in one commit.
- Not changed here: the tool that writes `chore: release X [skip ci]`; its `[skip ci]` suppresses
  the tag-push `Release` workflow and the version bump it makes does not re-render the block.

## Third cause, found by the local gate

`fuzz/Cargo.lock` pinned the workspace crates at 0.21.0; the 0.22.0 release commit updated only the
root lock, so `task fuzz-check` (`--locked --offline`) refuses at 0.22.0 as well. Updated here with
`cargo update --workspace --offline --manifest-path fuzz/Cargo.toml`.
