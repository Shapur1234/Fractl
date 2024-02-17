{
  description = "Fractl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";


    # wasm-bindgen-cli 0.2.91
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/38513315386e828b9d296805657726e63e338076";

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

          pname = "fractl_gui";
          version = "0.1.0";

          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        singlethreadArgs = commonArgs // {
          pname = "fractl_gui";
          cargoExtraArgs = ''--package=fractl_gui --no-default-features --features "f64"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        multithreadArgs = commonArgs // {
          pname = "fractl_gui-multithread";
          cargoExtraArgs = ''--package=fractl_gui --no-default-features --features "multithread f64"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        gpuArgs = commonArgs // {
          pname = "fractl_gui-gpu";
          cargoExtraArgs = ''--package=fractl_gui --no-default-features --features "gpu f32"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        wasmArgs = commonArgs // {
          pname = "fractl_gui-wasm";
          cargoExtraArgs = ''--package=fractl_gui'';

          trunkIndexPath = "fractl_gui/index.html";

          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

          wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.91";
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
            wrapProgram "$out/bin/fractl_gui" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        multithreadCrate = craneLib.buildPackage (multithreadArgs // {
          cargoArtifacts = multithreadCargoArtifacts;

          postInstall = ''
            mv $out/bin/fractl_gui $out/bin/fractl_gui-multithread
            wrapProgram "$out/bin/fractl_gui-multithread" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        gpuCrate = craneLib.buildPackage (gpuArgs // {
          cargoArtifacts = gpuCargoArtifacts;

          postInstall = ''
            mv $out/bin/fractl_gui $out/bin/fractl_gui-gpu
            wrapProgram "$out/bin/fractl_gui-gpu" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
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
            mv $out/bin/fractl_gui.exe $out/bin/fractl_gui-multithread.exe
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
            mv $out/bin/fractl_gui.exe $out/bin/fractl_gui-gpu.exe
          '';
        });
        wasmCrate = craneLib.buildTrunkPackage (wasmArgs // {
          cargoArtifacts = wasmCargoArtifacts;
        });

        serveWasm = pkgs.writeShellScriptBin "${wasmArgs.pname}" ''
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
          fractl_gui = singlethreadCrate;
          fractl_gui-multithread = multithreadCrate;
          fractl_gui-gpu = gpuCrate;
          fractl_gui-win = singlethreadWinCrate;
          fractl_gui-win-multithread = multithreadWinCrate;
          fractl_gui-win-gpu = gpuWinCrate;
          fractl_gui-wasm = wasmCrate;
        };

        apps = {
          fractl_gui = flake-utils.lib.mkApp {
            name = "fractl_gui";
            drv = singlethreadCrate;
          };
          fractl_gui-multithread = flake-utils.lib.mkApp {
            name = "fractl_gui-multithread";
            drv = multithreadCrate;
          };
          fractl_gui-gpu = flake-utils.lib.mkApp {
            name = "fractl_gui-gpu";
            drv = gpuCrate;
          };
          fractl_gui-wasm = flake-utils.lib.mkApp {
            name = "fractl_gui-wasm";
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

