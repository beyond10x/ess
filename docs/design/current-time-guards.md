# A Timestamp guard compared with the current time

beyond10x/ess#171. Source format `ess/16`; suites carrying the value it needs take
`ess-conformance/26` (ordinary) and `/27` (coverage).

## The gap

```yaml
- name: demo.jobs.ScheduleJob
  input: [{name: starts_at, type: Timestamp}]
  outcomes:
    - {name: start-in-past, when: starts_at < now - 60s, error: demo.jobs.StartInPast}
    - {name: scheduled, creates: demo.jobs.Job, instance: job_id, sets: {starts_at: input.starts_at}}
```

The service refuses a start more than a minute before the moment it handles the request. A
`Timestamp` guard could be ordered only against a fixed RFC 3339 literal, so `starts_at < now - 60s`
was refused as `type_mismatch`, and a fixed literal states a rule that goes false as time passes.

## The construct

On the right of `<`, `<=`, `>` or `>=` over a `Timestamp` (through newtypes and `Optional`), in a
command outcome's input guard — a plain `when:`, or the `when:` beside `when_subject_state:`,
`when_state_changes:`, `when_subject:` or an external cause — the literal may be the
**current-time operand**:

| spelling | instant |
|---|---|
| `now` | the moment the implementation selects the outcome |
| `now - 60s`, `now + 5m`, `now - 1h` | that moment moved by a whole number of seconds, minutes or hours |

The offset has no leading zero, spaces around its sign are optional, and it is at most one hundred
years (`ess_primitives::time::CurrentTime::MAX_OFFSET_SECONDS`). There are no days: a day is not
always 24 hours, and the operand reads nothing but a clock and a number of seconds. The IR carries
the predicate as written: the operand is a text literal, recognised by the type it is compared with,
so no IR shape changes and a document without it keeps its bytes.

| written | answer |
|---|---|
| below `ess/16` in a `when:` | `unsupported_format_version`, naming `ess/16` |
| an entity or type invariant, a view filter, a `when_subject:` predicate over stored fields, a selection | `type_mismatch`: those are not the predicate over a request's input read while it is handled |
| `now - 60`, `now - 1d`, `now - 060s` | `type_mismatch`, naming the spellings |
| `==` or `!=` against `now` from `ess/16` | `type_mismatch`: an instant is ordered against now, never equated with it |
| `label == now` over a `String` | unchanged: the four characters |

Below `ess/16` an equality with the word `now` keeps the meaning it had, the text `now`.

## Where `now` comes from

`timestamp-clock-provenance.md` infers no current-time fallback for a *reading*: an observed clock
value is never replaced by the time of the observer. That stands. The current-time operand is not a
reading and not a fallback; it is a clock the evaluator is **given**:

- **Rust evaluator.** `FactSource::now` answers the current time, `None` by default.
  `Predicate::evaluate` reads the operand against it, for a literal only — a caller who sends the
  text `now` sent no instant — and a source with no clock leaves the comparison `Unknown`, as it
  left any text that is no instant. `ess_primitives::predicate::WithNow` wraps a source with a
  clock the caller read.
- **Synthesis** decides every candidate against one fixed reference instant,
  `2019-12-30T23:59:59Z` (`ess_conformance::now_offset::reference`), and reads no clock.
- **The runner** resolves the value synthesis wrote (below) from its **wall clock**,
  `Clock::wall`. `ess-conformance` reads no clock of the machine's (`tests/suite.rs` scans its
  sources for one), so the wall defaults to `Clock::now`, the runner's own measure of budgets,
  which `AdvancingClock` starts in 2023. A caller running such a suite against an implementation
  that reads the machine's clock hands the runner that clock:
  `Runner::new(config, now_offset::WithWall::new(AdvancingClock::default(), machine_clock), ids)`.
  Without it the instants sent are the runner's, and the accepting witnesses fail. **The library
  default is deterministic; the CLI supplies the wall clock:** `ess verify conform run` builds its
  runner that way for every `--target` (`wall_clock_runner` in `crates/edge/ess-cli/src/main.rs`,
  reading `SystemTime`), so budgets and report durations stay on the deterministic clock and only
  `now_offset` values read the machine's.
- **The conformance target is told nothing new.** It receives an RFC 3339 instant like any other
  and decides the guard by its own clock.

## What synthesis does

A guard over the current time names no instant a suite could carry. So:

1. **Witnesses a second from each boundary, never on it.** `now - 60s` is tried at the reference
   less 59 and less 61 seconds; the operand's text is never tried. Boundary rows
   (`boundary_inputs`) move the same way: the accepting row a second inside the bound, the
   refuting row a second outside. For #171 that is `now - 61s` requiring `start-in-past` and its
   error, and `now - 59s` requiring `scheduled`. The boundary itself is what a latency flips — an
   input sent at `now - 60s` is refused by any target that reads its clock a moment later — so it
   is never sent, and the mutant that moves the boundary by less than a second is not caught.
2. **A value chosen for a now-guarded field is written as a `now_offset`.** `{kind: now_offset,
   seconds: -61}` is the candidate's distance from the reference. It replaces the literal in every
   `execute_command` input for that command, and it travels through `sets:` into the row the
   scenario reads back, so the view is required to hold what was sent. The event payload does not
   assert the field's value, only its shape: an `expect_event` payload carries literal nodes.
   An input no guard moves gets the plain witness, a day and a second after the reference
   (`now + 86401s`), clear of every boundary written in whole minutes or hours.
   **A field also ordered against a fixed instant** (`starts_at > '2030-01-01T00:00:00Z'` beside
   `starts_at < now - 60s`) keeps the values chosen from that instant's boundary as literal
   instants: a candidate at, or a second either side of, a fixed bound is that bound's, unless it
   is also a second from a `now` boundary (`now_offset::Orderings::chosen_from_now`). An equality
   with a fixed instant (`starts_at == '2024-06-01T00:00:00Z'`) counts as such a bound. Each value
   is decided once, at the reference, so it holds at a run only when the run falls on the same side
   of every fixed instant as the reference does. A fixed bound before the reference
   (`2019-12-30T23:59:59Z`) is on the same side at every run. An ordering against a fixed instant
   after the reference and not after `2026-09-27T00:00:00Z`, the day the operand was implemented
   and so no later than any run (`now_offset::earliest_run`), is on the other side at every run: a
   field ordered against one beside `now` is refused by name (a `NoWitness` naming the field, the
   instant and `now`), and every scenario sending the command is dropped. Two cases stay open. A
   fixed bound after that day (`2030-01-01`) is decided correctly until a run passes it. An
   equality in the refused window is not refused, and its literal witness is decided the other way
   when an earlier outcome's `now` guard reads it first.
3. **A now-ordered path a `now_offset` cannot carry is refused by name** (a `NoWitness` synthesis
   refusal naming the path and `now_offset`), and every scenario sending the command is dropped:
   a `Timestamp` inside a structure (`window.starts_at < now`), and an element a quantifier binds
   (`exists: {in: starts, as: s, that: s > now + 5m}`). A `now_offset` replaces a whole input
   field, and nothing carries one inside a literal or a list. `ess-domain` admits both, as it does
   the same guard against a fixed instant; the refusal is synthesis's, as for the structure.

## What the runner does

`now_offset` values are resolved per scenario. When a step first names a number of seconds, the
runner reads its wall clock once for that step, rounds **up** to the next whole second, and adds
the offset; every later `now_offset` of the same number in the scenario reads the same instant. A
value sent at `now - 59s` and required back in the row is the one instant.

Rounding up is what makes a second of margin sufficient. With the runner's reading `r`, the target's
reading `r + δ` and offsets a second from the boundary `d`, every witness decides as required for
`0 <= δ < 1s`: the accepting `<` row, `⌈r⌉ + d - 1 < r + δ + d`, holds because `⌈r⌉ < r + 1`; the
refusing row, `⌈r⌉ + d + 1 >= r + δ + d`, because `δ <= 1`; and the `>` side mirrors them. A target
whose clock trails the runner's, or which handles the request more than a second later, can fail
a correct implementation: the suite's claim is bounded by that second, and it says so here.

## A suite that fixes the target's `now` (beyond10x/ess#586)

A guard `expiry < now` and its `guard-boundary` mutant `expiry <= now` answer differently only when
`expiry` equals the instant the target decides at, and the off-boundary witnesses above never send
that value. A target that can be told which instant to read as `now` closes that gap; one that
cannot keeps today's suite claim.

**The capability.** `ConformanceTarget::hold_now(NowHold)` asks the target to decide the **next**
command the scenario executes — and only that one — at the instant the request names, an RFC 3339
`Timestamp` at whole-second precision. Its default body answers `TargetError::Unsupported` with the
observation `hold_now`, so a target written before it keeps compiling and every scenario that needs
it is reported `unsupported`, naming `hold_now`, rather than wrong. A target that reads the runner's
command clock (`Runner::command_clock`, beyond10x/ess#510) needs no method: the runner writes the
held instant into that clock's cell for the held step instead of its rounded wall reading, so the
interpreted target and the interpreters `mutate` runs offer the capability by construction.

**The step.** `{step: hold_now, seconds: <n>}` stands immediately before the `execute_command` it
holds. `seconds` is a `now_offset` number, resolved through the same per-scenario table: the first
step naming `n` fixes it against one wall reading rounded up to a whole second, and every later
`now_offset` of `n` reads that instant. The held command's own `now_offset` values that no earlier step
fixed resolve against the reading the held instant was fixed from, not a fresh one, so a held step
and the values it is compared with refer to one instant exactly, with no latency between them:

| guard | arrangement | held step |
|---|---|---|
| `when_subject: {predicate: expiry < now}` | `Issue` sends `expiry: {kind: now_offset, seconds: 0}` | `hold_now 0`, then `Redeem` |
| `when: starts_at < now - 60s` | — | `hold_now 0`, then `ScheduleJob` with `starts_at: {kind: now_offset, seconds: -60}` |

**What synthesis adds.** For every ordering of a field against `now` that the off-boundary rule
above witnesses — a command input guard, a `when_subject:` or `when_related:` predicate over a
stored field written from a whole input field — synthesis adds one scenario **on** the boundary:
the field at the reference offset the guard names, decided at the reference itself
(`now_offset::reference`), and sent as `now_offset` with a `hold_now 0` before the command under
test. The scenario requires the branch that value takes at that instant: the accepting branch for
`expiry < now` at `expiry == now`, so the `<=` mutant, which refuses there, is killed. The
off-boundary scenarios stay as they are, with their bytes; the at-boundary scenario is one more.
A path a `now_offset` cannot carry, and a field `straddled` refuses, get no held scenario either.

How the held scenario is built: it is the off-boundary scenario of the branch the boundary takes —
the one sending the value a second from the boundary on the side whose verdict the boundary
shares (`now - 59s` for `< now - 60s`, `now - 61s` for `<= now - 60s`) — with that value moved
onto the boundary, wherever the scenario asserts it, and everything after the command under test's
own expectations left out. It is filed as `<command>/outcome/<outcome>/held/<field>/<seconds>`
(`ScenarioId::HeldOutcome`), the field being the input field sent on the boundary and the seconds
the reference offset, beside the scenario it was taken from. None is added where moving the value
could change another decision: a field also compared with a fixed instant, another `now` boundary
on the value moved from, comparisons at the boundary that disagree about the side, or a scenario
naming either offset elsewhere. A stored field written with an offset other than `0` gets a
`hold_now 0` before the writing command too, so the value it writes and the instant the command
under test is held at come from one reading.


**The runner.** On `hold_now n` the runner resolves `n`, then either writes it into the command
clock it handed out or calls `hold_now` on the target; `Unsupported` ends the scenario `unsupported`
naming `hold_now`, any other error is the target's failure. A synthesized held scenario
(`<command>/outcome/<outcome>/held/<field>/<seconds>`) that ends `unsupported` for this reason
alone stays reported, named and counted, but does not fail the run (`ConformanceReport::verdict`;
the Go and TypeScript runtimes skip it under report/2 too): synthesis adds
it to suites a target may have passed before it existed. Every other unsupported scenario fails
the run as before. A `hold_now` not followed by an
`execute_command` is refused at admission. The Go and TypeScript runtimes resolve the step the same
way and expose the same optional capability: the Rust, Go and TypeScript runners give one step the
same instant and the same verdict.

**Formats.** The step needs a suite format pair, `ess-conformance/50` (ordinary) and `/51`
(coverage), for any suite carrying a `hold_now` step; a `/49` reader refuses it by version, and a
suite without one keeps its format and bytes. A suite synthesized from a model with a now-guard the
held rule witnesses carries one, so its format moves.

**Not yet.** `now` in a view filter or field: `ess-domain` refuses the operand there, and the view
request clock of `generic-language-features.md` (G1, wave 3) is this same instant set on a
`query_view` step once the source admits it.

## Formats

| surface | change |
|---|---|
| source | `ess/16`; no IR shape change |
| suite | `ess-conformance/26` and `/27`, the round-3 pair `leaf_payloads.rs` registers, for any suite carrying a `now_offset` (`now_offset.rs`: `ORDINARY`, `COVERAGE`, `used_by`, `admit_format`). Admission reads `{kind: now_offset, seconds}` from /26, bounded at twice the largest offset; below /26 the kind is `UnsupportedScenarioValue`. A suite without one keeps its format and bytes. |
| Go and TypeScript runtimes | execute /26 and /27 and resolve `now_offset` as the Rust runner does (beyond10x/ess#188) |

## Entity Runtime and generated targets

entity-core has no clock operand: its `Condition` reads "There is no `$now`, and there will not be
(R-62): the clock is read at the edge and handed in as an argument." Lowering refuses the guard with
`LoweringCode::CurrentTimeUnsupported`, from the `current_time` paths `check_predicate` records,
rather than lowering a `before` against the text `now - 60s`, which entity-core reads as no instant
and leaves `Unknown` for every request. Generated Rust and Go targets leave the command decision to
an owed method and carry no guard to render. The operand never reaches a view filter or a selection
plan, which `ess-domain` refuses it in.

## Not in this design

- Days, months or calendar arithmetic in the offset.
- `now` on the left of an ordering, or compared with another fact (`starts_at < ends_at - 60s`).
- The operand in a stored-field predicate (`when_subject:`), an invariant, a filter or a selection.
- A suite for a guard over a `Timestamp` inside a structure or a list element.
- A suite claim at the boundary itself, or within less than a second of it, against a target that
  does not offer `hold_now` (above).
