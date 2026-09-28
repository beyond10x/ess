---
format: aep.planning-md/3
id: review-result:direct-return-current-correction
kind: review-result
status: active
title: Direct-return corrected boundary recheck
relations:
- reviews: story:direct-library-return-observations
revision: 1
---
unit: ESS direct-return port working tree over 70ff257686897669bbb24dda1e8228cca000989a, corrected depth admission
verdict: nothing found after the bounded correction/recheck
cases: executed 21→27, red 0; six reviewer contract cases passed, including the unchanged first-pass failure
origin: introduced 0 / pre-existing 0 / undecided 0 outstanding
wrote-outside-worktree: assigned $TASK_SCRATCH paths only, listed in the separate local scratch manifest
needs-coordinator: retain the initial red report and complete repository/CI gates and publication

`git --no-pager diff --stat` described implementor/coordinator-owned changes: 33 tracked files, 536 insertions and 68 deletions, plus new source/tests/fixtures/evidence and the coordinator-owned planning artifact. Reviewer-attributable repository diff: empty. No source, planning, or Git file was edited by this reviewer; all probes remained in assigned scratch.

The first pass is retained as `$TASK_SCRATCH/report-initial.md`: one new boundary failure, where a valid depth-128 direct-response literal failed original-suite byte admission. The implementor corrected the bounded structural reader. Only an exact suite/28 or /29 direct-response expectation field receives its own depth allowance; every other position retains the previous document bound. Typed decoding bypasses its separate recursion limit only after structural validation. The original failing test was not modified: it now passes against the rebuilt libraries.

All six reviewer contract cases pass in `$TASK_SCRATCH/corrected-probes.log`: depth-128 original-byte admission, response isolation after a second no-input invocation of the same command, exact JSON byte/escaping boundaries, typed map member limits, exact signed integer extremes and adjacent-value refusal, and native Binary64 refusal. The implementor separately reports 19 direct-return tests plus two port tests green. I read the added regression checks for depth-129 refusal and unchanged legacy/non-response depth limits; I did not rerun the full repository gate.

The exploratory authored-YAML depth probe remains unchanged and retained as a transport diagnostic, not a declared contract case. The inherited YAML parser counts scenario-envelope nesting. The corrected design now states this limitation explicitly without reducing runtime/suite response checks or substituting partial assertions. Replacing that parser was outside the accepted correction.

The review also covered source/17 and suite/28–29 format gates, legacy source/16 and suite/26–27 preservation, exact released-byte fixtures, optional/nested presence, finite declaration admission, literal/resource checking, unsupported Go/TypeScript generation, and the absence of a compiled AEP dependency. Expectations are never sent to the target; actual invocation results supply response observations. Historical prototype evidence is labeled historical, not relabeled to the new formats.

Corrected source identities (SHA-256):

```text
ca9508191fbd2dd46201f4dd5ea97932994ccbf638178c85f07ebad7aa4f8dd7  crates/verify/ess-conformance/src/count_json.rs
10a00527fc5e60216d59f8e7200b00d43ee87d057477ee0f1ffd5909e1cf1ff9  crates/verify/ess-conformance/src/admission.rs
68b4070d2067064ccb45d70e579d24f9ec81d3fb4048c823aed8b9b7a441f05a  crates/verify/ess-conformance/src/direct_response.rs
c7981947b2183cf07d389156ffc6ed94c058aa80b16709b847f4822dd43fbda0  crates/verify/ess-conformance/Cargo.toml
```

Reviewer-written retained paths use `$TASK_SCRATCH`, the coordinator-assigned ESS review directory: `probe.rs`, `probe`, `probe-fixed`, `depth128-red.log`, `authored-depth128-red.log`, `remaining-probes.log`, `corrected-probes.log`, `report-initial.md`, and `report.md`. Exact local paths, library artifacts and lease identity are held separately in `scratch-manifest.md`. The reviewer released its own ESS lease on handoff. This is a bounded agent review, not approval or independent verifier evidence.

```findings
[]
```
