#! /bin/sh

rm -rf ./docs
mkdir ./docs

nix build .#gui-wasm
cp -a ./result/. ./docs/ --no-preserve=mode,ownership

sed -i 's/\/gui/$.\/gui/g' ./docs/index.html