.PHONY: all build release clean install test run

# Default target
all: build

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# Install locally
install: release
	cargo install --path cortex-cli --force

# Run tests
test:
	cargo test

# Run the application
run:
	cargo run --bin cortex

# Run with debug logging
debug:
	RUST_LOG=debug cargo run --bin cortex

# Format code
fmt:
	cargo fmt --all

# Check code
check:
	cargo check --all-targets
	cargo clippy --all-targets

# Build for all platforms
dist: dist-linux dist-macos dist-windows

dist-linux:
	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target aarch64-unknown-linux-gnu

dist-macos:
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin

dist-windows:
	cargo build --release --target x86_64-pc-windows-msvc

# Package release
package: release
	mkdir -p dist
	cp target/release/cortex dist/
	cp README.md dist/
	tar -czf cortex-$(shell cargo pkgid | cut -d# -f2).tar.gz -C dist .