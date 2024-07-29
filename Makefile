#!/usr/bin/env -S make -f

MAKEFLAGS += --warn-undefined-variable
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --silent

-include Makefile.*

SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
.DEFAULT_GOAL := help

help: Makefile  ## Show help
	for makefile in $(MAKEFILE_LIST)
	do
		@echo "$${makefile}"
		@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' "$${makefile}" | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'
	done


# =============================================================================
# Common
# =============================================================================
install:  ## Install the app locally
	cd src-tauri && cargo fetch && cd -
	yarn install
	pre-commit install --install-hooks
.PHONY: install

update:  ## Update deps and tools
	cd src-tauri && cargo update && cd -
	yarn upgrade
	pre-commit autoupdate
.PHONY: update

run:  ## Run development application
	cargo tauri dev
.PHONY: run


# =============================================================================
# CI
# =============================================================================
ci: lint scan test benchmark  ## Run CI tasks
.PHONY: ci

format:  ## Run autoformatters
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --allow-no-vcs
.PHONY: format

lint:  ## Run linters
	cargo fmt --check
	cargo clippy
.PHONY: lint

scan:  ## Run scans

.PHONY: scan

test:  ## Run tests
	cargo llvm-cov nextest --workspace --lcov --output-path lcov.info \
		&& cargo llvm-cov report --summary-only
.PHONY: test

benchmark:  ## Run benchmarks
	cargo bench --workspace
.PHONY: benchmark
