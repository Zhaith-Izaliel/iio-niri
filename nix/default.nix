{
  rustPlatform,
  lib,
  name,
  version,
}:
rustPlatform.buildRustPackage {
  inherit version;
  pname = name;
  cargoLock.lockFile = ../Cargo.lock;
  src = lib.cleanSource ../.;
}
