---
format: aep.planning-md/3
id: story:an-overlay-s-params-values-that-read-state-or-belong-to-a-shell-overlay-are-read-where-their-opener-stands
kind: story
status: draft
title: An overlay's params values that read state, and the params values of a shell overlay, are read where their opener stands
summary: 'The two cases the fix of an opened overlay''s params leaves: a state.<x> root in a params value and a shell overlay''s params values; a defect candidate with no red case yet'
tags:
- defect
- ess-ui
relations:
- serves: vision:O2
- informed_by: story:an-opened-overlay-reads-its-openers-params
revision: 1
---
## Need

The fix of `story:an-opened-overlay-reads-its-openers-params` (commit `6b3c131773`) moves the check and the two targets to the
opener's scope for `params.<name>` in the `params:` values of a page overlay. Two cases it does not move:

- a `state.<x>` root in an overlay's `params:` values: the reference's `expr_paths` scope for it is unchanged, and whether the value
  is read where the opener stands is not decided;
- an overlay of a shell: the opener lookup in `crates/ui/ess-ui-check/src/sites.rs` (`openers`) takes the path
  `pages/<page>/overlays/<overlay>/params/<key>` only, so a shell overlay's `params:` values keep the previous reading.

## Fit review

1. **Need.** The same defect class as the sibling story for the roots it did not touch: a value written in `params:` is passed by the
   opener, whatever root it reads.
2. **Class.** A defect candidate, not measured: no case is red for either gap yet. The first step is a red case for each.
3. **Existing stories.** `story:an-opened-overlay-reads-its-openers-params` is the sibling and borders it (informed_by); it records
   these two as its `## Not covered`.
4. **Release train:** next minor, or a 0.99.x patch if a red case shows the same defect.

## Decisions

None open. Whether `state.<x>` read at the opener is a defect or a different scope is decided by the first red case against the reference
(`ess-ui.schema.yaml:1805`).

## Acceptance

- a_state_root_in_a_params_value_is_judged_where_the_opener_stands, or a recorded decision that it is not.
- a_shell_overlay_params_value_is_judged_where_its_opener_stands, or a recorded decision that it is not.

## Scope

- cited: `crates/ui/ess-ui-check/src/sites.rs` (`openers`), `schemas/ui/ess-ui.schema.yaml` (1805)
