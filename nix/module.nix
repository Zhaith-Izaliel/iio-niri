{iio-niri}: {
  config,
  lib,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types mkIf;
  cfg = config.programs.iio-niri;
in {
  options.programs.iio-niri = {
    enable = mkEnableOption "IIO-Niri";

    package = mkOption {
      type = types.package;
      default = iio-niri;
      description = "The iio-niri package to use.";
    };
  };

  config = mkIf cfg.enable {
    hardware.sensor.iio.enable = true;

    environment.systemPackages = [cfg.package];
  };
}
