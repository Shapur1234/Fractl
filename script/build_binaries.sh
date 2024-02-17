#! /bin/sh

function get_binary_path() {
  echo | grep -o '"[^"]\+"' $1 | tr -d '"' | grep -o '.*-wrapped'
}

rm -rf ./bin
mkdir ./bin

nix build .#fractl_gui
cp $(get_binary_path ./result/bin/fractl_gui) ./bin/fractl_gui-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/fractl_gui-linux_amd64

nix build .#fractl_gui-gpu
cp $(get_binary_path ./result/bin/fractl_gui-gpu) ./bin/fractl_gui-gpu-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/fractl_gui-gpu-linux_amd64

nix build .#fractl_gui-multithread
cp $(get_binary_path ./result/bin/fractl_gui-multithread) ./bin/fractl_gui-multithread-linux_amd64 --no-preserve=mode,ownership
chmod +x ./bin/fractl_gui-multithread-linux_amd64

nix build .#fractl_gui-win
cp ./result/bin/fractl_gui.exe ./bin/fractl_gui-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/fractl_gui-win_amd64.exe

nix build .#fractl_gui-win-gpu
cp ./result/bin/fractl_gui-gpu.exe ./bin/fractl_gui-gpu-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/fractl_gui-gpu-win_amd64.exe

nix build .#fractl_gui-win-multithread
cp ./result/bin/fractl_gui-multithread.exe ./bin/fractl_gui-multithread-win_amd64.exe --no-preserve=mode,ownership
chmod +x ./bin/fractl_gui-multithread-win_amd64.exe
