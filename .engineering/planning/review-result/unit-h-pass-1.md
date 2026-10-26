---
format: aep.planning-md/3
id: review-result:unit-h-pass-1
kind: review-result
status: active
title: Unit H adversary pass 1
relations:
- reviews: story:a-view-field-from-names-the-related-spelling
revision: 1
---
## Adversary pass 1, unit H (ess-0700 wave)

Worktree ess-0700-h, base 0948f833e; adversary added 4 cases in ess-cli validate_parse_refusals.rs and ess-domain row_queries_navigate.rs (1 red). Cases executed 24 to 28.

Held: from: outside a view field keeps unknown field from; a grant naming the withheld view gives one refusal; a source declared in another file still gets the rewrite; empty path segments get no guessed rewrite; nothing else reads RawViewField.

```findings
- {file: crates/specify/ess-compiler/src/resolve.rs, line: 773, category: acceptance, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "the view-field from refusal is cited at the view name line instead of the from line whenever the spec has any other from:, such as a lifecycle transition"}
```
