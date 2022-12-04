build: test
	cargo build

test:
	cargo test

start: build
	sudo RUST_LOG=info BACKUP_LOCAL_RS_CONFIG=/home/hannes/Repository/backupArchival/backup-local-rs/config.json target/debug/backup-local-rs
