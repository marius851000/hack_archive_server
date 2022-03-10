{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, crate2nix }:
    utils.lib.eachSystem (utils.lib.defaultSystems ++ ["armv7l-linux"]) (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      crate2nix-tools = import "${crate2nix}/tools.nix" {inherit pkgs;};
      generatedCargoNix = (crate2nix-tools.generatedCargoNix {
        name = "pmd_hack_archive_server";
        src = ./.;
      });
      importedCargoNix = import "${generatedCargoNix}/default.nix" { inherit pkgs; };
    in rec {
      # `nix build`
      packages.pmd_hack_archive_server = importedCargoNix.workspaceMembers.server.build;
      defaultPackage = packages.pmd_hack_archive_server;
      
      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };
    });
}
