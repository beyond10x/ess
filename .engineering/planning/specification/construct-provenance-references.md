---
format: aep.planning-md/1
id: specification:construct-provenance-references
kind: specification
status: implemented
title: Construct provenance references
summary: Supported ESS constructs retain typed external records explaining why they exist.
owner: ess
tags:
- github-issue-migration
refs:
- provider: github
  reference: beyond10x/ess#3
revision: 5
---
# Specification: Construct provenance references

## Context

This record migrates `beyond10x/ess` GitHub issue 3, “A command, event or binding has nowhere to record why it exists.” The issue was opened by `timofriedlberlin` on 2026-09-02 at 14:13:07Z, remained open with no labels, assignees, milestone, or comments, and proposed both free-form `because` prose and structured external references on commands, events, errors, and bindings.

No implementation was added during this migration. Commit `9f13631d66583b56b08ce9bdeff01042842e5ade`, released in ESS 0.4.0 and every later release through 0.9.2, resolved the underlying provenance problem with a deliberately narrower contract: structured `provider:key` references on commands, outcomes, bindings, and components. It explicitly rejected a general `because` field because prose beside the external record can go stale. `crates/ess-domain/src/refs.rs:1-31` owns that decision and spelling; the typed fields are visible at `crates/ess-domain/src/binding.rs:166-170`, `crates/ess-domain/src/command.rs:793-797`, `:922-926`, `:1739-1743`, and `:1810-1814`.

The original proposal is preserved below, including the broader event/error and `because` requests. Those parts are recorded as deliberately outside the shipped resolution, not silently reported as implemented.

## Requirements

1. A supported ESS construct can name the external record that explains why it exists using a typed `provider:key` reference.
2. Commands, outcomes, bindings, and components accept an ordered `refs` collection.
3. References survive validation and compilation into resolved IR and reach `ess generate --kind docs`.
4. An empty reference collection emits no `refs` key.
5. A URL is refused where a `provider:key` reference is required; URL expansion belongs to a project-level provider map.
6. Strict readers continue to deny unknown fields.

## Constraints

- ESS has no AEP dependency; the two systems share only the `provider:key` spelling.
- References are typed values rather than arbitrary JSON or a generic extension bag.
- The persisted ESS format changes only through an explicit format decision when compatibility rules require one.

## Out of Scope

- A general free-form `because` field on commands, outcomes, events, errors, bindings, or components. Commit `9f13631` rejects this in favor of the external record as the durable explanation.
- `refs` on `RawEventSpec` or `RawErrorSpec`; the released implementation does not claim those surfaces (`crates/ess-domain/src/command.rs:1817-1842`).
- Embedding tracker URLs in each construct.
- Resolving provider keys to URLs inside the documentation projection.

## Invariants

- A reference round-trips without losing its provider or key.
- The provider and key are non-empty and retain their literal spelling.
- A reference is never mistaken for a URL.
- All projections that claim construct provenance derive it from the same resolved typed reference.

## Acceptance Criteria

- `jira:DEV-630` parses and round-trips as one external reference.
- malformed references and URL-shaped values are refused by name.
- A specification containing references on supported constructs compiles and its generated documentation includes them.
- `task check` passes at the migrated repository revision.

## Resolution

Implemented before migration by `9f13631d66583b56b08ce9bdeff01042842e5ade` as the narrower structured-reference contract above and included in the published 0.4.0 release. This migration records what was built and the explicit decision not to build the broader prose field; it does not pretend that every field proposed by the original issue shipped.

## Ambiguities

- `inferable` — the external-reference syntax and URL boundary are stated in `crates/ess-domain/src/refs.rs:15-29`.
- `inferable` — the excluded `because` field is a deliberate decision in the implementation commit message, not an omission inferred by this migration.
- `inferable` — the behavior is released: `git tag --contains 9f13631` includes 0.4.0 through 0.9.2.

## Open Questions

None for the implemented specification. Adding provenance to events or errors would be new work and requires a new artifact rather than widening this completed record after migration.

## Original GitHub Issue

Source: `https://github.com/beyond10x/ess/issues/3`

> `Conversion` requires `because` — prose stating why a crossing is allowed — and the reasoning in
> `crates/ess-domain/src/types.rs:738-742` is that a conversion with no reason is "a silent widening
> that someone added to make a build pass".
>
> That reasoning applies to a command and a binding at least as strongly, and neither has anywhere to
> put it. `RawCommandSpec`, `RawOutcome`, `RawEventSpec` and the binding types are all
> `#[serde(deny_unknown_fields)]`, so there is no key to write into and no `extra` to fall through to.
>
> ### What this costs, concretely
>
> Modelling the ACD v3 → backend boundary in `sbf/acd` produced three bindings, each declaring
> `delivery: at_least_once` and an `on_failure` escalation. Those declarations exist *because* the v1
> event path silently drops every event other than two, which was found by reading
> `adapters/backendsvc/svc.go:165`. That sentence is the most valuable thing about the binding and
> there is nowhere in the model to write it — it lives in a hand-written `README.md` beside the spec,
> where nothing checks it and `ess generate` cannot reach it.
>
> The same is true of a ticket id. A command that exists because of `DEV-630` cannot say so.
>
> ### What would fix it
>
> Two fields, both optional, on commands, events, errors and bindings:
>
> - `because:` — free prose, the same shape `Conversion` already has.
> - `refs:` — external references, `provider:key`. AEP added exactly this to its planning artifacts in
>   0.41.0 (`{provider: jira, reference: DEV-630}`, with the URL pattern configured once per project
>   rather than written per artifact), and the two models would then agree on how a ticket is named.
>
> Both should reach `ess generate --kind docs`, since documenting the contract is what the generated
> docs are for.

## Provenance

Migrated from GitHub issue `beyond10x/ess#3`.

- Created 2026-09-02T14:13:07Z · last updated 2026-09-02T14:13:07Z
- Original state: `OPEN`
- Original author: `timofriedlberlin`
- Original issue body: 31 lines, 22 nonblank
- Original labels, assignees, milestone, comments, and issue relations: none
- AEP reference: `github:beyond10x/ess#3`
