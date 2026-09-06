---
format: aep.planning-md/1
id: review-result:binary64-count-default-gate-harness
kind: review-result
status: active
title: Default gate exposes optional Go compiler override assumption
owner: coordinator
relations:
- reviews: story:model-binary64-fields
revision: 1
---
unit: Binary64/count integration default workspace gate
verdict: request changes
cases: executed 1 passing, 1 failed in the new CLI test target before later targets ran
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: retained gate log and this report in coordinator scratch
needs-coordinator: make the new Go test honor the ordinary toolchain discovery contract

At combined source 12fb11a9225cea39975ad3303edc907046634238 plus the two pass-1 test files, literal `task check` exited 201. Its test subcommand exited 101. The new native CLI test panicked at binary64_count_adversary.rs:99 because ESS_GO_COMPILER was absent. The independent review set that optional override; the default CI gate does not. CI explicitly installs Go on PATH before task check, and existing CLI conformance tests invoke that toolchain.

Exact relevant output:

```text
---- generated_go_binary64_shapes_refuse_before_factory_and_report_publication stdout ----
thread 'generated_go_binary64_shapes_refuse_before_factory_and_report_publication' (1553795) panicked at crates/edge/ess-cli/tests/binary64_count_adversary.rs:99:70:
called `Option::unwrap()` on a `None` value
```

Required correction: preserve an explicit absolute ESS_GO_COMPILER override when present; otherwise invoke Go from PATH, as the existing CLI gate does. Keep native execution and every positive/negative factory/destination assertion. Do not skip the test or merely add the private override to the root gate environment. Re-run the isolated native case with the override absent, then the unchanged literal full gate.

```findings
- file: crates/edge/ess-cli/tests/binary64_count_adversary.rs
  line: 99
  category: correctness
  severity: blocker
  origin: introduced
  message: The new unconditional CLI test requires an environment variable absent from the supported default gate; preserve explicit overrides and otherwise use Go from PATH without skipping native assertions.
```