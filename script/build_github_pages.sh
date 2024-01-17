#! /bin/sh

rm -rf ./docs
mkdir ./docs

nix build .#fractl_gui-wasm
cp -a ./result/. ./docs/ --no-preserve=mode,ownership

sed -i 's/\/fractl_gui/.\/fractl_gui/g' ./docs/index.html
