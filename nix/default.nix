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

  meta = with lib; {
    description = "Listen to iio-sensor-proxy and updates Niri output orientation depending on the accelerometer orientation.";
    homepage = "https://github.com/Zhaith/iio-niri";
    license = licenses.mit;
    maintainers = with maintainers; [];
    mainProgram = name;
    platforms = platforms.all;
  };
}
