{iio-niri}: {
  config,
  lib,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types mkIf getExe concatStringsSep;
  cfg = config.programs.iio-niri;
in {
  options.programs.iio-niri = {
    enable = mkEnableOption "IIO-Niri";

    package = mkOption {
      type = types.package;
      default = iio-niri;
      description = "The iio-niri package to use.";
    };

    service = {
      enable = mkEnableOption "the systemd user service to run IIO-Niri";

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
  };

  config = mkIf cfg.enable {
    hardware.sensor.iio.enable = true;

    environment.systemPackages = [cfg.package];

    systemd.user.services.iio-niri = mkIf cfg.service.enable {
      description = "IIO-Niri";
      wantedBy = [cfg.service.niriUnit];
      bindsTo = [cfg.service.niriUnit];
      partOf = [cfg.service.niriUnit];
      after = [cfg.service.niriUnit];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${getExe cfg.package} ${concatStringsSep " " cfg.service.extraArgs}";
        Restart = "on-failure";
      };
    };
  };
}
