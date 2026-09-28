unit: ESS direct-return port working tree over 70ff257686897669bbb24dda1e8228cca000989a
verdict: NEEDS-CHANGE
cases: executed 19→25, red 1; an additional exploratory source-parser probe is discussed separately
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: assigned $TASK_SCRATCH files only; exact local paths retained in scratch-manifest.md
needs-coordinator: retain this first-pass finding and obtain the implementor correction/recheck

`git --no-pager diff --stat` described the implementor/coordinator-owned working changes: 31 tracked files, 432 insertions and 60 deletions, plus the new direct-response module, tests, fixture/evidence documents and the coordinator's planning artifact. Reviewer-attributable repository diff: empty. No implementation, test, planning or Git metadata file in the source tree was edited by this reviewer. All probes remained in the assigned scratch directory to avoid disturbing source-identity checks.

The initial 19 passing cases are the implementor's retained direct_returns (17) and direct_return_port (2) results, not a preliminary suite run by this reviewer. Six new contract cases were written before execution and linked directly to the implementor's built Rust libraries. No full gate was rerun.

| Location | Category | Severity | Verdict | Origin | Finding |
| --- | --- | --- | --- | --- | --- |
| `crates/verify/ess-conformance/src/direct_response.rs:104` | boundary | blocker | NEEDS-CHANGE | introduced | A depth-128 Json response literal accepted by the new observation validator cannot pass original-suite byte admission because the suite envelope consumes the same nesting allowance. |

Measured: `reviewer_documented_depth128_literal_survives_original_byte_admission` constructs a source/17 command with a Json response, synthesizes its suite/28 scenario, and attaches a literal containing exactly 128 nested arrays. Both `Observation::validate` and `compare` succeed. `AdmittedSuite::from_suite` then returns `InvalidDocument: JSON nesting exceeds 128`; the test exits 101. The exact first execution is retained at `$TASK_SCRATCH/depth128-red.log` and its source at `$TASK_SCRATCH/probe.rs`.

Reached by: the public direct-response observation/standalone-suite APIs that the new design tells adapters to use; exact suite-byte admission is required before running them. This is not a malformed result invented outside the supported profile: the new validator accepts both its schema/literal and its actual value. The existing generic parser limit predates the port, but the new profile exposes that boundary to values it now promises to admit. Minimal correction: carry the new response depth profile through original-byte admission with bounded parsing while preserving existing formats' resource limits and response depth/byte/member checks.

Five other reviewer contract cases passed:

- A second invocation of the same command through `execute_command_without_input` uses its own response; the correct second value passes, a stale first value and missing response fail.
- The 1 MiB actual-response bound counts JSON escaping and field names, accepting the exact boundary and refusing the next byte.
- Typed maps accept 65,536 members and reject 65,537 while both remain below the byte bound.
- Exact i64 extremes and values just beyond binary64's consecutive-integer range survive suite bytes; their adjacent integers fail literal comparison.
- A Binary64 field inserted into a direct-response contract is refused by admission.

An exploratory seventh probe put a 128-deep literal directly in authored YAML. It reached serde_yaml's inherited envelope recursion cap before compilation. The port does not clearly promise replacing that source transport parser, and the coordinator explicitly bounded the required correction to runtime/suite behavior. This diagnostic is retained at `$TASK_SCRATCH/authored-depth128-red.log`; it is not a second merge-blocking finding or a requirement to broaden parsing. Documentation should distinguish source-transport limits from the new runtime response profile. The probe remains unchanged for audit, excluded from the six declared contract cases.

Reviewed source identities (SHA-256), before correction:

```text
68b4070d2067064ccb45d70e579d24f9ec81d3fb4048c823aed8b9b7a441f05a  crates/verify/ess-conformance/src/direct_response.rs
616ea596a8a6757fdce2633c1610adcf8c0edf3262f68edecf66ee6b7e2662e5  crates/verify/ess-conformance/src/selection.rs
fcc322f5e399726277512f4e184fd3f48bc233ebb331ee09fddd9fb97db5b354  crates/verify/ess-conformance/src/admission.rs
25b5b9f2676e96dbd5ddd461480011faf91752bf30e2a1f1a6190e5b871919f3  crates/verify/ess-conformance/src/authored.rs
```

The compatibility review found the format allocation preserved: source/17 and suites/28–29 are new; source/16 and released suites/26–27 retain their prior meaning. The retained released fixtures are checked against fresh generation byte for byte. False `returns` is omitted from existing IR bytes. The direct-return shape and literals come from the model/authored input, and the target supplies actual results without receiving expectations. Go/TypeScript emission explicitly refuses the unsupported observation. Those observations do not resolve the boundary finding above or constitute approval.

```findings
- file: crates/verify/ess-conformance/src/direct_response.rs
  line: 104
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: A depth-128 Json response literal accepted by the new observation validator cannot pass original-suite byte admission because the suite envelope consumes the same nesting allowance.
```
