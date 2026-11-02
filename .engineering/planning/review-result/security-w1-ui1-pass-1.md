---
format: aep.planning-md/3
id: review-result:security-w1-ui1-pass-1
kind: review-result
status: active
title: 'W1 UI1 security pass 1: ess-ui stories'
summary: 'NEEDS-CHANGE: React routePath sends dot-segment values to another route; account-menu drops; one pre-existing item to the backlog'
relations:
- reviews: story:account-menu-entries-keep-sets-and-navigate
- reviews: story:a-generated-ui-calls-the-routes-its-http-binding-serves
- reviews: story:a-row-actions-confirm-is-addressed-by-the-views-key
- reviews: story:a-record-action-in-the-terminal-renderer-sees-its-row
revision: 1
---
unit: W1 UI1 (four ess-ui stories), reviewed in tree ess-w1-ui1 at 939c2caf4
verdict: NEEDS-CHANGE
cases: executed 6→11, red 3
origin: introduced 3 / pre-existing 0 / undecided 0

**Findings and what became of them (coordinator).**
- S1 (story 4, integrity): React `routePath` fills `.` or `..` unrefused and fetch normalization sends the
  binding's method to another route; React refuses such a value. Back to the implementor.
- S2 (story 3): `sets` beside `does` dropped silently; same remedy as adversary A2.
- S3 (story 3): an entry that does not parse as an Action loses `sets` and `navigate` in React; same remedy
  as adversary A1, and the emitter refuses instead of `.ok()`.
- Out of scope, pre-existing: React's `runAction` runs one effect per action. Backlog draft
  story:an-action-declaring-several-effects-runs-them-all-or-is-refused. The terminal's drop of an
  unparseable entry is covered by the check refusing it.

**Attacked, not broken.** Path values holding URL syntax stay one segment in both runtimes; CRLF cannot reach
the request line; methods come from the binding type; story 1 and 2 keying matches the runner and the view.
