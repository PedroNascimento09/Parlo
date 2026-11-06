{
  description = "Parlo, offline dictation, translation and meetings";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" ];
      each = nixpkgs.lib.genAttrs systems;
    in {
      packages = each (system: {
        default = nixpkgs.legacyPackages.${system}.callPackage ./default.nix { };
      });
    };
}
