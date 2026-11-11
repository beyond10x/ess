---
format: aep.planning-md/3
id: review-result:zi-t2-format-adv-2
kind: review-result
status: active
title: Turn 2 adversary pass 2 on story:a-declared-field-format-renders-or-is-refused
relations:
- reviews: story:a-declared-field-format-renders-or-is-refused
revision: 1
---
```
unit: story:a-declared-field-format-renders-or-is-refused, commits b8a2adbcea + ea2c1a30b8 (HEAD) in ~/.local/state/worktree/trees/b10x/ess/zi-t2-3
verdict: broken
cases: executed 52→64, red 7
origin: introduced 6 / pre-existing 1 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: none
```

```
unit: story:a-declared-field-format-renders-or-is-refused
verdict: broken
new failing cases: 7 (adversary_t2_value_formats_p2_react.rs::{a_zoneless_instant_shows_the_clock_the_terminal_shows, a_date_shows_the_day_its_timestamp_shows, a_zoneless_timestamp_with_seconds_writes_back_an_instant, an_epoch_ms_timestamp_writes_back_a_number}, adversary_t2_value_formats_p2_tui.rs::{a_percent_on_a_tie_rounds_as_react_does, a_date_only_timestamp_shows_midnight, an_http_date_time_shows_the_time_of_day})
mutants survived: none
```

**1. Diff.** `git diff --stat` is empty. The only changes are three untracked test files:
```
?? crates/ui/ess-ui-check/tests/adversary_t2_value_formats_p2_check.rs
?? crates/ui/ess-ui-react/tests/adversary_t2_value_formats_p2_react.rs
?? crates/ui/ess-ui-tui/tests/adversary_t2_value_formats_p2_tui.rs
```

**2. Cases, run alone** (`adv2/run1-cases.txt`, exit 101, slot B waited 1 s, ran 45 s)

- **React, ISO text without a time zone.** Under TZ=Europe/Berlin, `as: time` of `2026-01-02T03:04:05` fails with `actual: '02:04', expected: '03:04'`. JavaScript reads a text without a zone in the browser's local zone, and the correction then prints it in UTC. The terminal reads the same text as UTC and shows `03:04`; its own doc comment in `view.rs` says it does this "as the React target shows it". Before the correction both showed `03:04`.
- **React, `as: date` against `as: timestamp`.** In Berlin, `as: date` of `2026-01-01T23:30:00Z` fails with `actual: '1/2/2026', expected: '1/1/2026'`. React's `as: timestamp` of the same value shows Jan 1, 23:30, and so does the terminal's `as: date`. React's date is still in local time.
- **React form, timestamp with seconds.** A held `2026-01-02T03:04:05` is put into the datetime-local box with its seconds. Writing it back gives `"2026-01-02T03:04:05:00Z"`, which is not a valid date.
- **React form, epoch number.** A held `1767323045000` is written back as `"2026-01-02T03:04:00Z"`: `actual: 'string', expected: 'number'`. The 5 seconds are lost as well.
- **Terminal, percent on a tie.** 0.1225 shows `12.2%` and 0.0025 shows `0.2%`. React shows `12.3%` and `0.3%`, measured with node 22. Rust's `{:.1}` rounds the binary value half to even; `Intl` rounds the decimal value half away from zero.
- **Terminal, date-only timestamp.** `as: timestamp` of `2026-01-02` shows `2026-01-02` with no time. React shows `01/02/2026, 00:00`.
- **Terminal, HTTP-style date.** `as: time` of `Fri, 02 Jan 2026 03:04:05 GMT` shows an empty cell: the fallback splits the text at the `T` of `GMT`. React shows `03:04`.

Four of my new tests pass and stay as pins, because the suite covered none of this:
- `instants_convert_as_javascript_does`: negative epoch, 1899/1900, the 2024 leap day, `+05:45` with fractional seconds, `-00:00`, `+0545`, no zone, the year 275760, and the day at a day edge.
- `the_form_round_trip_keeps_the_instant_and_the_share`
- The three check cases: a form field's `scale`, the `UNMAPPED:` guards, and header-metric `unit`/`scale`.

**3. Suite run** (`adv2/run2-suite.txt`, exit 101, slot B waited 71 s, ran 105 s).
- Command: `test -p ess-ui-check -p ess-ui-react -p ess-ui-tui --locked --test value_formats --test live_binding --test adversary_t2_value_formats_{check,react,tui} --test adversary_t2_value_formats_p2_{check,react,tui} --no-fail-fast`
- Unit targets pass: check value_formats 6, React value_formats 17, live_binding 11, terminal value_formats 10.
- All 8 of pass 1's cases pass now (check 2, React 3, terminal 3).
- My p2 targets: React 1 passed / 4 failed, terminal 1 passed / 3 failed, check 3 passed.
- That is 64 cases in total, 7 red.

**4. Findings** (they cover HEAD ea2c1a30b8)

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| core.tsx.tmpl:579 | An ISO text without a zone is read as local time, then shown in UTC; the terminal shows the clock as written. Blocker | NEEDS-CHANGE | introduced | A backend that sends naive datetimes, in any browser outside UTC |
| fields.tsx.tmpl:412 | Editing a held value that has seconds writes `…:05:00Z`, which is not a date. Blocker | NEEDS-CHANGE | introduced | An edit form pre-filled from a record with seconds and no zone |
| core.tsx.tmpl:585 | React `as: date` uses local time while `time`/`timestamp` and the terminal's `date` use UTC. Warning | NEEDS-CHANGE | introduced | Any instant near midnight UTC |
| fields.tsx.tmpl:412 | An epoch-ms number is written back as an ISO text and loses its seconds. Warning | CONFIRMED | introduced | A command input that holds epoch ms |
| view.rs:1749 | Percent ties round differently from React (`12.2%` against `12.3%`). Warning | CONFIRMED | introduced | Shares with 4 decimals, such as 0.1225 |
| view.rs:1695 | Terminal `as: timestamp` of a date-only text shows no time; React shows 00:00. Warning | CONFIRMED | introduced | A timestamp field that holds a date-only value |
| view.rs:1686 | Terminal `as: time` falls back to splitting at any `T`: an HTTP date shows empty. Backlog | CONFIRMED | pre-existing | A backend that sends HTTP-style dates |

Fix directions; I applied none:
- **Terminal, date-only:** treat a date-only text as midnight UTC.
- **Terminal, percent:** round half away from zero on the shortest decimal form of the value.
- **React, display:** read a text without a zone as UTC by appending `Z` before `new Date`, and give `date` a `timeZone: "UTC"`.
- **React, form timestamp:** cut the box's value to `YYYY-MM-DDTHH:mm` (or keep the seconds and add only `Z`), and write a number back when the held value was a number.

**5. Could not break**
- The terminal's day-to-date arithmetic, on all ten pinned instants: pre-1970, 1900 (no leap year), 2024-02-29, +05:45, -00:00, +0545, fractional seconds, the year 275760.
- The percent box: 0.123 shows `12.3` and writes back exactly 0.123; typing 0.07 writes 0.0007; an empty box writes null; the hundredths scale. Non-numeric typing is unreachable, because a real number box gives `""`.
- The datetime-local box with a zoned value: `+05:45` shows the UTC minute and writes back the same instant; an empty box writes null.
- Check refusals:
  - A form field with `as: number, scale` is refused at `…/fields/rate/scale`.
  - A form field with `as: percent, scale` is accepted.
  - An `UNMAPPED:` `as` keeps its `scale`/`unit`, and an `UNMAPPED:` metric `format` keeps them too.
  - Header metrics with percent+`unit`, duration+`scale`, or no format+`unit` are refused at the key.

**6. Paths written outside the worktree**
All under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-declared-field-format-renders-or-is-refused/adv2/`:
- `probe.js`
- `probe2.js`
- `probe.rs`
- `probe` (rustc output)
- `run1-cases.txt` and `run2-suite.txt`, with their `.touch` notes from the build script

Two of five cargo-free probes; 2 of the 4 allowed cargo runs were used.

**7. Findings block**
```findings
- file: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
  line: 579
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an ISO text without offset is parsed in the browser zone and then shown in UTC, so 2026-01-02T03:04:05 shows 02:04 in Berlin while the terminal shows 03:04
- file: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
  line: 412
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a timestamp form field holding a zoneless value with seconds writes back 2026-01-02T03:04:05:00Z, which is no valid instant
- file: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
  line: 585
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: React as date stays in the browser zone while time, timestamp and the terminal date use UTC, so 2026-01-01T23:30:00Z is 1/2/2026 as a date and 01/01/2026 23:30 as a timestamp
- file: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
  line: 412
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a timestamp form field holding epoch milliseconds writes back an ISO string truncated to the minute instead of a number
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1749
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the terminal rounds a percent tie half to even on the binary value (0.1225 shows 12.2%) while React's Intl rounds half away from zero (12.3%)
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1695
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the terminal shows a date-only timestamp as the bare date while React shows it as UTC midnight with 00:00
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1686
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the terminal time fallback splits any text at its first T, so an HTTP date shows an empty cell where React shows 03:04
```
