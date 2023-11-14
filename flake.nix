{
  description = "Fractaller";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, fenix, rust-overlay, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;

        craneLib = crane.lib.${system};
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        runtimeLibraries = with pkgs; [
          wayland
          wayland-protocols

          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        commonArgs = {
          inherit src;
          strictDeps = true;

          cargoExtraArgs = "--features rayon";

          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];
          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibraries;
        };

        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);

        rust_wasm32_target = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-src" ];
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        fractaller = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          postInstall = ''
            wrapProgram "$out/bin/fractaller" --set LD_LIBRARY_PATH ${lib.makeLibraryPath runtimeLibraries};
          '';
        });
      in
      {
        checks = {
          my-crate = fractaller;

          my-crate-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          my-crate-fmt = craneLib.cargoFmt {
            inherit src;
          };

          my-crate-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };
        };

        packages = {
          default = fractaller;
          my-crate-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        apps.default = flake-utils.lib.mkApp {
          drv = fractaller;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs;[
            rust_wasm32_target

            cargo-flamegraph
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibraries;
        };
      });
}
