#! /bin/sh

rm -rf ./bin
mkdir ./bin

nix build .#fractl-gui
cp ./result/bin/fractl-gui ./bin/fractl-gui-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/fractl-gui-linux_amd64

nix build .#fractl-gui-gpu
cp ./result/bin/fractl-gui-gpu ./bin/fractl-gui-gpu-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/fractl-gui-gpu-linux_amd64

nix build .#fractl-gui-multithread
cp ./result/bin/fractl-gui-multithread ./bin/fractl-gui-multithread-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/fractl-gui-multithread-linux_amd64

nix build .#fractl-gui-win
cp ./result/bin/fractl-gui.exe ./bin/fractl-gui-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/fractl-gui-win_amd64.exe

nix build .#fractl-gui-win-gpu
cp ./result/bin/fractl-gui-gpu.exe ./bin/fractl-gui-gpu-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/fractl-gui-gpu-win_amd64.exe

nix build .#fractl-gui-win-multithread
cp ./result/bin/fractl-gui-multithread.exe ./bin/fractl-gui-multithread-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/fractl-gui-multithread-win_amd64.exe
