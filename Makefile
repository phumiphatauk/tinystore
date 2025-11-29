.PHONY: build build-ui build-server clean run test help

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: build-server ## Build the entire project (server with SSR UI)

build-ui: ## Build the UI with Trunk (for client-side rendering)
	trunk build --release

build-server: ## Build the server with SSR
	cargo build --release

clean: ## Clean build artifacts
	cargo clean
	rm -rf public/pkg

run: ## Run the server in development mode
	cargo run -- serve

run-release: build ## Run the server in release mode
	./target/release/tinystore serve

test: ## Run tests
	cargo test --all

watch: ## Watch and rebuild on changes
	cargo watch -x 'run -- serve'

fmt: ## Format code
	cargo fmt --all

lint: ## Run clippy linter
	cargo clippy --all -- -D warnings
