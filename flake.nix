{
  inputs = {
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachSystem (utils.lib.defaultSystems ++ ["armv7l-linux"]) (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
    in rec {
      # `nix build`
      packages.pmd_hack_archive_server = pkgs.rustPlatform.buildRustPackage rec {
        pname = "hacknews-server";
        version = "0.1.0";

        cargoSha256 = "sha256-7zNHqFgrUKYyNEROHAZPsnxgNF2tJJmcF+Ov9/4m4gM=";

        src = ./.;
      };
      defaultPackage = packages.pmd_hack_archive_server;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };
    });
}
