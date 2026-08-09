.PHONY: help build pack install-bin test lint check clean

RELEASE_BINARY := target/release/fig
STAGED_BINARY := bin/fig

help:
	@echo "Available targets:"
	@echo "  make build      - Build and stage the uncompressed release binary"
	@echo "  make pack       - UPX-compress the staged release binary"
	@echo "  make install-bin - Build and stage as ./bin/fig"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make test       - Run tests"
	@echo "  make lint       - Run clippy and fmt check"
	@echo "  make check      - lint + test"

build:
	cargo build --release
	mkdir -p bin
	cp $(RELEASE_BINARY) $(STAGED_BINARY)

pack: build
	upx --best --lzma --force-macos $(STAGED_BINARY)

install-bin: build

clean:
	cargo clean

test:
	cargo test --offline

lint:
	cargo fmt --check
	cargo clippy --offline --all-targets -- -D warnings

check: lint test

.DEFAULT_GOAL := help
