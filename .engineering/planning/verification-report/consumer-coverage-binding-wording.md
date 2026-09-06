---
format: aep.planning-md/1
id: verification-report:consumer-coverage-binding-wording
kind: verification-report
status: draft
title: Consumer coverage binding Draft7 wording correction
relations:
- verifies: story:review-consumer-coverage
revision: 1
---
# Candidate consumer-binding vocabulary correction

The two document reviews are retained unchanged. Root verified all 58 pass1 and 65 pass2 inputs, read both complete reports and accepted all findings. Pass2 closes the four original findings and identifies one narrow Draft7 vocabulary warning. Root inspected the locked schemars 0.8.22 source: SchemaSettings defaults to draft07, definitions_path is #/definitions/, and RootSchema serializes definitions. The $defs serde alias is only a deserialization alias.

A separate v3 draft corrects that wording without changing produced schemas or silently admitting another dialect. No third broad review or executable check is claimed. Candidate source/profile and baseline output still require their staged future checkpoints after coverage integration; neither this draft nor any unknown matrix is selected or accepted here.

V3 SHA256 63d6075781dc7f348fff838962c4392df3c8e9335eaf7b5d06f8a690f630812c

```diff
--- consumer-binding-draft-v2.md
+++ consumer-binding-draft-v3.md
@@ -46,9 +46,9 @@
 
 Wire obligations have their own tagged namespace, separate from Rust obligations. A wire ID is
 `wire:RawSpecFile#` plus the RFC6901 JSON pointer into the freshly generated root schema document.
-The root, every schema node, every structural keyword/value and every properties/$defs entry are
+The root, every schema node, every structural keyword/value and every properties/definitions entry are
 accounted; each occurrence under flattened/alternative paths stays distinct. A custom schema name
-is the actual $defs key, not a guessed Rust type name. Local $ref values are obligations themselves
+is the actual definitions key, not a guessed Rust type name. Local $ref values are obligations themselves
 and must resolve against that same root's definitions; traverse each definition once and retain
 reference edges without infinite expansion. External or unresolved references fail. No total
 wire-to-Rust mapping is presumed or required: both tagged namespaces enter exact cell accounting.
@@ -63,8 +63,9 @@
 inside a manual JsonSchema implementation creates a new wire ID even if all Rust members stay
 the same. A changed existing schema value changes its shape fingerprint.
 
-Use an explicit closed schema-keyword/position grammar covering the actual current schemars
-output. Distinguish schema objects, maps of property/definition names and arbitrary literal data;
+Use an explicit closed schema-keyword/position grammar covering the locked schemars 0.8.22
+Draft 7 output, including definitions and #/definitions/ references. A different schema profile
+is a reviewed migration, not automatic $defs support. Distinguish schema objects, maps of property/definition names and arbitrary literal data;
 do not misinterpret property names or literal object keys as keywords. Unknown schema keywords,
 unrecognized shapes or reference forms fail extraction. Boolean schemas have concrete true/false
 obligations. Freeze and test this canonical wire profile before accepting a baseline; changes to
```

All prior draft/review files are preserved. One root readback logging cell raised KeyError for optional read_limit after all65 file hashes had passed; no write occurred before that failure. The completed readback was then recorded separately. This was an orchestration logging error, not a product or test failure.
