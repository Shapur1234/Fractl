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
          vulkan-headers
          vulkan-loader
          vulkan-tools
          vulkan-validation-layers

          wayland
          wayland-protocols

          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
        ];
        LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;

        commonArgs = {
          inherit src;
          strictDeps = true;

          pname = "fractl";
          version = "0.1.0";

          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        singlethreadArgs = commonArgs // {
          pname = "fractl";
          cargoExtraArgs = ''--package=fractl --no-default-features --features "f64"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        multithreadArgs = commonArgs // {
          pname = "fractl-multithread";
          cargoExtraArgs = ''--package=fractl --no-default-features --features "multithread f64"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        gpuArgs = commonArgs // {
          pname = "fractl-gpu";
          cargoExtraArgs = ''--package=fractl --no-default-features --features "gpu f32"'';

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        wasmArgs = commonArgs // {
          pname = "fractl-wasm";
          cargoExtraArgs = ''--package=fractl'';

          trunkIndexPath = "fractl/index.html";

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
            wrapProgram "$out/bin/fractl" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        multithreadCrate = craneLib.buildPackage (multithreadArgs // {
          cargoArtifacts = multithreadCargoArtifacts;

          postInstall = ''
            wrapProgram "$out/bin/fractl" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        gpuCrate = craneLib.buildPackage (gpuArgs // {
          cargoArtifacts = gpuCargoArtifacts;

          postInstall = ''
            mv $out/bin/fractl $out/bin/fractl-gpu
            wrapProgram "$out/bin/fractl-gpu" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        wasmCrate = craneLib.buildTrunkPackage (wasmArgs // {
          cargoArtifacts = wasmCargoArtifacts;
        });

        serveWasm = pkgs.writeShellScriptBin "fractl-wasm" ''
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
          fractl = singlethreadCrate;
          fractl-multithread = multithreadCrate;
          fractl-gpu = gpuCrate;
          fractl-wasm = wasmCrate;
        };

        apps = {
          fractl = flake-utils.lib.mkApp {
            name = "fractl";
            drv = singlethreadCrate;
          };
          fractl-multithread = flake-utils.lib.mkApp {
            name = "fractl-multithread";
            drv = multithreadCrate;
          };
          fractl-gpu = flake-utils.lib.mkApp {
            name = "fractl-gpu";
            drv = gpuCrate;
          };
          fractl-wasm = flake-utils.lib.mkApp {
            name = "fractl-wasm";
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

