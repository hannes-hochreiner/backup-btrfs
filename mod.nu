export def build [] {
  test
	cargo build
}

export def test [] {
	cargo test
}

export def nix-build [] {
  ^nix build
}

export def update [] {
	let deps_info = (get-deps-info)

  ^cargo update
  {
    "nix_hash": ($deps_info.hash),
		"vendor_output": ($deps_info.cargo_config)
  } | to json | save -f hashes.json
  ^nix flake update
}

def get-deps-info [] {
  let temp_path = $"/tmp/backup_btrfs_deps_(random uuid)"

  mkdir $temp_path
	let deps_info = {
		cargo_config: (cargo vendor $temp_path)
		hash: (nix hash path --format nix32 $temp_path)
	}

  rm -r $temp_path

  $deps_info
}

export def start [] {
  build
	sudo RUST_LOG=info BACKUP_LOCAL_RS_CONFIG=/home/hannes/Repository/backup-btrfs/config.json /opt/hannes/cargo_target/debug/backup-btrfs
}

export def docs [] {
  run-external dot "-Tsvg" "-odocs/bld/architecture.svg" docs/src/architecture.dot
}