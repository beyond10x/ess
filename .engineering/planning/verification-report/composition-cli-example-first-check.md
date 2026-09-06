---
format: aep.planning-md/1
id: verification-report:composition-cli-example-first-check
kind: verification-report
status: draft
title: First execution of the composition CLI example
relations:
- verifies: story:review-composition-contract
revision: 1
---
## Measured result

The full Taskfile gate and site-build passed at d565eb0206e0139da9e3ab6ad89aacac3c373927, with all10 recorded lanes exit0 and1948 workspace tests passing. Root then executed the new public compose example with its output base replaced by a fresh assigned scratch directory. The real CLI exited1 before emitting outputs because the parent directory did not exist. This additional check does not invalidate the recorded gate exits, and it prevents source publication until the copyable command is corrected.

## What reaches the failure

website/docs/reference/cli.md's new Composition clients example writes target/composition-example files without first creating that directory. A reader following the block from a fresh checkout reaches the same output-parent preflight. This is introduced example setup, not a production failure or a requested relaxation of destination validation.

## Exact invocation evidence

```json
{
  "head": "d565eb0206e0139da9e3ab6ad89aacac3c373927",
  "argv": [
    "target/debug/ess",
    "specify",
    "compose",
    "--path",
    "crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml",
    "--service",
    "todo=crates/specify/ess-composition/tests/fixtures/two-components",
    "--service",
    "usage=crates/specify/ess-composition/tests/fixtures/two-components",
    "--out",
    "target/review-boundaries-9/composition-cli-example/output/composition.json",
    "--client-plan-out",
    "target/review-boundaries-9/composition-cli-example/output/client-plan.json",
    "--client-rust-out",
    "target/review-boundaries-9/composition-cli-example/output/rust-client"
  ],
  "exit_code": 1,
  "at": "2026-09-06T13:45:59.822585+00:00",
  "binary_sha256": "58736100f486d77e86925ff837a4930b9ce35d09d4ab79e7bc069a5d1696787c"
}
```

```text
error: inspecting output parent /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-9/composition-cli-example/output: No such file or directory (os error 2)
```

## Disposition

The same implementor received a narrow correction brief to add explicit directory setup and execute the corrected command using this exact already-gated binary. Only the public CLI page may change. The first source attack had found no defect and did not execute the CLI; root has recorded this separate command check as verification, not another full attack or conformance evidence. A second independent source attack and changed-input gate verification precede publication.
