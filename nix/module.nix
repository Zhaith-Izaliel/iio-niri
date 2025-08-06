{iio-niri}: {
  config,
  lib,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types mkIf getExe concatStringsSep optional;
  cfg = config.programs.iio-niri;

  generatedArgs = concatStringsSep " " ((optional (cfg.monitor != null) "--monitor ${cfg.monitor}") ++ cfg.extraArgs);
in {
  options.programs.iio-niri = {
    enable = mkEnableOption "IIO-Niri";

    package = mkOption {
      type = types.package;
      default = iio-niri;
      description = "The iio-niri package to use.";
    };

    monitor = mkOption {
      type = types.nullOr types.nonEmptyStr;
      default = null;
      description = "The monitor to rotate. Can be null.";
    };

    extraArgs = mkOption {
      type = types.listOf types.nonEmptyStr;
      default = [];
      description = "Extra arguments to pass to IIO-Niri";
    };

    target = mkOption {
      type = types.nonEmptyStr;
      default = "niri.service";
      description = "The Niri target to start IIO-Niri after.";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.iio-niri = {
      description = "IIO-Niri";
      wantedBy = [cfg.target];
      after = [cfg.target];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${getExe cfg.package} ${generatedArgs}";
        Restart = "on-failure";
      };
    };

    hardware.sensor.iio.enable = true;
  };
}
