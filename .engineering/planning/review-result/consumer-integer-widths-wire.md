---
format: aep.planning-md/3
id: review-result:consumer-integer-widths-wire
kind: review-result
status: active
title: Native Rust integer-width wire review
relations:
- reviews: story:feature-request-394
revision: 1
---
# #394 Rust wire boundary test: independent review

Reviewed read-only on 2026-10-03. Source: ess-backlog-next-20261002, HEAD 13e40c33f9, only the 53-line append to crates/generate/schema-contract/tests/integer_widths.rs. Reviewer ran no build/test and made no repository edits.

Verdict: approved. Findings: [].

The test builds the actual generated Rust declaration and Cargo manifest, then exercises serde_json decoding/encoding. It checks both i32 boundaries, distinguishes 9007199254740992 from 9007199254740993 through the generated i64 field and JSON encoding, and rejects i32/i64 overflow, null and fractional input for the required integer field. This is execution evidence at the codec seam, additional to the existing declaration-width and constant-obligation tests. It does not claim that the native width discharges arbitrary schema range obligations. Root reports 4/0 focused tests; that execution was not independently repeated.
