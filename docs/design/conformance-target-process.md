# A conformance run drives a target process (`ess-conformance-target/1`)

Status: binding design, 2026-11-21, for
`story:a-conformance-run-drives-a-target-process-in-any-language`. Nothing here is built yet.

**Decision in one line.** `ess verify conform run --target process -- <program> [args…]` starts the
program once, asks it every question of `ConformanceTarget` as one JSON object per line on its
standard input, reads one JSON answer per line from its standard output, and the Rust runner judges
the answers and writes the report/2 a built-in target gets.

This page builds on [running-a-suite.md](running-a-suite.md): a level is a property of the target,
an unsupported scenario fails the run, and the out-of-process adapter is the open gap it names
(`running-a-suite.md:96`). Those decisions are not restated here.

## Where the seam is today

- `ConformanceTarget` is a Rust trait with 29 methods (`crates/verify/ess-conformance/src/
  target.rs:103-584`); the runner is synchronous and calls one at a time (`target.rs:68-76`).
- `ess verify conform run --target` takes `billing`, `oracle-fixture` and `interpreted`
  (`crates/edge/ess-cli/src/main.rs:1207-1215`), and the guide says only built-in targets run
  (`website/docs/guides/verify/runners.md:47`).
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
| `--target-timeout <seconds>` | how long `ess` waits for one answer, an integer from 1 to 3600; default 30. See "Time limits" |

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
| the first handshake fails in any way | the fault kind (below); for a version, the versions this `ess` reads |
| `ess verify conform mutate --target process` | the split route: `--emit DIR`, then `run --target process` per suite writing `report.json` beside it, then `--collect DIR` |

`mutate` shares `ReferenceTarget` (`main.rs:991-992`, `:4533-4558`), so `process` becomes a value
it parses and refuses by name rather than one clap rejects as unknown.

### Exit status of a run

Unchanged. `render_conformance_report` decides it from the report: with `--report-format 2` and
`--strict`, 0 for passed, 1 for failed, 3 for inconclusive (`main.rs:4860-4865`); otherwise 0, 1
and 3 for passed, failed and error (`main.rs:4877-4881`). `--strict` still requires
`--report-format 2` (`main.rs:4265-4267`), still conflicts with `--allow-incomplete`
(`main.rs:798-802`), and still cannot pass a legacy suite with unknown coverage (`main.rs:798`).
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
   answer to the next request and fails it by `id`.
5. **Faults.** See the fault table. A fault ends the current scenario `error`, `ess` kills the
   process group and reaps the child, and every further call in that scenario answers without
   contacting anything: `end_scenario` answers success (the killed process closed the context, and
   the scenario already carries its one error check), every other call answers the same fault.
6. **Restart.** The first call of the next scenario spawns a fresh process and handshakes again.
   The restarted process must name the same `implementation` (name and version) as the first;
   another identity is a failed handshake, because one report names one implementation. When a
   restart's handshake fails, that scenario and every remaining one end `error` with that fault, and
   `ess` does not restart again.
7. **End.** When the run's last scenario has ended, `ess` closes the child's stdin. A target exits
   when its standard input closes; that rule is also what ends a target whose `ess` died, because
   the child's own process group does not receive a terminal's interrupt. `ess` waits up to 5 s for
   the exit, then sends `SIGKILL` to the process group, and reaps the child. The child's exit status
   is not part of the report.

**No child left behind.** On a fault and at the end, `ess` signals the whole process group, so a
server the adapter started in its own group dies with it. `ess` signals the group before it reaps
the child, so the group id cannot have been reused (`waitid` with `NOWAIT` from `rustix`'s
`process` feature, which `ess-cli` already enables, `crates/edge/ess-cli/Cargo.toml`; `NOWAIT`
not verified). A process the adapter moved into another session or group is the adapter's to end.
The released `ess` archives are Unix only (`.github/workflows/package.yml:78-84`).

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

- **Answers are closed.** An unknown member, a missing required member, a duplicated member name
  and a member of the wrong type make the answer invalid, at every depth outside a value map. A
  value map (`input`, `payload`, `fields`, `response`, a row, `context`, `caller`, `compared`) holds
  whatever names the target reports, and the runner judges them against the specification as it
  judges a built-in target's.
- **Requests are open.** A target ignores a request member it does not know. That is what lets a
  later `ess` add an optional request member without a new version (see "Format consequences").
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
in-process target's do. No message lets a target say a scenario passed: there is no verdict member
anywhere in the format, and the status is the runner's fold of its checks (`runner.rs:537-540`).

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
(`crates/specify/ess-primitives/src/node.rs:139-141`), so neither is ever displayed: the process target keeps only `serde_json::Error::classify()` and
`column()`, which on a one-line document is the byte offset. A one-time scenario's checks are
further replaced by the runner's fixed redacted diagnostic (`runner.rs:3311-3323`), unchanged.

| kind | when | `detail` (example) |
|---|---|---|
| `exited` | the child exits while a request is outstanding | `target process exited (status 1) before answering 7 (execute_command)` |
| `closed-stdout` | stdout reaches end of file while a request is outstanding | `target process closed its standard output before answering 7 (execute_command)` |
| `timeout` | no complete line within `limit_ms` | `no answer to 7 (observe_events) within 30000 ms` |
| `line-too-long` | 16 MiB read with no `\n` | `answer to 7 (query_view) exceeds 16777216 bytes` |
| `not-utf8` | the line is not UTF-8 | `answer to 7 (query_view) is not UTF-8 at byte 412` |
| `not-json` | the line is not one JSON value, or is blank | `answer to 7 (query_view) is not JSON at byte 0` |
| `not-an-answer` | valid JSON that breaks the envelope or the operation's answer shape | `answer to 7 (execute_command) does not match its operation at byte 88` |
| `wrong-id` | `id` is not the outstanding request's | `answer names id 9 while 7 (execute_command) is outstanding` |
| `number-out-of-range` | a number `Number::from_json_token` refuses (below) | `answer to 7 (execute_command) holds a number outside the domain at byte 121` |

A handshake answer naming a version `ess` does not read is not a fault kind; it is the refusal
`target speaks ess-conformance-target/2; this ess reads ess-conformance-target/1`.

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
  done once in Rust instead of in each adapter.
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
            "suite_digest": "9e04aa…"}}}
```

```json
{"id": 0, "ok": {"speaks": "ess-conformance-target/1",
                 "implementation": {"name": "tasks-py", "version": "1.0.0"}}}
```

| member | JSON type | required | meaning |
|---|---|---|---|
| `request.reads` | array of string | yes | the versions this `ess` reads, oldest first |
| `request.suite.system`, `.specification_version`, `.suite_version`, `.spec_digest` | string | yes | the suite's `SuiteProvenance` (`src/scenario.rs:466-503`) |
| `request.suite.component` | string | no | present when the suite is scoped to one component (`scenario.rs:499-500`) |
| `request.suite.suite_digest` | string | yes | `AdmittedSuite::digest()`, `sha256-json-bytes/1` of the suite's bytes (`src/admission.rs:180-183`) |
| `ok.speaks` | string | yes | the one version the target speaks; must be in `reads` |
| `ok.implementation.name`, `.version` | string | yes | `ImplementationIdentity` (`target.rs:593-601`) |

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

This adapter is documentation for adopters, not a checker in this repository: nothing here runs it,
and `AGENTS.md`'s rule that everything executable is Rust is untouched. It serves the `tasks`
specification from memory. The guide's tested example comes with the implementation.

```python
#!/usr/bin/env python3
"""An ess-conformance-target/1 adapter for the `tasks` specification, held in memory."""
import json
import sys
import uuid

SPEAKS = "ess-conformance-target/1"
L = "tasks.list."


class Unsupported(Exception):
    pass


class Tasks:
    def handshake(self, request):
        return {"speaks": SPEAKS, "implementation": {"name": "tasks-py", "version": "1.0.0"}}

    def begin_scenario(self, request):
        self.tasks, self.events, self.writes = {}, [], 0

    def end_scenario(self, request):
        self.tasks, self.events = {}, []

    def execute_command(self, request):
        command, values = request["command"], request["input"]
        self.writes += 1
        result = {"consistency": str(self.writes), "direct_events": []}

        def took(outcome, error=None, fields=None, event=None):
            result["outcome"] = {"command": command, "outcome": outcome}
            if error:
                result["error"] = {"error": L + error, "fields": fields}
            if event:
                published = {"event": L + event[0], "payload": event[1]}
                result["direct_events"].append(published)
                self.events.append(published)
            return result

        if command == L + "AddTask":
            if values["priority"] < 0:
                return took("rejected", "InvalidPriority", {"submitted": values["priority"]})
            task_id = str(uuid.UUID(int=len(self.tasks) + 1, version=4))
            self.tasks[task_id] = {"task_id": task_id, "title": values["title"],
                                   "priority": values["priority"], "state": "Open"}
            return took("added", event=("TaskAdded", {"task_id": task_id,
                                                      "title": values["title"]}))
        if command == L + "CompleteTask":
            task = self.tasks.get(values["task_id"])
            if task is None:
                return {}  # an instance never created: no declared outcome covers it
            if task["state"] == "Done":
                return took("already-done", "AlreadyDone", {"state": "Done"})
            task["state"] = "Done"
            return took("completed", event=("TaskCompleted", {"task_id": task["task_id"]}))
        raise Unsupported("this adapter does not serve " + command)

    def query_view(self, request):
        if request["view"] != L + "Tasks":
            raise Unsupported("this adapter does not serve " + request["view"])
        return {"rows": sorted(self.tasks.values(), key=lambda row: row["task_id"])}

    def observe_events(self, request):
        return [event for event in self.events if event["event"] == request["event"]]


def main():
    target = Tasks()
    for line in sys.stdin:  # ends when ess closes standard input
        message = json.loads(line)
        answer = {"id": message["id"]}
        method = getattr(target, message["op"], None)
        try:
            if method is None:
                raise Unsupported("this adapter does not implement " + message["op"])
            answer["ok"] = method(message["request"])
        except Unsupported as why:
            answer["target_error"] = {"kind": "unsupported", "observation": message["op"],
                                      "why": str(why)}
        print(json.dumps(answer), flush=True)


if __name__ == "__main__":
    main()
```

Run as `ess verify conform run --path spec --target process --report-format 2 --report-out
report.json -- ./adapter.py`. Notes for authors: it answers every operation it lacks `unsupported`,
`configure_external_outcome` and `redeliver_event` included, which this specification never reaches;
it returns a consistency token from every command because `Tasks` is `read_your_writes`; its ids are
deterministic, so two runs write identical reports. The `tasks` specification has no `Decimal`; an
adapter for one reads with `json.loads(line, parse_float=decimal.Decimal)` and writes each decimal
as its exact token (see "Numbers"), which the standard library's `json.dumps` does not do for
`Decimal`.

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
  it cannot decode (stderr names the fault kind, never the line). `serve` fits the conform verbs
  (`synthesize`, `run`, `report`, `mutate`, …; `main.rs:612-1107`) and takes its own two-value enum,
  so `interpreted` and `process` are not offered.

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
`ess-recovery-fake` are (`crates/edge/ess-cli/Cargo.toml:67-80`). It wraps `serve` over `Billing`
and injects one misbehaviour per option. It costs one more link of `ess-conformance` in the
`ess-cli` test build.

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

- `crates/verify/ess-conformance/src/process.rs` (and submodules): `TARGET_FORMAT`, `READS`, the
  envelope, a request writer with exact numbers, an answer reader over `Node::from_json_exact` that
  refuses duplicate member names (`serde`'s map visitors keep the last value, not verified for this
  build), the fault kinds, a `Peer` trait (send a line, receive a line within a limit, kill and
  reap, respawn), `ProcessTarget<P: Peer>` implementing `ConformanceTarget` with the default
  fallbacks above, and `serve`.
- `crates/edge/ess-cli/src/target_process.rs`: the `Peer` over a child process. It resolves the
  program against `--target-dir`, spawns it with `process_group(0)`, reads stdout on a thread into
  bounded lines, waits with `recv_timeout`, and signals the group with `rustix::process` before
  reaping.
- `crates/edge/ess-cli/src/main.rs`: `ReferenceTarget::Process`, `Run`'s `command`,
  `target_dir` and `target_timeout`, the refusals, the dispatch at `:4321-4336`, the `mutate`
  refusal, and the `Serve` verb.
- `serde` derives. Requests gain `Deserialize`, for `serve`: `ScenarioContext`,
  `SemanticCommandRequest`, `AbsentInputRequest`, `SemanticViewRequest`, `EventObservationRequest`,
  `Deadline`, `InstantMark`, `ElapsedObservationRequest`, `RedeliveryRequest`,
  `EventDeliveryRequest`, `OrderedScanRequest`, `InvocationObservationRequest`, `AwaitRequest`,
  `ExternalOutcomeControl`. Five types have no `serde` at all and gain both:
  `EntitySetupRequest` (`target.rs:620`), `NowHold` (`target.rs:944`, its instant as RFC 3339
  text), `periodic::Open` (`periodic.rs:68`), `periodic::Observe` (`periodic.rs:87`) and
  `reading::ReadingObservationRequest` (`reading.rs:99`). Results gain decoders through the exact
  reader: `SemanticCommandResult`, `DeclaredErrorValue`, `ObservedEvent`, `SemanticViewResult`,
  `ObservedInvocation`, `AwaitAnswer`, `TargetError`. `ElapsedObservation`, `OrderedScan`,
  `ImplementationIdentity`, the periodic results and `ReadingEvidence` already derive both.
  `InstantName`, `StateName`, the semantic refs and `BindingRef` already have hand-written
  `Deserialize` (`scenario.rs:1672-1677`, `crates/specify/ess-domain/src/entity.rs:124`,
  `refs.rs:94-101`, `:165-172`).
- `crates/edge/ess-cli/tests/process_target.rs` and the test-only `ess-conformance-peer`.
- Documentation: `website/docs/guides/verify/runners.md` gains "An implementation in any language"
  with the adapter; `website/docs/reference/formats.md`, `FORMAT_RELEASES`, `CHANGELOG.md`.

## Tests the acceptance requires

Each line is one acceptance statement of the story, in its order. CLI tests live in
`crates/edge/ess-cli/tests/process_target.rs`; `peer` is `ess-conformance-peer`.

1. Design page: this page; no test.
2. `a_served_billing_reference_reports_what_the_built_in_one_reports`: runs
   `suites/generated/billing/suite.json` with `--target billing` and with `--target process --
   $CARGO_BIN_EXE_ess verify conform serve --target billing`, both `--report-format 2
   --report-out`, and compares the two reports' `outcomes` and `counts`.
3. `strict_exits_nonzero_unless_every_scenario_passed` and
   `each_non_passing_scenario_is_listed_under_its_own_category`: on a coverage suite from
   `--path examples/billing --suite-format 5` (strict cannot pass unknown coverage, `main.rs:798`),
   `serve` exits 0 under `--strict`, and a peer with one failed, one errored and one unsupported
   scenario exits 1 and lists each under `failed`, `error` and `unsupported`.
4. `one_wrong_outcome_fails_that_scenario_and_no_other`: `peer --wrong-outcome
   billing.invoice.CreateInvoice/outcome/accepted` answers `rejected` for that scenario's first
   command; `outcomes.failed` is exactly that id and every other scenario keeps the built-in run's
   status.
5. One test per fault, each asserting that scenario is `error`, the scenarios after it keep their
   built-in status (the restart works), and the peer's pid and a grandchild's pid it wrote to
   `--pid-file` are gone once `ess` exits: `an_exit_mid_scenario_is_an_error` (`--exit-in`),
   `a_line_that_is_not_an_answer_is_an_error` (`--garbage-in`),
   `an_answer_to_another_request_is_an_error` (`--wrong-id-in`),
   `an_answer_past_the_time_limit_is_an_error` (`--stall-in`, with `--target-timeout 1`). Beside
   them: `a_failed_restart_errors_every_remaining_scenario` (`--fail-restart`) and, in
   `ess-conformance`, `a_fault_never_quotes_the_answer`, which feeds lines holding a marker secret
   for every fault kind and asserts that no fault text contains it.
6. `an_unsupported_answer_is_unsupported_with_its_reason`: `peer --unsupported query_view` makes
   each scenario that reads `unsupported`; the detailed `--format json` run carries the peer's
   `why` in those checks' diagnostics, and the exit status is 1.
7. `two_runs_write_identical_reports`: two runs against `serve --target billing` write equal
   report/2 bytes.
8. `an_unread_protocol_version_is_refused_before_any_scenario`: `peer --speaks
   ess-conformance-target/2` exits 2, stderr names `ess-conformance-target/1`, no report is
   written, and the peer recorded no `begin_scenario`.
9. `task test-xtask`: the `FORMAT_RELEASES` row, the `formats.md` row and the runners guide
   section. Also `cargo test -p ess-conformance --lib process` for the codec: a round trip of every
   operation, exact numbers past binary64, refusal of unknown and duplicate answer members.

## Format consequences

`FORMAT_RELEASES` (`crates/edge/ess-xtask/src/docs.rs:147`) gains `("ess-conformance-target", 1,
None)` after `("ess-conformance-execution", 1, Some("0.53.0"))` (`docs.rs:281`), and the release
that ships it fills the version in. The family has no `SUPPORTED_*` constant, so its row is read
from the quoted literal in non-test source, as `docs.rs:116-121` describes.
`website/docs/reference/formats.md` gains a row after `ess-conformance-execution/1`
(`formats.md:394`), in the table whose columns are "Document and discriminator | Separate identity |
Reader and byte contract" (`formats.md:353`):

> | `ess-conformance-target/1` (handshake `speaks`) | The implementation identity a target names in
> its handshake, and the suite it is asked to run (`suite_digest`, `spec_digest`, `suite_version`) |
> Not yet in a release. The line protocol of `ess verify conform run --target process`: `ess`
> starts the command given after `--` and exchanges one JSON object per line on its standard input
> and output, request and answer in lockstep under a repeated `id`. Operations are
> `ConformanceTarget`'s methods in snake_case with their Rust `serde` members; an operation the
> target does not implement, or does not know, is answered `unsupported`. Answers are closed and
> their numbers exact. A fault (the process exits, an invalid line, a wrong `id`, a line over
> 16 MiB, no answer within the limit) ends the scenario `error` and restarts the process. Served by
> `ess verify conform serve`. [Design](…) [Source][process] |

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

## Open questions

1. **`--known-failing` with a process target.** The declaration binds the build that ran, and `run`
   binds the `ess` executable's SHA-256 (`known_failures.rs:162-167`), which says nothing about the
   target. Default: refused, exit 2, in this version. Later, take the target's build from the
   operator, as `report --implementation-build` does (`main.rs:886-889`).
2. **`serve --target interpreted`.** It would make the model itself a protocol peer, a fake
   backend for any adapter's tests, but the interpreter decides `now` guards only with the runner's
   clock (`main.rs:4326-4334`), which does not cross the wire. Default: not in this version.
3. **Whether the runners guide's Python adapter is executed by a test.** `tutorial_page.rs` runs
   the Start-here pages' `go` steps wherever `go` is on `PATH` and skips otherwise (`AGENTS.md`,
   Gate). Default: the guide's adapter runs the same way, wherever `python3` is on `PATH`; it stays
   documentation, not a checker.
4. **The 16 MiB line bound and the 5 s end grace.** Neither is derived from the code. Default: keep
   both fixed in `/1`; a view large enough to pass 16 MiB on one line is a suite worth narrowing.
