---
format: aep.planning-md/3
id: task:land-the-schema-tools-wave-branch
kind: task
status: draft
title: 'Gate and merge integrate/ess-wave-20261017 (#590, #591, #593)'
summary: The finished schema-tools branch lands on the integration branch
refs:
- provider: github
  reference: beyond10x/ess#590
- provider: github
  reference: beyond10x/ess#591
- provider: github
  reference: beyond10x/ess#593
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

Branch `integrate/ess-wave-20261017` (head `7c468d06f`, six commits on `a500f9f56`, which is
`origin/main` at 0.62.0) is gated and merged into `integrate/ess-downstream-20261018`, so #590,
#591 and #593 ship and the branch, its unit branch `unit/ess-schema-tools` and its tree
`ess-wave-20261017` can be retired.

## What the branch holds

| commit | content |
|---|---|
| `5d1a4c85c` | planning: `story:schema-tools-accept-projection` (#590–#593), `story:json-type-discriminated-union` and `decision-blocker:json-type-discriminated-union` (#594, deferred; superseded by `unit/ess-u594`'s cleared version) |
| `fa4bfa4df` | `models/toolchain/domains/generate.yaml`: `PlanProjection` takes a schema id base (#590); the commit body reports the toolchain model and its authored scenarios green on ess 0.61.0 |
| `f8a206904` | `--schema-id-base` on `ess generate --kind schema` (#590), YAML instances in `schema validate` and `validate-bundle` (#591), `x-ess-*` retained by `import-document`/`import-bundle` (#593); CHANGELOG, `website/docs/reference/cli.md`, `website/docs/guides/generate-artifacts.md`, `docs/design/review-schema-resource-identity.md` |
| `f9f9a3cf0` | merge of `unit/ess-schema-tools` |
| `2beb2ae96` | `AGENTS.md`: local builds use a compile cache of their own |
| `7c468d06f` | `crates/edge/ess-cli/tests/support/browser.rs`: a browser startup refusal reads stderr after Firefox stopped (a release-gate flake) |

Status: finished, unshipped. No commit body records the package gates of `f8a206904`; the tree
`ess-wave-20261017` is clean. #592 is not on the branch (second slice,
`story:alphabet-projects-an-anchored-pattern`).

## Steps

1. In a fresh managed worktree on the integration branch: `git merge --no-ff --no-commit
   integrate/ess-wave-20261017` (expected clean: the branch is on `origin/main`; `CHANGELOG.md`
   `[Unreleased]` merges beside the in-flight units' bullets).
2. Gates the change needs (AGENTS.md package-gate table): `cargo test -p ess-gen --test schema
   --locked`, `cargo test -p ess-cli --test schema_registry_identity --locked`, `cargo test -p
   schema-contract --test bundle --locked`, `cargo test -p ess-cli --test <the browser fixture's
   binary> --locked`, clippy `-D warnings` on `ess-gen`, `ess-cli`, `schema-contract`, `task
   fmt-check`; the model change: `ess specify validate --path models/toolchain` and `ess verify
   conform author --path models/toolchain --scenarios models/toolchain-scenarios`, then
   `task projection-check` (the CLI reference is generated from the command definition).
3. Bot commit (`b10x-gates --repository beyond10x/ess bot -- commit -F <file>`), issue numbers
   already in the CHANGELOG bullets.
4. `aep plan artifact validate`: three store files arrive. Move
   `story:schema-tools-accept-projection` draft → proposed → active → implemented for its first
   slice, with `aep plan artifact evidence … --kind test_result --source "<gate command>"` from
   step 2, and name #592's second slice as `story:alphabet-projects-an-anchored-pattern` in its
   body (`aep plan artifact body --section`).
5. Retire: `worktree finish --discard-cache --archive ess-wave-20261017`; delete
   `unit/ess-schema-tools` and `integrate/ess-wave-20261017` after the merge is on the integration
   branch (both local only).

## Done when

The integration branch carries `f8a206904`'s tests green under step 2, `ess generate --kind schema
--schema-id-base https://example.org/schemas/ --path examples/billing` followed by `ess generate
schema validate` accepts the projection as written, and the three store files validate.

## Scope

`crates/edge/ess-cli/src/{main.rs,schema.rs,schema_bundle.rs}`, `crates/generate/ess-gen/src/{schema.rs,types.rs}`,
`crates/generate/schema-contract/src/bundle.rs`, `models/toolchain/domains/generate.yaml`,
`website/docs/reference/cli.md`, `website/docs/guides/generate-artifacts.md`, `CHANGELOG.md`,
`AGENTS.md`, `crates/edge/ess-cli/tests/support/browser.rs`.
