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

        packages = {
          default = pkgs.callPackage ./nix {inherit version name;};
        };
      };

      flake = {
        nixosModules.default = {pkgs, ...}: import ./nix/module.nix {iio-niri = withSystem pkgs.stdenv.hostPlatform.system ({config, ...}: config.packages.default);};

        overlays.default = {pkgs, ...}: let
          packages = withSystem pkgs.stdenv.hostPlatform.system ({config, ...}: config.packages);
        in
          final: prev: {iio-niri = packages.default;};
      };
    });
}
