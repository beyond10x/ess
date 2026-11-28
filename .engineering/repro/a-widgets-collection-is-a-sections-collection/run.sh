#!/bin/sh
# A collection that a widget's body draws is not addressed or declared like a section's collection.
# Usage: sh run.sh   (needs ess on PATH; reads nothing outside this directory)
set -u
cd "$(dirname "$0")" || exit 2

echo '$ ess --version'
ess --version

echo
echo '$ ess ui check --path ui/states.ui.yaml --model spec   (states on the collection in a widget body, and on a section'"'"'s own collection)'
ess ui check --path ui/states.ui.yaml --model spec

echo
echo '$ ess ui check --path ui/section-states.ui.yaml --model spec   (states on the section that draws the widget)'
ess ui check --path ui/section-states.ui.yaml --model spec
echo
echo '$ ess ui test --path ui/section-states.ui.yaml --model spec tests/empty.yaml   (the widget reads no row)'
ess ui test --path ui/section-states.ui.yaml --model spec tests/empty.yaml

echo
echo '$ ess ui test --path ui/widget.ui.yaml --model spec tests/addressing.yaml'
ess ui test --path ui/widget.ui.yaml --model spec tests/addressing.yaml

echo
echo '$ ess ui test --path ui/widget.ui.yaml --model spec tests/width.yaml   (a label longer than the screen)'
ess ui test --path ui/widget.ui.yaml --model spec tests/width.yaml
