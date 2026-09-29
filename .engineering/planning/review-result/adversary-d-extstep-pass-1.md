---
format: aep.planning-md/3
id: review-result:adversary-d-extstep-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit extstep
relations:
- reviews: story:authored-external-steps-state-their-answer
revision: 1
---
unit: story:authored-external-steps-state-their-answer (beyond10x/ess#243), uncommitted working tree over 2ae8d6bda at ~/.local/state/worktree/trees/b10x/ess/ess-f-extstep
verdict: NEEDS-CHANGE
cases: executed 1768→1772, red 4
origin: introduced 0 / pre-existing 0 / undecided 4
wrote-outside-worktree: 3 paths (listed in part 6)
needs-coordinator: none

## 1. git --no-pager diff --stat

```
 .../ess-conformance/assets/coverage-admission.js   |   2 +-
 crates/verify/ess-conformance/src/authored.rs      | 159 ++++++++++++++++++++-
 crates/verify/ess-conformance/src/coverage.rs      |   2 +-
 .../verify/ess-conformance/src/coverage_build.rs   |   3 +-
 crates/verify/ess-conformance/src/go/runtime.go    |   5 +-
 crates/verify/ess-conformance/src/ts/runtime.ts    |   5 +-
 crates/verify/ess-conformance/tests/authored.rs    |  14 +-
 website/docs/guides/verify-conformance.md          |  19 +++
 8 files changed, 201 insertions(+), 8 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_extstep_pass1.rs   (mine, untracked)
```

This is the implementor's diff, unchanged. The only path I added is the untracked test file. No implementation file was touched.

## 2. Cases added: crates/verify/ess-conformance/tests/adversary_extstep_pass1.rs

Every case checks the story's acceptance directly. It passes only if the act is refused with ESS-AUTHOR-037, or if it compiles to a scenario that the reference target passes (for the courier fixture: configured with `SendMail/queued`). All 4 were red on their first run (`cargo test -p ess-conformance --test adversary_extstep_pass1`, EXIT=101):

| # | test | claim the act makes | red line (verbatim) |
|---|---|---|---|
| B1 | `an_escalation_only_a_downstream_external_failure_publishes_is_refused_or_armed` | `CreateInvoice`, `outcome: accepted`, `events: [billing.email.DeliveryEscalated]` | `compiled without ESS-AUTHOR-037, with external answers configured [], and the reference target cannot satisfy it` … `CheckResult { code: Event, about: "event billing.email.DeliveryEscalated", status: Failed` |
| B2 | `an_escalation_claim_on_an_act_with_no_outcome_is_refused_or_armed` | same act, no `outcome:` (so `unstated_external` does run) | same line, `event billing.email.DeliveryEscalated`, `status: Failed` |
| B3 | `a_no_events_claim_only_an_external_branch_satisfies_is_refused_or_armed` | `SendEmail`, no `outcome:`, `no_events: [billing.email.EmailSent]` | `compiled without ESS-AUTHOR-037, with external answers configured []` … `CheckResult { code: NoEvent, about: "no event billing.email.EmailSent", status: Failed` |
| W1 | `an_event_decided_only_by_a_command_the_act_never_reaches_is_still_refused` | `SendMail`, no `outcome:`, `events: [MailQueued]`, where an unrelated `QueueMail` emits `MailQueued` on a decided branch | `an act on SendMail claiming MailQueued, which only SendMail/queued (external) can publish for it, compiled with refusals [] and configured []` |

## 3. Suite run (after the cases existed)

`CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-f-extstep CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p ess-conformance --no-fail-fast` → EXIT=101

```
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
error: 1 target failed:
    `-p ess-conformance --test adversary_extstep_pass1`
```

Totals: 1768 passed and 4 failed. The implementor's `conformance2.log` has 1768 passed and 0 failed, which gives `<before>` = 1768. `cargo clippy -p ess-conformance --test adversary_extstep_pass1 -- -D warnings` is clean. `cargo xtask generate --check` printed `projections are up to date`.

## 4. Findings

**Blockers:**

- **B1 and B2: an escalation reached only through a downstream external failure is silently accepted.**
  - What was measured: `src/authored.rs:2052-2064` treats any binding's `escalation` event as "decided elsewhere". But in billing, `DeliveryEscalated` is published only when the `SendEmail` call made by `notify-on-invoice-created` answers its `external:` branch `failed`. Synthesis reaches that event only by arming `ConfigureExternalOutcome` (`synthesize.rs` `on_failure`).
  - `src/authored.rs:1939` also skips the check completely whenever `outcome:` is written (B1).
  - Result: the act compiles with no configure step, and the reference target fails the scenario.
  - The authored format has no way to state the answer for a command that a binding invokes, because `outcome:` names only the act's own command.
  - Fix, named but not applied: refuse with 037, naming `SendEmail/failed`, when a claimed event is an escalation (or is emitted only by external branches of commands downstream of the act). Run the check whatever the act's `outcome:` is. The alternative is a format key that states a downstream answer.
  - What reaches it: `examples/billing/components.yaml:63-68`. Its comment says a scenario should "force `SendEmail` to fail and then assert `billing.email.DeliveryEscalated`", so this is the documented use.
- **B3: `no_events:` is never read, which is a silent drop (defect class 2).**
  - `unstated_external` reads only `error:` and `events:`.
  - `SendEmail`'s only branch decided by the input, `sent`, emits `EmailSent`. So an act with no `outcome:` that claims `no_events: [EmailSent]` can hold only on `failed`. It compiles unarmed and fails on the reference target.
  - What reaches it: an author asserting that "the mail was not sent" without writing `outcome: failed`. This is the same authoring slip the issue describes, spelled as a negative claim.

**Warning:**

- **W1: the event check is global over all commands (`src/authored.rs:2052-2059`).**
  - Any decided branch of any command that emits the event exempts the claim, even when the act cannot reach that command. The claim then compiles unarmed.
  - What reaches it: a model in which two commands publish one event. That is plausible, but I found no example in `examples/` that does it, so this finding rests on a fixture I built.

**Notes (no failing case written):**

- **N1: the arming can be taken by the wrong call.** The step is armed immediately before `ExecuteCommand`. A binding from an earlier act that invokes the same command asynchronously (billing: `CreateInvoice` → `SendEmail`, `at_least_once`) can consume the armed `SendEmail/failed` answer on a target that delivers asynchronously. The reference target is synchronous, so no red case is possible here (INFEASIBLE locally). Synthesis carries the same hazard.
- **N2: the story's acceptance says "refused by `ess author`".** No test drives the `ess author` CLI. All tests call `authored::compile` directly.

## 5. Attacked and could not break

- Several external branches on one command: each gets its own control, the forced branch is the one written, and an accepting act between them gets no control. The implementor's test covers this, and I read it against the code.
- Order of the configure step: it comes after windows, input and arrange steps and immediately before `ExecuteCommand`. Nothing between them spends it.
- `times`: None matches synthesis. It is not used for acts on a bounded-retry binding, because an act invokes directly, not through a retry.
- An `external:` branch with a `when:` guard: its test strategy is `InjectFault` (`ess-domain command.rs:567`), so it is armed.
- Go and TS runtimes, `coverage-admission.js` and `coverage.rs`: all four agree on 1..=37 without 36. No schema regex restates the range.
- Bytes for models without external acts: `cargo xtask generate --check` is clean.
- Loops or repeated steps: the authored format has none (`Act` has no repeat key). Repeated acts are covered by the six-act case.

## 6. Paths written outside the worktree

- `~/.cache/ess-wave-n2/extstep/adv1/red.log`
- `~/.cache/ess-wave-n2/extstep/adv1/suite.log`
- `~/.cache/ess-wave-n2/extstep/adv1/review.md`
- Build output went to the assigned `~/.cache/b10x-target/ess-f-extstep`. I added no new directory.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/authored.rs
  line: 2061
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: a binding escalation published only on a downstream external failure counts as decided, so an act claiming DeliveryEscalated compiles unarmed and the reference target fails it (tests an_escalation_*_is_refused_or_armed)
- file: crates/verify/ess-conformance/src/authored.rs
  line: 1939
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: the external-only claim check is skipped whenever outcome is written, so an act naming a decided outcome that claims a downstream-external event is neither refused nor armed
- file: crates/verify/ess-conformance/src/authored.rs
  line: 2013
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: unstated_external never reads no_events, so an act claiming the accepting branch's event absent expects the external answer without stating it and compiles unarmed (a_no_events_claim_only_an_external_branch_satisfies_is_refused_or_armed)
- file: crates/verify/ess-conformance/src/authored.rs
  line: 2052
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: an event emitted by a decided branch of any command, even one the act never reaches, exempts the claim, so an external-only claim on the act's own command compiles unarmed
```
