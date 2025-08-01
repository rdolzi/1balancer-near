.PHONY: all build build-dev test test-integration test-unit deploy-local deploy-testnet deploy-mainnet clean install

RUST_VERSION := 1.86.0
CONTRACT_DIR := contracts
SOLVER_DIR := shade-agent-solver
SCRIPTS_DIR := scripts

# Default target
all: check-rust build test

# Environment checks
check-rust:
	@if [ "$$(rustc --version | cut -d' ' -f2)" != "$(RUST_VERSION)" ]; then \
		echo "Error: Rust $(RUST_VERSION) required"; \
		echo "Run: rustup install $(RUST_VERSION) && rustup default $(RUST_VERSION)"; \
		exit 1; \
	fi
	@if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then \
		echo "Installing wasm32-unknown-unknown target..."; \
		rustup target add wasm32-unknown-unknown; \
	fi

# Installation
install: check-rust
	@echo "Installing dependencies..."
	@if ! command -v near &> /dev/null; then \
		echo "Installing NEAR CLI..."; \
		npm install -g near-cli; \
	fi
	@if ! command -v cargo-near &> /dev/null; then \
		echo "Installing cargo-near (required for building NEAR contracts)..."; \
		echo "This is the official NEAR contract build tool"; \
		cargo install cargo-near --version 0.16.1; \
	fi
	@cd $(SOLVER_DIR) && npm install
	@cd integration-tests && npm install

# Build targets
build: check-rust
	@echo "🔨 Building all contracts..."
	@$(SCRIPTS_DIR)/build/build-all.sh

build-dev: check-rust
	@echo "🔧 Starting development build watcher..."
	@$(SCRIPTS_DIR)/build/build-dev.sh

build-htlc: check-rust
	@echo "Building HTLC contract..."
	@cd $(CONTRACT_DIR)/fusion-plus-htlc && ./build.sh

build-solver-registry: check-rust
	@echo "Building Solver Registry contract..."
	@cd $(CONTRACT_DIR)/solver-registry && cargo near build non-reproducible-wasm

build-solver:
	@echo "Building Shade Agent Solver..."
	@cd $(SOLVER_DIR) && npm run build

# Test targets
test: test-unit test-integration

test-unit: check-rust
	@echo "Running unit tests..."
	@echo "Note: Unit tests for NEAR contracts are currently disabled due to NEAR SDK 5.x limitations"
	@echo "Integration tests will verify contract functionality"

test-integration:
	@echo "Running integration tests..."
	@cd integration-tests && npm test

test-cross-chain:
	@echo "Running cross-chain tests..."
	@cd integration-tests && npm run test:happy

test-security:
	@echo "Running security tests..."
	@cd integration-tests && npm run test:security

# Deployment targets
deploy-local: build
	@echo "Deploying to local NEAR node..."
	@$(SCRIPTS_DIR)/deploy/deploy-local.sh

deploy-testnet: build
	@echo "Deploying to NEAR testnet..."
	@$(SCRIPTS_DIR)/deploy/deploy-testnet.sh

deploy-mainnet: build
	@echo "⚠️  Deploying to NEAR mainnet..."
	@echo "This requires mainnet credentials. Continue? [y/N]"
	@read -r response && [ "$$response" = "y" ] && $(SCRIPTS_DIR)/deploy/deploy-mainnet.sh

deploy-solver-tee: build-solver
	@echo "Deploying solver to TEE..."
	@$(SCRIPTS_DIR)/deploy/deploy-solver-tee.sh

# Development helpers
dev-near:
	@echo "Starting local NEAR node..."
	@near-local-node --home /tmp/near-local

logs-testnet:
	@echo "Streaming testnet logs..."
	@near logs $(NEAR_HTLC_CONTRACT) --follow

# Clean up
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@rm -rf $(SOLVER_DIR)/dist/
	@rm -rf integration-tests/node_modules
	@find . -name "*.log" -delete

clean-all: clean
	@echo "Deep clean including dependencies..."
	@rm -rf $(SOLVER_DIR)/node_modules
	@rm -rf Cargo.lock

# Documentation
docs:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open

# Help
help:
	@echo "1Balancer NEAR - Makefile targets:"
	@echo ""
	@echo "  make install          - Install all dependencies"
	@echo "  make build           - Build all contracts"
	@echo "  make build-dev       - Start development build watcher"
	@echo "  make test            - Run all tests"
	@echo "  make test-unit       - Run unit tests only"
	@echo "  make test-integration - Run integration tests"
	@echo "  make test-cross-chain - Run cross-chain swap tests"
	@echo "  make deploy-testnet  - Deploy to NEAR testnet"
	@echo "  make deploy-local    - Deploy to local NEAR node"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make docs            - Generate documentation"
	@echo ""
	@echo "Environment variables needed:"
	@echo "  NEAR_NETWORK         - Network ID (testnet/mainnet)"
	@echo "  NEAR_ACCOUNT        - Your NEAR account"
	@echo "  NEAR_HTLC_CONTRACT  - HTLC contract address"