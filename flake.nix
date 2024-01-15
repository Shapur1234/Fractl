{
  description = "Fractl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # The version of wasm-bindgen-cli needs to match the version in Cargo.lock
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/75c13bf6aac049d5fec26c07c28389a72c25a30b";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, nixpkgs-for-wasm-bindgen, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [
            "wasm32-unknown-unknown"
            "x86_64-pc-windows-gnu"
          ];
          extensions = [ "rust-src" ];
        };
        craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope' (_final: _prev: {
          inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
        });

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.wgsl" path) ||
            (lib.hasSuffix "\.ttf" path) ||
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.css" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };

        runtimeLibs = with pkgs; [
          vulkan-loader

          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi

          wayland
          wayland-protocols
        ];
        LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;

        commonArgs = {
          inherit src;
          strictDeps = true;

          pname = "gui";
          version = "0.1.0";

          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        singlethreadArgs = commonArgs // {
          pname = "gui";
          cargoExtraArgs = ''--package=gui --no-default-features --features "f64"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        multithreadArgs = commonArgs // {
          pname = "gui-multithread";
          cargoExtraArgs = ''--package=gui --no-default-features --features "multithread f64"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        gpuArgs = commonArgs // {
          pname = "gui-gpu";
          cargoExtraArgs = ''--package=gui --no-default-features --features "gpu f32"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        wasmArgs = commonArgs // {
          pname = "gui-wasm";
          cargoExtraArgs = ''--package=gui'';

          trunkIndexPath = "gui/index.html";

          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

          wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.89";
          };
        };

        singlethreadCargoArtifacts = craneLib.buildDepsOnly singlethreadArgs;
        multithreadCargoArtifacts = craneLib.buildDepsOnly multithreadArgs;
        gpuCargoArtifacts = craneLib.buildDepsOnly gpuArgs;
        wasmCargoArtifacts = craneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });

        singlethreadCrate = craneLib.buildPackage (singlethreadArgs // {
          cargoArtifacts = singlethreadCargoArtifacts;

          postInstall = ''
            wrapProgram "$out/bin/gui" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        multithreadCrate = craneLib.buildPackage (multithreadArgs // {
          cargoArtifacts = multithreadCargoArtifacts;

          postInstall = ''
            mv $out/bin/gui $out/bin/gui-multithread
            wrapProgram "$out/bin/gui-multithread" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        gpuCrate = craneLib.buildPackage (gpuArgs // {
          cargoArtifacts = gpuCargoArtifacts;

          postInstall = ''
            mv $out/bin/gui $out/bin/gui-gpu
            wrapProgram "$out/bin/gui-gpu" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });

        singlethreadWinCrate = craneLib.buildPackage (singlethreadArgs // {
          strictDeps = true;
          doCheck = false;

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
          ];
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS =
            "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
        });
        multithreadWinCrate = craneLib.buildPackage (multithreadArgs // {
          strictDeps = true;
          doCheck = false;

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
          ];
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS =
            "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";

          postInstall = ''
            mv $out/bin/gui.exe $out/bin/gui-multithread.exe
          '';
        });
        gpuWinCrate = craneLib.buildPackage (gpuArgs // {
          strictDeps = true;
          doCheck = false;

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
          ];
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS =
            "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";

          postInstall = ''
            mv $out/bin/gui.exe $out/bin/gui-gpu.exe
          '';
        });

        wasmCrate = craneLib.buildTrunkPackage (wasmArgs // {
          cargoArtifacts = wasmCargoArtifacts;
        });

        serveWasm = pkgs.writeShellScriptBin "gui-wasm" ''
          ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${wasmCrate} 8000
        '';

        signlethreadCrateClippy = craneLib.cargoClippy (singlethreadArgs // {
          inherit src;
          cargoArtifacts = singlethreadCargoArtifacts;

          cargoClippyExtraArgs = "-- --deny warnings";
        });
        multithreadCrateClippy = craneLib.cargoClippy (multithreadArgs // {
          inherit src;
          cargoArtifacts = multithreadCargoArtifacts;

          cargoClippyExtraArgs = "-- --deny warnings";
        });
        gpuCrateClippy = craneLib.cargoClippy (gpuArgs // {
          inherit src;
          cargoArtifacts = gpuCargoArtifacts;

          cargoClippyExtraArgs = "-- --deny warnings";
        });
      in
      {
        checks = {
          inherit singlethreadCrate;
          inherit gpuCrate;
          inherit wasmCrate;

          inherit signlethreadCrateClippy;
          inherit multithreadCrateClippy;
          inherit gpuCrateClippy;

          fmt = craneLib.cargoFmt commonArgs;
        };

        packages = {
          gui = singlethreadCrate;
          gui-multithread = multithreadCrate;
          gui-gpu = gpuCrate;
          gui-win = singlethreadWinCrate;
          gui-win-multithread = multithreadWinCrate;
          gui-win-gpu = gpuWinCrate;
          gui-wasm = wasmCrate;
        };

        apps = {
          gui = flake-utils.lib.mkApp {
            name = "gui";
            drv = singlethreadCrate;
          };
          gui-multithread = flake-utils.lib.mkApp {
            name = "gui-multithread";
            drv = multithreadCrate;
          };
          gui-gpu = flake-utils.lib.mkApp {
            name = "gui-gpu";
            drv = gpuCrate;
          };
          gui-wasm = flake-utils.lib.mkApp {
            name = "gui-wasm";
            drv = serveWasm;
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            rustToolchain
            runtimeLibs

            trunk
            cargo-flamegraph
            cargo-outdated
            gdb
          ];

          inherit LD_LIBRARY_PATH;
        };
      }
    );
}

