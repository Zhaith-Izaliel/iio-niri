{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/25.05";
  };

  outputs = inputs @ {flake-parts, ...}: let
    inherit (cargoToml.package) name version;
    cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  in
    flake-parts.lib.mkFlake {inherit inputs;} ({withSystem, ...}: {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {pkgs, ...}: {
        devShells = {
          # nix develop
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              rust-analyzer
              dbus
              pkg-config
            ];
          };
        };

        packages = rec {
          default = pkgs.callPackage ./nix {inherit version name;};

          doc-gen = pkgs.callPackage ./nix/doc.nix {iio-niri = default;};
        };
      };

      flake = {
        nixosModules.default = {pkgs, ...}: let
          module = import ./nix/module.nix {iio-niri = withSystem pkgs.stdenv.hostPlatform.system ({config, ...}: config.packages.default);};
        in {
          imports = [module];
        };

        overlays.default = final: prev: let
          packages = withSystem prev.stdenv.hostPlatform.system ({config, ...}: config.packages);
        in {iio-niri = packages.default;};
      };
    });
}
