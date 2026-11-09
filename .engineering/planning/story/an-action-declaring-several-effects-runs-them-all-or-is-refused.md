---
format: aep.planning-md/3
id: story:an-action-declaring-several-effects-runs-them-all-or-is-refused
kind: story
status: archived
title: An action declaring several effects runs them all or is refused
summary: React runAction runs one effect per action (W1 UI1 review, pre-existing, backlog)
tags:
- backlog
- not-a-defect
- review-finding
- zero-open-issues
relations:
- informed_by: review-result:security-w1-ui1-pass-1
- decomposes: epic:zero-open-issues-2026-11
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:48Z", actor: "human:timo", revision: 4}
---
## Need

React's `runAction` runs one effect per action (an else-if chain), so an ordinary action declaring two
effects runs only the first (pre-existing at b81deac78, read by review-result:security-w1-ui1-pass-1). W1 refuses
the combinations on account-menu entries only.

## Acceptance

- An action outside the account menu that declares more than one effect is either run in full by both
  renderers or refused by `ess ui check`, naming the action.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.


## Verification on 0.92.0 (2026-11-09)

Result: **not-a-defect**. Checked against main at 4ab2ea1449 and the 0.92.0 release binary by the
zero-open-issues run (epic `epic:zero-open-issues-2026-11`); this draft leaves the plan as archived.

Proof: verify/an-action-declaring-several-effects-runs-them-all-or-is-refused/repro/run.txt; crates/ui/ess-ui/src/path.rs:526-545 (check_action)

The case cannot occur: no document whose action declares two effects reaches either renderer. On 0.92.0 a row action with `does` + `navigate` fails at load in `ess ui check` (with and without `--model`) and in `ess generate ui --target react`, exit 1: `an action has exactly one of does, opens, navigate, export, upload, copy, sets; this one has 2`.
The loader runs `check_action` (path.rs:526) on every located `NodeRef::Action` (path.rs:430). The walker locates actions at every site: header, empty state, tab form, buttons, row/bulk/collection/record/form/bar/confirm/board/graph actions (path.rs:235-384). The rule has been there since 51aee77acb (2026-09-30, the first ess-ui/1 loader). The reference states it as "Exactly one of" under Action (website/docs/reference/ess-ui.md:2044 ff.).
React's else-if chain in runAction (actions.tsx.tmpl:169-225) is therefore never given two effects. The acceptance ("refused ... naming the action") already holds: the error is at the action's node path.
Suggest closing it as not-a-defect, citing check_action.
