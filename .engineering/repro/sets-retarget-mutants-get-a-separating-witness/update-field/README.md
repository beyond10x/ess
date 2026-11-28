Run from this directory with ess 0.99.9; the parent directory's `ess-inputs.yaml` pins 0.99.2, so
name the release: `ESS_TOOLCHAIN=0.99.9 ess verify conform mutate --path . --target interpreted
--class sets-retarget --class sets-drop`.

The model is the parent directory's, with an `Adjust` command added that updates both fields from
two `Integer` inputs. On 0.99.9: 6 mutants, 4 killed, 2 unwitnessed. The 2 unwitnessed are the
`sets-retarget` mutants of `Define` (the create shape and its `exists` trigger, still red). The 4
mutants of `Adjust` (`sets-retarget` and `sets-drop` of each field) are killed: the update shape a
downstream audit reported as unwitnessed does not reproduce here.
