---
title: Explore command sequences and histories
sidebar_position: 5
description: Random command sequences, concurrent-history checks, client-lane drawings and imported logs.
---

# Explore command sequences and histories

## Explore random command sequences

Generated and authored scenarios are short, fixed paths. The TypeScript and Go packages that
`ess verify conform synthesize --target typescript|go` writes also carry an explorer: seeded random
walks over the commands, driven through the same `Target` you implement for the suite, and checked
after every step against a reference model interpreted from the specification. It finds faults
that only show later in a sequence: a view that drops rows after the fifth, a refusal that still
writes, an identity reused on the fourth create.

```ts
import { assertExplored, explore } from './index.js';

const result = await explore(() => newTarget(), { seeds: 200, steps: 60 });
assertExplored(result);                          // fails on a disagreement or an unreached outcome
assertExplored(result, { allowExcluded: true }); // also accepts outcomes the explorer left out
```

```go
result, err := essconform.Explore(func() essconform.Target { return newTarget() },
    essconform.ExploreOptions{Seeds: 200, Steps: 60})
if err != nil { t.Fatal(err) }
essconform.AssertExplored(t, result, essconform.AssertOptions{})
```

`seeds` sequences run, seeded 1 to `seeds`, each on a fresh target; `steps` is the number of
commands in each (defaults 200 and 60). A failure names its seed, and `seed` runs exactly that one
sequence again. One seed draws the same sequence in both languages.

After every step the explorer compares the outcome, the error, the direct events and every payload
field the specification determines, then every view without parameters over an entity: its row
count, identities, determined fields and `order_by`. A `read_your_writes` view is read once with the
command's consistency token; an `eventual` view is polled until it agrees, up to the eight attempts
an `eventually` step allows. Last, every invariant is evaluated over the model's records; a record
that breaks one is reported as a specification defect, because the guards allowed a sequence the
invariants forbid. A failure is shrunk by removing steps while the shorter trace still fails the
same way, for at most 1,000 replays.

`assertExplored` (`AssertExplored`) fails on a disagreement, on a declared outcome of an included
command that no sequence reached, and on the outcomes of an excluded command. The explorer models a
subset: `when`, `otherwise`, `wrong_state`, `unknown_instance`, `existing_instance` and external
conditions, and the conditions read from the stored row — `when_subject_state`,
`when_state_changes` and `when_subject` (a stored field or a predicate over the stored fields);
`creates` with an observed identity and `moves`/`updates` of a supplied subject; integer, boolean,
string and UUID inputs, their newtypes, enums and structs of them. Anything else is excluded with
the reason in `excluded` — `when_related`, which reads another entity's row, among them — and
accepting that is an explicit `allowExcluded`. Where two guards both hold — which the model admits
over an infinite domain — the draw is reported in `ambiguous` and redrawn rather than decided; a
view filter, invariant or stored-row guard over a field no command set, or one the row holds as
null, is reported in `undetermined` and the command stays in exploration. Neither fails.

A text input is drawn from `""`, `"a"`, `"b"`, the text literals of the command's guards and its
own `example:`; for every `.count` a guard compares it with, say `secret.count < 12`, it is also
drawn at 11, 12 and 13 characters, cut from its example, or from its own name where it has none.
An Integer or enum input also draws its `example:`. These draws are part of what a seed names, so a
failure recorded under a release before beyond10x/ess#221 and #223 replays a different sequence
wherever a command reads its stored row, declares `existing_instance:`, or has a text input with an
`example:` or a `.count` guard: replay it with the release that recorded it.

The model decides which outcome a step expects in the order Entity Runtime and synthesis use:

1. an input-guarded refusal (an outcome with a `when:` and an `error:`), the first declared whose
   guard holds, before the record, its state or an external branch is read;
2. then existence: `existing_instance:` where a creation names an identity a record already
   carries, and `unknown_instance:` where a command reading its stored row names one none carries;
3. then the branches selected by the stored row (`when_subject_state:`, `when_state_changes:`,
   `when_subject:`), the first declared that holds, before any accepting guard; validation refuses
   an accepting `when:` or external branch declared before them that one request can satisfy
   together with them, so declaration order gives the same answer;
4. otherwise the one accepting `when:` that holds, or the default when none does;
5. then `wrong_state`, where the outcome from step 3 or 4 moves the subject from a state no move of the
   command starts from. An outcome that moves nothing answers in every state, an eligible external
   branch included.

A target that answers `wrong_state` to an input a refusal claims disagrees with the model. Before
0.43.0 the explorer answered `wrong_state` before any guard, and so reported a target in this order
as disagreeing.

The model is `ir.json`, the compact IR the suite's `spec_digest` is taken over. The explorer refuses
a package whose `ir.json` does not hash to `suite.json`'s digest; regenerate the package rather than
editing either file.

### Restart the target between commands

Every sequence runs in one process lifetime unless you ask for restarts. An implementation that
mints identities from a counter kept only in its process passes every such sequence, and after a
restart its next creation reuses an identity it has already stored. To check that, give the target
a `restart` method (Go: implement `RestartTarget`) that stops every process of your implementation
and starts it again over the same durable state, and ask for restarts:

```ts
const result = await explore(() => newTarget(), { seeds: 200, steps: 60, restartEvery: 10 });
assertExplored(result);
```

```go
result, err := essconform.Explore(func() essconform.Target { return newTarget() },
    essconform.ExploreOptions{Seeds: 200, Steps: 60, RestartEvery: 10})
```

After every `restartEvery` commands of a sequence the explorer restarts the target and reads every
view again. A row the restart lost fails at the `restart` step of the trace, and a later creation
that mints an identity a record already carries fails as a reused identity. A restart draws no
random number, so a seed runs the same commands with restarts as without. A restart is a check
only once a command has followed it: a restart after the last command of a sequence is followed by
one more drawn command, and only a restart a command followed counts as performed.

The explorer cannot see your processes. Clearing memory inside a process that keeps running is not
a restart, because the counter lives in the process and survives it, and a `restart` that answers
without restarting anything is reported as performed and passes. Both are defects in the target,
and only its author can rule them out.

`restarts` in the result reports the interval and how many restarts were performed. A target
without `restart`, or whose `restart` throws `unsupported` (returns `ErrUnsupported`), is reported
in `restarts.unsupported` and the rest of the exploration runs without restarts. `assertExplored`
fails on it, and on restarts no sequence was long enough to reach, whatever `allowExcluded` says.
Without `restartEvery` the result has no `restarts` and nothing changes. Restarts are
sequential-only: `exploreConcurrent` refuses options carrying `restartEvery`, and Go
`ConcurrentOptions` has no such field.

## Check a concurrent history

A suite and the explorer drive a target one call at a time, so a race between two clients never
happens under them. `check-history` reads an
`ess-history/1` document, one run of several clients with each call's invoke and return instants,
and searches for an order of the calls that the specification's own model accepts, answer for
answer.

```shell-session
$ ess verify conform check-history \
    --path examples/billing \
    --history target/history.json
```

| exit | meaning |
|---|---|
| 0 | `Linearizable`: some order of the calls explains every recorded answer. |
| 1 | `Violation`: no order does. The report names the longest partial order found and a shrunk history that is still a violation. |
| 3 | `Unknown`: the search spent `--budget` model executions (default 1,000,000) first. Unknown is not a pass. |
| 2 | The specification did not load, or the history was refused, for example because it was recorded against another specification. |

The search is split by subject: calls on different instances are checked apart. A call that never
answered may have taken effect or not, and is placed after every other call. A history records no
inputs, so a call is explained by any input the suite would submit for its command. A read of a view
that records its rows is judged at the consistency the view declares: under `read_your_writes` no
client reads a state older than its own last write, and under `eventual` each client's reads converge
once its first `--settle` reads after the last write (default 4) are past. Reads that cannot be judged
are listed with their reason. The same history and budget always print the same report;
`--format json` prints it as JSON. What the check holds a history to, and the limits of that
promise, is [when commands race](../../concepts/ess.md#when-commands-race).

### Draw a history as client lanes

```shell-session
$ ess verify conform web \
    --path examples/billing \
    --history target/history.json \
    --out target/lanes
```

`web --history` checks the history as `check-history` does and writes one `index.html`, or prints
it when `--out` is absent. Each client is a lane, each call a bar from its invoke to its return, and
each call the search placed carries its position in the order found. A history that declares more
than 16 clients draws a lane for each client that made a call and counts the rest in one row. For a violation, the page
marks the call where the search failed. Where one other call explains the failure, it names that
call too, with the state each of the two needed and the state the other order left. Below that is
the shrunk history, drawn the same way. The page carries its stylesheet and no script, so it opens
from disk and fetches nothing. The same history renders to the same bytes. It exits 0 whatever the
verdict; the verdict as an exit status is `check-history`'s.

`--out` replaces the files `ess` owns in that directory, including a scenario player's, so write
history pages and the player to different directories.
### Import a recorded log

A service that logs its calls can be judged from its log. `import-history` reads a JSON Lines log,
one call per line in the log's own shape, through an adapter you write, and writes `ess-history/1`:

```yaml
format: ess-history-adapter/1
fields:
  operation_id: { pointer: /correlation }
  client: { pointer: /request/client }
  command: { pointer: /request/command }
  subject_key: { pointer: /request/subject }
  invoked_at: { pointer: /request/at_ms }
  returned_at: { pointer: /response/at_ms }
  outcome: { pointer: /response/outcome }
  completion:
    pointer: /response/status
    values: { ok: Returned, timeout: Indeterminate }
```

```shell-session
$ ess verify conform import-history --path examples/billing \
    --log calls.jsonl --adapter adapter.yaml --output target/history.json
$ ess verify conform check-history --path examples/billing --history target/history.json
```

Every field is either a JSON pointer or `absent`. Nothing is guessed. If a line lacks a field that
the call cannot be judged without, the import is refused (exit 2) and each such field is named on
its line. Those fields are the client, command, subject, invoke instant and completion, plus the
return instant and outcome of a `Returned` call. Refusals name the log line and the field. Other
fields can be missing, and each case is reported as a `coverage-gap`:

- A missing `operation_id` is given a generated version-8 UUID. No ESS writer uses version 8, so a
  generated ID cannot collide with a carried one.
- `rows` is optional in the adapter. A view read without rows is imported, but `check-history` will
  not judge it.
- The document's `seed` is never carried and is written as 0.

Gaps are printed on stderr. With `--output FILE`, they are also written as a JSON array to
`FILE.gaps.json`. This file is always written; it is never empty because `seed` is always a gap.
A history imported with gaps carries seed 0 and generated IDs by construction, and only the gaps
file records which values were not in the log. An `--output` is refused if it or its gaps file is
the `--log` or `--adapter` file (hard links included) or a file of the `--path` specification.
Both files are written to temporary siblings, and replace existing files only once both writes have
succeeded.

Instants must be unsigned integers, such as epoch milliseconds. Client labels are numbered in the
order they first appear.
