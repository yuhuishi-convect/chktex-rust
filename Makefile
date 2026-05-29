# chktex-rust — common development commands
#
# Run `make` or `make help` to list targets.

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

CARGO ?= cargo
ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TOOLS := $(ROOT)/tools
TARGET := $(ROOT)/target
CHKTEX := $(TARGET)/debug/chktex
CHKTEX_RELEASE := $(TARGET)/release/chktex
CHKTEX_WINDOWS_MSVC := $(TARGET)/x86_64-pc-windows-msvc/release/chktex.exe
CHKTEX_WINDOWS_GNU := $(TARGET)/x86_64-pc-windows-gnu/release/chktex.exe
ORACLE_ENV := $(TARGET)/oracle.env
WINDOWS_FLAVOR ?= msvc

# Upstream oracle paths (overridable; see tools/setup-oracle.sh)
CHKTEX_UPSTREAM_PARENT ?= /tmp/chktex-upstream
CHKTEX_UPSTREAM_DIR ?= $(CHKTEX_UPSTREAM_PARENT)/chktex/chktex
CHKTEX_ORACLE ?= $(CHKTEX_UPSTREAM_DIR)/chktex
TEST_TEX ?= $(CHKTEX_UPSTREAM_DIR)/Test.tex

.DEFAULT_GOAL := help

.PHONY: help build release release-windows package-windows run check test test-core test-cli \
        oracle-setup oracle-setup-tests install-oracle-env \
        oracle oracle-tests diff diff-warnings \
        fmt fmt-check clippy clean

help: ## Show available targets
	@printf "chktex-rust\n\n"
	@printf "Usage: make <target>\n\n"
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / { \
		printf "  %-18s %s\n", $$1, $$2 \
	}' $(MAKEFILE_LIST) | sort
	@printf "\nExamples:\n"
	@printf "  make test\n"
	@printf "  make oracle-setup oracle-tests\n"
	@printf "  make release-windows\n"
	@printf "  make release-windows WINDOWS_FLAVOR=gnu\n"
	@printf "  make diff-warnings TEST_TEX=\$$CHKTEX_UPSTREAM_DIR/Test.tex\n"

build: ## Build debug chktex binary
	$(CARGO) build -p chktex-cli

release: ## Build optimized release chktex binary
	$(CARGO) build --release -p chktex-cli

release-windows: ## Cross-compile chktex.exe for Windows (WINDOWS_FLAVOR=msvc|gnu)
	$(TOOLS)/cross-windows.sh $(WINDOWS_FLAVOR)

package-windows: release-windows ## Build Windows binary and stage chktex.exe + chktexrc
	@out="$(TARGET)/windows-$(WINDOWS_FLAVOR)"; \
	bin="$(if $(filter gnu,$(WINDOWS_FLAVOR)),$(CHKTEX_WINDOWS_GNU),$(CHKTEX_WINDOWS_MSVC))"; \
	rm -rf "$$out"; \
	mkdir -p "$$out"; \
	cp "$$bin" "$$out/chktex.exe"; \
	cp tests/fixtures/upstream/chktexrc "$$out/chktexrc"; \
	printf "Packaged %s\n" "$$out"

run: build ## Run chktex on a file (FILE=path/to/doc.tex)
	@test -n "$(FILE)" || { echo "error: set FILE=path/to/doc.tex"; exit 1; }
	$(CHKTEX) $(ARGS) $(FILE)

check: ## cargo check all workspace crates
	$(CARGO) check --workspace

test: ## Run unit and integration tests (excludes oracle suite)
	$(CARGO) test --workspace

test-core: ## Run chktex-core unit tests only
	$(CARGO) test -p chktex-core

test-cli: ## Run chktex-cli tests (non-oracle)
	$(CARGO) test -p chktex-cli --test cli

oracle-setup: ## Clone/build upstream C chktex and write target/oracle.env
	$(TOOLS)/setup-oracle.sh

oracle-setup-tests: ## Setup oracle, then run the differential suite
	$(TOOLS)/setup-oracle.sh --run-tests

install-oracle-env: oracle-setup ## Setup oracle and print source command for target/oracle.env
	@test -f "$(ORACLE_ENV)"
	@printf "source %s\n" "$(ORACLE_ENV)"

oracle: ## Run differential tests against upstream C binary
	$(TOOLS)/run-oracle-tests.sh

oracle-tests: oracle ## Alias for `make oracle`

diff: ## Compare warning output vs upstream (TEST_TEX=...)
	@test -f "$(CHKTEX_ORACLE)" || { \
		echo "error: oracle not found at $(CHKTEX_ORACLE); run 'make oracle-setup'"; \
		exit 1; \
	}
	@test -f "$(TEST_TEX)" || { \
		echo "error: fixture not found at $(TEST_TEX)"; \
		exit 1; \
	}
	$(TOOLS)/diff-warnings.sh "$(TEST_TEX)"

diff-warnings: diff ## Alias for `make diff`

fmt: ## Format Rust sources
	$(CARGO) fmt --all

fmt-check: ## Check Rust formatting without writing
	$(CARGO) fmt --all -- --check

clippy: ## Run clippy on the workspace
	$(CARGO) clippy --workspace --all-targets -- -D warnings

clean: ## Remove build artifacts
	$(CARGO) clean
