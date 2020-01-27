ifndef VERSION
	VERSION := latest
endif

.PHONY: build

build:
	cargo build --release
