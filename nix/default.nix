{
  rustPlatform,
  lib,
  name,
  version,
  dbus,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  inherit version;
  pname = name;

  PKG_CONFIG_PATH = "${dbus.dev}/lib/pkgconfig";

  nativeBuildInputs = [
    dbus
    pkg-config
  ];

  cargoLock.lockFile = ../Cargo.lock;
  src = lib.cleanSource ../.;
}
