.PHONY: init update-version clean build test scan release

init:
	rm -rf .git/hooks
	ln -s ../scripts/git-hooks .git/hooks
	chmod -R +x ./scripts/*

update-version:
	sed 's/version = "0.0.0"/version = "$(VERSION)"/g' Cargo.toml > Cargo.toml.tmp
	sed 's/.version("0.0.0")/.version("$(VERSION)")/g' src/args/mod.rs > src/args/mod.rs.tmp
	mv Cargo.toml.tmp Cargo.toml
	mv src/args/mod.rs.tmp src/args/mod.rs

clean:
	cargo clean

build:
	cargo build

test:
	cargo test

scan:
	cargo clippy --all-targets --all-features -- -D warnings

release:
	cargo build --release
