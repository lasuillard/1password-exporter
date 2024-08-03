#!/usr/bin/env -S make -f

MAKEFLAGS += --warn-undefined-variable
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --silent

SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
.DEFAULT_GOAL := help

help: Makefile  ## Show help
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' "$(MAKEFILE_LIST)" | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'


# =============================================================================
# Common
# =============================================================================
install:  ## Install the app locally
	cargo fetch
	pre-commit install --install-hooks
.PHONY: install

update:  ## Update deps and tools
	cargo update
	pre-commit autoupdate
.PHONY: update

run:  ## Run development application
	cargo watch -x 'run -- --log-level debug'
.PHONY: run


# =============================================================================
# CI
# =============================================================================
ci: lint test  ## Run CI tasks
.PHONY: ci

format:  ## Run autoformatters
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --allow-no-vcs
.PHONY: format

lint:  ## Run linters
	cargo fmt --check
	cargo clippy
.PHONY: lint

test:  ## Run tests
	cargo llvm-cov nextest --workspace --lcov --output-path lcov.info \
		&& cargo llvm-cov report --summary-only
.PHONY: test
