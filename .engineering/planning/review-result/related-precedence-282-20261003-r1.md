---
format: aep.planning-md/3
id: review-result:related-precedence-282-20261003-r1
kind: review-result
status: active
title: Related-refusal precedence implementation independent review pass 1
relations:
- reviews: story:feature-request-282
revision: 1
---
needs-revision

# #282 related-refusal / wrong-state precedence — independent whole review pass 1

Candidate `226af8bfeac6db007c8e9da9ee90005346acdd89` has one blocking correctness defect and one known formatting defect.

The new source-22 held-state preflight drops a valid open-provider alternative. `Externals::Open` requires an eligible external outcome declared before a later selected branch to remain possible beside it (`crates/verify/ess-conformance/src/interpret/execute.rs:23-25,646-660,781-808`). The new preflight returns one transition immediately (`:552-568`): it skips an earlier nonmoving external outcome, discovers that the later moving fallback is in the wrong state, and returns only `wrong_state` (`:620-643,829-864`). The independent prospective regression expected `{provider-declined, wrong-state}` and received `{wrong-state}`. Preserve each open-provider alternative while applying the lifecycle answer only to the invalid moving alternative, and retain the regression.

The candidate also fails repository `cargo fmt` on import order and one `map_err` wrap in `crates/verify/ess-conformance/src/interpret/execute/history/values.rs:1,257`. These are mechanical and behavior-neutral, but the frozen candidate is not formatting-green.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/interpret/execute.rs",
    "line": 566,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the ess/22 held-state preflight returns one wrong_state transition and discards an earlier eligible Externals::Open alternative, although the interpreter contract requires that external outcome to remain possible beside the later selected branch"
  },
  {
    "file": "crates/verify/ess-conformance/src/interpret/execute/history/values.rs",
    "line": 1,
    "category": "formatting",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the frozen candidate fails the repository cargo fmt check on import order and one map_err wrap"
  }
]
```

Independent execution used Rust 1.98.1, one job, debug/incremental disabled, dev/test stripping, locked/offline mode, an outside-repository temp directory, and the review tree's own target. Results:

- prospective open-external + wrong-state regression: exit 101; 0 passed, 1 failed;
- accepted #282 wrong-state-before-related control: exit 0; 1 passed;
- pre-existing open-external authority control: exit 0; 1 passed.

The prospective patch SHA-256 is `f0c54df35f7cbd28f46858785b3aab8d36d45542c49594bdf40b6ece47c2ad17`; its red log is `df454b8b6acf519a99cc4bb99547346053c758e5417386416cd0bbe2a0927bb5`. The two control logs are `edfc51ebf352fe01be8f7cebc73380ff41d44b799e75ab8be4329a6ffca02205` and `7237b3e98dcb18f151e4531e22f6f4c1d6e24c0d7eb10dcf3478a1f3211b0da8`.

Author evidence, inspected rather than claimed as independent compilation, had all 12 focused #282 tests green and strict affected all-target Clippy green. The refreshed formatter check was red only for the two cited mechanical lines. A full three-package attempt exited 101 with 31 targets / 166 failures inventoried as unrelated generated-suite/version baseline drift; it is not claimed as clean candidate evidence.

Static review found the remaining allocation and ordering sound: source 22 admission only for the actual present-related-refusal + `wrong_state` composition, older formats refused, no-`wrong_state` behavior retained, multiple selected related refusals still ambiguous, real related-row synthesis retained, and no widening into successor `via` or generated-guard stories. The four final history-proof lint edits preserve generated fixture bytes by direct construction equivalence.

The candidate HEAD remains unchanged. The review tree contains only the preserved prospective Rust test patch, `git diff --check` is green, all reviewer compiler processes are terminal, and the compiler lane has been handed back.
