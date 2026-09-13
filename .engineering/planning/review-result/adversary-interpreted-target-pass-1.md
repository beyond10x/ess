---
format: aep.planning-md/1
id: review-result:adversary-interpreted-target-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: a generated support row and an over-strong module claim'
summary: 'Red: 4 introduced, 1 blocker on a gate the package-scoped suite never runs'
relations:
- reviews: story:interpreted-target-selection
revision: 1
---
# Adversary pass 1 — story:interpreted-target-selection

Verdict **red**. Cases executed 933 → 935, 2 red. Origin: introduced 4, pre-existing 0, undecided 0.

## Attacked and could not break

No `Status::Error` on any constructible path: `begin_scenario` refuses first and `runner.rs:394-410`
skips the steps and `end_scenario`, so every non-empty suite yields one `unsupported` per scenario —
measured at 29 unsupported exit 1 from `--path`, 30 unsupported exit 1 from the committed suite, and
30 unsupported exit 1 under `--strict`. All 139 lines of `interpret.rs` read: every method returns
`TargetError::unsupported`, so the target answers no command, view, event, binding or elapsed claim.
The 8 defaulted trait methods all return `unsupported` too, so none agrees with anything silently.
No `HashMap`, `SystemTime`, `Instant::now`, `rand::` or `thread_rng`. All 14 classification ids match
the sibling naming exactly, and no `impl<Interpreted;Default>` entry is needed because derived impls
do not appear in the inventory. `conform run` is the only subcommand taking `--target`.

`task consumer-check` could not be executed — it pins `RUSTUP_TOOLCHAIN=1.98.1`, which is not
installed on this machine. The classification check above is static only.

```findings
- file: website/docs/status/where-this-stands.md
  line: 68
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: "the Conformance targets row is generated from `conform run --help` by ess-xtask/src/support.rs:334 and compared back by `cargo xtask support --check`, which `task check` runs, so adding the `interpreted` value without updating the row breaks a gate the unit's package-scoped suite never executes."
- file: crates/verify/ess-conformance/src/interpret.rs
  line: 25
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the module doc claims a run against this target can never be green and always exits non-zero, but a zero-scenario admitted suite — reachable through the `[]` selection workflow documented at website/docs/guides/verify-conformance.md:238 — reports `'status': 'passed'` under `'implementation': {'name': 'interpreted'}` and exits 0."
- file: website/docs/guides/verify-conformance.md
  line: 146
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the published guide still says the built-in choices are `billing` and `oracle-fixture` while the CLI now offers three, and nothing machine-checks this sentence."
- file: crates/verify/ess-conformance/src/interpret.rs
  line: 51
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the IDENTITY doc states as a general convention that a report's implementation name is the `--target` value that selected it, which is true only of this target — `billing` reports `billing-reference` and `oracle-fixture` reports `oracle-reference`."
```
