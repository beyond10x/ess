---
format: aep.planning-md/3
id: review-result:ui-tui-live-binding-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:ui-tui-live-binding (wave ui-live-apps-w3)
relations:
- reviews: story:ui-tui-live-binding
revision: 1
---
needs-change

Adversary pass 1 on story:ui-tui-live-binding at 5c8ef11aa. Cases executed 61 to 67 (ess-ui-tui), red 6, all introduced. Test file: crates/ui/ess-ui-tui/tests/adversary_tui_live_pass1.rs.

- warning: a reopened inline confirm shows the last attempt's refusal (app.rs:2669).
- warning: a retry after a refused confirmed action resends the accepted confirm command (app.rs:2507).
- warning: Unfinished{committed:true} does not re-read the screen (app.rs:2537).
- warning: Base::parse accepts malformed authorities, so a bad --base-url is not refused at startup (http.rs:66).
- note: an interim 1xx answer is taken as final (http.rs:438).
- note: the reader ignores Content-Length and waits for close (http.rs:220).
- note (no test): the key-drain runs in unbound runs too and drops ctrl-c during a stall (lib.rs:214).

Held: all 22 vectors reach the open form as the right state over a raw TCP stub; IPv6 base URL with path prefix and trailing slash; Host and Authorization headers; CR/LF/NEL/DEL in ESS_UI_AUTHORIZATION refused; as: number for -5, 1.5, empty, abc; a failed live read polls again.

```findings
[
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":2669,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"Opening an inline confirm does not clear its o: UI state, so a reopened confirm shows the previous attempt's refusal before anything is sent."},
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":2507,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"After the confirmed action is refused, confirming again resends the confirm's own command that was already accepted."},
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":2537,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"An Unfinished{committed:true} answer does not invalidate reads, so the screen stays stale after a committed effect."},
{"file":"crates/ui/ess-ui-tui/src/http.rs","line":66,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"Base::parse accepts an out-of-range, non-numeric or empty port or an unclosed IPv6 bracket, so a bad --base-url is not refused before the terminal is touched."},
{"file":"crates/ui/ess-ui-tui/src/http.rs","line":438,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"parse_answer takes an interim 1xx answer as the final one."},
{"file":"crates/ui/ess-ui-tui/src/http.rs","line":220,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"The reader waits for the connection to close instead of stopping at Content-Length."},
{"file":"crates/ui/ess-ui-tui/src/lib.rs","line":214,"category":"judgement","severity":"note","verdict":"approve","origin":"introduced","message":"The key-drain after a command also runs in unbound fixture runs and drops a ctrl-c pressed during a stalled command."}
]
```
