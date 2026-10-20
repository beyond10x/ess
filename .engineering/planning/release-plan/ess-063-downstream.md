---
format: aep.planning-md/3
id: release-plan:ess-063-downstream
kind: release-plan
status: implemented
title: 'Release ESS 0.63.0: the downstream asks of 2026-10-18 and the salvaged work of 2026-10-13 and 2026-10-17, under ess/27'
summary: One integration branch, one source format, one suite pair, one delta format; waves 1-4 and what follows
relations:
- serves: vision:O2
- depends_on: release-plan:ess-062
- supersedes: release-plan:ess-063
- delivers: task:land-the-schema-tools-wave-branch
- delivers: task:land-the-2026-10-13-port
- delivers: task:land-the-struct-shorthand-unit
- delivers: task:salvage-the-related-rows-quantifier-tree
- delivers: story:alphabet-projects-an-anchored-pattern
- delivers: story:each-entry-moves-the-rows-its-elements-name
- delivers: story:a-committed-suite-is-compact-drift-checked-and-reviewable
- delivers: story:binding-conditions-compare-boolean-event-fields
- delivers: story:optional-value-invariant-observation
- delivers: story:system-issued-command-is-awaited-not-driven
- delivers: story:system-issued-command-await-in-go-and-typescript-runners
- delivers: story:payload-source-reads-the-head-of-an-ordered-selection
- delivers: story:authored-exact-value-on-generated-field-is-refused
- delivers: story:minted-value-present-and-shared-with-its-payload
revision: 4
transitions:
- {from: "draft", to: "implemented", at: "2026-10-20T10:58:49Z", actor: "human:timo", revision: 4}
---
## Intent

Release ESS 0.63.0 from the one integration branch `integrate/ess-downstream-20261018`: the
downstream asks of 2026-10-18 (the in-flight units U1–U6), the finished-but-unshipped work of
2026-10-13 and 2026-10-17 (branches the port branch (head `30de58532`), `integrate/ess-wave-20261017`,
`unit/ess-u594`), the half-done #551 tree, and the two new stories the reconciliation of
2026-10-18 adds (#592, #596). Every open issue from #542 to #596 that this repository's own work
closes is listed below with its vehicle; the four generic language features (#554, #560, #561,
#564) and #584 follow in the next releases under their own waves.

The branch stays PR-ready after each merge: the package gate of the merged change runs on the
merged head, and the branch is merged with `origin/main` whenever main moves.

## Format versions, shared by the whole batch

| format | number | carried by |
|---|---|---|
| source | `ess/27` | U3 (Boolean `==`/`!=` in binding conditions and selections), the port (`system: true` actors and `await_within`, ordered heads, present mints and `{payload: …}` reads, identity-pinned set-effect filters, `payload.` selector operands), #594 (`shorthand:` on a struct), #551 (rows quantifier), #596 (`each:` with `moves:`/`deletes:`) |
| suite | `ess-conformance/52` ordinary, `/53` coverage, cumulative over `/50`,`/51` | U5 (value invariants at wrapped and List positions), the port (`await_outcome`, `expect_not_issued`, `{kind: awaited}`), #572 if it lands before the release closes the pair |
| delta | `ess-diff/20` | the port (`actor/<name>/system-changed`, `command/<name>/await-within-changed`), #594 (`type/<name>/shorthand-changed`) |

One `FORMAT_HISTORY` row 27, one suite catalogue pair, one delta vocabulary: each later merge adds
its items to the existing row rather than allocating a number. The release closes all three; work
that misses it opens the next number. G4's "format A" is `ess/27` only if its wave lands before
that close, otherwise `ess/28`.

## Waves, in order

| wave | unit | closes | vehicle | branch to salvage | adversary |
|---|---|---|---|---|---|
| 0 (in flight) | U1, U2, U3, U5, U6 | #492 gap 1 and the 2026-10-18 report | the integrator's units | — | per unit |
| 1 | `task:land-the-schema-tools-wave-branch` | #590, #591, #593 | `story:schema-tools-accept-projection` (first slice) | `integrate/ess-wave-20261017` | no |
| 1 | `task:land-the-2026-10-13-port` | #542, #543, #544, #545, #546, #547, #549, #552 | nine stories (see the task) | the port branch (head `30de58532`) | yes (one pass over the merged head) |
| 2 | `task:land-the-struct-shorthand-unit` | #594 | `story:json-type-discriminated-union` | `unit/ess-u594` | yes |
| 2 | `story:alphabet-projects-an-anchored-pattern` | #592 | new | — | no |
| 3 | `task:salvage-the-related-rows-quantifier-tree` | #551 | `story:related-rows-quantifier-in-a-selector` | tree `ess-u551-impl` | yes |
| 3 | `story:each-entry-moves-the-rows-its-elements-name` | #596 | new | — | yes |
| 4 | `story:a-committed-suite-is-compact-drift-checked-and-reviewable` | #572 (and the closed #570, #571) | existing | — | yes |

Order inside a wave: wave 1 lands the schema-tools branch first (it sits on `origin/main` and
touches no synthesis file), then the port after U1–U6 (it rewrites the synthesis files they edit).
Wave 2 runs after the schema-tools merge (both regenerate `schemas/generated/ess.schema.json`;
#592 lands alone because it moves committed projections). Wave 3 runs after the port (both units
edit `set_effects.rs` and the row-set synthesis). Wave 4 is independent of the source format and
may run beside wave 3; its suite-format decision is taken at dispatch.

## After this release

| wave | stories | format |
|---|---|---|
| G4 | `story:the-precedence-rule-places-every-branch`, `story:every-guard-combination-validates-under-the-rule`, `story:declaration-order-agrees-with-every-step-pair`, `story:unknown-instance-beside-preserving-and-update-selection`, `story:the-language-policy-is-counted-and-cited`; `story:external-branch-narrowed-by-subject-guard` (#584 option 2) | format A: `ess/27` if open, else `ess/28` |
| G1 | `story:expression-roots-a-branch-establishes-are-readable`, `story:expression-arithmetic-and-time`, `story:digest-and-encoding-functions-in-expressions` (#584 option 1, the chosen answer) | format B |
| G2 | `story:row-queries-navigate-a-reference`, `story:row-queries-quantify-over-a-list-of-references` (#564) | format C |
| G3 | `story:aggregate-measures-over-expressions`, `story:aggregate-group-keys-are-expressions`, `story:derived-measures-and-interval-union` (#561) | format D |

`docs/design/generic-language-features.md` § Delivery is the order; its "format A … `ess/26` if
nothing allocates it first" is overtaken (`ess/26` shipped in 0.62.0).

## Required completion

As `release-plan:ess-062`: one full gate on the final head, one push, one pull request with a
`Fixes` line per issue closed, one release; `aep plan artifact validate` after every store merge;
the stories above moved to `implemented` with the gate as evidence.

## Decisions

## Decisions

- **D1, 2026-10-19 (operator).** The local branches and the stash that
  `review-result:reconcile-20261018-r2` § B lists as shipped or superseded stay until 0.63.0 ships,
  then are deleted. Nothing in this release needs them gone, and each is local-only.
- **D2, 2026-10-19 (operator).** Unit checks are the unit's own tests, the test binaries of the code
  it touches, Clippy on the touched crates and `task fmt-check`. The full gate runs on the
  integration head only, by pushing it to `queue/ess-integrate-20261018`, whose pull request runs `ci.yml` in full:
  after wave 1, after wave 2 and on the final head.
