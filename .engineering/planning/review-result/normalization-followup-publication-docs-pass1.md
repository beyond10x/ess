---
format: aep.planning-md/1
id: review-result:normalization-followup-publication-docs-pass1
kind: review-result
status: active
title: Shared normalization documentation review
relations:
- reviews: story:normalization-followup-publication
revision: 1
---
unit: shared public documentation drafts against coordinator 213d4b8a8338763346cad2bc92cee826430726c9 and positional eb2e5d60e9e803993417df39563bc744dbcd36fc
verdict: one documentation finding
scope: changed draft prose only; no native execution or final positional review

The three base/draft SHA-256 pairs match the supplied manifest. Reviewed only the changes in `CHANGELOG.md`, `website/docs/reference/formats.md` and `website/docs/guides/generate-artifacts.md`. Source declarations and retained diff are recorded in `source-proof.json` and `reviewed-draft.diff`. No repository file was modified and no build or test was run.

| File:line | Verdict / origin | Measured wording and source comparison |
|---|---|---|
| `website/docs/guides/generate-artifacts.md:557` | CONFIRMED / introduced; warning | The new sentence says active positional policies require `run_json` or the retained-base64 helper, without qualifying this as the reference Plan API. `run_json` is the reference method (`normalize.rs:259`); generated Rust accepts positional text through `Normalizer::normalize` (`rust_runtime.rs.txt:97`) and Go through `Normalizer.Normalize` (`go_runtime.go.txt:105`). Both directly pass the checked positional policies to input preparation. The new requirement therefore misnames the required API for users of the generated targets. |

Replace the sentence spanning draft lines 556–558, starting at “Active”, with:

> Active positional policies require an original-text entrypoint (`Plan::run_json`, Rust `Normalizer::normalize`, Go `Normalizer.Normalize`, or CLI `normalize-run`) or a retained-base64 helper. `Plan::run` and Rust `normalize_value` refuse them even when the selected field is absent.

The existing reference and generated APIs are also listed earlier in the guide at lines 440–447; the replacement keeps the new positional section consistent with them. This is a documentation correction, not an observed runtime failure.

The remaining changed claims match the inspected contracts: checked finite wrapper APIs, Go positive-zero default, raw-capable Rust source decoding and prior-Value limitation, the Rust mixed declared-field/Binary64-extra refusal, retained structural obligations, closed fixed-string-array policy members, recipe-6-only declarations/Position, first-entry preparation, report-3 reuse, and same-generator legacy-map compatibility. No claim of new native qualification or completed positional adversarial review is made here.

All outputs are confined to the assigned documentation-review scratch directory: `public-report.md`, `receipt.json`, `reviewed-draft.diff` and `source-proof.json`. The public report has no terminal line feed.

```findings
- file: website/docs/guides/generate-artifacts.md
  line: 557
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The positional API requirement names reference-only run_json without accounting for generated Rust Normalizer::normalize and Go Normalizer.Normalize text entrypoints.
```