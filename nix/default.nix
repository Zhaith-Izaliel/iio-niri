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

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    dbus
  ];

  meta = {
    description = "Listen to iio-sensor-proxy and updates Niri output orientation depending on the accelerometer orientation";
    homepage = "https://github.com/Zhaith-Izaliel/iio-niri";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [zhaithizaliel];
    mainProgram = "iio-niri";
    platforms = lib.platforms.linux;
  };
}
