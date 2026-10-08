---
format: aep.planning-md/3
id: review-result:adversary-wave-20261008e-e1-pass-1
kind: review-result
status: active
title: Adversary pass 1, e1 String-newtype response constraints (wave 2026-10-08e)
relations:
- reviews: story:response-string-newtype-constraints-checked-not-refused
revision: 1
---
## Verdict

NEEDS-CHANGE, no blocker. Cases 15 → 27 on unit e1's lanes; 2 red, both in a held file.

## Findings

| severity | finding | outcome |
|---|---|---|
| should-fix | The model interpreter returns no value for a response String newtype with an upper-bound count invariant (`value.count <= 2`) or `value.count >= 3` with `starts_with`/`ends_with`; its own suite ends `unsupported` (`interpret/response.rs:130` tries only the invariant's literals and a plain String). Newly reached, because the outcome was refused before. | Escalated: `src/interpret/` is held by another wave. Tests `adversary_e1_reachability.rs::the_model_interpreter_*` are committed `#[ignore]` with the reason |
| should-fix | Deleting the payload-observation check in generated Go (`go/response.go:357`) or TypeScript (`ts/response.ts:906`) left the unit's suite green, because every parity case also took the direct-response step. | Fixed: `adversary_e1_payload_mutant.rs` runs suites without the direct step |
| note | `typed_fields::declarations` also serves fixture inputs (`fixtures.rs:74`), so a fixture refusal names a "response type". | No change: the story asks for these exact texts |
| note | The split refusal texts change coverage-suite bytes for reading-attached or non-String-constrained response types. | No change: the split is the story's decision |

## Held

Byte stability (request-only and one-time constraints keep `/34`), 20 parity vectors across native, Go and TypeScript, admission of `/46` by older readers, and the prefix and native payload mutants.
