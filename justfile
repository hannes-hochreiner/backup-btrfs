build: test
	cargo build

test:
	cargo test

start: build
	sudo RUST_LOG=info BACKUP_LOCAL_RS_CONFIG=/home/hannes/Repository/backup-btrfs/config.json /opt/hannes/cargo_target/debug/backup-btrfs

docs:
  dot -Tsvg -odocs/bld/architecture.svg docs/src/architecture.dot