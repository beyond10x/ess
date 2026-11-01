---
format: aep.planning-md/3
id: story:a-browser-start-bounds-session-new-by-its-deadline
kind: story
status: draft
title: A browser start bounds its first session.new by the startup deadline
relations:
- serves: vision:O2
revision: 1
---
## Finding

Out of scope in adversary pass 1 of wave D357 U2 (`review-result:adversary-d357-u2-pass-1`): since that unit, the first `session.new` round trip is part of a browser start in the ess-cli test fixture (`crates/edge/ess-cli/tests/support/browser.rs`), but it is bounded by the session timeout (120 s), not by the startup deadline the caller gave, so a silent `session.new` holds a 1 s start for 120 s.

## Acceptance

- The first `session.new` round trip of a start is bounded by the time left of the caller's startup deadline; when it runs out, the start is a fixture environment refusal at the session stage that carries the measured startup time.
- A boundary case with a stand-in that accepts the upgrade and never answers `session.new` refuses within the startup deadline plus a stated margin.
