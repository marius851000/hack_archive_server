{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "nixpkgs";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};
        src = ./.;

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
          buildInputs = [ pkgs.openssl ];
          nativeBuildInputs = [ pkgs.pkg-config ];
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        pmd_hack_archive_server = craneLib.buildPackage {
          inherit src cargoArtifacts;
          buildInputs = [ pkgs.openssl ];
          nativeBuildInputs = [ pkgs.pkg-config ];
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit pmd_hack_archive_server;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          pmd_hack_archive_server-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src;
            cargoClippyExtraArgs = "-- --deny warnings";
            buildInputs = [ pkgs.openssl ];
            nativeBuildInputs = [ pkgs.pkg-config ];
          };

          # Check formatting
          pmd_hack_archive_server-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Check code coverage (note: this will not upload coverage anywhere)
          /*pmd_hack_archive_server-coverage = craneLib.cargoTarpaulin {
            inherit cargoArtifacts src;
          };*/
        };

        packages.pmd_hack_archive_server = pmd_hack_archive_server;
        packages.default = pmd_hack_archive_server;

        apps.my-app = flake-utils.lib.mkApp {
          drv = pmd_hack_archive_server;
        };
        defaultApp = self.apps.${system}.my-app;

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            openssl
            pkg-config
          ];
        };
      });
}
