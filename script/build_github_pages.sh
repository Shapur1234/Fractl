#! /bin/sh

rm -rf ./docs
mkdir ./docs

nix build .#fractl-gui-wasm
cp -a ./result/. ./docs/ --no-preserve=mode,ownership

sed -i 's/\/fractl-gui/.\/fractl-gui/g' ./docs/index.html
