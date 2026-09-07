#!/usr/bin/env bash
set -euo pipefail

: "${COMPONENT_SERVICE:?component service is required}"
: "${COMPONENT_VERSION:?component version is required}"
: "${ESS_REVISION:?ESS revision is required}"
: "${CHECK_COMMAND:?check command is required}"
: "${COMPONENT_PATH:?component path is required}"
: "${BUILD_PATH:?build path is required}"
: "${BUILD_IR:?build IR path is required}"
: "${RUNTIME_IR:?runtime IR path is required}"
: "${IMAGE_REPOSITORY:?image repository is required}"
: "${BUILD_IMAGE:?build-image choice is required}"
: "${CHART_NAMESPACE:?chart namespace is required}"
: "${EVIDENCE_REPOSITORY:?evidence repository is required}"
: "${BUNDLE_REPOSITORY:?bundle repository is required}"
: "${SPEC_PATH:?explicit ESS model path is required}"
: "${CONFORMANCE_REPORT:?original conformance report/2 file is required}"

if { [ -n "${CONFORMANCE_SUITE:-}" ] && [ -n "${CONFORMANCE_SUITE_INPUT:-}" ]; } ||
   { [ -z "${CONFORMANCE_SUITE:-}" ] && [ -z "${CONFORMANCE_SUITE_INPUT:-}" ]; }; then
  printf 'exactly one conformance-suite or conformance-suite-input is required\n' >&2
  exit 2
fi

case "$COMPONENT_SERVICE" in
  "" | *[!a-z0-9-]*)
    printf 'invalid ESS service identifier: %s\n' "$COMPONENT_SERVICE" >&2
    exit 2
    ;;
esac
case "$COMPONENT_VERSION" in
  *.*.*) ;;
  *)
    printf 'invalid component version: %s\n' "$COMPONENT_VERSION" >&2
    exit 2
    ;;
esac
case "$BUILD_IMAGE" in
  true | false) ;;
  *)
    printf 'build-image must be true or false\n' >&2
    exit 2
    ;;
esac
if [ "${GITHUB_REF_TYPE:-}" = tag ]; then
  test "$GITHUB_REF_NAME" = "$COMPONENT_VERSION"
fi

mkdir -p target/release
ess_bin="$PWD/target/ess/$ESS_REVISION/bin/ess"
test -x "$ess_bin"
component_ir=target/release/component.ir.json
"$ess_bin" generate component compile --path "$COMPONENT_PATH" --out "$component_ir"
test "$(jq -r .system "$component_ir")" = "$COMPONENT_SERVICE"

# The caller supplied these originals before the action. Keep a byte snapshot and pin it for
# every later gate; the generic repository check is never the report producer for this action.
snapshot_root="$(mktemp -d target/release/conformance-inputs.XXXXXX)"
report_snapshot="$snapshot_root/conformance-report.json"
input_snapshot="$snapshot_root/expected-input.json"
cp -- "$CONFORMANCE_REPORT" "$report_snapshot"
expected_option=--expected-suite
expected_path="${CONFORMANCE_SUITE:-}"
if [ -n "${CONFORMANCE_SUITE_INPUT:-}" ]; then
  expected_option=--expected-suite-input
  expected_path="$CONFORMANCE_SUITE_INPUT"
fi
cp -- "$expected_path" "$input_snapshot"
report_bytes_digest="sha256:$(sha256sum "$report_snapshot" | awk '{print $1}')"
input_bytes_digest="sha256:$(sha256sum "$input_snapshot" | awk '{print $1}')"
qualification=(--spec "$SPEC_PATH" --report "$report_snapshot"
  "$expected_option" "$input_snapshot"
  --report-sha256 "$report_bytes_digest" --expected-input-sha256 "$input_bytes_digest")
deployment_context=(--component-ir "$component_ir" --build-ir "$BUILD_IR" --runtime-ir "$RUNTIME_IR")
"$ess_bin" generate release check-conformance "${qualification[@]}" "${deployment_context[@]}"

bash -euo pipefail -c "$CHECK_COMMAND" 2>&1 | tee target/release/check.log
image_tag="$IMAGE_REPOSITORY:$COMPONENT_VERSION"

if [ "$BUILD_IMAGE" = true ]; then
  "$ess_bin" generate build execute \
    --path "$BUILD_PATH" \
    --workdir . \
    --projection-out target/release/app-buildkit \
    --target app \
    --set "app.tags=$image_tag" \
    --push
fi
image_digest="$(docker buildx imagetools inspect "$image_tag" --format '{{json .Manifest.Digest}}' | jq -r .)"
image_raw="$(docker buildx imagetools inspect "$image_tag" --raw)"
image_platform_digest="$(
  jq -r '
    if has("manifests") then
      first(.manifests[] | select(.platform.os == "linux" and .platform.architecture == "amd64") | .digest)
    else empty
    end
  ' <<<"$image_raw"
)"
if [ -z "$image_platform_digest" ]; then
  image_platform_digest="$image_digest"
fi

"$ess_bin" generate build execute \
  --path "$BUILD_PATH" \
  --workdir . \
  --projection-out target/release/chart-buildkit \
  --target chart
chart_archive="$(find out/chart -type f -name '*-chart.tgz' -print -quit)"
test -n "$chart_archive"
chart_name="$(helm show chart "$chart_archive" | awk '$1 == "name:" { print $2; exit }')"
chart_push="$(helm push "$chart_archive" "$CHART_NAMESPACE" 2>&1)"
printf '%s\n' "$chart_push"
chart_digest="$(grep -Eo 'sha256:[0-9a-f]{64}' <<<"$chart_push" | tail -1)" || chart_digest=""
chart_reference="${CHART_NAMESPACE#oci://}/$chart_name"
if [ -z "$chart_digest" ]; then
  printf 'helm push did not report a chart digest\n' >&2
  exit 1
fi

cosign sign --yes "${IMAGE_REPOSITORY}@${image_digest}"
cosign sign --yes "${chart_reference}@${chart_digest}"
image_signature_reference="$(cosign triangulate "${IMAGE_REPOSITORY}@${image_digest}")"
chart_signature_reference="$(cosign triangulate "${chart_reference}@${chart_digest}")"
image_signature_digest="$(docker buildx imagetools inspect "$image_signature_reference" --format '{{json .Manifest.Digest}}' | jq -r .)"
chart_signature_digest="$(docker buildx imagetools inspect "$chart_signature_reference" --format '{{json .Manifest.Digest}}' | jq -r .)"

syft scan "${IMAGE_REPOSITORY}@${image_digest}" -o cyclonedx-json=target/release/sbom.json
semantic_digest="$(jq -r .semantic_digest "$RUNTIME_IR")"
build_digest="$(jq -r .build_digest "$RUNTIME_IR")"
runtime_digest="sha256:$(sha256sum "$RUNTIME_IR" | awk '{print $1}')"
source_commit="${GITHUB_SHA:-$(git rev-parse HEAD)}"
run_reference="${GITHUB_SERVER_URL:-local}/${GITHUB_REPOSITORY:-unknown}/actions/runs/${GITHUB_RUN_ID:-unknown}"

jq -n \
  --arg source_commit "$source_commit" \
  --arg run "$run_reference" \
  --arg image "${IMAGE_REPOSITORY}@${image_digest}" \
  --arg chart "${chart_reference}@${chart_digest}" \
  --arg semantic_digest "$semantic_digest" \
  --arg build_digest "$build_digest" \
  --arg runtime_digest "$runtime_digest" \
  '{format: "slsa-provenance-summary/1", source_commit: $source_commit, run: $run,
    subjects: [$image, $chart],
    ess: {semantic_digest: $semantic_digest, build_digest: $build_digest, runtime_digest: $runtime_digest}}' \
  > target/release/provenance.json
jq -n \
  --arg image_reference "$image_signature_reference" \
  --arg image_digest "$image_signature_digest" \
  --arg chart_reference "$chart_signature_reference" \
  --arg chart_digest "$chart_signature_digest" \
  '{format: "sigstore-signature-set/1", signatures: [
    {subject: "runtime", reference: $image_reference, digest: $image_digest},
    {subject: "chart", reference: $chart_reference, digest: $chart_digest}]}' \
  > target/release/signatures.json

publish_evidence() {
  local kind="$1"
  local path="$2"
  local media_type="$3"
  local destination="$EVIDENCE_REPOSITORY:$COMPONENT_VERSION-$kind"
  local digest
  digest="$(oras push --no-tty \
    --artifact-type "application/vnd.beyond10x.ess.evidence.$kind.v1" \
    --format 'go-template={{.digest}}' \
    "$destination" "$path:$media_type")" || return $?
  printf '%s\n' "$digest"
}

# Images/charts may already have been uploaded. A refusal here blocks all subsequent evidence
# and bundle uploads; it makes no rollback claim about those earlier operations.
"$ess_bin" generate release check-conformance "${qualification[@]}" "${deployment_context[@]}"
provenance_reference="$EVIDENCE_REPOSITORY:$COMPONENT_VERSION-provenance"
sbom_reference="$EVIDENCE_REPOSITORY:$COMPONENT_VERSION-sbom"
signature_reference="$EVIDENCE_REPOSITORY:$COMPONENT_VERSION-signature"
conformance_reference="$EVIDENCE_REPOSITORY:$COMPONENT_VERSION-conformance"
# Preserve child failure in the assignment. A read over process substitution masks that status.
provenance_digest="$(publish_evidence provenance target/release/provenance.json application/json)"
sbom_digest="$(publish_evidence sbom target/release/sbom.json application/vnd.cyclonedx+json)"
signature_digest="$(publish_evidence signature target/release/signatures.json application/json)"
conformance_output="$("$ess_bin" generate release publish-conformance \
  "${qualification[@]}" "${deployment_context[@]}" --to "$conformance_reference")"
printf '%s\n' "$conformance_output"
conformance_digest="${conformance_output##* }"

evidence="$(jq -n \
  --arg provenance_reference "$provenance_reference" --arg provenance_digest "$provenance_digest" \
  --arg sbom_reference "$sbom_reference" --arg sbom_digest "$sbom_digest" \
  --arg signature_reference "$signature_reference" --arg signature_digest "$signature_digest" \
  --arg conformance_reference "$conformance_reference" --arg conformance_digest "$conformance_digest" \
  '{provenance: {reference: $provenance_reference, digest: $provenance_digest},
    sbom: {reference: $sbom_reference, digest: $sbom_digest},
    signature: {reference: $signature_reference, digest: $signature_digest},
    conformance: {reference: $conformance_reference, digest: $conformance_digest}}')"

runtime_release_unit="$(jq -r .release_units.runtime "$component_ir")"
chart_release_unit="$(jq -r .release_units.chart "$component_ir")"
jq -n \
  --arg release_unit "$runtime_release_unit" --arg system "$COMPONENT_SERVICE" \
  --arg version "$COMPONENT_VERSION" --arg source_commit "$source_commit" \
  --arg semantic_digest "$semantic_digest" --arg build_digest "$build_digest" \
  --arg runtime_digest "$runtime_digest" --arg reference "$IMAGE_REPOSITORY" \
  --arg digest "$image_digest" --arg platform_digest "$image_platform_digest" \
  --argjson evidence "$evidence" \
  '{format: "ess-release/1", release_unit: $release_unit, system: $system, version: $version,
    source_commit: $source_commit, semantic_digest: $semantic_digest, build_digest: $build_digest,
    runtime_digest: $runtime_digest, artifacts: {app: {build_output: "app", kind: "oci_image",
    reference: $reference, digest: $digest, platforms: {"linux/amd64": $platform_digest}}},
    evidence: $evidence}' > target/release/runtime-release.json
jq -n \
  --arg release_unit "$chart_release_unit" --arg system "$COMPONENT_SERVICE" \
  --arg version "$COMPONENT_VERSION" --arg source_commit "$source_commit" \
  --arg semantic_digest "$semantic_digest" --arg build_digest "$build_digest" \
  --arg runtime_digest "$runtime_digest" --arg reference "$chart_reference" \
  --arg digest "$chart_digest" --argjson evidence "$evidence" \
  '{format: "ess-release/1", release_unit: $release_unit, system: $system, version: $version,
    source_commit: $source_commit, semantic_digest: $semantic_digest, build_digest: $build_digest,
    runtime_digest: $runtime_digest, artifacts: {chart: {build_output: "chart", kind: "helm_chart",
    reference: $reference, digest: $digest}}, evidence: $evidence}' > target/release/chart-release.json

"$ess_bin" generate release verify --path target/release/runtime-release.json --build-ir "$BUILD_IR" --runtime-ir "$RUNTIME_IR"
"$ess_bin" generate release verify --path target/release/chart-release.json --build-ir "$BUILD_IR" --runtime-ir "$RUNTIME_IR"
"$ess_bin" generate release bundle \
  --component-ir "$component_ir" --build-ir "$BUILD_IR" --runtime-ir "$RUNTIME_IR" \
  --release target/release/runtime-release.json --release target/release/chart-release.json \
  --out target/release/ess-release-bundle.json
publish_output="$("$ess_bin" generate release publish \
  --path target/release/ess-release-bundle.json --to "$BUNDLE_REPOSITORY:$COMPONENT_VERSION" \
  "${qualification[@]}")"
printf '%s\n' "$publish_output"
bundle_digest="${publish_output##* }"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    printf '### %s ESS component %s\n\n' "$COMPONENT_SERVICE" "$COMPONENT_VERSION"
    printf -- '- Bundle: %s@%s\n' "$BUNDLE_REPOSITORY" "$bundle_digest"
    printf -- '- Runtime: %s@%s\n' "$IMAGE_REPOSITORY" "$image_digest"
    printf -- '- Chart: %s@%s\n' "$chart_reference" "$chart_digest"
    printf -- '- conformance: passed for the supplied exact declared selection\n'
    printf -- '- attachment binding: unverified\n'
    printf -- '- producer origin: unverified\n'
    printf -- '- artifact execution: unverified\n'
    printf -- '- signature verification: unsupported\n'
  } >> "$GITHUB_STEP_SUMMARY"
fi
