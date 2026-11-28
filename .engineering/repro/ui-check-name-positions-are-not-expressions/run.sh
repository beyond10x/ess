#!/bin/sh
# A name that begins `channel.` is read as a channel expression where a name stands.
# Usage: sh run.sh   (needs ess on PATH; reads nothing outside this directory)
set -u
cd "$(dirname "$0")" || exit 2

echo '$ ess --version'
ess --version

echo
echo '$ ess specify validate --path spec   (the model: domain shop.channel, view Items)'
ess specify validate --path spec

echo
echo '$ ess ui check --path ui/read.ui.yaml --model spec   (a section reads {view: channel.Items}, a name in reads/view)'
ess ui check --path ui/read.ui.yaml --model spec

echo
echo '$ ess ui check --path ui/widget-relative.ui.yaml --model spec   (a widget named channel.Items, used by component: channel.Items)'
ess ui check --path ui/widget-relative.ui.yaml --model spec

echo
echo '$ ess ui check --path ui/widget-full.ui.yaml --model spec   (the same widget named shop.channel.Items)'
ess ui check --path ui/widget-full.ui.yaml --model spec

echo
echo '$ ess ui check --path ui/names.ui.yaml --model spec   (a command channel.AddItem in does, a page channel.list in navigate/to and in navigation)'
ess ui check --path ui/names.ui.yaml --model spec
