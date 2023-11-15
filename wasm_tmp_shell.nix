let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  # rust_channel = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

  rust_channel = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "wasm32-unknown-unknown" ];
    extensions = [ "rust-src" ];
  };
in
with pkgs;

mkShell rec {
  nativeBuildInputs = [
    rust_channel

    pkg-config
  ];
  buildInputs = [
    alsa-lib
    libxkbcommon
    udev
    vulkan-loader

    wayland

    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
