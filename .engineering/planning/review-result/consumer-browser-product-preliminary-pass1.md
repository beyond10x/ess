---
format: aep.planning-md/3
id: review-result:consumer-browser-product-preliminary-pass1
kind: review-result
status: active
title: 'Preliminary browser source review: depth and artifact collisions'
relations:
- reviews: story:browser-response-conformance
revision: 1
---
needs-revision

Preliminary bounded root review of uncompiled browser implementation based on d849ea01c, not a final implementation adversary or approval. Private owner WIP checkpoint patch SHA256 7ea9c9992bc1f1752f301d93f0d5c76043a6d04453d79b3a473501a183cbf103 includes tracked and new source. Source was held stable for the deciding probe; exact player asset SHA256 715ee9f33ee65ddfacb3e93b1fdd6abdb7e9c42bde9b297c42fa137b5d8dcbd1. Root inspected the product/ABI/bundle/presentation modules, browser assets, CLI acquisition/emission delta and bounded admission/presentation tests. The full independent feature fixture and final assembled product were not verified.

Root actual execution: Node v22.23.2 imported the exact frozen player asset without a DOM or browser. A complete closed presentation with valid digest and source metadata, nested Object DisplayValues and one terminal Text passes the asset's validatePresentation at logical depths 0,128,256,340,341,400,1024. The same asset's parseControl passes through340 and refuses341,400,1024 as resource_limit, at only12988,15171,38259 serialized bytes respectively. The named BROWSER_DISPLAY_DEPTH_WRAPPER_MISMATCH assertion exits1. Private red log SHA256 3b6463cc71ca5675a6dcd91479da16dca78cadab5263f1167fe1825bc6a71fdc and original asset are retained in ess-browser-preliminary-review-20261003. This is an actual player-level red; no claim that a real compiled ESS model at depth1024 was emitted or loaded in Firefox follows. The first exploratory probe omitted required metadata and was not the deciding test.

```findings
[
  {
    "file": "crates/verify/ess-conformance/assets/browser-player.js",
    "line": 28,
    "category": "boundary",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "parseControl bounds raw JSON nesting at1024 while the same product's typed display validator and Rust Budget permit logical value depth1024. Object encoding adds object/value-array/pair-array wrappers, so valid display depth341 already refuses. The actual Node control reproduces this independently. Align bounded control parsing with the encoded envelope while retaining the logical depth/node/byte bounds and duplicate-key rejection. Add persistent Rust-orchestrated object/list/scenario-envelope controls including logical1024 acceptance and1025 refusal, and check the Rust writer/parser for the same wrapper mismatch. Compiler/browser validation remains held."
  },
  {
    "file": "crates/verify/ess-conformance/src/web.rs",
    "line": 393,
    "category": "boundary",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "emit_product accepts arbitrary validated SourceDocument paths, inserts their blobs, then overwrites colliding keys with fixed product artifacts. A source labeled browser.json, index.html, player.js, README.md or rust/browser_host.rs can pass bundle::create's reference uniqueness checks but be replaced afterward, contradicting the emitted manifest. Default CLI sources/... labels avoid this case; the new public emitter does not. Refuse output collisions before returning artifacts or relabel before manifest construction. Add a public-emitter Rust regression with browser.json and a valid noncolliding control. This finding is source-derived, not an executed Rust/Firefox result."
  }
]
```

Both findings were delivered to the existing browser owner for bounded correction within current scope. No production edit by root, compilation, browser launch, cache allocation, remote write or source handoff occurred. Build clearance remains explicit; neither these probes nor subsequent source corrections establish complete product support.
