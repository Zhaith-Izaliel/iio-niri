{
  rustPlatform,
  stdenv,
  lib,
  name,
  version,
  dbus,
  pkg-config,
  installShellFiles,
}:
rustPlatform.buildRustPackage {
  inherit version;
  pname = name;

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    installShellFiles
  ];

  buildInputs = [
    dbus
  ];

  postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    installShellCompletion --cmd ${name} \
      --bash <($out/bin/${name} completions bash) \
      --zsh <($out/bin/${name} completions zsh) \
      --fish <($out/bin/${name} completions fish)
  '';

  meta = {
    description = "Listen to iio-sensor-proxy and updates Niri output orientation depending on the accelerometer orientation";
    homepage = "https://github.com/Zhaith-Izaliel/iio-niri";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ zhaithizaliel ];
    mainProgram = "iio-niri";
    platforms = lib.platforms.linux;
  };
}
