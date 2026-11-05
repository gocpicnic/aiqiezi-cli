CARGO ?= cargo
PACKAGE := aiqiezi-cli
BUILD_DATE ?= $(shell date -u +%Y%m%d%H%M%S)
TARGET ?=
TARGET_ARG := $(if $(TARGET),--target $(TARGET),)
DIST_DIR ?= dist
OS ?= $(shell uname -s)
EXE_EXT ?= $(if $(filter Windows_NT,$(OS)),.exe,)
ARGS ?= --help

ifneq ($(TARGET),)
SOURCE_BIN := target/$(TARGET)/release/aiqiezi$(EXE_EXT)
DIST_BIN := $(DIST_DIR)/$(TARGET)/aiqiezi$(EXE_EXT)
else
SOURCE_BIN := target/release/aiqiezi$(EXE_EXT)
DIST_BIN := $(DIST_DIR)/aiqiezi$(EXE_EXT)
endif

.DEFAULT_GOAL := help

.PHONY: help build build-debug run version fmt fmt-check check test clippy ci clean

help:
	@printf 'Usage:\n'
	@printf '  make build        Build the release CLI binary and copy it to %s\n' "$(DIST_BIN)"
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
	BUILD_DATE="$(BUILD_DATE)" $(CARGO) build --release --locked $(TARGET_ARG) -p $(PACKAGE)
	mkdir -p "$(dir $(DIST_BIN))"
	cp "$(SOURCE_BIN)" "$(DIST_BIN)"

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
