build: test
	cargo build

test:
	cargo test

start: build
	sudo RUST_LOG=info BACKUP_LOCAL_RS_CONFIG=/home/hannes/Repository/backupArchival/backup-local-rs/config.json /opt/hannes/cargo_target/debug/backup-local-rs

docs:
  dot -Tsvg -odocs/bld/architecture.svg docs/src/architecture.dot