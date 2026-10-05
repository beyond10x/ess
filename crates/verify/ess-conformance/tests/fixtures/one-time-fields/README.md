# Shared live multi-field exemption controls

The same admitted suite has required marked String fields `secret` and `recovery`, plus unmarked
`audit`. The shared FieldService executes real calls through the original Service and supplies both
fresh values. Healthy issuance/rotation passes. Copying the current secret into recovery, copying it
into audit, or returning a previous secret in recovery on rotation must fail ESS-CF-DISCLOSURE.
The manifest records actual native counts and callback traces; scenario/count serialization is
checked against every nonempty returned marked value and the original first token. The original
nineteen single-field vectors remain unchanged.
