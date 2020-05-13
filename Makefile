.PHONY: init update-version clean build test cover open-coverage-html scan release

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

cover-flags := CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off"
cover:
	$(cover-flags) cargo +nightly build
	$(cover-flags) cargo +nightly test
	grcov ./target/debug/ -s . -t lcov --llvm --ignore-not-existing -o ./target/debug/coverage
	genhtml -o ./target/debug/coverage-html --show-details --highlight ./target/debug/coverage

open-coverage-html:
	open ./target/debug/coverage-html/index.html

scan:
	cargo clippy --all-targets --all-features -- -D warnings

release:
	cargo build --release
