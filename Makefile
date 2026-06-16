# Makefile -- bevy-planet
# The whole dev workflow goes through here. Run `make help` for the list.
# The Rust crate lives in app/; docs and tooling stay at the repo root.

APP := app

.DEFAULT_GOAL := help
.PHONY: help run build release fmt fmt-check lint test check watch clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

run: ## Run the game in dev (dynamic linking = fast recompiles)
	cd $(APP) && cargo run --features bevy/dynamic_linking

build: ## Build in debug
	cd $(APP) && cargo build

release: ## Build in release (optimised, WITHOUT dynamic_linking)
	cd $(APP) && cargo build --release

fmt: ## Format the code
	cd $(APP) && cargo fmt

fmt-check: ## Check formatting without modifying (used as a gate)
	cd $(APP) && cargo fmt --check

lint: ## Clippy in strict mode: every warning becomes an error
	cd $(APP) && cargo clippy --all-targets -- -D warnings

test: ## Run the tests (pure logic: math, coordinate mapping, generation)
	cd $(APP) && cargo test

check: fmt-check lint test ## CI-like gate: must pass before every commit

watch: ## Rebuild + rerun on every save (requires cargo-watch)
	cd $(APP) && cargo watch -x 'run --features bevy/dynamic_linking'

clean: ## Clean build artifacts
	cd $(APP) && cargo clean
