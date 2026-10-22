---
format: aep.planning-md/3
id: review-result:release-0680-adversary-pass-1
kind: review-result
status: active
title: ESS 0.68.0 release diff, adversary pass 1
summary: One adversarial pass over origin/main...e6b9141a1; no blocker, three findings filed as draft stories.
relations:
- reviews: story:an-existing-instance-outcome-beside-related-guards-is-witnessed
revision: 1
---
findings

One adversarial pass over the whole 0.68.0 diff (`origin/main...e6b9141a1` plus the uncommitted
release edits), 2026-10-22. No blocker.

What was attacked with no finding: the #623 plain-race refusal at ess/29 (0.67.0 and 0.68.0 give
the same refusals and byte-identical suites; only the message text changed); #560 determinism
(synthesis uses a fixed reference instant; a verdict could flip only past 3600 s between write and
read); the auto-merged `synthesize.rs` at 47ac058ee (coherent); the existing-instance steering (only
the ess/3 toolchain digest moved, explained by `specify.yaml` in the diff); the served #509 faulty
control and AddPort without a node; coalesce and the generated-value pin below ess/30 (still
refused, the pin as ESS-AUTHOR-042); the `WrittenSource` deserializer and schema.

Runs: `explore_guards_first` 3/3, `ess-synth enforced_reference_served` 2/2,
`adversary_e1_reachability` 8/9 (TS2688 with `ESS_TYPES_NODE` unset, the documented environment
case).

The red tests are kept outside the repository with the coordinator; each draft story states the
failing case in its summary.

```findings
[{"file": "crates/verify/ess-conformance/src/interpret/response.rs", "category": "coverage", "severity": "note", "origin": "introduced", "message": "TextRules::candidates never overlaps a prefix and a suffix, so count <= 3, starts_with AB, ends_with BC (witness ABC) is Unsupported; never a wrong verdict. Owner: story:response-newtype-witness-lets-prefix-and-suffix-overlap"}, {"file": "crates/verify/ess-conformance/src/synthesize.rs", "category": "coverage", "severity": "warning", "origin": "pre-existing", "message": "The wrong-state family skips a command with a subject guard: StateBesideWrongState/not-held gets no scenario and no refusal at ess/27 and ess/30. Owner: story:wrong-state-branch-beside-a-subject-guard-is-witnessed"}, {"file": "crates/verify/ess-conformance/src/synthesize.rs", "category": "diagnostic", "severity": "note", "origin": "pre-existing", "message": "An unknown-port scenario is refused as ESS-SYNTH-008, the internal-drift code; 0.67.0 prints the same. Owner: story:unknown-port-refusal-names-its-own-code"}, {"file": "crates/specify/ess-domain/src/system.rs", "category": "documentation", "severity": "note", "origin": "introduced", "message": "The ess/30 FORMAT_HISTORY row lacked #560; fixed by the release commit"}]
```
