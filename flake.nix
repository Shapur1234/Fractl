{
  description = "Fractaller";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

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

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        craneLib = crane.lib.${system};

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasInfix "/resource/" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };

        LD_LIBRARY_PATH = lib.makeLibraryPath (with pkgs; [
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ]);

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
          cargoExtraArgs = "--package=fractl-cli";
        };

        guiArgs = commonArgs // {
          pname = "fractl-gui";
          cargoExtraArgs = "--package=fractl-gui";

          nativeBuildInputs = with pkgs; [
            makeWrapper
          ];

          inherit LD_LIBRARY_PATH;
        };

        cliCargoArtifacts = craneLib.buildDepsOnly cliArgs;
        guiCargoArtifacts = craneLib.buildDepsOnly guiArgs;

        cliCrate = craneLib.buildPackage (cliArgs // {
          cargoArtifacts = cliCargoArtifacts;
        });
        guiCrate = craneLib.buildPackage (guiArgs // {
          cargoArtifacts = guiCargoArtifacts;

          postInstall = ''
            wrapProgram "$out/bin/fractl-gui" --set LD_LIBRARY_PATH ${LD_LIBRARY_PATH};
          '';
        });


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
            guiCrateClippy;

          fmt = craneLib.cargoFmt commonArgs;
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
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs;[
            cargo-flamegraph
            cargo-outdated
            gdb
          ];

          inherit LD_LIBRARY_PATH;
        };
      }
    );
}
