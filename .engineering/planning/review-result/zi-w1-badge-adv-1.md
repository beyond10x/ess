---
format: aep.planning-md/3
id: review-result:zi-w1-badge-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-badge-field-takes-a-tone
relations:
- reviews: story:a-badge-field-takes-a-tone
revision: 1
---
unit: story:a-badge-field-takes-a-tone (beyond10x/ess#723), tree zi-t1-3 at HEAD a6e7af09d6 plus 2 uncommitted test files
verdict: broken
new failing cases: 3 (crates/ui/ess-ui-check/tests/field_tone.rs::adv_a_field_tone_by_written_as_a_map_with_descending_bands_is_refused, ::adv_a_field_tone_by_written_as_a_map_with_an_unknown_tone_is_refused, crates/ui/ess-ui-react/tests/field_tone.rs::adv_a_band_from_minus_infinity_tones_alike_in_react_and_the_model)
mutants survived: none (none tried; the 4 cargo runs went to the red cases and the suite runs)

- **Diff:** `crates/ui/ess-ui-check/tests/field_tone.rs | 34 +`, `crates/ui/ess-ui-react/tests/field_tone.rs | 50 +`. Only test files changed, nothing committed. Cases executed 10→13, 3 red. Each case was run alone first (`-- adv_`), then the suite.
- **1. Blocker, in the acceptance.** On a field, `tone_by: {map: {bands: [{from: 80, …}, {from: 0, …}]}}` loads and passes `ess ui check`. The schema says `{ref: tone_map}` and the design page says "not a `{value, tones}` record". The loader allows the map form because `FieldToneBy` (`crates/ui/ess-ui/src/model.rs:3339`) accepts `{map}`. `tone_bands` (`crates/ui/ess-ui-check/src/style.rs:397`) only checks badges and icons, so these bands are never checked. Run `ESS_RESERVE=4G build.sh adv/check-adv.txt … test -p ess-ui-check --locked --test field_tone -- adv_`; it fails at field_tone.rs:134 (exit 101), and the only finding is a `fixture_per_view` warning. Fix: the loader refuses a field `tone_by` that is not a name.
- **2. Backlog, same cause.** `tone_by: {map: {done: purple}}` on a field also passes, because nothing checks the tones of `Field.tone_by.map`. React then outputs `ui-tone-purple`. Fails at field_tone.rs:149, in the same outfile.
- **3. Backlog, outside the acceptance.** React and the model/TUI disagree on a band at `from: -.inf`. The loader accepts it and so does `tone_bands`. `ts::literal` (`crates/ui/ess-ui-react/src/ts.rs:33`) writes a non-finite number as `null`, and then `toneFor` (`json.ts.tmpl:72`) skips that band. So for -5, `tone_of` gives `danger` and React gives `ui-tone-neutral`. Run `build.sh adv/react-adv.txt … test -p ess-ui-react --locked --test field_tone -- adv_`; it fails with `AssertionError: React and the model disagree on -5 under toneBy {"bands":[{"from":null,…}` (exit 101). It only happens when an author writes `.inf`, `-.inf` or `.nan` as a `from`. Fix: `tone_bands` refuses a `from` that is not finite.
- **Suites:** `adv/check-suite.txt` shows `test result: FAILED. 5 passed; 2 failed` (exit 101). `adv/react-suite.txt` shows `test result: FAILED. 5 passed; 1 failed` (exit 101).
- **Held:** band edges where the value equals `from`, negative values, null and numeric strings agree between React and Rust. Bands written out on a badge or icon go through `tone_bands`. Field names resolve after widget expansion (`resolve_tones` runs after `expand_widgets`). The labelled rows used for `label_from` match React's.
- **Not run:** a widget param typed `{list: …}` used as `tone: args.p` is stripped as structural rather than refused when the widget is declared. Exact-map keys for whole-number floats (`String(2.0)` is "2", `Labels::key` gives "2.0") may also differ, and that is the same as the existing `labels` behaviour.
- **Origin:** introduced 3, pre-existing 0, undecided 0.
- **Written outside the worktree (all in the assigned scratch `adv/`):** `wait.sh`, `check-adv.txt`, `react-adv.txt`, `check-suite.txt`, `react-suite.txt`, plus the `.touch` files build.sh may have written. The scratch path `ess/targets/A/tmp/ess-ui-react-field-tone/adv-inf/` was written by the test run itself.

```findings
[
  {
    "file": "crates/ui/ess-ui/src/model.rs",
    "line": 3339,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "a field's tone_by written as {map: {bands}} loads although the schema types it as a tone map name, and tone_bands never checks it, so descending bands pass ess ui check"
  },
  {
    "file": "crates/ui/ess-ui-check/src/enums.rs",
    "category": "contract-drift",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "the tones of a field's written-out tone_by map are never enum-checked, so purple loads, passes the check and is emitted as ui-tone-purple"
  },
  {
    "file": "crates/ui/ess-ui-react/src/ts.rs",
    "line": 33,
    "category": "boundary",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "a band from of -.inf is accepted and emitted as null, so React skips the band while the model and TUI apply it, and -5 is neutral in React but danger in the terminal"
  }
]
```
