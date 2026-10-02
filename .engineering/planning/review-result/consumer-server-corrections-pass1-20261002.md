---
format: aep.planning-md/3
id: review-result:consumer-server-corrections-pass1-20261002
kind: review-result
status: active
title: Server correction source attack, no concrete counterexample
relations:
- reviews: task:consumer-server-gate-corrections-20261002
revision: 1
---
unit: server gate corrections; working-tree diff against d414cfc213d8871b04ac18b08c47fee3dc0b71ca
verdict: nothing found
cases: executed 453→not-run, red 0; before is coordinator-reported, review executed 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none; worktree CLI managed review lease metadata
needs-coordinator: resolve outstanding task test-xtask failures before declaring required checks complete

```text
 .../tests/d2_constraint_home_adversary.rs          | 75 ++++++++++++++--------
 .../generate/ess-entity-runtime/tests/lowering.rs  | 32 +++++----
 .../generate/ess-synth/tests/view_params_served.rs | 38 ++++++++++-
 3 files changed, 100 insertions(+), 45 deletions(-)
```

I added no cases, changed no files and ran no tests. The diff above is the implementor’s existing diff; every changed path is a test file. `git diff --check` returned zero.

Read-only attack found no concrete counterexample:

- **Badge assertions exercise the runtime.** `lowering.rs:998` seeds a different previous badge; :1006 requires badge to need no fulfillment; :1012 asserts the resulting stored badge equals the supplied badge; :1014 asserts the emitted badge matches that state; :1017 pins `OnSite`. This follows the actual specification’s independent state/event assignments at `examples/gatepass/domains/visit.yaml:213/:217`. Dropping the assignment or retaining previous state would fail the literal state assertion.
- **Scratch isolation reaches the reported collision.** All Rust/Go harness build directories and copied executables use `scratch`, while `OnceLock` handles same-process reuse. The subprocess regression uses the same label in two live processes, with child removal exercising destructive interference. I read the implementor’s `red-process-isolation.log`: the child ran exactly one test, and the parent failed on `"child"` versus `"parent"` before the fix. This is existing test-runner output, not an independently executed review result.
- **Obligation coverage references a real executable case.** The named function exists under `#[test]` at `examples/gatepass-realization/tests/conformance.rs:895`; it compares the plan and linker, pins the remaining obligation to `RegisterVisit`, and checks that generated behavior is no longer implemented manually. The changed xtask case also compares actual plan/list data, beyond checking the comment’s wording.

The review lease `codex-ess-server-adversary-scope-boolean` was acquired and released. Existing coordinator/implementor leases were untouched.

```findings
[]
```
