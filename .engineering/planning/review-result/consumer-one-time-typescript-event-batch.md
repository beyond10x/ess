---
format: aep.planning-md/3
id: review-result:consumer-one-time-typescript-event-batch
kind: review-result
status: active
title: Independent TypeScript event batch correction review
relations:
- reviews: story:feature-request-389
revision: 1
---
approve
unit: story:feature-request-389 — TypeScript ordinary event resource correction
verdict: approve
cases: executed 0→0, red 0; source rereview only
origin: introduced finding resolved
wrote-outside-worktree: none
needs-coordinator: yes — integrate and complete combined validation

Reviewed the entire two-file correction against c0e02bdeaa8b1f703dfbe2e3ebdcfc7cee79f1de. Patch SHA-256 7371997e2e6d7dd6f691d16e9bb7a096a2b5b18991c9beae2c47abf1fa32c78b and report ad7a14954fab9b16b68bd1ec3261c3bee7360c899f9068b6356709e3bc20b303; both frozen source hashes match. Own production/test edits: none; own executions: 0.

Both logCount and eventuallyEvent now call disclosureMaps on the complete returned payload array before remembering or counting any item, and immediately propagate refusal. logCount's undefined result causes countBefore to stop before the command. This resolves the aggregate resource-budget finding in the prior seven-path review without weakening ordinary unprotected behavior.

The two regression cases use admitted protected suite bytes, execute real shared Service callbacks through the emitted TypeScript runtime, and separately inject a one-off adapter batch on the ordinary callback. Two 500,000-byte payloads must pass; two 600,000-byte payloads must be Unsupported/TARGET. Later empty scans cannot hide the omission. The denial adapter control is correctly labeled separate from the shared native manifest. All five counts, total, report binding, code, exit and nondisclosure are checked. Author evidence reports final-test-byte red 0/2, treatment 28 dedicated plus 4 runtime tests green, and strict Clippy green; these are not reviewer executions.

No further concrete finding in this correction. The original seven-path scope review remains applicable, with its one blocker resolved. Combined package gates, source-generated feature inventory and final delivery remain coordinator work.

```findings
[]
```
