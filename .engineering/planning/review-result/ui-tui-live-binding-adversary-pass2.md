---
format: aep.planning-md/3
id: review-result:ui-tui-live-binding-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:ui-tui-live-binding (wave ui-live-apps-w3)
relations:
- reviews: story:ui-tui-live-binding
revision: 1
---
needs-change

Adversary pass 2 on story:ui-tui-live-binding at acd04ddcf. Cases executed 68 to 73, red 5, all introduced. Test file: crates/ui/ess-ui-tui/tests/adversary_tui_live_pass2.rs.

- warning: a confirm command answered Unfinished{committed:true} is resent on retry; binding.rs:115 says such a command is never retried.
- note: a form submit answered committed keeps the draft and the next ctrl-s resends it.
- note: a chunk size near usize::MAX overflows size + 2 (panic in debug).
- note: two differing Content-Length fields are not refused.
- note: Base::parse validates the port, not the host ([zz], percent-encoded host accepted).

Held: chunk extensions, uppercase hex, trailers, final chunk without CRLF, truncated chunk; no space after the colon; a 103 with headers; headers then close; short, negative and non-numeric Content-Length; a longer body cut at the stated length; userinfo, port 0, query and fragment refused; leading-zero port and uppercase scheme accepted; confirm memory resets on reopen; a committed answer re-reads once.

Not tested: the key-drain (no headless terminal events); an end-to-end bound session (the ess-ui-test runner drives only the fixture adapter; a gap for story:ui-tui-app-generator).

```findings
[
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":2511,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A confirm command answered Unfinished{committed:true} is not remembered as done, so confirming again resends a command the contract says must not be retried."},
{"file":"crates/ui/ess-ui-tui/src/app.rs","line":2545,"category":"acceptance","severity":"note","verdict":"approve","origin":"introduced","message":"A form submit answered Unfinished{committed:true} keeps the draft open and the next ctrl-s sends the committed command again."},
{"file":"crates/ui/ess-ui-tui/src/http.rs","line":561,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"A chunk size near usize::MAX overflows size + 2 and panics in debug builds."},
{"file":"crates/ui/ess-ui-tui/src/http.rs","line":522,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"Two differing Content-Length fields are not refused; the last one wins."},
{"file":"crates/ui/ess-ui-tui/src/http.rs","line":104,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"Base::parse validates the port but not the host."}
]
```
