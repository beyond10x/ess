---
format: aep.planning-md/3
id: review-result:ui-react-plain-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:ui-react-plain (wave ui-live-apps-w1)
relations:
- reviews: story:ui-react-plain
revision: 1
---
needs-change

Adversary pass 2 on story:ui-react-plain at b506081412. Cases executed 54 to 56, red 2, introduced 2, pre-existing 1. Test file: crates/ui/ess-ui-react/tests/adversary_react_plain_pass2.rs.

- warning: an alias written with a leading slash becomes //legacy/thing/:id and never matches; react-router 7 matchRoutes (measured) collapsed // and matched.
- warning: matchPage compares static segments with the percent-encoded pathname, so an alias with a non-ASCII or space segment never matches; react-router decoded first.
- warning, pre-existing: npm run typecheck/build fail on the partner-portal example against real @types/react 19.3.0 (image.tsx TS2322 objectFit); the offline stub types hide it.

Held: real @types/react type-checks the router; the real build writes www/assets/main.js and main.css; headless Chrome renders a deep link under its shell and redirects an unknown path; the dev server rebuilds on change; preview with --watch=forever serves; /assets falls back; npm 12 install works; CI installs esbuild in the test job; the baseURL note matches the emitter; Link-to-current compares encoded paths correctly.

```findings
[
  {"file": "crates/ui/ess-ui-react/src/emit.rs", "line": 189, "category": "contract-drift", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "an alias written as a legacy path with a leading slash becomes //legacy/thing/:id and never matches, where react-router collapsed // and matched; test an_alias_written_with_a_leading_slash_reaches_its_page"},
  {"file": "crates/ui/ess-ui-react/src/emit.rs", "line": 163, "category": "contract-drift", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "matchPage compares static segments against the percent-encoded pathname without decoding, so an alias with a non-ASCII or space segment never matches; test an_alias_with_a_segment_the_address_bar_encodes_reaches_its_page"},
  {"file": "crates/ui/ess-ui-react/templates/runtime/primitives/image.tsx.tmpl", "line": 15, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "pre-existing", "message": "npm run typecheck and npm run build fail on the partner-portal example against real @types/react 19.3.0 (TS2322 objectFit string); the offline stub types hide it"}
]
```
