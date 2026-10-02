---
format: aep.planning-md/3
id: review-result:consumer-constrained-identity-308-pass1-20261002
kind: review-result
status: active
title: Independent review of constrained identity replay observation
relations:
- reviews: story:feature-request-308
revision: 1
---
## Outcome

Reviewer scope_aggregate found no concrete finding on the four frozen files in b05007e49357031568d92b12b37fa4042c9915c3. All four hashes verified; own executions: 0. Implementor execution is separately attributed: five focused binaries, exact production baseline with final tests 43 passed/3 failed, treatment 46 passed/0 failed/0 ignored; strict all-target Clippy and formatting pass. This is not the final combined package gate.

## Review

Only CompleteSubject admits constrained Newtype under the explicit declaration profile (replay.rs:161-191); RetainedResult remains strict at :117 and nested other constrained forms retain recursive refusal. SubjectShape preserves its serialized fields, ExactShape validation, required identity and row admission (subject.rs:42-109). No legacy-snapshot downgrade or runtime comparator change. Tests preserve descriptor bytes and demonstrate that the prior production reader accepts that structural descriptor; honest replay passes, changed rows fail, malformed original rows stop before retry. Optional/nested wrappers and numeric/recursion/resource/refusal bounds remain explicit. The binding design separates invariant satisfaction from structural preservation.

## Evidence

Synthesis worktree target/backlog-input/308-report.md, 308-frozen.patch, 308-source-sha256.txt, 308-baseline-final.log, 308-focused.log. Review was read-only.

```findings
[]
```
