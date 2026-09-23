---
format: aep.planning-md/1
id: review-result:wire-behavior-period-design-pass-1
kind: review-result
status: active
title: Wire container support needs complete member evidence
relations:
- reviews: task:consumer-accounting-wire-behavior-period-parity
revision: 1
---
needs-revision

task:consumer-accounting-wire-behavior-period-parity — Add complete loader controls and returned-IR or stage-attributed refusal observations for every member and local-reference branch contained by each whole `RawBindingSpec`, `NamedType`, `RawComponentSpec`, `RawCommandSpec`, and `RawErrorSpec` node claimed `Supported`, because the prescribed controls cover selected changed fields while a Supported cell preserves no ShapeDelta residual for the unobserved remainder — local-evidence:ess-evolution/waves/0004-ess-accounting/wire-obligation-scope-result.md:47-71; docs/design/consumer-accounting-applicability.md:100-110

What I read: 2 planning artifacts (the task and inherited parent story), 5 binding/accounting design contracts, the retained 58-row scope proposal, the exact Period/parser/schema and loader sources, the seven current loader cases, the 27-claim native authority, and aggregate validation/source pins, using `nl -ba ... | sed -n`, `rg -n`, and read-only `git status`, `git diff`, and `git rev-parse`.

Facts observed: `Period` currently derives an integer schema although Serde enters through `String`; the proposed bounded decimal arms partition `1..=4294967295`, and `minLength: 4`, `maxLength: 13`, the anchored pattern, and the separate forbidden-character `not` guard jointly exclude the named noncanonical and line-ending forms. The production loader parses, assembles, and compiles without evaluating JSON Schema, and its refusal fields distinguish the three stages. The wire extractor already admits `pattern`, `minLength`, `maxLength`, and `not`. The existing seven cases and all 27 adopted Rust claims bind the shared loader-case source file, so their source identities must be refreshed after the proposed edits. The four wire aggregate parents can retain ShapeDelta, unchanged profile/residual semantics, and frozen old witnesses while root and definitions receive newly generated current hashes.

What I could not establish: revised Period/root/definitions hashes, the post-fix finite wire inventory, actual paired pointer/loader execution, or exact refusal observations; no implementation or executable evidence exists yet. It is also unexecuted inference that the two RawOutcome aggregate hashes remain unchanged after the isolated Period schema correction.

```findings
- file: local-evidence:ess-evolution/waves/0004-ess-accounting/wire-obligation-scope-result.md
  line: 47
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Add complete loader controls and returned-IR or stage-attributed refusal observations for every member and local-reference branch contained by each whole `RawBindingSpec`, `NamedType`, `RawComponentSpec`, `RawCommandSpec`, and `RawErrorSpec` node claimed `Supported`, because the prescribed controls cover selected changed fields while a Supported cell preserves no ShapeDelta residual for the unobserved remainder
```
