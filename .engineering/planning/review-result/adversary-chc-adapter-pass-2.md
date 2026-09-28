---
format: aep.planning-md/3
id: review-result:adversary-chc-adapter-pass-2
kind: review-result
status: active
title: Adversary pass 2, concurrent-history wave 5 unit adapter
relations:
- reviews: story:recorded-log-adapter-domain
revision: 1
---
# Adversary pass 2 — story:recorded-log-adapter-domain

Dispatched as `aep:adversary` after correction round 2, 2026-09-28. Header and findings verbatim.

unit: story:recorded-log-adapter-domain, the uncommitted working tree on impl/recorded-log-adapter-domain (base e1b9468159 plus the coordinator's merge fix)
verdict: NEEDS-CHANGE
cases: executed 931→937, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: none left (test temp dirs under the assigned `$TMPDIR` were deleted; builds went to the assigned `$CARGO_TARGET_DIR`)
needs-coordinator: should a `{`-leading YAML flow document (base admitted it) still be YAML? That decides finding 5.

Cases (tests/recorded_adapter_adversary_pass2.rs), all RED: a core tag adjacent to a flow key colon is
refused; a bang before an escaped quote in a quoted word is text; a long word holding a bang is not
refused as a tag; a JSON word mapped twice is refused; a YAML flow-style adapter is read as YAML; a
JSON field written twice is refused.

Suite: ess-conformance 931 passed, 6 failed (the six new); ess-cli import tests green.

Coordinator routing: all `introduced` → correction round 3. Decided: remove the `!` rewriter; refuse
local tags through `serde_yaml::Value::Tagged`, admit YAML core tags (case 1 re-pinned to that
decision); JSON parsed straight into the typed adapter with duplicate refusal; a `{`-leading document
that is not valid JSON is read as YAML.

```findings
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 714
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "`tag_tokens` starts a token only after whitespace or `[{,`, so a core tag adjacent to a quoted flow key's colon (`{\"pointer\":!!str /request/client}`) is never rewritten and is admitted, though the reader's doc says a YAML tag anywhere is refused."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 747
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A `!` token inside a double-quoted word ends at the quote a backslash escapes, so rewriting `\"done !\\\"\"` deletes the backslash, the rewritten text stops parsing, and a tag-free adapter the base admitted is refused as 'begins like a YAML tag'."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 685
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "Rewriting `!` to the longer `!ess-tag-N` can push a quoted simple key past libyaml's 1024-character limit, so a valid tag-free adapter with a long word holding ` !` is refused; no real caller writing such a word was found."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 570
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "JSON adapters are read through serde_json::Value, which keeps the last duplicate member, so a JSON `values` mapping `ok` twice is admitted with `ok` silently Indeterminate while the YAML spelling is refused as 'mapped twice'."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 568
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Any `{`-leading document is handed to serde_json, so a YAML flow-style adapter with unquoted keys, admitted by the base reader, is now refused with a JSON syntax error although the reader reads documents 'written as YAML or JSON'."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 570
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A JSON adapter naming a source twice (`\"client\": \"absent\", \"client\": {…}`) is admitted with the last value, where the base reader refused it as 'duplicate field `client`'."
```
