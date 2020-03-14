.PHONY: update-version clean build release

update-version:
	sed 's/version = "0.0.0"/version = "$(VERSION)"/g' Cargo.toml > test.toml
	mv test.toml Cargo.toml

clean:
	cargo clean

build:
	cargo build

release:
	cargo build --release
