export def build [] {
  test
	cargo build
}

export def test [] {
	cargo test
}

export def start-nix [] {
	run-external "podman" "run" "--rm" "-it" "-v" $"($env.PWD):/workspace:z" "nixos/nix" "bash" "-c" "nix build --extra-experimental-features nix-command --extra-experimental-features flakes --recreate-lock-file /workspace"
}

export def start [] {
  build
	sudo RUST_LOG=info BACKUP_LOCAL_RS_CONFIG=/home/hannes/Repository/backup-btrfs/config.json /opt/hannes/cargo_target/debug/backup-btrfs
}

export def docs [] {
  run-external dot "-Tsvg" "-odocs/bld/architecture.svg" docs/src/architecture.dot
}