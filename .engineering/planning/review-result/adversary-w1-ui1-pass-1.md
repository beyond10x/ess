---
format: aep.planning-md/3
id: review-result:adversary-w1-ui1-pass-1
kind: review-result
status: active
title: 'W1 UI1 adversary pass 1: ess-ui stories'
summary: 'NEEDS-CHANGE: account-menu check misses unparseable entries and dropped sets; a GET command is never sent'
relations:
- reviews: story:account-menu-entries-keep-sets-and-navigate
- reviews: story:a-generated-ui-calls-the-routes-its-http-binding-serves
- reviews: story:a-row-actions-confirm-is-addressed-by-the-views-key
- reviews: story:a-record-action-in-the-terminal-renderer-sees-its-row
revision: 1
---
unit: W1 UI1 (four ess-ui stories), 642617588, c0daa4b1f, c942ffcd1, 939c2caf4 merged as cce755258, reviewed in tree ess-w1-ui1 at 939c2caf4
verdict: NEEDS-CHANGE
cases: executed 130→133, red 3
origin: introduced 3 / pre-existing 0 / undecided 0

**Findings and what became of them (coordinator).** All three go back to the implementor.
- A1 (story 3): `account_menu_keys` accepts entries the terminal cannot parse as an Action; the check now parses
  each entry and refuses a failed parse.
- A2 (story 3): React drops `sets` beside `does` or `navigate`; the check refuses `sets` beside `does`,
  `navigate` or `opens`.
- A3 (story 4): a command served on GET is sent with a JSON body and never leaves the browser; both runtimes
  send no body on GET/HEAD, else the generator refuses the command.
- Inferred, not raised: a path variable bound by the plain field name on a renamed input; the implementor checks
  whether story 4 introduced it.
