---
format: aep.planning-md/3
id: review-result:consumer-312-general-invocations
kind: review-result
status: active
title: Independent review of per-invocation caller synthesis
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

```findings
[]
```

Independent adversary review of the frozen #312 invocation-phase continuation.

Reviewed artifact: managed tree `ess-backlog-initial-state-20261002`, `target/backlog-input/312-general-candidate.patch`.
SHA256: `a4638d2c404f715012290675f851f6720e53c2d9c24b9888bcfb2fbdbaba2bcb`.
Scope: all eleven changed files in that patch, including `tests/caller_fresh_identity.rs`; paths below are relative to `crates/verify/ess-conformance`.
Reviewer test/build execution count: **0**. This review inspected source, the frozen diff, and retained owner evidence; it did not mutate source or rerun compilation.

The arrangement/acting distinction in `src/synthesize/caller.rs` supplies separate caller-rewritten models and stamps credentials at invocation phase boundaries. Already-attributed nested blocks retain their role. The corresponding changes in `src/synthesize.rs` and the existence, related-guard, set-effect and subject-fact planners use arrangement authority for row creation and acting authority for the command under test. In the same-command upsert path, the creating branch is explicitly selected from the arrangement model; the second invocation is attributed to the acting model. No late caller relabeling substitutes for recalculating caller-derived values and guards.

Mixed synthesis does not silently erase an ordinary source outcome when the mixed arrangement cannot witness it: the retained outcome receives an explanation containing the mixed refusal. Successful mixed scenarios preserve their relevant refusals. Reverse-order composition retains the first run's observations, attempts bounded fresh identity witnesses, and records an explicit note when the reverse cannot compose. Global set/count observations are not rewritten to accommodate retained rows. Optional supplied identities participate in redraw while null/generated fallback does not masquerade as a supplied identity. String replacement is now restricted to typed or source-derived value locations, preserving schema field metadata. The finite far/near/guided search and branch-preservation checks remain in place.

`tests/adversary_287_pass1.rs` exercises independent target behavior for caller-derived values, input guards, related guards, optional identities and same-command set effects. The healthy-target and deliberately faulty-target assertions remain substantive. Actual Go, TypeScript and WASM paths compare native outcomes/counts, diagnostic sets and command-call counts. Existing Go/TypeScript runtime changes only add stable diagnostic prefixes to the corresponding failure paths. `tests/caller_fresh_identity.rs` still requires reuse of the recorded identity; its caller comparison now requires the intended opposite caller rather than preserving the former same-caller expectation.

Retained evidence inspected: `target/backlog-input/312-general-regressions-final2.log` records eight test binaries with 26 + 8 + 13 + 6 + 3 + 10 + 14 + 21 = **101 passed, 0 failed**, and its exit file is 0. One stored-field format expectation is filtered in this owner tree; the integrator already migrated that expectation and owns the integrated run. `target/backlog-input/312-general-unit-final2.log` records **4 passed, 0 failed**. `target/backlog-input/312-general-lint-final.log` finishes successfully and its exit file is 0. These are owner executions, not reviewer executions.

This approval covers the exact frozen source patch, not release readiness or an unexecuted integrated gate. The older implementation report names an earlier patch hash; the hash above identifies the reviewed candidate. This report uses managed tree identifiers and repository-relative artifact paths only.
