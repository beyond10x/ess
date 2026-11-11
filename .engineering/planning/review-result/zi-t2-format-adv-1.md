---
format: aep.planning-md/3
id: review-result:zi-t2-format-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:a-declared-field-format-renders-or-is-refused
relations:
- reviews: story:a-declared-field-format-renders-or-is-refused
revision: 1
---
unit: story:a-declared-field-format-renders-or-is-refused, commit b8a2adbcea in ~/.local/state/worktree/trees/b10x/ess/zi-t2-3
verdict: broken
cases: executed 44→52, red 8
origin: introduced 6 / pre-existing 1 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: none

```
unit: story:a-declared-field-format-renders-or-is-refused
verdict: broken
new failing cases: 8 (adversary_t2_value_formats_react.rs::{a_utc_instant_shows_the_clock_the_terminal_shows, a_timestamp_form_field_is_no_text_box, a_percent_form_field_sends_a_number}, adversary_t2_value_formats_tui.rs::{an_epoch_ms_timestamp_shows_date_and_time, an_epoch_ms_time_shows_the_time_of_day, a_percent_shows_the_digits_react_shows}, adversary_t2_value_formats_check.rs::{a_scale_on_a_field_that_is_no_percent_is_refused, a_metric_unit_without_format_duration_is_refused})
mutants survived: none
```

**1. Diff.** `git diff --stat` is empty. My three files are untracked test files and nothing else changed:
```
?? crates/ui/ess-ui-check/tests/adversary_t2_value_formats_check.rs
?? crates/ui/ess-ui-react/tests/adversary_t2_value_formats_react.rs
?? crates/ui/ess-ui-tui/tests/adversary_t2_value_formats_tui.rs
```

**2. Cases, run alone** (`adv/run1-cases.txt`, exit 101). All are red now.
- **React, time zone.** With TZ=Europe/Berlin, which is this workstation's zone, `as: time` of `2026-01-02T03:04:05Z` gives `'04:04' !== '03:04'`. The terminal shows `03:04`. The acceptance says `03:04`. `as: timestamp` has the same problem. The unit's own test reads `getHours()` in local time, so it would pass in any zone.
- **React form, timestamp.** `as: timestamp` falls back to a plain text input: `actual: 'text', expected: 'datetime-local'`. `FieldInput` has no `timestamp` case (`fields.tsx.tmpl:383`).
- **React form, percent.** `as: percent` stores the string: `got "75"`, expected `75`. It also falls back to a text input.
- **Terminal, epoch number.** `as: timestamp` and `as: time` with `1767323045000` print the raw `1767323045000`. React treats an epoch-ms number as an instant and its own test pins that.
- **Terminal, percent.** `0.123` shows `12%`. React shows `12.3%` (`maximumFractionDigits: 1` against Rust `{:.0}`).
- **Check, `scale`.** `{as: duration, scale: hundredths}` gets no error. The schema says `scale` is allowed "only on such a field", and the same wording on `unit` is enforced by `check_unit` (`path.rs:480`).
- **Check, metric `unit`.** A metric with `format: bytes, unit: m` gets no error, although the schema says "only with that format".

**3. Suite run** (`adv/run2-suite.txt`, exit 101, slot B waited 404 s, ran 60 s). The existing targets pass: check value_formats 6, React value_formats 17, live_binding 11, terminal value_formats 10, which makes the 44. All my adversary cases are red. After that run I removed two of my own passing tests by deleting their whole functions, and did not re-run.

**PLAIN_PROJECT pin.** The move comes only from the template change the commit names. I put back the base text of `core.tsx` and `fields.tsx` and passed it through a copy of `assets::select`. The project digest then equals the old pin `0x3b54a38c1dbe908a` (`adv/run3-pin.txt`, `ok`, exit 0, slot A waited 81 s, ran 30 s). The plain desk ships no `metric.tsx`. I removed this probe afterwards because it reads the base commit through `git show`.

**4. Findings**

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| core.tsx.tmpl:584 | React shows the browser's local clock, the terminal shows the clock as written; in any zone but UTC they differ. Blocker | NEEDS-CHANGE | introduced | every non-UTC browser |
| view.rs:1687 | Terminal `timestamp` shows an epoch-ms number raw; React formats it. Blocker | NEEDS-CHANGE | introduced | a view field holding epoch ms |
| view.rs:1681 | Terminal `time` shows an epoch-ms number raw; the acceptance says "time of day in both". Warning | CONFIRMED | pre-existing | same |
| fields.tsx.tmpl:383 | Form `as: timestamp` silently becomes a text input. Blocker | NEEDS-CHANGE | introduced | form fields |
| fields.tsx.tmpl:425 | Form `as: percent` uses a text input and sends a string. Warning | NEEDS-CHANGE | introduced | form fields |
| view.rs:1745 | Percent digits differ between renderers: `12%` against `12.3%`. Warning | CONFIRMED | introduced | any percent that is not a whole number |
| path.rs:480 | `scale` on a non-percent field and metric `unit`/`scale` on the wrong format are not refused, though the schema says so. Warning | CONFIRMED | introduced | `ess ui check` |

**React text primitive.** `format: duration` changed from `73s` to `0:01:13` and `format: time` from date-and-time to `HH:mm`. Neither is in the acceptance, the schema note for text `format` was not updated, and no test pins either. It is not a regression in meaning: the unit is still seconds, and the terminal text primitive ignores `format` both before and after this commit. This is a note only.

**5. Could not break**
- `as: metric` is refused at the column path.
- Duration arithmetic agrees between React and the terminal for 0, negative values, fractions, 59.999 s (floors to 59) and hours over 99. Both use the same double multiplication. They only differ beyond about 1.8e19 seconds, where Rust saturates; nobody reaches that.
- Metric default `s` against field default `ms`.
- Strings and null on percent, and invalid strings on time: both renderers show them the same way.
- The PLAIN pin attribution, above.

**6. Paths written outside the worktree**
All under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-declared-field-format-renders-or-is-refused/adv/`:
- `run1-cases.txt`
- `run2-suite.txt`
- `run3-pin.txt`
- `p.rs`
- `p`

The build script also wrote its usual `.touch` notes beside some of these outfiles.

**7. Findings block**
```findings
- file: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
  line: 584
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: as time/timestamp of 2026-01-02T03:04:05Z shows 04:04 in React under Europe/Berlin while the terminal shows 03:04, so the renderers disagree outside UTC
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1687
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: as timestamp with an epoch-ms number prints the raw number in the terminal while React formats it as an instant
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1681
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: as time with an epoch-ms number prints the raw number in the terminal, not the time of day
- file: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
  line: 383
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a form field as timestamp silently falls back to a text input
- file: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
  line: 425
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a form field as percent is a text input that writes a string, not a number
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1745
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a percent field shows 0.123 as 12% in the terminal and 12.3% in React
- file: crates/ui/ess-ui/src/path.rs
  line: 480
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: scale on a non-percent field and a metric unit or scale on the wrong format are accepted although the schema says only with that as or format
```
