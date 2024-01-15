#! /bin/sh

rm -rf ./bin
mkdir ./bin

nix build .#gui
cp ./result/bin/gui ./bin/gui-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/gui-linux_amd64

nix build .#gui-gpu
cp ./result/bin/gui-gpu ./bin/gui-gpu-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/gui-gpu-linux_amd64

nix build .#gui-multithread
cp ./result/bin/gui-multithread ./bin/gui-multithread-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/gui-multithread-linux_amd64

nix build .#gui-win
cp ./result/bin/gui.exe ./bin/gui-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/gui-win_amd64.exe

nix build .#gui-win-gpu
cp ./result/bin/gui-gpu.exe ./bin/gui-gpu-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/gui-gpu-win_amd64.exe

nix build .#gui-win-multithread
cp ./result/bin/gui-multithread.exe ./bin/gui-multithread-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/gui-multithread-win_amd64.exe
