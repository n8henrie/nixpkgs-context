{
  description = "Flake for nixpkgs-context";
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      pname = (fromTOML (builtins.readFile ./Cargo.toml)).package.name;
      systems = [
        "x86_64-darwin"
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem =
        with nixpkgs.lib;
        f: foldAttrs mergeAttrs { } (map (s: mapAttrs (_: v: { ${s} = v; }) (f s)) systems);
    in
    eachSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          default = self.packages.${system}.${pname};
          ${pname} = pkgs.callPackage ./package.nix { inherit pname; };
        };

        apps.default = {
          type = "app";
          program = pkgs.lib.getExe' self.packages.${system}.${pname} pname;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.outputs.packages.${system}.default ];
          buildInputs = with pkgs; [
            bacon
            cargo
            clippy
            rust-analyzer
            rustfmt
            watchexec
          ];
        };
      }
    );
}
