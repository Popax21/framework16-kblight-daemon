{
  lib,
  pkgs,
  config,
  ...
}: {
  options.services.fw16-kblight-daemon = {
    enable = lib.mkEnableOption "the Framework Laptop 16 Keyboard Backlight Daemon";
    package = lib.mkPackageOption pkgs "fw16-kblight-daemon" {};

    settings = lib.mkOption {
      type = lib.types.attrsOf lib.types.toml;
      default = {};
      description = "Configuration options for the keyboard backlight daemon.s";
    };
  };
  config = let
    cfg = config.services.fw16-kblight-daemon;
  in
    lib.mkIf cfg.enable {
      systemd.packages = [cfg.package];
      systemd.services.fw16-kblight-daemon = {
        wantedBy = ["multi-user.target"];
        environment.CONFIG_FILE = (pkgs.formats.toml {}).generate "fw16-kblight-daemon-config.toml" cfg.settings;
      };
      boot.kernelModules = lib.mkIf (cfg.settings.expose_via_uleds or true) ["uleds"];
    };
}
