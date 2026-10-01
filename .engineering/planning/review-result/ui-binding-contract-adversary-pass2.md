---
format: aep.planning-md/3
id: review-result:ui-binding-contract-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:ui-binding-contract (wave ui-live-apps-w1)
relations:
- reviews: story:ui-binding-contract
revision: 1
---
needs-change

Adversary pass 2 on story:ui-binding-contract at 184abadd2a. Cases executed 130 to 135, red 1, introduced by correction 1. Test file: crates/ui/ess-ui-check/tests/adversary_binding_pass2.rs.

- warning: correction 1 treats an export as binding no params, although export has a params slot (ess-ui.md:1939); the documented `export: {reads: V, params: same_as(list)}` (ess-ui.md:1976, used at examples/partner-portal/ui.yaml:256,447) on a view with a required param is a false read_params error.
- not tested: a confirm's references and a channel's carried view on a view with a required param now also report "binds none" (partner portal delete confirm, line 293).

Held: Optional-only params give no finding; recorded refusals sit in the command's error routes at the recorded status; an error shared by two commands; a command with no declared errors binds {}; two spellings give one route each; JSON identical across model file order; wire codes agree with both servers; ess-ui depends only on serde, serde_json, serde_yaml; a paged view read with paging: client is refused.

```findings
[{"file":"crates/ui/ess-ui-check/src/model.rs","line":288,"category":"contract-drift","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"the correction treats an export as a read that binds no parameter, although export has a params slot; the documented `export: {reads: V, params: same_as(list)}` on a view with a required parameter is now a false read_params error (adv2_an_export_that_binds_its_params_with_same_as_is_not_an_unbound_param_error)"}]
```
