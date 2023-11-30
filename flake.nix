# TODO: Add multiple apps
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

  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ ];
        };
        inherit (pkgs) lib;

        runtimeLibs = with pkgs; [
          wayland
          wayland-protocols

          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasInfix "/resource/" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };


        craneLib = crane.lib.${system};

        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);


        commonArgs = {
          inherit src;
          strictDeps = true;

          pname = "gui";
          version = "0.1.0";
          cargoExtraArgs = "--package=gui";

          # cargoExtraArgs = "";

          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];
          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        fractaller = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          postInstall = ''
            wrapProgram "$out/bin/fractaller" --set LD_LIBRARY_PATH ${lib.makeLibraryPath runtimeLibs};
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
            cargo-flamegraph
            cargo-outdated
            gdb
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
        };
      });
}
