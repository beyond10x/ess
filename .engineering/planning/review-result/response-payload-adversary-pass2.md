---
format: aep.planning-md/1
id: review-result:response-payload-adversary-pass2
kind: review-result
status: active
title: Review closed response union correction
relations:
- reviews: story:typed-response-outcome-payloads
revision: 1
---
unit: typed-response-outcome-payloads — adversary pass 2, final
verdict: green — recorded closed-union finding corrected
cases executed: 0 by this review; inspected original regression 1/1, new Rust 1/1, new Go 1/1 green evidence
origin: n/a
wrote-outside-worktree: this private review report only
needs-coordinator: yes — record final review disposition and integrate correction
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 27ec173cb85cf9ec2f1e8793743f80dbbfa12265975969e09a524815e0875197, retained as local-evidence:runtime-gaps/publication-replay/snapshots/27ec173cb85cf9ec2f1e8793743f80dbbfa12265975969e09a524815e0875197.md. Source creation recorded at 2026-09-11T06:00:53Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 ee76b7785134cfb5ea469d4530dc789ea054cd89be3b0aca0a0def66ddafd5fd, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/ee76b7785134cfb5ea469d4530dc789ea054cd89be3b0aca0a0def66ddafd5fd-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

Reviewed union-correction.patch SHA256 833dbc0fe356ad987b02320e1d1d655e1b46c184e800527d8ad58357cb5babce and union-candidate.sha256 digest 9cd19e4beb2c659113f3f8c25b3db8d79ee8d1a9f0c6fc74a9635ed88db298ff. Independently verified both digests and every six-file manifest entry against the actual candidate tree.

Rust validate_value_inner and Go selectionObservation.validateValue now reject every union member except the authored discriminator and its computed content key when response mode is active. Both checks run before recursive content admission. The existing label lookup still rejects unknown variants; optional content continues through the existing missing/null handling; required content still refuses absence. The alternate tag `value` correctly uses `content`. Legacy selection mode does not enter either new rejection block, and native generation is unchanged.

The first-pass regression retains the same source text, actual valid/invalid values and deciding assertion. Its only changes are redundant Rust raw-string delimiters and rustfmt wrapping, independently confirmed against the retained original. The original green result is present in union-correction.log. The first new Rust invocation failed only while exporting contracts into a nonexistent directory, after its semantic assertions; union-boundaries.log records its successful corrected-path invocation. The Go fixture consumes the compiler-derived contracts and separately decodes response and event values. It covers both tag spellings with present, null and absent optional content, then extra members, unknown variants, missing required content and wrong content type. union-go.log records its green result. Scoped all-target Clippy is green in union-clippy-final.log.

No remaining counterexample was established within this bounded correction. Prior first-pass inspection of invocation authority, source compatibility, schema/native projection and typed diff remains applicable: this patch changes none of those paths. Root's separate centralized typed response admission correction is outside this unit verdict and was not independently retested here.

No new test, production, Git or AEP edits; no compiler slot used, no lease acquired, and no unchanged successful tests rerun. This is the second/final attack; no third review requested.
```findings
[]
```