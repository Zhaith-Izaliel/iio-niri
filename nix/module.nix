{iio-niri}: {
  config,
  lib,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types mkIf getExe concatStringsSep;
  cfg = config.service.iio-niri;
in {
  options.service.iio-niri = {
    enable = mkEnableOption "IIO-Niri";

    package = mkOption {
      type = types.package;
      default = iio-niri;
      description = "The iio-niri package to use.";
    };

    niriUnit = mkOption {
      type = types.nonEmptyStr;
      default = "niri.service";
      description = "The Niri **user** service unit to bind IIO-Niri's **user** service unit to.";
    };

    extraArgs = mkOption {
      type = types.listOf types.str;
      default = [];
      description = "Extra arguments to pass to IIO-Niri.";
    };
  };

  config = mkIf cfg.enable {
    hardware.sensor.iio.enable = true;

    environment.systemPackages = [cfg.package];

    systemd.user.services.iio-niri = {
      description = "IIO-Niri";
      wantedBy = [cfg.niriUnit];
      bindsTo = [cfg.niriUnit];
      partOf = [cfg.niriUnit];
      after = [cfg.niriUnit];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${getExe cfg.package} ${concatStringsSep " " cfg.extraArgs}";
        Restart = "on-failure";
      };
    };
  };
}
