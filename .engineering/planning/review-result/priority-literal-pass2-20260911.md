---
format: aep.planning-md/1
id: review-result:priority-literal-pass2-20260911
kind: review-result
status: active
title: Literal documentation final adversarial review
relations:
- reviews: story:docs-literal-mapping-claims-unchecked
revision: 1
---
unit: story:docs-literal-mapping-claims-unchecked at eb71c85d634c8e252a7ac380843a87c26aefcba3
verdict: nothing found
cases: executed 8→8 in the deciding docs lane, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 retained scratch files; assigned target/cache metadata and managed lease registry
needs-coordinator: record this final attack and retain the separate source-admission follow-up
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 0515dc2dc02d5ebcfdd77ad7283256461b865a56effd29af5151a014c7cc928a, retained as local-evidence:runtime-gaps/publication-replay/snapshots/0515dc2dc02d5ebcfdd77ad7283256461b865a56effd29af5151a014c7cc928a.md. Source creation recorded at 2026-09-11T00:05:07Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 976315c34a237d31ca125e8792c84070d762f3550076c944b8518d0c78d2700d, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/976315c34a237d31ca125e8792c84070d762f3550076c944b8518d0c78d2700d-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

```text
```

## 1. Diff and scope

`git --no-pager diff --stat` produced the empty output above. `git status --porcelain` was empty and `git diff --check` exited 0. No source or test files changed during this pass. Reviewed the correction diff against `75845afb2ffa71e65c4d781a2f14ca2011baf10d`, its interaction with the full unit diff from `59afcf8caec5230a88aadfd7c590a703e0b5500b`, the correction brief/report and the original finding.

## 2. Deciding cases

No additional case was needed: the correction's boundary matrix already tests both enum and String leaves with Optional-only, newtype-only and mixed chains at 31, 32 and 33 total wrappers. Each enum case additionally checks invalid membership admission on the appropriate side of the boundary. The five first-pass adversary cases and their assertions remain present. The existing original tests cover shallow enum/wrapper membership and String/newtype invariant/external-resource limits.

The brief explicitly permits a bounded correction-review execution of existing deciding cases because source changed since the first attack. No unchanged package baseline was run, and no cosmetic case was added merely to increase counts. The first-pass red records remain at `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-first.log` and `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-docs.log`; this pass produced no new failure.

## 3. Bounded execution

Executed once:

```console
cargo test --offline --locked -p ess-gen --test docs literal_docs
```

Used the assigned literal target with two jobs, development/test debug information disabled, incremental compilation disabled and the prescribed compiler wrapper. The exact local environment is retained privately at `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-pass2-private-inventory.md`.

Verbatim test-runner output:

```text
running 8 tests
test adversary_literal_docs_cannot_be_generated_from_nontext_literal_inputs ... ok
test adversary_literal_docs_do_not_invent_a_representation_for_optional_cycles ... ok
test adversary_literal_docs_preserve_source_enum_names_and_mixed_wrappers ... ok
test adversary_literal_docs_do_not_claim_unchecked_deep_enum_membership ... ok
test binding_literal_docs_report_enum_membership_through_wrappers ... ok
test binding_literal_docs_limit_string_guarantees_to_representation ... ok
test adversary_literal_docs_do_not_claim_unchecked_newtype_enum_membership ... ok
test literal_docs_match_the_checked_representation_depth_boundary ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.24s
```

Exit 0. The before count is the correction implementor's measured `literal_docs` lane count of 8; the current count is the runner's 8. The full docs count of 40 and package count of 222 remain prior recorded evidence, not suites rerun by this pass. Full original output is retained unedited at `local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-pass2-tests.log`, SHA256 `e43ed48122387a270890afd6675fa36b4c4b242fb9b83c7e912b94c1b63fb81e`.

## 4. Findings

Nothing found in this final bounded attack. The first-pass affirmative-membership overclaim did not recur in either reproduction or the broader boundary matrix.

## 5. Reviewed boundaries and limits

- `crates/specify/ess-domain/src/binding.rs:1228` exposes the existing value 32; both the actual validation loop at line 1240 and `crates/generate/ess-gen/src/docs.rs:1721` use that same constant. Each Optional/newtype visit consumes one iteration; recognizing the terminal enum/String consumes another. The parser nesting bound remains separate.
- Source comparison shows the validation function bodies and admission branches unchanged. The only domain changes are comments and public visibility of the existing constant. Boundary tests explicitly retain the existing unchecked admission outside the visit budget; this review does not promote that separately tracked defect into a new documentation requirement.
- The renderer's additional repeated-handle guard only ends a cycle sooner with the conservative fallback; it cannot create an affirmative representation claim.
- The IR diff changes commentary only, qualifying both mapping and payload literal guarantees by actual completion within the shared bound. No serialized field, variant, serialization attribute or compiler resolution behavior changed. This is source-diff evidence; this pass did not independently rerun byte-compatibility suites.
- Existing tests still distinguish source enum identity from wire/display names, preserve shallow invalid-literal refusals, and state String-backed invariant/external-resource limits. The optional-cycle test accepts source refusal as well as conservative rendering, so its passing result alone does not prove that recursive source was admitted.
- No full package, ownership, full-gate, site, release or remote checks ran in this pass. No implementation mutation, new test, planning write or Git-state operation occurred.

No approval or full-release correctness claim is made. Aggregate tokens, tool usage and wall time are unavailable from this role's harness and are not estimated as measured.

## 6. Retained evidence and hand-back

Four new retained scratch files under `local-evidence:ess-evolution-20260910/priority-wave/literal/`: `adversary-pass2-tests.log`, `adversary-pass2-tests.exit`, `adversary-pass2-report.md` and `adversary-pass2-private-inventory.md`. The private inventory records exact local paths, assigned target/cache use and the lease registry effects. The assigned tree and target remain for coordinator cleanup; only this review's lease was acquired and released.

```findings
[]
```