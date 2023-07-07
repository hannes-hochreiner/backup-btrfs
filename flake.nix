{
  description = "A backup tool based on btrfs snapshots";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};
        backup-btrfs = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          buildInputs = [
            # Add additional build inputs here
          ];
        };
      in
      {
        checks = {
          inherit backup-btrfs;
        };

        packages.default = backup-btrfs;

        apps.default = flake-utils.lib.mkApp {
          drv = backup-btrfs;
        };

        nixosModules.default = { config, lib, pkgs, ... }:
          with lib;
          let cfg = config.hochreiner.services.backup-btrfs;
          in {
            options.hochreiner.services.backup-btrfs = {
              enable = mkEnableOption "Enables the backup-btrfs service";
              config_file = mkOption {
                type = types.path;
                default = "";
                description = lib.mdDoc "Path of the configuration file";
              };
              log_level = mkOption {
                type = types.enum [ "error" "warn" "info" "debug" "trace" ];
                default = "info";
                description = lib.mdDoc "Log level";
              };
            };

            config = mkIf cfg.enable {
              systemd.services."hochreiner.backup-btrfs" = {
                description = "backup-btrfs service";
                wantedBy = [ "multi-user.target" ];

                serviceConfig = let pkg = self.packages.${system}.default;
                in {
                  Type = "oneshot";
                  ExecStart = "${pkg}/bin/backup-btrfs";
                  Environment = "RUST_LOG=${log_level} BACKUP_LOCAL_RS_CONFIG=${config_file}";
                };
              };
              systemd.timers."hochreiner.backup-btrfs" = {
                description = "timer for the backup-btrfs service";
                wantedBy = [ "multi-user.target" ];
                timerConfig = {
                  OnBootSec="5min";
                  OnUnitInactiveSec="5min";
                  Unit="hochreiner.backup-btrfs.service";
                };
              };
            };
          };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
          ];
        };
      }
    );
  
  nixConfig = {
    substituters = [
      "https://cache.nixos.org"
      "https://hannes-hochreiner.cachix.org"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "hannes-hochreiner.cachix.org-1:+ljzSuDIM6I+FbA0mdBTSGHcKOcEZSECEtYIEcDA4Hg="
    ];
  };
}