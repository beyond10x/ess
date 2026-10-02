---
format: aep.planning-md/3
id: review-result:ui-react-plain-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:ui-react-plain (wave ui-live-apps-w1)
relations:
- reviews: story:ui-react-plain
revision: 1
---
needs-change

Adversary pass 1 on story:ui-react-plain at ff9613260. Cases executed 50 to 54, red 4, all introduced. Test file: crates/ui/ess-ui-react/tests/adversary_react_plain_pass1.rs.

- warning: the dev script exits about 37 ms after start when stdin is closed (background job, CI step, docker without -i); Playwright webServer holds stdin open and is not affected. Fix: --watch=forever.
- warning: the preview script exits the same way.
- warning: --serve=5173/4173 listen on 0.0.0.0 where Vite listened on localhost. Fix: 127.0.0.1.
- note: a Link to the current location pushes a duplicate history entry.

Held: route ranking and duplicate refusal, param decoding (malformed %, empty, //, trailing slash, case), the fallback for dotted/encoded/non-ASCII paths, url state keeping other keys, a single replace on redirect, shells staying mounted, NODE_ENV under --minify, dependencies, data-ui-path, script names, determinism.

```findings
[
  {"file": "crates/ui/ess-ui-react/templates/project/package.json.tmpl", "line": 7, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the dev script's esbuild serve exits at once when stdin is closed; --watch=forever keeps it serving; test the_dev_script_keeps_serving_when_started_without_stdin"},
  {"file": "crates/ui/ess-ui-react/templates/project/package.json.tmpl", "line": 11, "category": "acceptance", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "the preview script exits at once when stdin is closed; test the_preview_script_keeps_serving_when_started_without_stdin"},
  {"file": "crates/ui/ess-ui-react/templates/project/package.json.tmpl", "line": 7, "category": "judgement", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "--serve=5173 and --serve=4173 listen on 0.0.0.0 where Vite listened on localhost; test the_serve_scripts_listen_on_loopback_only"},
  {"file": "crates/ui/ess-ui-react/templates/runtime/router.tsx.tmpl", "line": 101, "category": "contract-drift", "severity": "note", "verdict": "needs-revision", "origin": "introduced", "message": "Link to the current location pushes a duplicate history entry; test a_link_to_the_current_location_replaces_instead_of_pushing"}
]
```
