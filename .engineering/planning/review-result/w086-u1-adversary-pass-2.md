---
format: aep.planning-md/3
id: review-result:w086-u1-adversary-pass-2
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 2 of unit U1 (value labels)'
relations:
- reviews: story:a-value-shows-its-declared-label-beside-its-tone
prose_only: adversary pass 2 approved with no findings (findings block empty)
revision: 1
---
# UI-adopter wave 1: adversary pass 2 of unit U1 (value labels)

Unit: `story:a-value-shows-its-declared-label-beside-its-tone`, fix commit `4f160cbe6f` (fix round 1), attacked at integration `6b1798614c` in tree `ess-w086-adv-a`.

- Verdict: approve. Cases executed 49 -> 63, 0 red (`crates/ui/ess-ui/tests/adversary_w086_u1_pass2.rs`, 14 cases; `adversary_w086_u1_pass1` 4 passed).
- Attacked, not broken: a field or badge wherever it can stand (shorthand columns, `text:` badges, widget bodies and arguments, nested widgets, tabs, items, expand, selectable columns, page kinds, `same_as` overlays); `field` and `labels` as sibling names; label-map keys spelled like document structure against all four walkers; tones beside labels in containers named `labels` or `field`.
- Two new cases pin both halves of `holds_labels` (the `primitive: badge` arm and `field:` as text), which no earlier case pinned.
