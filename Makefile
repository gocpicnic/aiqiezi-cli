CARGO ?= cargo
PACKAGE := aiqiezi-cli
BUILD_DATE ?= $(shell date -u +%Y%m%d%H%M%S)
TARGET ?=
TARGET_ARG := $(if $(TARGET),--target $(TARGET),)
DIST_DIR ?= dist
OS ?= $(shell uname -s)
IS_WINDOWS_TARGET := $(findstring windows,$(TARGET))
EXE_EXT ?= $(if $(IS_WINDOWS_TARGET),.exe,$(if $(filter Windows_NT,$(OS)),.exe,))
WINDOWS_TARGET ?= x86_64-pc-windows-msvc
WINDOWS_RUSTFLAGS ?= -C target-feature=+crt-static
ARGS ?= --help

ifneq ($(TARGET),)
SOURCE_BIN := target/$(TARGET)/release/aiqiezi$(EXE_EXT)
DIST_BIN := $(DIST_DIR)/$(TARGET)/aiqiezi$(EXE_EXT)
else
SOURCE_BIN := target/release/aiqiezi$(EXE_EXT)
DIST_BIN := $(DIST_DIR)/aiqiezi$(EXE_EXT)
endif

ifneq ($(IS_WINDOWS_TARGET),)
BUILD_COMMAND = BUILD_DATE="$(BUILD_DATE)" RUSTFLAGS="$(WINDOWS_RUSTFLAGS)" $(CARGO) xwin build --release --locked $(TARGET_ARG) -p $(PACKAGE)
else
BUILD_COMMAND = BUILD_DATE="$(BUILD_DATE)" $(CARGO) build --release --locked $(TARGET_ARG) -p $(PACKAGE)
endif

.DEFAULT_GOAL := help

.PHONY: help build build-windows check-xwin build-debug run version fmt fmt-check check test clippy ci clean

help:
	@printf 'Usage:\n'
	@printf '  make build        Build the release CLI binary and copy it to %s\n' "$(DIST_BIN)"
	@printf '  make build-windows  Cross-build the Windows MSVC executable with cargo-xwin\n'
	@printf '  make check-xwin   Verify that cargo-xwin is installed\n'
	@printf '  make build-debug  Build the debug CLI binary\n'
	@printf '  make run          Run the CLI; override ARGS, for example ARGS="start"\n'
	@printf '  make version      Print the CLI version and build date\n'
	@printf '  make fmt          Format all Rust code\n'
	@printf '  make fmt-check    Check Rust formatting\n'
	@printf '  make check        Run cargo check for the workspace\n'
	@printf '  make test         Run all workspace tests\n'
	@printf '  make clippy       Run clippy with warnings denied\n'
	@printf '  make ci           Run all checks used by CI\n'
	@printf '  make clean        Remove build artifacts including %s\n' "$(DIST_DIR)"
	@printf '\n'
	@printf 'Optional: make build TARGET=x86_64-unknown-linux-musl\n'

build:
	$(BUILD_COMMAND)
	mkdir -p "$(dir $(DIST_BIN))"
	cp "$(SOURCE_BIN)" "$(DIST_BIN)"

build-windows: check-xwin
	$(MAKE) build TARGET=$(WINDOWS_TARGET)

check-xwin:
	@$(CARGO) xwin --version >/dev/null 2>&1 || { \
		echo "error: cargo-xwin is not installed; run 'cargo install cargo-xwin --locked'" >&2; \
		exit 1; \
	}

build-debug:
	BUILD_DATE="$(BUILD_DATE)" $(CARGO) build --locked $(TARGET_ARG) -p $(PACKAGE)

run:
	BUILD_DATE="$(BUILD_DATE)" $(CARGO) run --locked $(TARGET_ARG) -p $(PACKAGE) -- $(ARGS)

version:
	BUILD_DATE="$(BUILD_DATE)" $(CARGO) run --quiet --locked $(TARGET_ARG) -p $(PACKAGE) -- --version

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

check:
	$(CARGO) check --workspace --all-targets --locked

test:
	$(CARGO) test --workspace --locked

clippy:
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

ci: fmt-check check test clippy

clean:
	$(CARGO) clean
	rm -rf "$(DIST_DIR)"
