# A conformance run drives a target process (`ess-conformance-target/1`)

Status: binding design, written 2026-11-21 before the code, for
`story:a-conformance-run-drives-a-target-process-in-any-language`. Built on the branch
`unit/outside-conformance-target`, 2026-11-21: where the build differs from the first design this
page states the build and cites it. Citations of code the build did not write (`target.rs`,
`main.rs`, `runner.rs` and the like) keep the line numbers of the tree before it (commit
`205d6c7339`); the build moved some of them. The code is `crates/verify/ess-conformance/src/
process.rs` and `process/{wire,exact,serve}.rs`, and `crates/edge/ess-cli/src/target_process.rs`.

**Decision in one line.** `ess verify conform run --target process -- <program> [args…]` starts the
program once, asks it every question of `ConformanceTarget` as one JSON object per line on its
standard input, reads one JSON answer per line from its standard output, and the Rust runner judges
the answers and writes the report/2 a built-in target gets.

This page builds on [running-a-suite.md](running-a-suite.md): a level is a property of the target,
an unsupported scenario fails the run, and the out-of-process adapter is the open gap it names
(`running-a-suite.md:96`). Those decisions are not restated here.

## Where the seam was

What follows describes the tree this design was written against.

- `ConformanceTarget` is a Rust trait with 29 methods (`crates/verify/ess-conformance/src/
  target.rs:103-584`); the runner is synchronous and calls one at a time (`target.rs:68-76`).
- `ess verify conform run --target` takes `billing`, `oracle-fixture` and `interpreted`
  (`crates/edge/ess-cli/src/main.rs:1207-1215`), and the guide said only built-in targets run; it
  now names `process` beside them (`website/docs/guides/verify/runners.md:22-24`).
- The CLI runs every built-in target on `wall_clock_runner` (`main.rs:4396-4412`): budgets,
  deadlines and report instants come from `AdvancingClock`, which starts at
  1 700 000 000 000 ms and moves 100 ms per read (`runner.rs:146-180`); only a `now_offset` value
  reads the machine clock (`main.rs:4383-4389`). This fixes the deadline rule below.
- The generated TypeScript and Go runners already put the same questions to a foreign-language
  target as JSON-like data (`src/ts/runtime.ts:2013-2108`, `src/go/runtime.go:1182-1225`). Their
  member spellings differ from the Rust types; see "Member spelling" below.

## The command line

```console
ess verify conform run --suite <suite> --target process \
    [--target-dir <dir>] [--target-timeout <seconds>] \
    --report-format 2 --report-out <file> [--strict | --allow-incomplete] \
    -- <program> [args…]
```

Every other option of `run` (`main.rs:769-818`) keeps its meaning. `--path`, `--suite-input`,
`--suite-format` and `--scenarios` choose the suite exactly as for a built-in target
(`main.rs:4276-4297`).

| option | meaning |
|---|---|
| `--target process` | a new value of `ReferenceTarget` (`main.rs:1208`), spelled `process`, with no doc comment for the reason `main.rs:1212-1213` gives |
| `-- <program> [args…]` | the command, as an argv: everything after `--` (clap `last = true`). No shell: the program is executed directly, its arguments are passed as given, nothing is expanded |
| `--target-dir <dir>` | the working directory the program starts in; default the current directory. `ess` itself does not change directory |
| `--target-timeout <seconds>` | how long `ess` waits for one answer, an integer from 1 to 3600; 30 when absent. Clap checks the range, so 0 and a value over 3600 exit 2 (`main.rs:1242-1249`). The option has no clap default: 30 is applied in code (`main.rs:4434`) so that its presence with another target can be refused. See "Time limits" |

How the program is found. A program containing `/` that is relative is joined to `--target-dir`
and made absolute before it is spawned, so `-- ./adapter` names the adapter in the
implementation's tree whatever directory `ess` runs in (`std::process::Command` leaves a relative
path with a changed working directory platform-dependent). A bare name is looked up on `ess`'s
`PATH`. `ess` adds no argument and no environment variable: the child inherits `ess`'s
environment unchanged, its standard input and output are pipes to `ess`, and its standard error is
`ess`'s standard error, passed through and never parsed.

The mechanism was chosen without precedent to follow: `mutate` has no per-mutant project command.
It runs a built-in target or splits into `--emit` and `--collect` and leaves running to the
adopter (`main.rs:965-975`), and `ess-cli` spawns no other program than `docker` and `oras`
(`main.rs:2043`, `:2260`, `:2354`). So the trailing `--` form stands.

### Refusals

Each refusal exits 2, writes no report and starts no scenario. `bail!` exits 1
(`main.rs:1652-1657`), so these return `ExitCode::from(2)` as `known_failures::REFUSED` does
(`crates/edge/ess-cli/src/known_failures.rs:21`).

| refused | message names |
|---|---|
| `--target process` with nothing after `--` | that the process target needs a command after `--` |
| a command after `--` with any other `--target` | that only `--target process` runs a command |
| `--target-dir` or `--target-timeout` without `--target process` | the option and `--target process` |
| `--target-dir` that is not an existing directory | the path |
| `--known-failing` with `--target process` | that the declaration binds `ess`'s own build (`known_failures.rs:162-167`), which is not the target's; see "Open questions" |
| the program cannot be spawned | the program and the OS error kind |
| the first handshake fails in any way | the fault kind (below); for a version, the versions this `ess` reads, never quoting what the target wrote; for an `implementation` `name` or `version` over 256 bytes, `identity-too-long` (`process.rs:407-413`) |
| `ess verify conform mutate --target process` | the split route: `--emit DIR`, then `run --target process` per suite writing `report.json` beside it, then `--collect DIR` |

`mutate` shares `ReferenceTarget` (`main.rs:991-992`, `:4533-4558`), so `process` becomes a value
it parses and refuses by name rather than one clap rejects as unknown.

### Exit status of a run

Unchanged. `render_conformance_report` decides it from the report: with `--report-format 2` and
`--strict`, 0 for passed, 1 for failed, 3 for inconclusive (`main.rs:4860-4865`); otherwise 0, 1
and 3 for passed, failed and error (`main.rs:4877-4881`). `--strict` still requires
`--report-format 2` (`main.rs:4265-4267`), still conflicts with `--allow-incomplete`
(`main.rs:798-802`), and still cannot pass a legacy suite with unknown coverage (`main.rs:798`).
A signal to `ess` during a process-target run kills the program's process group and then ends
`ess` by that signal, so a shell reports 128 plus the signal number and a Ctrl-C still stops a
shell loop; no report is written (see "A signalled `ess`").
Every failed, errored or unsupported scenario is listed under its own category in report/2's
`outcomes` (`crates/verify/ess-conformance/src/counts.rs:117-125`, `:264-285`).

The report's `producer_profile` stays `rust-scenario-status/1` (`counts.rs:227`): the Rust runner
executed and judged every scenario. The target is named by `implementation`, which is the
handshake's identity (`counts.rs:226`, `target.rs:613-617`).

## The process lifecycle

```
ess                                                  target process
 | spawn argv in --target-dir, own process group      |
 |-- {"id":0,"op":"handshake",...} ----------------->  |
 |<- {"id":0,"ok":{"speaks":...,"implementation":...}} |
 |   (per scenario, in suite order)                   |
 |-- {"id":1,"op":"begin_scenario",...} ------------>  |
 |<- {"id":1,"ok":null} ----------------------------   |
 |-- {"id":2,"op":"execute_command",...} ----------->  |
 |<- {"id":2,"ok":{...}} ---------------------------   |
 |   ...                                              |
 |-- {"id":n,"op":"end_scenario",...} -------------->  |
 |<- {"id":n,"ok":null} ----------------------------   |
 | close stdin after the last scenario                |
 | wait up to 5 s, then kill the process group        |
```

1. **Start.** `ess` spawns the argv in its own process group (`process_group(0)`), with stdin and
   stdout piped and stderr inherited, before the run begins. One process serves the whole run.
2. **Handshake.** `ess` writes the handshake request and reads its answer before `Runner` is
   made. Any fault here, or an answer naming a version `ess` does not read, is a refusal: exit 2,
   no report. The handshake's `implementation` is what `identity()` returns for the rest of the
   run. The runner calls `identity()` once, before the first scenario (`runner.rs:410-418`), and
   replaces it for a suite with one-time responses (`runner.rs:412-413`); that stays as it is.
3. **Per scenario.** The runner brackets each scenario itself. The first call of a scenario is
   `fixture_values` when its first step resolves fixtures, otherwise `begin_scenario`
   (`runner.rs:459-467`); `end_scenario` closes it, and is not called when `begin_scenario` failed
   (`runner.rs:467-518`). The process target forwards every call as one request.
4. **Lockstep.** One request is outstanding at a time. `ess` writes a request, reads exactly one
   line, and writes nothing more until that line is decoded. `id` is 0 for the handshake and then
   1, 2, 3… per process; the answer repeats it. A line the target writes unasked is read as the
   answer to the next request and fails it by `id`; one that carries the next request's own `id` is
   accepted (see "Known limits").
5. **Faults.** See the fault table. A fault ends the current scenario `error`, `ess` kills the
   process group and reaps the child, and every further call in that scenario answers without
   contacting anything: `end_scenario` answers success (the killed process closed the context, and
   the scenario already carries its one error check), every other call answers the same fault.
6. **Restart.** The first call of the next scenario spawns a fresh process and handshakes again
   (`process.rs:617-650`). The restarted process must name the same `implementation` (name and
   version) as the first; another identity is a failed handshake, because one report names one
   implementation. The two are compared as the target sent them, not as their escaped text
   (`process.rs:628-630`, `wire.rs:289-316`), so two names that would print alike are still two
   implementations. A failed restart is a respawn that cannot start (fault kind `not-started`) or a
   handshake that fails, and it is `Unavailable { operation: "handshake" }` carrying the refusal's
   text (`process.rs:645`). That scenario and every remaining one end `error` with it, `ess` does
   not restart again (`process.rs:646`), and `end_scenario` answers success, as it does after any
   fault (`process.rs:1001-1011`).
7. **End.** When the run's last scenario has ended, `ess` closes the child's stdin, waits up to 5 s
   for the exit, and then sends `SIGKILL` to the process group and reaps the child. The group is
   killed after the wait always, even when the child has already exited, so a server it left in
   its group dies too (`target_process.rs:338-344`; a group that is already gone is ignored,
   `:84-87`). A target exits when its standard input closes. That rule ends a target whose `ess`
   was killed with `SIGKILL` or crashed; any other signal to `ess` is handled (below). The child's
   exit status is not part of the report.

**No child left behind.** On a fault and at the end, `ess` signals the whole process group, so a
server the adapter started in its own group dies with it. `ess` signals the group before it reaps
the child, so the group id cannot have been reused: it reads the child's status with `waitid` and
`NOWAIT` (`rustix`'s `process` feature, which `ess-cli` already enables) without reaping it
(`target_process.rs:270-283`), and reaps it only after the kill (`:306-317`). A process the adapter
moved into another session or group is the adapter's to end. The released `ess` archives are Unix
only (`.github/workflows/package.yml:78-84`).

**A signalled `ess`.** The child's own process group does not receive the interrupt a terminal
sends to `ess`'s, and a process that dies of a signal runs no `Drop`. From before the first spawn, a
`signal-hook` iterator thread waits for `SIGINT`, `SIGTERM` and `SIGHUP`
(`target_process.rs:50-68`). On one it kills the target's process group and its leader, then reaps
the child, waiting at most 2 s (`SIGNAL_REAP`, `:70-75`, polled by `reap_within`, `:92-105`): a
child killed with `SIGKILL` is reaped within milliseconds, one that is not reaped in 2 s will not
be, and every later signal is swallowed while `ess` waits, so a leader that cannot be reaped does
not leave `ess` ignoring termination. Then `ess` ends by that same signal, with no report:
`signal_hook::low_level::emulate_default_handler`, a safe function, restores the default action,
unblocks the signal and raises it again, and for `SIGINT`, `SIGTERM` and `SIGHUP`, whose default
is to terminate, that ends the process; should it return, `ess` exits `128 + the signal` (`:82-90`).
A parent therefore sees a process terminated by the signal, as it would without the handler, so a
shell reports 128 plus the signal number and a Ctrl-C still stops a shell loop over runs. One lock
orders this. A spawn registers the child under it before releasing it, and a kill, reap and clear
all happen under it (`:174-181`, `:306-317`); the signal thread takes the same lock and never
releases it, so it sees either no target or one that is registered and not yet reaped, and no
target is spawned after it. `signal-hook` is a direct dependency of `ess-cli`
(`crates/edge/ess-cli/Cargo.toml:58`); it was already locked through the terminal renderer, so
`Cargo.lock` gains a dependency line and no package. `SIGKILL` cannot be handled.

### Time limits

The `deadline` a request carries is not a wall-clock instant. The CLI's runner takes it from
`AdvancingClock` (`main.rs:4404-4411`, `runner.rs:555-562`), so it reads as a moment in November
2023 and has passed on every wall clock. An eventual step does not wait in wall time either: it asks
again until its logical budget of 5 000 ms is spent at 100 ms per clock read
(`runner.rs:150-154`, `:1104-1180`). "Deadline plus a margin" therefore cannot bound a wait, and
the limit is:

- `--target-timeout` seconds of wall time for every request, the handshake included;
- plus `hold` seconds for `observe_elapsed`, and `elapsed.hold` seconds for `observe_periodic`,
  because an end-to-end target may sleep through the window (`target.rs:458-461`);
- plus `within` seconds for `await_outcome` where the request carries one (`target.rs:565-571`).

Each request carries its limit as `limit_ms`, so a target that waits for something returns what it
has before then. The `deadline` member keeps its Rust meaning and spelling: the runner's own
instant, echoed by a target that cares, compared by none. A target that answers what it has now is
correct; the runner asks again (`target.rs:265-269`).

## The envelope

Every line is one UTF-8 JSON object followed by `\n`. A blank line, a line that is not UTF-8 and a
line of more than 16 MiB (16 777 216 bytes before the `\n`) are faults.

Request, `ess` to target:

| member | JSON type | required | meaning |
|---|---|---|---|
| `id` | integer | yes | 0 for the handshake, then 1, 2, … per process |
| `op` | string | yes | `handshake`, or a method name of `ConformanceTarget` in its own snake_case |
| `limit_ms` | integer | yes | the wall-clock milliseconds `ess` waits for this answer |
| `request` | object | yes | the operation's request, below |

Answer, target to `ess`: exactly one of `ok` and `target_error`.

| member | JSON type | required | meaning |
|---|---|---|---|
| `id` | integer | yes | the request's `id` |
| `ok` | any | one of the two | the operation's result; `null` for an operation whose Rust result is `()` |
| `target_error` | object | one of the two | a `TargetError`, below |

Reading rules:

- **Answers are closed.** An unknown member, a missing required member and a member of the wrong
  type make the answer invalid, at every depth outside a value map (`exact::object`,
  `exact.rs:339-357`). A value map (`input`, `payload`, `fields`, `response`, a row, `context`,
  `caller`, `compared`) holds whatever names the target reports, and the runner judges them against
  the specification as it judges a built-in target's.
- **A repeated member name is refused at every depth**, value maps included: a map visitor keeps the
  last value, so a repeated name would otherwise be read as one member (`exact.rs:184-189`, fault
  `not-an-answer` at the repeated member's value).
- **Integer members are read by value**: `20.0` is `20`, as `Number` compares (`exact.rs:381-392`).
  A fraction, a negative number and a number past `u64` are refused where an unsigned integer is
  required.
- **`not_granted.actor` may be absent or `null`** in an answer (`wire.rs:337-342`,
  `exact.rs:329-335`); both mean no actor.
- **An answer holds exactly one of `ok` and `target_error`.** `{"ok": …, "target_error": null}` is
  refused as it is with a non-null one (`process.rs:776-781`).
- **`ElapsedObservation`, `OrderedScan` and `ImplementationIdentity` answers are decoded by hand**
  (`wire.rs:302-316`, `:439-453`), because the types derive `Deserialize` without
  `deny_unknown_fields` and would silently drop an unknown member.
- **Requests are open, at every depth.** A target ignores a request member it does not know,
  wherever it sits in the request. That is what lets a later `ess` add an optional request member
  without a new version (see "Format consequences"). `ess` itself reads requests only in `serve`,
  and `serve` implements the rule by dropping a member the typed decode refuses as `unknown field`
  and decoding again, at most 256 members and 4096 reads per request (`exact.rs:436-466`); beyond
  either bound the request is refused as not a request. The suite types keep
  `deny_unknown_fields`, because they also read closed suite documents, and a test holds a
  suite-side `Scope`, `Contract` and `Check` to that (`process/tests.rs:1702`). The decoder finds
  the member from serde's `unknown field` wording; if that ever changes it fails closed.
- **Optional members.** In requests `ess` writes `null` for an absent optional value, as the Rust
  `Serialize` does, and omits `params` when empty and `total` when absent, as the Rust types skip
  them (`target.rs:1045`, `:1070`, `:1109`). In answers an optional member may be omitted or `null`.

### `target_error` and how it maps

`TargetError` already serializes as `{"kind": …}` with snake_case variants (`target.rs:1262-1292`);
the wire uses that shape.

| wire | `TargetError` | what the runner does | scenario status |
|---|---|---|---|
| `"ok": {…}` with `outcome` and `error` | none: a declared refusal is a normal command result (`target.rs:1257-1261`) | judges it like any other answer | passed or failed by the suite |
| `{"kind":"unsupported","observation":S,"why":S}` | `Unsupported` | `target_failure` records `unsupported`, observed `cannot expose {observation}: {why}` (`runner.rs:4377-4385`, `target.rs:1327-1329`) | `unsupported`, failing the run (`report.rs:605-611`), except a held scenario whose only gap is `hold_now` (`report.rs:548-561`) |
| `{"kind":"unavailable","operation":S,"detail":S}` | `Unavailable` | records `error`; the process stays up, no restart | `error` |
| `{"kind":"not_granted","actor":S\|null}` to a command | `NotGranted` | the command's answer, as `ExpectNotGranted` requires (`runner.rs:1435-1445`, `:1497-1506`) | passed or failed by the suite |
| `{"kind":"not_granted",…}` to a read | `NotGranted` | kept for a following `ExpectNotGranted`, otherwise a failed read (`runner.rs:1079-1092`, `:1220-1226`) | passed or failed by the suite |
| `{"kind":"not_granted",…}` to any other operation | `NotGranted` | `target_failure`, not unsupported | `error` |
| a protocol fault (table below) | `Unavailable { operation: <op>, detail: <fault> }`, minted by `ess` | records `error`; `ess` kills the process group; the next scenario restarts it | `error` |

`unsupported` and `unavailable` strings are the target's own and reach the report as the
in-process target's do, with two bounds that an in-process target's strings do not get. Every
string of a `target_error` (`observation`, `why`, `operation`, `detail`, `actor`) is cut at 4096
bytes at a character boundary, ending ` … (<n> bytes more)`, where `<n>` counts the bytes cut, and
has every control character (Cc), line or paragraph separator (U+2028, U+2029) and bidirectional
override or isolate (U+202A to U+202E, U+2066 to U+2069) written as its escape, `\u{1b}` style,
and every backslash doubled, so the six characters `\u{1b}` print as `\\u{1b}` and never as an
escaped ESC does (`wire.rs:240-286`, `process.rs:72-74`). The 4096 bound counts the bytes the
target sent, before escaping. No message lets a target say a scenario passed: there is no verdict
member anywhere in the format, and the status is the runner's fold of its checks
(`runner.rs:537-540`).

**An operation the target does not implement**, including an `op` it has never heard of, is
answered `{"kind":"unsupported","observation":<op>,"why":<reason>}`. For four optional operations
the Rust default body is not `Unsupported` but a delegation, and the process target keeps it: an
`unsupported` answer to `query_view_as` or `query_view_anonymous` makes `ess` send `query_view` with
the same request, and one to `scan_view_as` or `scan_view_anonymous` makes it send `scan_view`
(`target.rs:240-263`, `:533-546`). A target that checks no read grant therefore fails the
`<view>/grant/read/denied` scenario, exactly as a Rust target written without those methods does
(`target.rs:237-239`).

### Protocol faults

A fault is described by kind, request `id`, operation and byte offset only. No diagnostic, report
member, `detail` text or line on `ess`'s stderr quotes an answer line or a value from it.
`serde_json`'s error text can quote a value (`invalid type: string "…"`) and
`Node::from_json_exact`'s quotes a number token
(`crates/specify/ess-primitives/src/node.rs:139-141`), so neither is ever displayed: the process
target keeps only `serde_json::Error::classify()` and `column()`, which on a one-line document is
the byte offset (`exact.rs:97-99`). A one-time scenario's checks are further replaced by the
runner's fixed redacted diagnostic (`runner.rs:3311-3323`), unchanged.

| kind | when | `detail` (example) |
|---|---|---|
| `exited` | the child exits while a request is outstanding, or a write to it fails and it exits within 500 ms | `target process exited (status 1) before answering 7 (execute_command)` |
| `closed-stdout` | stdout ends while a request is outstanding, or a write fails, and no exit status appears within 500 ms | `target process closed its standard output before answering 7 (execute_command)` |
| `timeout` | no complete line within `limit_ms` | `no answer to 7 (observe_events) within 30000 ms` |
| `line-too-long` | 16 MiB read with no `\n` | `answer to 7 (query_view) exceeds 16777216 bytes` |
| `not-utf8` | the line is not UTF-8 | `answer to 7 (query_view) is not UTF-8 at byte 412` |
| `not-json` | the line is not one JSON value, or holds only whitespace (reported at byte 0, `exact.rs:83-85`) | `answer to 7 (query_view) is not JSON at byte 0` |
| `not-an-answer` | valid JSON that breaks the envelope or the operation's answer shape | `answer to 7 (execute_command) does not match its operation at byte 88` |
| `wrong-id` | `id` is not the outstanding request's | `answer names id 9 while 7 (execute_command) is outstanding` |
| `number-out-of-range` | a number `Number::from_json_token` refuses (below) | `answer to 7 (execute_command) holds a number outside the domain at byte 121` |
| `not-started` | a restart's respawn cannot start the program | `target process could not be started again (NotFound) before 0 (handshake)` |

`not-started` is a fault of a restart only: it is a failed restart (lifecycle step 6), reported as
`Unavailable { operation: "handshake" }` with this text (`process.rs:281-282`, `:376-379`,
`:617-624`). A failed first start is not a fault but the refusal "cannot start the target program"
(exit 2, `main.rs:4522-4530`).

There is no `closed-stdin` kind. A failed write to the child's standard input is reported as
`exited` or `closed-stdout` by the same 500 ms rule (`target_process.rs:348-362`).

A handshake answer naming a version `ess` does not read is not a fault kind; it is the refusal
`target speaks ess-conformance-target/2; this ess reads ess-conformance-target/1`. The refusal never
quotes `speaks`: its text names a version only when the value is `ess-conformance-target/` followed
by one to nine decimal digits (`process.rs:439-448`, `:703-709`), and any other value is refused as
`target speaks no version of ess-conformance-target; this ess reads ess-conformance-target/1`.

### Writing, reading and the end of a stream

`ChildPeer` hands each request line to a writer thread and reads answer lines on a reader thread
(`target_process.rs:188-198`), so a target that stops reading its standard input cannot hold `ess`
in a write, and every wait for an answer ends at `limit_ms` (`:338-368`). The reader passes lines
over a channel of two (`:171`): a target that writes lines nobody asked for stalls on its own after
two, and the next request reads the first of them as its answer and fails it by `id` (`wrong-id`).
When stdout ends (end of file, a read error, or a last line with no `\n`) or a write fails, `ess`
waits up to 500 ms for an exit status (`SETTLE`, `:93-95`, `:273-278`): `exited` when it has one,
`closed-stdout` when it has none. A child that exits while another process holds its stdout open
is found by looking at the child every 50 ms rather than by running to the limit (`:352-364`).

### Numbers

Values travel exactly, by value rather than by spelling.

- **`ess` writes** every number in a request from `Number::exact_text()` (`crates/specify/
  ess-primitives/src/facts.rs:465-474`), never through `Number`'s `Serialize`. That `Serialize`
  writes the binary64 for every value binary64 carries and would round a decimal whose digits it
  does not carry, because "the run-time writer half of decimal serialization is not built"
  (`facts.rs:378-410`, `:591-592`). A suite's `1.0` is therefore sent as `1`; the value is the same.
- **`ess` reads** every value in an answer through `Node::from_json_exact` (`node.rs:89-112`), never
  through `Node`'s `Deserialize`, which reads every non-integer through binary64
  (`node.rs:33-41`). An integer in `i64` is that integer; any other token keeps the value its digits
  spell (`facts.rs:588-602`). This is the exact decoding `target.rs:98-102` asks of a JSON adapter,
  done once in Rust instead of in each adapter. The reader is `process/exact.rs`, which reads each
  number token through `Number::from_json_token`, the door `Node::from_json_exact` uses
  (`exact.rs:1-12`). It is two linear passes over a line: `serde_json` checks that it is one JSON
  value, and a second pass builds the tree, so a line nested deep is not read again at every level
  (the first reader parsed the rest of the line again at each level and visited 23,241,414 bytes for
  a 180,294-byte line). A value nested past 128 levels below the line's own value is refused
  (`exact.rs:26`, `process/exact/tests.rs:215-260`). An integer member is read by value, so `20.0`
  is `20` (`exact.rs:381-392`).
- **A number `ess` cannot hold exactly** is one `Number::from_json_token` refuses: outside
  binary64's magnitude, longer than 512 characters, or with an exponent past ±512
  (`facts.rs:594-602`). It is the fault `number-out-of-range`; the scenario is `error`, never a
  rounded pass. The Go and TypeScript runners refuse the same numbers (`facts.rs:594-596`).
- **A target in a language whose JSON numbers are binary64** (JavaScript's `JSON.parse`, many
  others) must not round a number it echoes or stores. It reads number tokens as decimals or
  strings of digits (Python: `json.loads(line, parse_float=decimal.Decimal)`, integers are already
  exact) and writes each number as the token of its exact value, never as the nearest binary64.
  A target that cannot do that for a field answers the operation that needs it `unsupported`, as
  `target.rs:101-102` requires of an in-process JSON adapter.

## Member spelling

The wire spells every request and result with the existing Rust `serde` shape of its type. Many
request types already derive `Serialize` (`target.rs:639-1245`); the types that lack it gain it
(see "What the implementation adds"). The generated TypeScript `Target` exchanges a different
spelling of the same questions, and the wire does not use it:

| where | TypeScript (`src/ts/runtime.ts`) | wire (Rust `serde`) |
|---|---|---|
| member case | camelCase: `directEvents`, `stopAfter`, `elapsedMillis` (`:2219`, `:2385`, `:2462`) | snake_case: `direct_events`, `stop_after`, `elapsed_ms` |
| a command's branch | `outcome: string` (`:2211`) | `outcome: {command, outcome}`, an `OutcomeRef` (`target.rs:769`, `crates/specify/ess-compiler/src/refs.rs:179-187`) |
| a declared error | `error: string` and `errorPayload` (`:2213-2215`) | `error: {error, fields}` (`target.rs:771`, `:841-847`) |
| freshness | result `consistency: string`, read `atLeast: string` (`:2217`, `:2272`) | result `consistency: string\|null`; read `consistency: {"consistency":"current"}` or `{"consistency":"at_least","token":…}` (`crates/specify/ess-primitives/src/consistency.rs:72-85`) |
| not granted | flags inside the result: `notGranted`, `notGrantedActor` (`:2227-2229`, `:2296-2298`) | `target_error` `not_granted` (`target.rs:1287-1291`) |
| reading as an actor | `actor` and `anonymous` on `ViewRequest` and `ScanRequest` (`:2280-2286`, `:2391-2393`) | the operations `query_view_as`, `query_view_anonymous`, `scan_view_as`, `scan_view_anonymous` (`target.rs:240-263`, `:533-546`) |
| deadline | `{attempts}` (`:2473-2476`) | the runner's instant in epoch ms (`target.rs:663-664`, `crates/specify/ess-primitives/src/time.rs:27-41`) |
| forcing an outcome | `{command, outcome, correlation}`, `times` beside them (`:2083-2085`, `:2332-2336`) | `{force: {command, outcome}, correlation}`; `times` beside it for the repeated form (`target.rs:1240-1245`, `:296-300`) |
| an event | `{event, payload}` (`:2233-2236`) | `{event, payload, correlation, sequence}` (`target.rs:873-883`) |
| a binding's invocation | `{command, input}` (`:2326-2329`) | `{binding, command, input}` (`target.rs:1161-1169`) |
| an awaited step | `{reached: true\|false, elapsedMillis}` (`:2131-2141`) | `{"answer":"reached"\|"not_issued", elapsed_ms}` (`target.rs:1217-1234`) |
| a command's actor | `actor: string`, required (`:2179`) | `actor: string\|null` (`target.rs:697`) |
| unsupported | a thrown `ErrUnsupported` (`:237-244`) | `target_error` `unsupported` with `observation` and `why` |

Same in both: `ScenarioContext`, `Identity`/`ImplementationIdentity`, `InstantMark`, `NowHold`, the
elapsed request's `instant`, `hold` (whole seconds) and `correlation`, and `ViewResult`'s `rows` and
`total`.

The Rust shape is the better single spelling. `ess` decodes into the Rust types, so any other
spelling is a translation table the in-process and process targets could drift across. Several
TypeScript shapes are lossy: an invocation without `binding`, an event without `sequence` or
`correlation`, and `{attempts}`, which no runner deadline maps onto. And folding `not_granted` into
results would give `TargetError` two wire forms.

### Deviation from the story's fit review

Fit review item 4 says "its request and answer members are the ones the generated TypeScript
`Target` already exchanges as plain JSON". The comparison above contradicts that sentence. The wire
uses the Rust `serde` members instead, and the story's sentence should read: "its request and answer
members are the Rust `serde` shapes of the trait's types; the generated TypeScript `Target` asks the
same questions in another spelling, listed on the design page." "No second spelling of a request is
introduced" still holds: the Rust shape is the existing one.

## Message catalogue

Examples use the `tasks` specification of
[the first-specification tutorial](../../website/docs/start/first-specification.md). Correlation ids
and digests are shortened; a real correlation id is `<system>-<spec_digest>-<6 digits>`, minted by
`Ids` (`runner.rs:195-240`). On the wire each example is one line. Value maps (`input`, `payload`,
`fields`, `response`, rows) hold the specification's values as JSON values (`Node` serializes
untagged, `node.rs:15-31`).

| `op` | trait method | required | default when answered `unsupported` | reached by |
|---|---|---|---|---|
| `handshake` | `identity` (`target.rs:174`) | yes | none: the run is refused | every run |
| `fixture_values` | `:108` | no | unsupported | `ResolveFixtures` |
| `open_periodic` | `:120` | no | unsupported | periodic binding checks |
| `observe_periodic` | `:130` | no | unsupported | periodic binding checks |
| `close_periodic` | `:140` | no | unsupported | periodic binding checks |
| `observe_clock_reading` | `:152` | no | unsupported | reading comparisons |
| `establish_entity` | `:167` | no | unsupported | authored entity setup |
| `begin_scenario` | `:181` | yes | unsupported | every scenario |
| `execute_command` | `:184` | yes | unsupported | `ExecuteCommand` |
| `execute_command_without_input` | `:215` | no | unsupported | `ExecuteCommandWithoutInput` |
| `query_view` | `:226` | yes | unsupported | `QueryView`, `EventuallyView` |
| `query_view_as` | `:240` | no | `query_view` | a read after `ReadAs` an actor |
| `query_view_anonymous` | `:258` | no | `query_view` | a read after `ReadAs` no actor |
| `observe_events` | `:270` | yes | unsupported | `EventuallyEvent`, refusal counts |
| `configure_external_outcome` | `:281` | yes | unsupported | `ConfigureExternalOutcome` |
| `configure_external_outcome_repeatedly` | `:296` | no | unsupported | bounded retry |
| `redeliver_event` | `:325` | yes | unsupported | `RedeliverEvent` |
| `deliver_event` | `:342` | no | unsupported | `DeliverEvent` |
| `observe_invocations` | `:377` | no | unsupported | `ExpectInvocation`, `ExpectEveryInvocation` |
| `mark_instant` | `:399` | no | unsupported | `MarkInstant` |
| `hold_now` | `:418` | no | unsupported | a held scenario |
| `observe_elapsed` | `:465` | no | unsupported | `ExpectNotBefore`, `ExpectWithin`, `ExpectQuiet` |
| `scan_view` | `:518` | no | unsupported | `ExpectHalt`, `EventuallyHalt` |
| `scan_view_as` | `:533` | no | `scan_view` | a scan after `ReadAs` an actor |
| `scan_view_anonymous` | `:544` | no | `scan_view` | a scan after `ReadAs` no actor |
| `await_outcome` | `:575` | no | unsupported | `AwaitOutcome`, `ExpectNotIssued` |
| `end_scenario` | `:583` | yes | unsupported | every scenario that began |

"Required" means the trait gives the method no default body. A target may still answer any
operation `unsupported`; for a required one that makes every scenario reaching it `unsupported`.

Three methods are not operations. `identity` is answered by the handshake.
`execute_command_recorded` (`target.rs:198-206`) is called only by the concurrent-history recorder
(`src/record.rs:100`), which is out of scope; the process target keeps its default, one
`execute_command`. `take_command_clock` (`target.rs:435-440`) hands over an in-process clock and is
called only by the mutation audit (`src/mutate.rs:5652`); the process target keeps its default,
`false`, so the runner sends `hold_now` (`runner.rs:2506-2515`).

### `handshake`

```json
{"id": 0, "op": "handshake", "limit_ms": 30000, "request": {
  "reads": ["ess-conformance-target/1"],
  "suite": {"system": "tasks", "specification_version": "v1",
            "suite_version": "ess-conformance/34", "spec_digest": "5b1d0c…",
            "suite_digest": "sha256:9e04aa…"}}}
```

```json
{"id": 0, "ok": {"speaks": "ess-conformance-target/1",
                 "implementation": {"name": "tasks-py", "version": "1.0.0"}}}
```

| member | JSON type | required | meaning |
|---|---|---|---|
| `request.reads` | array of string | yes | the versions this `ess` reads, oldest first |
| `request.suite.system`, `.specification_version`, `.suite_version`, `.spec_digest` | string | yes | the suite's `SuiteProvenance` (`src/scenario.rs:466-503`); `spec_digest` is bare hex, as the suite writes it |
| `request.suite.component` | string | no | present when the suite is scoped to one component (`scenario.rs:499-500`) |
| `request.suite.suite_digest` | string | yes | `AdmittedSuite::digest()`, `sha256-json-bytes/1` of the suite's bytes, written `sha256:` and 64 lowercase hex digits, not bare hex (`src/admission.rs:154-160`, `:180-183`) |
| `ok.speaks` | string | yes | the one version the target speaks; must be in `reads` |
| `ok.implementation.name`, `.version` | string | yes | `ImplementationIdentity` (`target.rs:593-601`), each at most 256 bytes: a longer one is `identity-too-long` (`wire.rs:258-268`), counted in the bytes sent, and each is stored with control, line-separator and bidirectional characters written as escapes and a backslash doubled (`wire.rs:240-256`), while a restart is compared by the name and version as sent (`process.rs:628-630`) |

A target may refuse a suite it does not implement by answering the handshake with its identity and
then answering what it can; the runner's verdict says the rest. A handshake answered with
`target_error` is a failed handshake.

### `begin_scenario` and `end_scenario`

```json
{"id": 1, "op": "begin_scenario", "limit_ms": 30000, "request": {
  "scenario": "tasks.list.AddTask/outcome/added", "correlation": "tasks-5b1d0c-000001"}}
```

```json
{"id": 1, "ok": null}
```

| member | JSON type | required | meaning |
|---|---|---|---|
| `scenario` | string | yes | the scenario id as the suite writes it (`scenario.rs:1279-1290`) |
| `correlation` | string | yes | the activity every request of this scenario carries (`target.rs:639-645`) |

`end_scenario` has the same request and answer. Each scenario must start from the empty state the
suite requires (`scenario_initial_state`, printed by `run` at `main.rs:4299-4301`).

### `execute_command`

```json
{"id": 2, "op": "execute_command", "limit_ms": 30000, "request": {
  "command": "tasks.list.AddTask", "actor": "tasks.list.User", "caller": null,
  "input": {"title": "title", "priority": 0}, "correlation": "tasks-5b1d0c-000001"}}
```

```json
{"id": 2, "ok": {
  "outcome": {"command": "tasks.list.AddTask", "outcome": "added"},
  "error": null, "consistency": "1", "response": null,
  "direct_events": [{"event": "tasks.list.TaskAdded",
                     "payload": {"task_id": "00000000-0000-4000-8000-000000000001",
                                 "title": "title"},
                     "correlation": null, "sequence": null}]}}
```

A declared refusal is the same answer with the refusing branch and its error:
`"outcome": {"command": "tasks.list.AddTask", "outcome": "rejected"}, "error": {"error":
"tasks.list.InvalidPriority", "fields": {"submitted": -1}}`.

Request (`SemanticCommandRequest`, `target.rs:692-710`):

| member | JSON type | required | meaning |
|---|---|---|---|
| `command` | string | yes | the command's qualified name |
| `actor` | string or null | yes | the actor the step sends as |
| `caller` | object or null | yes | caller attribute values to authenticate as; `null` when the actor declares none |
| `input` | object | yes | input values by declared field name, every reference resolved |
| `correlation` | string | yes | the scenario's activity |

Answer (`SemanticCommandResult`, `target.rs:748-793`):

| member | JSON type | required | meaning |
|---|---|---|---|
| `outcome` | object or null | no | `{command, outcome}` of the declared branch taken; absent or `null` when no declared branch covers the refusal (`target.rs:752-768`) |
| `error` | object or null | no | `{error, fields}`: the declared error's qualified name and its payload |
| `consistency` | string or null | no | a non-empty token a later `read_your_writes` read demands (`consistency.rs:30-42`); a command answering none makes such a read `unsupported` |
| `direct_events` | array of event | no, default `[]` | the events this invocation published |
| `response` | object or null | no | the command's returned response fields, read exactly |

Event (`ObservedEvent`, `target.rs:873-883`), used here and by `observe_events`:

| member | JSON type | required | meaning |
|---|---|---|---|
| `event` | string | yes | the event's qualified name |
| `payload` | object | yes | its fields by declared name |
| `correlation` | string or null | no | the activity, where the target propagates one |
| `sequence` | integer or null | no | its position in the target's publication order |

### `execute_command_without_input`

The request is `execute_command`'s without `input` (`AbsentInputRequest`, `target.rs:732-742`): an
absent request body, not `{}`. The answer is `execute_command`'s.

```json
{"id": 3, "op": "execute_command_without_input", "limit_ms": 30000, "request": {
  "command": "tasks.list.AddTask", "actor": "tasks.list.User", "caller": null,
  "correlation": "tasks-5b1d0c-000004"}}
```

```json
{"id": 3, "ok": {"outcome": null, "error": null, "consistency": null,
                 "direct_events": [], "response": null}}
```

### `query_view`, `query_view_as`, `query_view_anonymous`

```json
{"id": 4, "op": "query_view", "limit_ms": 30000, "request": {
  "view": "tasks.list.Tasks", "consistency": {"consistency": "at_least", "token": "1"},
  "correlation": "tasks-5b1d0c-000001", "deadline": 1700000005300}}
```

```json
{"id": 4, "ok": {"rows": [{"task_id": "00000000-0000-4000-8000-000000000001",
                           "title": "title", "priority": 0, "state": "Open"}]}}
```

Request (`SemanticViewRequest`, `target.rs:1035-1057`):

| member | JSON type | required | meaning |
|---|---|---|---|
| `view` | string | yes | the view's qualified name |
| `params` | object | no | each declared parameter's value; omitted when the view declares none |
| `consistency` | object | yes | `{"consistency":"current"}` or `{"consistency":"at_least","token":S}` |
| `correlation` | string | yes | the scenario's activity |
| `deadline` | integer | yes | the runner's instant in epoch ms (see "Time limits") |
| `reader` | string | `query_view_as` only | the actor the read is sent as |

Answer (`SemanticViewResult`, `target.rs:1060-1072`):

| member | JSON type | required | meaning |
|---|---|---|---|
| `rows` | array of object | yes | each row's values by projected field name |
| `total` | integer | no | the rows the filter admits, for a view declaring `paging: {total: true}` |

`query_view_anonymous` has `query_view`'s request and answer. A read-granted view answers a reader
it does not admit with `target_error` `not_granted`, naming the reader or `null`.

### `observe_events`

```json
{"id": 5, "op": "observe_events", "limit_ms": 30000, "request": {
  "event": "tasks.list.TaskAdded", "correlation": "tasks-5b1d0c-000001",
  "deadline": 1700000005400}}
```

```json
{"id": 5, "ok": [{"event": "tasks.list.TaskAdded",
                  "payload": {"task_id": "00000000-0000-4000-8000-000000000001",
                              "title": "title"}}]}
```

Request `EventObservationRequest` (`target.rs:919-930`): `event` (string), `correlation` (string),
`deadline` (integer), all required. Answer: an array of event, as above.

### `configure_external_outcome` and `configure_external_outcome_repeatedly`

```json
{"id": 6, "op": "configure_external_outcome_repeatedly", "limit_ms": 30000, "request": {
  "force": {"command": "billing.email.SendEmail", "outcome": "failed"},
  "correlation": "billing-096efa-000002", "times": 3}}
```

```json
{"id": 6, "ok": null}
```

| member | JSON type | required | meaning |
|---|---|---|---|
| `force` | object | yes | `{command, outcome}`: the branch the next invocation(s) must take (`target.rs:1240-1245`) |
| `correlation` | string | yes | the scenario's activity |
| `times` | integer ≥ 1 | repeated form only | how many invocations it applies to (`target.rs:296-300`) |

### `redeliver_event` and `deliver_event`

```json
{"id": 7, "op": "deliver_event", "limit_ms": 30000, "request": {
  "event": "acme.inbound.MessageReceived", "authority": "mailbox",
  "payload": {"message_id": "m-1"}, "context": {"tenant": "t-1"},
  "correlation": "acme-1f00aa-000003"}}
```

```json
{"id": 7, "ok": null}
```

`redeliver_event` (`RedeliveryRequest`, `target.rs:1002-1011`) carries `event` and `correlation`.
`deliver_event` (`EventDeliveryRequest`, `target.rs:1018-1030`) adds `authority` (string, the
binding-local channel name), `payload` and `context` (objects). All members are required.

### `observe_invocations`

```json
{"id": 8, "op": "observe_invocations", "limit_ms": 30000, "request": {
  "binding": "notify-on-invoice-created", "command": "billing.email.SendEmail",
  "correlation": "billing-096efa-000031", "deadline": 1700000105200}}
```

```json
{"id": 8, "ok": [{"binding": "notify-on-invoice-created",
                  "command": "billing.email.SendEmail",
                  "input": {"recipient": "customer_email"}}]}
```

Request `InvocationObservationRequest` (`target.rs:1148-1158`): `binding`, `command`,
`correlation`, `deadline`, all required. Answer: an array of `ObservedInvocation`
(`target.rs:1161-1169`): `binding`, `command` (strings) and `input` (object), all required. The
answer is the cumulative snapshot `target.rs:364-372` defines.

### `mark_instant`, `observe_elapsed`, `hold_now`

```json
{"id": 9, "op": "observe_elapsed", "limit_ms": 50000, "request": {
  "instant": "sent", "hold": 20, "watching": "billing.email.DeliveryEscalated",
  "correlation": "billing-096efa-000007"}}
```

```json
{"id": 9, "ok": {"elapsed_ms": 20004, "published": 1}}
```

| op | request members (all required) | answer |
|---|---|---|
| `mark_instant` | `instant` (lower-kebab name), `correlation` (`target.rs:935-941`) | `null` |
| `observe_elapsed` | `instant`, `hold` (whole seconds), `watching` (string or null), `correlation` (`target.rs:961-979`) | `{elapsed_ms, published}`, both integers, required (`target.rs:982-995`) |
| `hold_now` | `instant` (RFC 3339 UTC at whole seconds, `2026-11-21T10:00:00Z`), `correlation` (`target.rs:943-950`) | `null` |

`hold_now`'s instant is written by `Rfc3339Instant::to_rfc3339` and read by `parse_rfc3339`
(`time.rs:741`, `:912`). `limit_ms` above is 30 000 plus the 20 s `hold`.

### `scan_view`, `scan_view_as`, `scan_view_anonymous`

```json
{"id": 10, "op": "scan_view", "limit_ms": 30000, "request": {
  "view": "billing.invoice.OutstandingInvoices", "stop_after": 1,
  "consistency": {"consistency": "current"}, "correlation": "billing-096efa-000025",
  "deadline": 1700000085300}}
```

```json
{"id": 10, "ok": {"produced": 1, "halted": true}}
```

Request `OrderedScanRequest` (`target.rs:1104-1122`): `view`, `params` (omitted when empty),
`stop_after` (integer ≥ 1), `consistency`, `correlation`, `deadline`; `scan_view_as` adds `reader`.
Answer `OrderedScan` (`target.rs:1128-1143`): `produced` (integer) and `halted` (boolean), both
required.

### `await_outcome`

```json
{"id": 11, "op": "await_outcome", "limit_ms": 90000, "request": {
  "command": "acme.billing.ExpireInvoice", "instance": "00000000-0000-4000-8000-000000000001",
  "within": 60, "deadline": 1700000009100, "compared": {},
  "correlation": "acme-1f00aa-000012"}}
```

```json
{"id": 11, "ok": {"answer": "reached",
  "result": {"outcome": {"command": "acme.billing.ExpireInvoice", "outcome": "expired"},
             "direct_events": []},
  "input": {"invoice_id": "00000000-0000-4000-8000-000000000001"}, "elapsed_ms": 60000}}
```

Request `AwaitRequest` (`target.rs:1193-1214`): `command` (string), `instance` (value), `within`
(whole seconds or null), `deadline`, `compared` (object), `correlation`; all present. Answer
`AwaitAnswer` (`target.rs:1217-1234`), tagged by `answer`: `"reached"` with `result` (a command
result), `input` (object) and `elapsed_ms` (integer, optional); or `"not_issued"` with `elapsed_ms`
(optional).

### `establish_entity`

```json
{"id": 12, "op": "establish_entity", "limit_ms": 30000, "request": {
  "entity": "tasks.list.Task", "identity": "00000000-0000-4000-8000-0000000000aa",
  "fields": {"title": "seeded", "priority": 2}, "state": "Done",
  "correlation": "tasks-5b1d0c-000009"}}
```

```json
{"id": 12, "ok": null}
```

`EntitySetupRequest` (`target.rs:620-632`): `entity` (string), `identity` (value), `fields`
(object, an absent Optional field absent and a present null `null`), `state` (string),
`correlation` (string), all required. Validation failure or inability is `unsupported`, never `ok`
(`target.rs:161-166`).

### `fixture_values`

```json
{"id": 13, "op": "fixture_values", "limit_ms": 30000, "request": {
  "scenario": {"scenario": "acme.session.Open/outcome/opened",
               "correlation": "acme-1f00aa-000020"},
  "contract": {"fields": [{"name": "principal-id", "type": "Uuid"}],
               "declarations": {}}}}
```

```json
{"id": 13, "ok": {"principal-id": "00000000-0000-4000-8000-0000000000bb"}}
```

`request.scenario` is a `ScenarioContext`; `request.contract` is `fixtures::Contract` as it
serializes (`src/fixtures.rs:26-53`: `fields` of `{name, type, entity?, state?}` and
`declarations`); the example's `type` spelling is illustrative, not verified against a suite. The
answer is an object of values by fixture name, which the runner validates and freezes
(`target.rs:104-107`).

### `open_periodic`, `observe_periodic`, `close_periodic`

```json
{"id": 14, "op": "observe_periodic", "limit_ms": 40000, "request": {
  "scope": {"correlation": "acme-1f00aa-000030", "binding": "poll-mailbox", "lifetime": "l-1"},
  "elapsed": {"instant": "opened", "hold": 10, "watching": null,
              "correlation": "acme-1f00aa-000030"},
  "after": 0}}
```

```json
{"id": 14, "ok": {"elapsed_ms": 10000, "complete_through_ms": 10000, "cursor": 1,
  "records": [{"scope": {"correlation": "acme-1f00aa-000030", "binding": "poll-mailbox",
                         "lifetime": "l-1"},
               "fact": {"kind": "received", "ordinal": 1, "due_ms": 5000, "at_ms": 5000,
                        "eligible": true}}]}}
```

| op | request | answer |
|---|---|---|
| `open_periodic` | `periodic::Open` (`src/periodic.rs:67-74`): `check` (the suite's `periodic::Check` verbatim, `periodic.rs:27-40`) and `mark` (an instant mark) | `Opened` (`periodic.rs:76-85`): `scope`, `anchor_ms`, `context` |
| `observe_periodic` | `periodic::Observe` (`periodic.rs:87-95`): `scope`, `elapsed` (an elapsed request), `after` | `Observation` (`periodic.rs:181-192`): `elapsed_ms`, `complete_through_ms`, `cursor`, `records` of `{scope, fact}` with `fact` tagged by `kind` (`periodic.rs:97-158`) |
| `close_periodic` | `periodic::Scope` (`periodic.rs:57-66`): `correlation`, `binding`, `lifetime` | `Closed` (`periodic.rs:194-201`): `scope`, `at_ms` |

The periodic result types already deny unknown members (`periodic.rs:28`, `:58`, `:77`, `:98`,
`:173`, `:182`, `:195`).

### `observe_clock_reading`

```json
{"id": 15, "op": "observe_clock_reading", "limit_ms": 30000, "request": {
  "reading": {"event": "acme.audit.Stamped", "occurrence": 0, "field": "at",
              "declared_type": "acme.audit.Instant",
              "contract": {"encoding": "unix_seconds", "origins": ["producer_process"]}},
  "correlation": "acme-1f00aa-000040"}}
```

```json
{"id": 15, "ok": {"correlation": "acme-1f00aa-000040", "occurrence": "acme.audit.Stamped#0.at",
  "source": {"process_instance": "p-1", "epoch": "e-1"}, "origin": "producer_process",
  "formatter": {"kind": "unix_seconds"}}}
```

Request `reading::ReadingObservationRequest` (`src/reading.rs:98-105`): `reading`, a
`ReadingReference` (`reading.rs:13-26`), and `correlation`. The `origins` spelling in the example is
not verified (`ReadingOrigin` was not read). Answer `ReadingEvidence`
(`crates/specify/ess-domain/src/reading.rs:143-157`): `correlation`, `occurrence`, `source`
(`{process_instance, epoch}` or null), `origin` (`producer_process`, `consumer_process`,
`unknown`), `formatter` tagged by `kind`.

## A minimal adapter in Python 3

The adapter is in [the process-target guide
page](../../website/docs/guides/verify/target-process.md#a-complete-adapter): Python 3 on
the standard library, serving the `tasks` specification of [the first-specification
tutorial](../../website/docs/start/first-specification.md) from memory. It is documentation that a
test runs, and `AGENTS.md`'s rule that everything executable is Rust is untouched: no Python file is
committed. `crates/edge/ess-cli/tests/process_target_guide.rs` takes the guide's code block out of
the page byte for byte and takes the specification out of the tutorial page, synthesizes the suite
with `--suite-format 5`, and runs the documented command under `--strict`. All 6 scenarios pass and
the exit status is 0
(`the_guides_python_adapter_passes_every_scenario_of_its_specification`). A second case changes one
answer of the adapter and requires exit 1 with exactly `tasks.list.AddTask/outcome/rejected` failed,
so the first cannot pass by accident
(`the_guides_adapter_with_one_wrong_outcome_fails_that_scenario`). Both print a skip and pass where
`python3` is not on `PATH`.

The sketch this page held before the build is gone. It answered no outcome for `CompleteTask` of a
task that does not exist, where the suite expects `already-done` (`wrong_state: true` covers a
task that does not exist), and it failed 1 of the 6 scenarios. The guide's adapter answers
`already-done` with empty `fields` there.

Notes for authors that stand: the adapter answers every operation it lacks `unsupported`,
`configure_external_outcome` and `redeliver_event` included, which this specification never reaches;
it returns a consistency token from every command because `Tasks` is `read_your_writes`; and its
ids come from a counter, so two runs write identical reports. The `tasks` specification has no
`Decimal`; an adapter for one reads with `json.loads(line, parse_float=decimal.Decimal)` and writes
each decimal as its exact token (see "Numbers"), which the standard library's `json.dumps` does not
do for `Decimal`.

## Test peer: `ess verify conform serve`

The acceptance test needs a process that serves the built-in `billing` reference over the protocol,
and no test may build a binary with cargo (`crates/edge/ess-xtask/tests/no_test_builds.rs:1-6`).

**Chosen: a public library function and one verb,** option (a):

- `ess_conformance::process::serve(target: &impl ConformanceTarget, input: impl BufRead, output:
  impl Write) -> Result<(), ServeError>`, the inverse of the process target: it answers the
  handshake with `target.identity()`, decodes each request, calls the method, writes the answer,
  answers an unknown `op` `unsupported`, and returns at end of input.
- `ess verify conform serve --target billing|oracle-fixture`, which serves that built-in reference
  on its standard input and output until input closes: exit 0 at end of input, exit 2 on a request
  it cannot decode (stderr names the fault kind, never the line: `serve.<kind>: …`,
  `main.rs:4544-4568`). `serve` fits the conform verbs (`synthesize`, `run`, `report`, `mutate`, …;
  `main.rs:612-1107`) and takes its own two-value enum, `ServedTarget` (`main.rs:1256-1263`), so
  `interpreted` and `process` are not offered.

It is the protocol's reference peer for adopters: an adapter author can run the billing suite
against `ess verify conform serve --target billing` and compare transcripts. And the acceptance
test spawns it through `CARGO_BIN_EXE_ess`, which the test build already links.

Cost: a public CLI verb and public API, so `task ci-lint` is part of the gate (`AGENTS.md`, the
package-gate table), a `CHANGELOG.md` line and the CLI reference. The agent plugin's
`agentplugins-check tools` runs only the commands the plugin spells, so it is unaffected until the
plugin spells `serve`.

**The fault cases need a peer that misbehaves**, which a public verb should not offer. They use a
test-only executable, `ess-conformance-peer` (`crates/edge/ess-cli/tests/support/
conformance_peer.rs`), declared as a `[[bin]]` the way `ess-recovery-driver` and
`ess-recovery-fake` are (`crates/edge/ess-cli/Cargo.toml:82-87`). It wraps `serve` over `Billing`
and injects one misbehaviour per option: `--wrong-outcome`, `--exit-in`, `--garbage-in`,
`--wrong-id-in`, `--stall-in`, `--unsupported`, `--speaks`, `--pid-file`, `--fail-restart` and
`--record <file>`, which appends the `op` of each request it receives to the file, one per line,
so a test can say which requests the peer saw (`conformance_peer.rs:9-22`). It costs one more link
of `ess-conformance` in the `ess-cli` test build.

## Determinism

Scenario order is the suite's (`runner.rs:424-426`). Values `ess` mints:

| value | source | stable across runs |
|---|---|---|
| correlation ids | `Ids::for_suite`, a counter seeded from system and spec digest (`runner.rs:195-240`) | yes |
| `deadline` | `AdvancingClock` plus the 5 000 ms budget (`runner.rs:555-562`) | yes, against a target that answers the same |
| `started_at`, `completed_at`, `duration_ms` | the same clock (`runner.rs:410`, `:428`, `:545-550`); report/2 `completed_at` is `completed_at` (`counts.rs:228`) | yes, against a target that answers the same |
| request `id`, `limit_ms` | per process counter; the options | yes |
| `now_offset` instants, including `hold_now`'s | the machine clock (`main.rs:4383-4412`, `runner.rs:2496-2516`) | no, and the same holds for built-in targets |

So two runs of one suite against one deterministic target write report/2 files that are equal byte
for byte. The acceptance allows them to differ in the completion instant, and the test asserts the
stronger equality. A restart changes nothing that is minted.

## What the implementation adds

- `crates/verify/ess-conformance/src/process.rs` and `process/{wire,exact,serve}.rs`:
  `TARGET_FORMAT`, `READS`, the envelope, a request writer with exact numbers, an answer reader
  (`exact.rs`) that reads number tokens through `Number::from_json_token` and refuses repeated
  member names (checked for this build, `exact.rs:184-189`), the fault kinds, a `Peer` trait (send a
  line, receive a line within a limit, kill and reap, respawn), `ProcessTarget<P: Peer>`
  implementing `ConformanceTarget` with the default fallbacks above, and `serve`.
- `crates/edge/ess-cli/src/target_process.rs`: the `Peer` over a child process. It resolves the
  program against `--target-dir`, spawns it with `process_group(0)`, writes requests from a writer
  thread, reads stdout on a thread into bounded lines, waits with `recv_timeout`, signals the group
  with `rustix::process` before reaping, and ends the target when `ess` is signalled.
- `crates/edge/ess-cli/src/main.rs`: `ReferenceTarget::Process`, `Run`'s `command`,
  `target_dir` and `target_timeout` (`ProcessArgs`, `:1237-1254`), the refusals
  (`process_refusal`, `:4460-4511`), the dispatch (`run_target`, `:4388-4431`), the `mutate`
  refusal (`:4649-4658`), and the `Serve` verb (`conform_serve`, `:4544-4568`).
- `crates/edge/ess-cli/Cargo.toml`: `signal-hook` as a direct dependency (`:58`).
- `serde` derives. Requests gain `Deserialize`, for `serve`: `ScenarioContext`,
  `SemanticCommandRequest`, `AbsentInputRequest`, `SemanticViewRequest`, `EventObservationRequest`,
  `Deadline`, `InstantMark`, `ElapsedObservationRequest`, `RedeliveryRequest`,
  `EventDeliveryRequest`, `OrderedScanRequest`, `InvocationObservationRequest`, `AwaitRequest`,
  `ExternalOutcomeControl`. Five types have no `serde` at all and gain both:
  `EntitySetupRequest` (`target.rs:620`), `NowHold` (`target.rs:944`, its instant as RFC 3339
  text), `periodic::Open` (`periodic.rs:68`), `periodic::Observe` (`periodic.rs:87`) and
  `reading::ReadingObservationRequest` (`reading.rs:99`). Results gain decoders through the exact
  reader: `SemanticCommandResult`, `DeclaredErrorValue`, `ObservedEvent`, `SemanticViewResult`,
  `ObservedInvocation`, `AwaitAnswer`, `TargetError`. `ElapsedObservation`, `OrderedScan` and
  `ImplementationIdentity` already derive both but are decoded by hand too, so that an unknown
  member is refused (`wire.rs:302-316`, `:439-453`). The periodic results and `ReadingEvidence`
  already derive both, and the periodic result types deny unknown members (`periodic.rs:28`,
  `:58`, `:77`, `:98`, `:173`, `:182`, `:195`).
  `InstantName`, `StateName`, the semantic refs and `BindingRef` already have hand-written
  `Deserialize` (`scenario.rs:1672-1677`, `crates/specify/ess-domain/src/entity.rs:124`,
  `refs.rs:94-101`, `:165-172`).
- `crates/edge/ess-cli/tests/process_target.rs` and the test-only `ess-conformance-peer`.
- Documentation: `website/docs/guides/verify/target-process.md` is "An implementation in any
  language", with the adapter, and `runners.md` keeps a short section of that name;
  `website/docs/reference/formats.md`, `FORMAT_RELEASES`, `CHANGELOG.md`.

## Tests the acceptance requires

Each line is one acceptance statement of the story, in its order, with the tests that exist. CLI
tests live in `crates/edge/ess-cli/tests/process_target.rs` unless a file is named; `peer` is
`ess-conformance-peer`, and `lib` is `crates/verify/ess-conformance/src/process/tests.rs`.

1. Design page: this page; no test.
2. `a_served_billing_reference_reports_what_the_built_in_one_reports`: runs
   `suites/generated/billing/suite.json` with `--target billing` and with `--target process --
   $CARGO_BIN_EXE_ess verify conform serve --target billing`, both `--report-format 2
   --report-out`, and compares the two reports' `outcomes` and `counts`. In `lib`,
   `a_billing_run_through_serve_reports_what_the_built_in_run_reports` does the same through
   `ProcessTarget` and `serve` with no process.
3. `strict_exits_nonzero_unless_every_scenario_passed` and
   `each_non_passing_scenario_is_listed_under_its_own_category`: on a coverage suite from
   `--path examples/billing --suite-format 5` (strict cannot pass unknown coverage, `main.rs:798`),
   `serve` exits 0 under `--strict`, and a peer with one failed, one errored and one unsupported
   scenario exits 1 and lists each under `failed`, `error` and `unsupported`.
4. `one_wrong_outcome_fails_that_scenario_and_no_other`: `peer --wrong-outcome
   billing.invoice.CreateInvoice/outcome/accepted` answers `rejected` for that scenario's first
   command; `outcomes.failed` is exactly that id and every other scenario keeps the built-in run's
   status. `crates/edge/ess-cli/tests/process_target_guide.rs` repeats it for the guide's adapter:
   `the_guides_adapter_with_one_wrong_outcome_fails_that_scenario`.
5. One test per fault, each asserting that scenario is `error`, the scenarios after it keep their
   built-in status (the restart works), and the peer's pid and a grandchild's pid it wrote to
   `--pid-file` are gone once `ess` exits: `an_exit_mid_scenario_is_an_error` (`--exit-in`),
   `a_line_that_is_not_an_answer_is_an_error` (`--garbage-in`),
   `an_answer_to_another_request_is_an_error` (`--wrong-id-in`),
   `an_answer_past_the_time_limit_is_an_error` (`--stall-in`, with `--target-timeout 1`). Beside
   them: `a_failed_restart_errors_every_remaining_scenario` (`--fail-restart`),
   `a_sighup_to_ess_waiting_for_an_answer_ends_the_targets_group_and_writes_no_report`, and, in
   `crates/edge/ess-cli/tests/adversary_process_target_pass1.rs`,
   `a_sigterm_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind` and
   `a_sigint_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind`. In `lib`, one test per
   fault kind (`an_exit_is_the_exited_fault`, `a_closed_stdout_is_the_closed_stdout_fault`,
   `no_answer_in_time_is_the_timeout_fault`, `a_long_line_is_the_line_too_long_fault`,
   `a_line_that_is_not_utf8_is_the_not_utf8_fault`, `a_blank_or_broken_line_is_the_not_json_fault`,
   `a_shape_the_operation_does_not_have_is_the_not_an_answer_fault`,
   `an_answer_to_another_request_is_the_wrong_id_fault`,
   `a_number_outside_the_domain_is_the_number_out_of_range_fault`,
   `a_target_that_cannot_be_started_again_is_the_not_started_fault`), the lifecycle
   (`after_a_fault_the_scenario_answers_alone_and_the_next_one_restarts`,
   `a_restart_naming_another_implementation_fails_every_remaining_scenario`,
   `a_failed_restart_fails_every_remaining_scenario_without_respawning_again`), and
   `a_fault_never_quotes_the_answer`, which feeds lines holding a marker secret for every fault kind
   and asserts that no fault text contains it.
6. `an_unsupported_answer_is_unsupported_with_its_reason`: `peer --unsupported query_view` makes
   each scenario that reads `unsupported`; the detailed `--format json` run carries the peer's
   `why` in those checks' diagnostics, and the exit status is 1. In `lib`,
   `an_unsupported_read_as_an_actor_falls_back_to_the_plain_read`,
   `a_target_error_answer_is_the_targets_and_keeps_the_process` and, in
   `crates/verify/ess-conformance/tests/adversary_process_target_pass1.rs`,
   `an_unsupported_command_answer_is_unsupported_and_keeps_the_process`.
7. `two_runs_write_identical_reports`: two runs against `serve --target billing` write equal
   report/2 bytes.
8. `an_unread_protocol_version_is_refused_before_any_scenario`: `peer --speaks
   ess-conformance-target/2` exits 2, stderr names `ess-conformance-target/1`, no report is
   written, and the peer recorded no `begin_scenario` (`--record`). In `lib`,
   `a_version_this_build_does_not_read_is_refused_naming_the_versions_read` and
   `a_handshake_answered_with_a_target_error_or_a_fault_is_refused`.
9. `task test-xtask`: the `FORMAT_RELEASES` row, the `formats.md` row and the process-target guide
   section. In `lib`, the codec: `every_operation_round_trips_through_serve`,
   `numbers_past_binary64_travel_exactly_both_ways`, the refusals of an unknown, repeated, missing
   and mistyped answer member (`an_unknown_answer_member_is_refused`,
   `a_repeated_member_name_is_refused`, `a_missing_answer_member_is_refused`,
   `a_member_of_the_wrong_type_is_refused`), and
   `a_value_map_holds_any_names_and_optional_members_may_be_absent`.
   In `process/exact/tests.rs`: `every_line_reads_as_the_reader_it_replaced_read_it`,
   `a_line_is_gone_over_once_however_deep_it_nests` and
   `the_nesting_limit_is_128_levels_below_the_line`.

Added by the review of the build, with the tests that pin them:

- **The guide's adapter runs.** `process_target_guide.rs`:
  `the_guides_python_adapter_passes_every_scenario_of_its_specification` (see "A minimal adapter in
  Python 3").
- **Requests are open at every depth.**
  `serve_drops_an_unknown_top_level_request_member_in_every_operation`,
  `serve_drops_an_unknown_request_member_at_every_depth_in_every_operation`,
  `a_request_member_unknown_in_one_object_is_dropped_there_and_kept_where_it_is_known` and
  `a_suite_scope_check_or_contract_with_a_member_it_does_not_know_is_still_refused` in `lib`, and
  `serve_ignores_a_request_member_it_does_not_know_in_every_operation` in the `ess-conformance`
  `adversary_process_target_pass1.rs`.
- **What a target supplies is escaped and bounded.**
  `a_handshake_identity_has_every_steering_character_written_as_text`,
  `a_handshake_name_or_version_past_256_bytes_is_refused_without_quoting_it`,
  `a_restarted_target_naming_the_same_steering_identity_is_the_same_implementation`,
  `every_string_of_a_target_error_has_every_steering_character_written_as_text`,
  `a_target_error_string_past_4096_bytes_is_cut_at_a_character_naming_what_was_cut`,
  `a_report_through_a_process_target_writes_no_steering_character_the_target_sent` in `lib`, and
  `the_text_report_prints_a_targets_implementation_name_without_control_characters` and
  `the_text_report_prints_a_targets_unsupported_reason_without_control_characters` in the
  `ess-conformance` `adversary_process_target_pass1.rs`.
- **The text report escapes names and values.**
  `a_text_report_writes_an_escape_in_a_row_member_name_as_text`,
  `a_text_report_writes_a_c1_control_or_a_bidirectional_override_in_a_value_as_text` and
  `a_text_report_prints_an_ordinary_row_member_name_and_value_unchanged` in `lib`.
- **Numbers at the edges.** `numbers_at_the_edges_travel_exactly_through_a_process_target_both_ways`
  and `a_number_past_the_domain_in_a_nested_answer_is_a_fault_not_a_value`
  (`ess-conformance` `adversary_process_target_pass1.rs`).
- **Refusals and `serve`.** `every_refusal_exits_2_and_writes_no_report`,
  `mutate_refuses_the_process_target_naming_the_split_route`,
  `a_relative_program_is_found_in_the_target_dir` and
  `serve_ends_at_end_of_input_and_refuses_a_line_by_kind_only`; in `lib`,
  `serve_answers_an_unknown_op_unsupported_and_returns_at_end_of_input` and
  `serve_refuses_a_request_it_cannot_decode_by_kind_and_line`.

## Format consequences

`FORMAT_RELEASES` (`crates/edge/ess-xtask/src/docs.rs:147`) has `("ess-conformance-target", 1,
None)` after `("ess-conformance-execution", 1, Some("0.53.0"))` (`docs.rs:281-282`): the format is
in no release, and the release that ships it fills the version in, as `("infra-ir", 4, None)` and
`("ess-conformance", 64, None)` do until theirs. The family has no `SUPPORTED_*` constant, so
`cargo xtask docs` reads its row from the quoted literal in non-test source, as `docs.rs:116-121`
describes, and refused without the row.

`website/docs/reference/formats.md` has the row after `ess-conformance-execution/1`, in the table
whose columns are "Document and discriminator | Separate identity | Reader and byte contract". It
says what the format is, who writes and reads it, and its rules:

- **Written and read by** `ess verify conform run --target process` (it writes requests and reads
  answers) and `ess verify conform serve` (the other direction).
- **Closed**: an answer's unknown member outside a value map, missing member, mistyped member, both
  `ok` and `target_error`, member name repeated at any depth, number binary64 cannot hold, line over
  16 MiB, and an implementation `name` or `version` over 256 bytes. Integers are read by value.
- **Open**: a request's reader ignores a member it does not know, at every depth, and a value map
  holds whatever names the specification gives.
- **Escaped and bounded**: strings a target supplies that `ess` shows are escaped for control,
  line-separator and bidirectional characters, with a backslash doubled, and a `target_error`
  string is cut at 4096 bytes.

**Within `/1`**, with no new version:

- a new operation: a target that does not know it answers `unsupported`, which is the honest answer
  for a capability it lacks;
- a new optional request member that a target ignoring it still answers correctly, because
  requests are open;
- a new optional answer member that `ess` reads when present: a `/1` target never sends it.

**A new version, `/2`**, is required when the envelope, the handshake, the line or number rules,
or the fault rules change; when an existing operation's member is renamed, retyped or given another
meaning; when an answer member becomes required; or when a request member is added that a target
must not ignore. Such a change goes into a new operation instead wherever it can. During one
release `ess` may read both, `reads: ["…/1", "…/2"]`, and the target picks.

## Rejected alternatives

- **An HTTP transport.** It is one more contract to version, with ports, a URL and a process `ess`
  does not start. A child process gives the run a defined start, time limit and end. A service that
  listens on HTTP is reached by an adapter process that calls it (fit review item 7).
- **One process per scenario.** Isolation is the target's business through `begin_scenario`
  (`target.rs:176-181`), and a service that takes seconds to start would pay that once per scenario.
  A restart happens only after a fault.
- **A shell command string.** Quoting differs per shell and per platform, the operator's string
  would be expanded where `ess` cannot say how, and the downstream adopter asked for no shell. An
  argv after `--` is exactly what `execve` takes.
- **Letting the target report verdicts.** That is the route the format refuses
  (`running-a-suite.md:173-192`; `target.rs:62-66`). A runner that judges itself already has its
  route, `ess verify conform report --results`, whose report says so (`main.rs:819-831`).
- **A Python emitter** (`--target python` on `synthesize`). One more hand-written runner to keep
  equal to the Rust, Go and TypeScript ones, priced at about 66 KB per language
  (`story:java-conformance-target`), and it covers Python only.

## Known limits

Five things are deliberately left as they are in `/1`.

- **A run in which every scenario times out takes scenarios × `--target-timeout`**, plus a restart
  handshake for each: 17 minutes for the 34-scenario billing suite at the default 30 s (34 × 30 s),
  and 34.2 s measured at `--target-timeout 1`. The guide tells an author to lower the option while
  building an adapter.
- **An answer written early with the next request's id is accepted**, and gains the target nothing.
  The reader holds up to two unasked lines (`target_process.rs:197`); when the next request is
  sent, the first of them is read as its answer, checked against that request's `id` and judged by
  the runner against the specification like any other answer. A target that answers early can be
  right or wrong, and cannot be passed for what it did not answer.
- **`serve` answers a last line with no line feed although `ess` refuses one.** `ess`'s reader
  treats a final line with no `\n` as the end of the stream (`target_process.rs:239-243`), and
  `serve` decodes it and answers (`serve.rs:123-128`). An adapter must end every answer with `\n`.
- **A member name in a diagnostic line prints `"`, `\`, `=` and `, ` as they are**
  (`crates/verify/ess-conformance/src/report.rs:456-473`, `quote_name`), so a row member a target
  names `a = 1, b` reads like two fields. Only control, line-separator and bidirectional characters
  are escaped, so the name stays within its one line and cannot steer a terminal.
- **The report file is written in place** (`crates/edge/ess-cli/src/main.rs:5082`, `:5099`,
  `fs::write`, which truncates and then writes), so a signal landing during that write can leave a
  truncated file where an earlier report was. It was so before this story.

## Open questions

All four are settled by the build.

1. **`--known-failing` with a process target.** The declaration binds the build that ran, and `run`
   binds the `ess` executable's SHA-256 (`known_failures.rs:162-167`), which says nothing about the
   target. Settled: refused, exit 2, with no report (`main.rs:4502-4511`), in this version. Later,
   take the target's build from the operator, as `report --implementation-build` does
   (`main.rs:886-889`).
2. **`serve --target interpreted`.** It would make the model itself a protocol peer, a fake
   backend for any adapter's tests, but the interpreter decides `now` guards only with the runner's
   clock (`main.rs:4326-4334`), which does not cross the wire. Settled: not offered; `ServedTarget`
   has `billing` and `oracle-fixture` only (`main.rs:1256-1263`).
3. **Whether the guide's Python adapter is executed by a test.** Settled: it is.
   `crates/edge/ess-cli/tests/process_target_guide.rs` runs the guide's copy of the adapter,
   wherever `python3` is on `PATH`, and prints a skip otherwise, as `tutorial_page.rs` does for
   the `go` steps; it stays documentation, not a checker.
4. **The 16 MiB line bound and the 5 s end grace.** Neither is derived from the code. Settled: both
   are fixed in `/1`; a view large enough to pass 16 MiB on one line is a suite worth narrowing.
   The other bounds the build added are chosen the same way and not derived from anything: 256
   bytes for an implementation `name` or `version`, 4096 bytes for a `target_error` string, 500 ms
   to wait for an exit status after a stream ends, and, in `serve`, 256 unknown request members and
   4096 reads to find them. A different value is a change a later release may make, and one that
   makes a previously valid answer invalid is a new version (see "Format consequences").
