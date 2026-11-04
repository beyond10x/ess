---
format: aep.planning-md/3
id: review-result:w086-u3-adversary-pass-3
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 3 of unit U3 (page size)'
relations:
- reviews: story:a-server-paged-collection-has-a-page-size-control
prose_only: adversary pass 3 approved with no findings (findings block empty)
revision: 1
---
# UI-adopter wave 1: adversary pass 3 of unit U3 (page size)

Unit: `story:a-server-paged-collection-has-a-page-size-control`, fix commit `c22f85428a` (fix round 2), attacked at integration `4815c9fbb8` in tree `ess-w086-adv-a`.

- Verdict: approve. Cases executed 112 -> 122, 0 red (`adversary_w086_u3_pass3.rs` in `ess-ui-check`, 6 cases, and `ess-ui-react`, 4 cases); every pass-1 and pass-2 case green.
- Attacked, not broken: paths sharing only a text prefix (`state.page`/`state.pages`), siblings in one record, one root at different depths either way round, padded texts, one param for both, bracketed and numeric segments, overlapping widget arguments; held values 0, -5, text, `[25]`, `true`, `null` and empty with Next, "of N", URL round trips and Back.
- `Reads::state_path` admits only identifier segments and trims the text, so `overlaps` never sees bracketed, numeric or spaced forms.
