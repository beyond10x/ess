---
format: aep.planning-md/3
id: review-result:served-view-params-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:served-view-params (wave ui-live-apps-w6)
relations:
- reviews: story:served-view-params
revision: 1
---
needs-change

Adversary pass 2 on story:served-view-params at 034c377fe. Cases executed 359 to 364, red 5, all pre-existing. Test file: crates/generate/ess-synth/tests/adversary_view_params_pass2.rs. Coordinator: fixed in this unit (served-surface parity; the early hang-up stops a dev server on a browser reload).

- warning: a caller that hangs up before reading its answer makes http::write fail with EPIPE and serve returns; the Rust server exits.
- warning: Bytes and boolean map keys are admitted by one server and refused by the other.
- note: Bytes and integer/boolean map-key refusals use different words in Go and Rust.
- note: a request with 101 headers: Rust 431, Go 200.
- note: one silent caller holds the single-threaded Rust accept loop with no read timeout.

Held: request-head limit at, over and on the last line agrees in both servers; a pipelined request after a 431 gets no second answer; no example, suite or realization sends a Decimal or Uuid the new check refuses; no test outside ess-synth pins the old Go wording; answers.json unaffected; Go params named error, string, any, len, iota and package names build; len with len_ is refused as a collision; the billing-web wasm release build and smoke.mjs pass (21 claims held).

```findings
[{"file":"crates/generate/ess-synth/src/rust/http.rs","line":703,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"pre-existing","message":"A caller that hangs up before reading its answer makes http::write fail with EPIPE and the ? ends serve, so the Rust server exits"},
 {"file":"crates/generate/ess-synth/src/rust/json.rs","line":627,"category":"contract-drift","severity":"warning","verdict":"needs-revision","origin":"pre-existing","message":"Bytes and boolean map keys are admitted by one server and refused by the other"},
 {"file":"crates/generate/ess-synth/src/go/http.rs","line":1869,"category":"contract-drift","severity":"note","verdict":"needs-revision","origin":"pre-existing","message":"Bytes and integer/boolean map-key refusals use different words in Go and Rust"},
 {"file":"crates/generate/ess-synth/src/rust/http.rs","line":1351,"category":"boundary","severity":"note","verdict":"needs-revision","origin":"pre-existing","message":"A request with 101 headers is 431 from Rust and 200 from Go"},
 {"file":"crates/generate/ess-synth/src/rust/http.rs","line":697,"category":"concurrency","severity":"note","verdict":"needs-revision","origin":"pre-existing","message":"One silent caller holds the single-threaded Rust accept loop with no read timeout"}]
```
