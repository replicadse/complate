.PHONY: update-version clean build release

update-version:
	sed 's/version = "0.0.0"/version = "$(VERSION)"/g' Cargo.toml > Cargo.toml.tmp
	sed 's/.version("0.0.0")/.version("$(VERSION)")/g' src/args/args.rs > src/args/args.rs.tmp
	mv Cargo.toml.tmp Cargo.toml
	mv src/args/args.rs.tmp src/args/args.rs

clean:
	cargo clean

build:
	cargo build

release:
	cargo build --release
