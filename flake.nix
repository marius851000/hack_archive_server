{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachSystem (utils.lib.defaultSystems ++ ["armv7l-linux"]) (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      naersk-lib = naersk.lib."${system}";
    in rec {
      # `nix build`
      packages.pmd_hack_archive_server = naersk-lib.buildPackage {
        pname = "pmd_hack_archive_server";
        root = ./.;
      };
      defaultPackage = packages.pmd_hack_archive_server;

      # `nix run`
      apps.pmd_hack_archive_server = utils.lib.mkApp {
        drv = packages.pmd_hack_archive_server;
      };
      defaultApp = apps.pmd_hack_archive_server;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };
    });
}
