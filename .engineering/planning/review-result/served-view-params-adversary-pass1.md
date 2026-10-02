---
format: aep.planning-md/3
id: review-result:served-view-params-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:served-view-params (wave ui-live-apps-w6)
relations:
- reviews: story:served-view-params
revision: 1
---
needs-change

Adversary pass 1 on story:served-view-params at b5100e2c7. Cases executed 348 to 358, red 5; introduced 3, pre-existing 4. Test file: crates/generate/ess-synth/tests/adversary_view_params_pass1.rs.

- warning: params minHours and min_hours emit code that compiles in neither language, and synthesis does not refuse the model.
- warning: a Go param named nil breaks the generated stub.
- warning, pre-existing: Go and Rust refuse undecodable Integer and Boolean values with different text (14 of 56 probes).
- note, pre-existing: a 2 MiB query: Go 431, Rust 200.
- note, pre-existing: Decimal and Uuid values outside the OpenAPI pattern reach the port.
- note: the new pub Request.query breaks downstream struct literals (E0063); needs a CHANGELOG breaking line.
- note, pre-existing: ess_ui_check::binding treats Binary64 params as scalars; every code target refuses Binary64.

Held: 42 of 56 query probes agree byte for byte (%20 and +, UTF-8, %00, malformed %, invalid UTF-8, repeated keys, empty values, keys without =, fragment, key case, wire vs declared name, undeclared keys, enum case); /openapi.json names and required flags equal what each server reads; params named type, ref, func, range, match, self build; duplicate wire names refused by the compiler; the web target builds for wasm32 with a param view; views without params keep their bytes.

```findings
[{"file":"crates/generate/ess-synth/src/go/port.rs","line":297,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"Go view params whose names normalise to one identifier emit a redeclared parameter instead of a synthesis refusal"},
 {"file":"crates/generate/ess-synth/src/rust/port.rs","line":334,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"Rust view params whose names normalise to one identifier emit E0415 instead of a synthesis refusal"},
 {"file":"crates/generate/ess-synth/src/go/port.rs","line":311,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A Go view param named nil shadows the nil the obligation stub returns"},
 {"file":"crates/generate/ess-synth/src/go/http.rs","line":860,"category":"contract-drift","severity":"warning","verdict":"needs-revision","origin":"pre-existing","message":"Go and Rust refuse undecodable Integer and Boolean query values with different refusal text"},
 {"file":"crates/generate/ess-synth/src/rust/http.rs","line":1473,"category":"boundary","severity":"note","verdict":"approve","origin":"pre-existing","message":"A 2 MiB query is answered 431 by Go and 200 by Rust"},
 {"file":"crates/generate/ess-synth/src/rust/wire.rs","line":794,"category":"contract-drift","severity":"note","verdict":"approve","origin":"pre-existing","message":"Decimal and Uuid query values outside the published OpenAPI pattern reach the port in both servers"},
 {"file":"crates/generate/ess-synth/src/rust/http.rs","line":1372,"category":"judgement","severity":"note","verdict":"approve","origin":"introduced","message":"The new pub field Request.query breaks every downstream http::Request struct literal with E0063"},
 {"file":"crates/ui/ess-ui-check/src/model.rs","line":944,"category":"contract-drift","severity":"note","verdict":"approve","origin":"pre-existing","message":"binding treats Binary64 params as scalars while every code target refuses Binary64"}]
```
