---
format: aep.planning-md/3
id: story:web-bridge-answers-like-http
kind: story
status: draft
title: The web bridge answers a command with the HTTP surface's shape
tags:
- defect
- severity-low
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web/bridge.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/page.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/web.rs
- confidence: inferred
  path: generated/web/
revision: 5
---
## Outcome

The web bridge answers a command with the same shape as the HTTP surface: `outcome`, flat
`error`/`payload` on a refusal, and `published`, so a client written against one surface reads the
other.

## Problem

Aligning every surface on `published` (story:generated-server-publishes-and-reads-headers) left the
bridge's refusal nested, as it was before. Reproduced on 0.92.0:

```
$ ess generate synthesize --path examples/billing --target web --out web
$ grep -n -A7 '"refusal"' web/crates/billing-web/src/wire.rs
576:            json::member(out, "refusal");
577-            out.push('{');
578-            json::member(out, "error");
579-            json::push_text(out, "billing.email.Undeliverable");
580-            json::member(out, "payload");
581-            encode_error_billing_email_undeliverable(error, out);
582-            out.push('}');
$ grep -n "outcome.refusal" web/index.html
383:  if (outcome.refusal) {
384:    holder.appendChild(element("p", { class: "refused", text: `refused with ${outcome.refusal.error}` }));
385:    holder.appendChild(json(outcome.refusal.payload));
```

The bridge's encoder is `encode_outcome_<command>` (`crates/generate/ess-synth/src/rust/wire.rs`),
including its unknown-instance arm. The HTTP surface (`crates/generate/ess-synth/src/rust/http.rs`,
the served answer) writes `error` at the top level, as the error's wire code rather than its
qualified name, and `payload` only when the error declares fields. So the two answers differ in
three ways: nesting, the `error` value, and an always-present `payload`.

## Acceptance

- The bridge's refusal is no longer nested under `refusal:`: a refused command answers
  `{"outcome": ..., "published": [...], "error": <wire code>, "payload": {...}}`, with `payload`
  present exactly when the HTTP surface writes it; the unknown-instance answer follows the same shape.
- The generic page (`index.html`) and `bridge.js` read the flat shape; the browser lab tests pass.
- One test compares a bridge answer with the HTTP answer for the same command and input (an
  accepted and a refused outcome) and finds them equal as JSON.

## Scope

- cited: `crates/generate/ess-synth/src/rust/wire.rs` (`encode_outcome_*` emitter, the `refusal` member)
- cited: `crates/generate/ess-synth/src/rust/http.rs` (the served answer, the shape to match)
- cited: `crates/generate/ess-synth/src/web/page.rs` (the page's `outcome.refusal` reader)
- inferred: `crates/generate/ess-synth/src/web/bridge.rs`
- inferred: `crates/generate/ess-synth/tests/web.rs`, `crates/generate/ess-synth/tests/served_correction_pass2_bridge.rs`
- inferred: `generated/web/` (regenerated)
- inferred: `CHANGELOG.md` (a breaking change for a client reading the nested shape)

confidence: medium

## Origin

Aligning every surface on `published` (story:generated-server-publishes-and-reads-headers) left the
bridge's refusal nested, as it was before.
