{
  description = "Fractaller";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # The version of wasm-bindgen-cli needs to match the version in Cargo.lock
    # Update this to include the version you need
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/067e11fb004fd21f18000b20e724eededd649544";

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
          xorg.libXrandr
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

        cliArgs = commonArgs // {
          pname = "fractl-cli";
          cargoExtraArgs = "--package=fractl-cli --features multithread";
        };
        guiArgs = commonArgs // {
          pname = "fractl-gui";
          cargoExtraArgs = "--package=fractl-gui --features multithread";

          buildInputs = [
            runtimeLibs
          ];
          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };
        wasmArgs = commonArgs // {
          pname = "fractl-gui-wasm";
          cargoExtraArgs = "--package=fractl-gui";

          trunkIndexPath = "fractl-gui/index.html";

          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };

        cliCargoArtifacts = craneLib.buildDepsOnly cliArgs;
        guiCargoArtifacts = craneLib.buildDepsOnly guiArgs;
        wasmCargoArtifacts = craneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });

        cliCrate = craneLib.buildPackage (cliArgs // {
          cargoArtifacts = cliCargoArtifacts;
        });
        guiCrate = craneLib.buildPackage (guiArgs // {
          cargoArtifacts = guiCargoArtifacts;

          postInstall = ''
            wrapProgram "$out/bin/fractl-gui" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });
        wasmCrate = craneLib.buildTrunkPackage (wasmArgs // {
          cargoArtifacts = wasmCargoArtifacts;
        });

        serveWasm = pkgs.writeShellScriptBin "fractl-wasm" ''
          ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${wasmCrate} 8000
        '';

        cliCrateClippy = craneLib.cargoClippy (cliArgs // {
          inherit src;
          cargoArtifacts = cliCargoArtifacts;

          cargoClippyExtraArgs = "-- --deny warnings";
        });
        guiCrateClippy = craneLib.cargoClippy (guiArgs // {
          inherit src;
          cargoArtifacts = guiCargoArtifacts;

          cargoClippyExtraArgs = "-- --deny warnings";
        });
      in
      {
        checks = {
          inherit
            cliCrate
            cliCrateClippy
            guiCrate
            guiCrateClippy
            wasmCrate;

          fmt = craneLib.cargoFmt commonArgs;
        };

        packages = {
          fractl-cli = cliCrate;
          fractl-gui = guiCrate;
          fractl-wasm = wasmCrate;
        };

        apps = {
          fractl-cli = flake-utils.lib.mkApp {
            name = "fractl-cli";
            drv = cliCrate;
          };
          fractl-gui = flake-utils.lib.mkApp {
            name = "fractl-gui";
            drv = guiCrate;
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

