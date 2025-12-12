{
  description = "A backup tool based on btrfs snapshots";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
      };

      backup-btrfs-cargo-toml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
      hashes-json = (builtins.fromJSON (builtins.readFile ./hashes.json));
      inputs = with pkgs; [ rustc cargo gcc_multi ];
      path = builtins.concatStringsSep ":" (builtins.map (input: "${input}/bin") inputs);

      backup-btrfs-deps = derivation {
        inherit system;
        name = "${backup-btrfs-cargo-toml.package.name}-${hashes-json.nix_hash}-deps";
        builder = "${pkgs.nushell}/bin/nu";
        PATH = path;
        args = [ ./builder.nu "vendor" ./. ];

        outputHashAlgo = "sha256";
        outputHashMode = "recursive";
        outputHash = hashes-json.nix_hash;
        # outputHash = pkgs.lib.fakeHash;
      };

      backup-btrfs-bin = derivation {
          inherit system;
          name = "${backup-btrfs-cargo-toml.package.name}-v${backup-btrfs-cargo-toml.package.version}";
          builder = "${pkgs.nushell}/bin/nu";
          PATH = path;
          args = [ ./builder.nu "build" ./. backup-btrfs-deps "backup-btrfs" hashes-json.vendor_output ];
      };
    in {
      packages.${system} = {
        deps = backup-btrfs-deps;
        bin = backup-btrfs-bin;
        default = backup-btrfs-bin;
      };

      nixosModules.${system}.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.hochreiner.services.backup-btrfs;
    
          policyOptions = {
            options = {
              value = mkOption {
                type = types.int;
              };
              unit = mkOption {
                type = types.enum [ "minutes" "hours" "days" "weeks" ];
              };
            };
          };

          configuration = pkgs.writeTextFile {
            name = "backup-btrfs-config";
            text = ''
              {
                "source_subvolume_path": "${cfg.source_subvolume_path}",
                "snapshot_device": "${cfg.snapshot_device}",
                "snapshot_subvolume_path": "${cfg.snapshot_subvolume_path}",
                "snapshot_path": "${cfg.snapshot_path}",
                "snapshot_suffix": "${cfg.snapshot_suffix}",
                "user_local": "${cfg.user_local}",
                "policy_local": ['' + (lib.strings.concatStringsSep ", " (lib.map (elem: "{ \"${elem.unit}\": ${builtins.toString elem.value} }") cfg.policy_local)) + ''],
                "config_ssh": {
                  "host": "${cfg.ssh_host}",
                  "config": "${cfg.ssh_config}"
                },
                "backup_device": "${cfg.backup_device}",
                "backup_subvolume_path": "${cfg.backup_subvolume_path}",
                "backup_path": "${cfg.backup_path}",
                "policy_remote": ['' + (lib.strings.concatStringsSep ", " (lib.map (elem: "{ \"${elem.unit}\": ${builtins.toString elem.value} }") cfg.policy_remote)) + '']
              }
            '';
          };
        in {
          # https://britter.dev/blog/2025/01/09/nixos-modules/
          options.hochreiner.services.backup-btrfs = {
            enable = mkEnableOption "Enables the backup-btrfs service";
            config_file = mkOption {
              type = types.path;
              description = lib.mdDoc "Path of the configuration file";
            };

            log_level = mkOption {
              type = types.enum [ "error" "warn" "info" "debug" "trace" ];
              default = "info";
              description = lib.mdDoc "Log level";
            };
            source_subvolume_path = mkOption {
              type = types.path;
              description = lib.mdDoc "path of the subvolume to back up";
              example = "/home";
            };
            snapshot_device = mkOption {
              type = types.path;
              description = lib.mdDoc "path of the device the subvolume resides on";
              example = "/dev/mapper/new";
            };
            snapshot_subvolume_path = mkOption {
              type = types.path;
              description = lib.mdDoc "path of the subvolume for snapshots";
              example = "/snapshots";
            };
            snapshot_path = mkOption {
              type = types.path;
              description = lib.mdDoc "path of the snapshots";
              example = "/snapshots";
            };
            snapshot_suffix = mkOption {
              type = types.str;
              description = lib.mdDoc "snapshot suffix";
              example = "laptop";
            };
            user_local = mkOption {
              type = types.str;
              description = lib.mdDoc "local user running the backup";
              example = "root";
            };
            policy_local = mkOption {
              description = lib.mdDoc "policy for retaining local snapshots";
              type = types.listOf (types.submodule policyOptions);
            };
            ssh_host = mkOption {
              type = types.str;
              description = lib.mdDoc "name of the remote host";
            };
            ssh_config = mkOption {
              type = types.path;
              description = lib.mdDoc "path of the ssh configuration file";
            };
            backup_device = mkOption {
              type = types.path;
              description = lib.mdDoc "device path on the remote host";
              example = "/dev/mapper/volume";
            };
            backup_subvolume_path = mkOption {
              type = types.path;
              description = lib.mdDoc "subvolume path on the remote host";
              example = "/volume/backups";
            };
            backup_path = mkOption {
              type = types.path;
              description = lib.mdDoc "path of the snapshots on the remote host";
              example = "/volume/backups/snapshots";
            };
            policy_remote = mkOption {
              description = lib.mdDoc "policy for retaining remote snapshots";
              type = types.listOf (types.submodule policyOptions);
            };
          };

          config = mkIf cfg.enable {
            systemd.services."hochreiner.backup-btrfs" = {
              description = "backup-btrfs service";
              serviceConfig = let pkg = self.packages.${system}.default;
              in {
                Type = "oneshot";
                ExecStart = "${pkg}/bin/backup-btrfs";
                Environment = "RUST_LOG='${cfg.log_level}' BACKUP_BTRFS_CONFIG='${configuration}' PATH=/run/current-system/sw/bin";
              };
            };
            systemd.timers."hochreiner.backup-btrfs" = {
              description = "timer for the backup-btrfs service";
              wantedBy = [ "multi-user.target" ];
              timerConfig = {
                OnBootSec="15min";
                OnUnitInactiveSec="15min";
                Unit="hochreiner.backup-btrfs.service";
              };
            };
          };
        };

      devShells.${system}.default = pkgs.mkShell {
        name = "backup-btrfs";

        # Inherit inputs from checks.
        # checks = self.checks.${system}
        # shellHook = ''
        #   exec zellij -l zellij.kdl
        # '';
        shellHook = ''
          exec nu
        '';
        # Additional dev-shell environment variables can be set directly
        # MY_CUSTOM_DEVELOPMENT_VAR = "something else"
        # Extra inputs can be added here; cargo and rustc are provided by default.
        buildInputs = with pkgs; [
          # zellij
          rustc
          cargo
        ];
      };
    };
  
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