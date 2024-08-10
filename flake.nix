{
  description = "Immutable directory manipulation library and CLI tool";

  inputs = {
    devenv.url = "github:cachix/devenv/v1.0.7";
    nixpkgs.follows = "devenv/nixpkgs";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs@{
      self,
      devenv,
      nixpkgs,
      systems,
    }:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);

      mkPkgs =
        system:
        import nixpkgs {
          overlays = [ self.overlays.${system}.default ];
          system = system;
        };
    in
    {
      devShells = forEachSystem (
        system:
        let
          pkgs = mkPkgs system;
        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs; # NB. `pkgs` has overlays!
            modules = [ ./devenv.nix ];
          };
        }
      );

      overlays = forEachSystem (system: {
        default = _final: _prev: { dirtabase = self.packages.${system}.default; };
      });

      packages = forEachSystem (
        system:
        let
          pkgs = mkPkgs system;
        in
        rec {
          default = dirtabase;
          dirtabase = pkgs.callPackage ./. { };
        }
      );
    };
}
