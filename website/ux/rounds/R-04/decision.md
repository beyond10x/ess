# R-04 decisions

## Ratchet decision `R-04-worse`: the index of the reference rows adds text to read

Change 5 (UXF-010) puts an index of every `ess/N` row under the title of the format version page
and an index of every class number under the title of the diagnostics page. The screen-only user
reaches its row in 3 to 5 actions instead of 26 and 33, and reads more on the first screen:

| scenario | key | baseline | R-04 |
| --- | --- | --- | --- |
| `ess-31-release/desktop-1280` | `reading_load.max` | 291 | 358 |
| `ess-31-release/mobile-390` | `reading_load.max` | 142 | 164 |
| `ess-31-release/mobile-390` | `reading_load.median` | 117 | 137 |
| `diagnostic-type-009/mobile-390` | `reading_load.max` | 142 | 165 |
| `diagnostic-type-009/mobile-390` | `reading_load.median` | 120 | 137 |

- `ess-31-release/desktop-1280 reading_load.max`
- `ess-31-release/mobile-390 reading_load.max`
- `ess-31-release/mobile-390 reading_load.median`
- `diagnostic-type-009/mobile-390 reading_load.max`
- `diagnostic-type-009/mobile-390 reading_load.median`

## Ratchet decision `R-04-worse`: the docs introduction holds one more link before the target

Change 4 (UXF-011) adds the link "Verify conformance" to the second paragraph of the docs
introduction. Two keyboard cards pass that page on their way to a link further down, and tab over
it once more:

| scenario | key | baseline | R-04 |
| --- | --- | --- | --- |
| `first-conformance-run/mobile-390/keyboard` | `screen_only.actions` | 130 | 131 |
| `install-linux-x86_64/mobile-390/keyboard` | `screen_only.actions` | 128 | 129 |

- `first-conformance-run/mobile-390/keyboard screen_only.actions`
- `install-linux-x86_64/mobile-390/keyboard screen_only.actions`

## Ratchet decision `R-04-worse`: a layout shift measured once under a load of 120 to 170

Two pages that changes 1 to 5 did not touch shifted at load in the slow lane, in the dark theme,
where the layout shift is measured:

| scenario | key | baseline | R-04 |
| --- | --- | --- | --- |
| `/ess/docs/concepts/overview@desktop-1280/dark` | `layout_shift.load` | 0 | 0.0342 |
| `/ess/docs/status/roadmap@desktop-1280/dark` | `layout_shift.load` | 0 | 0.0697 |

Two probe runs of both pages afterwards, at a load of 122 and 128 (`.out/probe-cls-1`,
`.out/probe-cls-2`, not part of the round), measured 0 on both. The values are taken so the round
stays complete; the next round's run replaces them in the baseline if it measures 0 again.

- `/ess/docs/concepts/overview@desktop-1280/dark layout_shift.load`
- `/ess/docs/status/roadmap@desktop-1280/dark layout_shift.load`

## Ratchet decision `R-04-worse`: the gate lane measures a lighter reference page

Change 1 (UXF-025) moved the two longest reference pages out of the every-merge gate to the slow
lane, where every value of theirs is still measured and held, and put
`/ess/docs/reference/entity-runtime-lowering` in their place. The gate lane's baseline drops their
eight scenarios (48 values) and takes the four new ones:

- `gate: /ess/docs/reference/diagnostics@desktop-1280/dark`
- `gate: /ess/docs/reference/diagnostics@desktop-1280/light`
- `gate: /ess/docs/reference/diagnostics@mobile-390/dark`
- `gate: /ess/docs/reference/diagnostics@mobile-390/light`
- `gate: /ess/docs/reference/spec-versions@desktop-1280/dark`
- `gate: /ess/docs/reference/spec-versions@desktop-1280/light`
- `gate: /ess/docs/reference/spec-versions@mobile-390/dark`
- `gate: /ess/docs/reference/spec-versions@mobile-390/light`

The lane `full` is taken with `cargo xtask ux-ratchet --round website/ux/rounds/R-04/round.json
--update --decision R-04-worse`, the lane `gate` with the same command on the round of `gate-14`
(this file copied beside it), then checked with `gate-12` and `gate-13`. Decided in round R-04 under its brief ("a worse value only under a decision that lists
it"), 2026-11-23.
