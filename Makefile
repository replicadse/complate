.PHONY: clean build release

clean:
	cargo clean

build:
	cargo build

release:
	cargo build --release
