# Calendar-window guards (source `ess/22`)

beyond10x/ess#244, part (b): "a calendar window: only Monday to Thursday, 08:00 to 16:00 in a named
time zone". Story `feature-request-244`; this page resolves
`decision-blocker:calendar-window-time-zone`. Part (a), elapsed time since a stored instant, shipped
as family F A2/A3 (`expression-family-source22.md`).

## Decision: UTC or a fixed offset, never a zone database

Coordinator decision, 2026-10-05. A calendar window is evaluated in **UTC or a fixed UTC offset**
written in the guard (`Z`, `+01:00`, `-05:30`). A named IANA zone (`Europe/Berlin`) and any other zone
name (`UTC`, `CET`) is **refused by name at source**, with a message saying to write a fixed offset.

The reason is the three evaluators. Rust, Go and TypeScript decide a window with the same few lines
of integer arithmetic over an instant, and no data file. A zone rule needs a zone database, and the
three lanes would have to agree on its version, on every past and future rule change, and on what a
wall-clock time that a daylight-saving change skips or repeats means. None of that is needed to
answer "is this instant inside the window", and all of it is a source of disagreement between lanes
that a suite cannot see.

The consequence, stated here and in the issue's closing note: **a window does not follow daylight
saving.** `08:00 to 16:00 at +01:00` is 07:00 to 15:00 UTC all year. A rule meant in Berlin local time
is written as two guards or two offsets by the specification's owner, or accepted as one hour off for
half the year. DST-shifting local windows are out of scope; adding them later is a new construct with
a pinned zone-data dependency, not a reinterpretation of this one.

## Source

Structured form only, inside any command-guard predicate. There is no compact spelling: a window is
five values, and a compact one would be a second grammar.

```yaml
when:
  window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}
```

| key | value | rule |
|---|---|---|
| `at` | `now`, or a fact path | the instant tested. `now` is the decision's current time; a path must resolve to a `Timestamp` (through newtypes and `Optional`) |
| `days` | a list of `mon`, `tue`, `wed`, `thu`, `fri`, `sat`, `sun` | at least one, no repeats; any order, written back Monday first |
| `from` | `"HH:MM"`, `00:00`–`23:59` | inclusive start of the window on each listed day |
| `to` | `"HH:MM"`, `00:01`–`24:00` | exclusive end. `24:00` is the end of the day; `00:00` is refused naming `24:00`, its one spelling |
| `offset` | `Z` or `±HH:MM`, at most `14:00` either way | required. `+00:00` reads as `Z`; `-00:00` is refused (RFC 3339: unknown offset) |

Exactly those five keys. A key `zone`, `tz`, `timezone` or `time_zone`, and an `offset` that is a
zone name, are refused naming the fixed-offset spelling. `from` equal to `to` is refused: it is either
empty or the whole day, and the whole day is `from: "00:00", to: "24:00"`. A time that is not text (a
YAML 1.1 sexagesimal number) is refused asking for the quoted spelling.

`window:` is the window only when its value is a mapping carrying `at`. Any other `window:` entry
remains a constraint on a fact named `window`, as in every earlier format, so no admitted document
changes meaning.

The canonical form is the same mapping with every key written: `days` Monday first, `from`/`to` as
`HH:MM`, `offset` as `Z` or `±HH:MM`, `at` as `now` or the path. The IR carries it through
`Predicate::to_node`, so a model without a window keeps its bytes.

### Where it is admitted

Where the current-time operand is admitted from `ess/22`: a command outcome's `when:` over its input
(alone or beside a held state, a state change, a stored field, a `when_subject:` or an external
cause), its `when_subject:` predicate, and the predicate of an identity-addressed `when_related:`. `at`
reads what the site reads: an input field in `when:`, a stored field or `input.<path>` in a stored
row's predicate, a binder inside a quantifier.

Anywhere else — an entity, type or newtype invariant, a view filter, a selection, a set-effect
filter, an aggregate's `where`, a binding's `where` — it is refused as `type_mismatch` naming the
places it is admitted, whether `at` is `now` or a fact. An authored scenario's `satisfies:` is refused
by name too (`InvalidPredicate`, naming the window). That keeps windows out of every suite predicate
(`satisfies`, observed selections), so no suite format changes.

`at: now` where the site also names a root or a binder `now` is refused: the word would mean two
things. Rename the field or the binder.

## Semantics

For a known instant `t` (UTC seconds since the epoch, fraction dropped), the guard's offset `o`
(seconds east of UTC), `local = t + o`:

- `day(local)` is the weekday of the civil date of `local` (1970-01-01 is a Thursday);
- `tod(local)` is the second of that day, `0`–`86399`;
- with `from < to` (minutes): inside iff `day ∈ days` and `from·60 ≤ tod < to·60`;
- with `from > to`, the window **crosses midnight and belongs to the day it opens**: inside iff
  (`day ∈ days` and `tod ≥ from·60`) or (`previous(day) ∈ days` and `tod < to·60`). So
  `days: [fri], from: "22:00", to: "02:00"` holds Friday 22:00 to Saturday 02:00, and not Friday 01:00.

Dropping the fraction is exact: every boundary is a whole minute, so `15:59:59.999` is inside a window
ending `16:00` and `16:00:00.001` is not.

The instant is compared, never its spelling: `2020-01-06T02:30:00-05:00` is 07:30 UTC and 08:30 at
`+01:00`, whatever offset it was written with.

### Unknown

Kleene, as every predicate:

- the fact at `at` unobserved, or an `Optional` on its path absent: `Unknown`;
- the text at `at` not an RFC 3339 instant: `Unknown`, with the evaluator's ill-defined note;
- `at: now` and no clock given to the evaluator (`FactSource::now` is `None`): `Unknown`.

`Unknown` never selects a branch. Negation keeps it.

### One decision instant (E-U4) and stored rows (E-U5)

`at: now` reads exactly the instant every other `now` of the decision reads: the command edge's one
frozen reading, handed to the evaluator through `WithNow`. A window in a `when_subject:` or
`when_related:` predicate reads the stored row from the pre-outcome snapshot, like every other stored
read of that decision. No window reads a clock of its own, rereads per row, or reads a host time
zone. The `ess-history/2` `decision_time` E-U4 records is that one reading; a window adds nothing to
the history format.

## Formats

| surface | change |
|---|---|
| source | `ess/22`, no new major. Below it the window is refused naming `ess/22`: while the document is read (the reader of an older source has no `window` form) and again at assembly for a source with no format the reader knew |
| IR | the canonical mapping, only where written; every other predicate keeps its bytes |
| suite | none: a window never reaches a suite predicate (above). The Rust, Go and TypeScript readers still count one (`expression_format::reads`, `admitPredicateVersion`), so a hand-built suite carrying a window is held to `/40`/`/41` like the rest of the persisted family F vocabulary, and one relabelled `/39` is refused before any step runs |
| history | none: `ess-history/2`'s `decision_time` is the instant `at: now` reads |

## Synthesis

**A window over an input fact** is witnessed both ways at each side of each boundary. The instants are
built on a reference week (Monday 2020-01-06 through Sunday 2020-01-12, in the guard's offset). For
every listed day and every day following a listed one: `from − 1s`, `from`, `to − 1s` and `to` (`to`
on the next day for a midnight-crossing window), each classified inside or outside by the evaluator.
The plain witness search tries them as candidates, written in UTC with `Z`. The boundary rows send
the guarded branch every inside instant and the default branch every outside instant where the
conjunct holding the window alone refuses, every other conjunct held — for a window at any depth of
the guard, under `any:`, `all:` and `not:` alike, so the two-interval form `any:` of two windows is
witnessed at every boundary of both. A boundary row spells its instant at a whole-hour offset under
which the written clock, read as UTC, falls on the other side of the window
(`CalendarWindow::spelled_against`): a target that drops an instant's own offset, or compares the
written clock, decides it otherwise. A boundary row asserts only the branch and its error, so the
spelling is never compared with an echo of the value.

**A window over the command's input read from a stored row's predicate** (`at: input.<path>` in a
`when_subject:` or `when_related:` predicate) steers the input as a window over that input path, as an
ordering of the input does there; the stored row's further rows send the input at each deciding
instant (below). A wrong-state scenario on such a command is sent an input inside the window, so the
guarded branch is selected and answered by the wrong state whichever a target takes first.

**A window over a stored fact** is arranged through the creator input that writes the field, with the
same candidates; where no creator input carries it, the existing arrangement refusal stands. Each
branch is further witnessed on rows holding the field at each of the window's deciding instants
(`CalendarWindow::deciding_instants`, bounded by the existing limit of further rows per branch):
`from` on the first listed day, `to` after it, `from` on each unlisted day, a second before the first
`from`, a second before the last `to`, and `from` on every other listed day, each row kept for the
branch it selects. An inclusive `to`, an extra or a dropped day, a moved `from` and an ignored offset
each decide one of them otherwise. A value
chosen at a window's boundary is a fixed instant: where the same stored field is also ordered against
`now` by another guard, it is sent as that instant and never as a `now_offset`, and a field held to a
window and ordered against `now` together is refused by name (the existing fixed-instant-beside-`now`
refusal), because a literal 2020 instant reads differently against `now` at the synthesis reference
than at any run.

**A window over `now`** is refused by name (`NoWitness`, naming the window and the clock), and every
scenario sending the command is dropped; a refusal an earlier stage already made for such a scenario
is restated with that reason. Such a command orders nothing for any other command's arrangement,
because no scenario sends it. The suite holds no clock: a target decides `now` by its own,
and no suite step places that clock on a weekday or an hour. Synthesizing a window over the runner's
wall clock would be a suite whose verdict depends on the day it runs. Its acceptance is the scripted
command clock of the interpreter and generated targets, below. A target-clock step in the suite would
lift the refusal; it needs a suite major and is not taken here.

## Lanes

| lane | obligation | decisive control |
|---|---|---|
| shared Rust evaluator (`ess-primitives`) | parse, canonical form, display, evaluate as above | each side of each boundary; two offsets, one instant, two days; a different-offset spelling; a crossing window on both days; Unknown rows; refusals; shared vectors |
| domain / compiler | one checker: `ess/22` gate, site admission, `at` type, `now` ambiguity | below-`ess/22`, wrong site, non-Timestamp `at`, named zone, `now` collision |
| native interpreter | evaluate with the decision's instant | scripted command clock at each boundary side, two offsets, one read per decision with a stored-row `now` beside the window |
| Rust runner, Go and TypeScript suite runtimes | run the synthesized suite of a window over input and stored instants; identical verdicts | the healthy interpreter passes; faulty targets fail named scenarios in all three |
| Go and TypeScript predicate readers | parse and evaluate a window the IR carries (explorers) | the shared vector file |
| generated Rust and Go | the command stays owed, naming the calendar window; no guard is rendered | the owed reason names the window |
| Entity Runtime | `CalendarWindowUnsupported`, never lowered | lowering refusal |
| semantic diff | a window added, removed or edited is a behaviour change (canonical `to_node`) | diff control |
| mutation | `guard-boundary` moves a window's `from` a minute later, and has no site where `from` cannot move (23:59, or onto `to`); `guard-outward` moves its `to` a minute later | both mutants of a window over an input, at any depth of the guard, killed by the synthesized suite; one over `now` is `Unwitnessed`, never `Survived`; no mutant is the unchanged window |
| suite readers | a window under `/40` is admitted by the Rust, Go and TypeScript readers alike, and refused below | relabelled `/39` refusal in each |
| docs and schema | predicates reference, Entity Runtime lowering table, schema description | site examples |

### Faulty targets

Each is the interpreter running the model with one fault written into the window, the same position a
wrong implementation is in:

| fault | written as |
|---|---|
| local host time | the guard's offset replaced by a host zone, `-07:00` |
| offset ignored | the offset replaced by `Z` |
| `to` inclusive | `to` one minute later: on every whole second up to and including `to` it decides as an inclusive `to` does |
| midnight crossing attributed to the instant's own day | the window split into `from`–`24:00` and `00:00`–`to` on the same listed days |

Each must fail a synthesized scenario under the Rust runner and the Go and TypeScript runtimes, and
the healthy model none; for a window over `now`, the first three are written into the `Deploy`
window alone and decide otherwise than the healthy model at the interpreter's scripted command
clock. The shared vectors hold the same four, and a reader of the written clock instead of the
instant, to disagree with a vector in Go and TypeScript.

## Implementation seams (2026-10-05)

- Construct, reader, canonical form, evaluation and the synthesis boundaries:
  `crates/specify/ess-primitives/src/window.rs` (`CalendarWindow::parse_mapping`, `to_node`,
  `contains`, `evaluate`, `boundaries`); `Predicate::Window`, `reads_window` and `windows` in
  `predicate.rs`; `Rfc3339Instant::epoch_seconds` and `CivilDate::from_epoch_day` in `time.rs`.
- Checker: `Checker::window` in `crates/specify/ess-domain/src/expression.rs`.
- Synthesis: the `Predicate::Window` arm of `collect_literals` (`witness.rs`); `window_bounds` in
  `boundary_inputs` and the restated refusal after `now_offset::install` (`synthesize.rs`);
  `clocked_window`, `window_bounds` and the window arms of `Orderings` in `now_offset.rs`.
- Mutation: `is_ordering`, `swap_strictness`, `outward`, `window_from_later` and `window_to_later`
  in `mutate.rs`.
- Go and TypeScript: `windowPredicate`/`parseWindow` in `go/predicate.go`, `CalendarWindow`/
  `parseWindow` in `ts/predicate.ts`, and the `window` arms of `admitPredicateEnvelope` and
  `admitPredicateVersion` in `go/runtime.go` and `ts/runtime.ts`.
- Generated behaviour: the `Predicate::Window` arm of `determined::supported` (`ess-synth`);
  generated invariants refuse it by name (`rust/invariant.rs`, `go/invariant.rs`).
- Entity Runtime: `LoweringCode::CalendarWindowUnsupported` (`lib.rs`, `subset.rs`).

## Not in this design

- Named zones and daylight saving (above).
- A window in an invariant, a view filter, a selection or any suite predicate.
- A compact spelling, seconds in `from`/`to`, more than one interval per window (write `any:` of two
  windows), dates or holidays.
- Generated Rust and Go evaluating a window, and Entity Runtime lowering one.
- A suite step that sets a target's clock, which synthesizing a window over `now` would need.
