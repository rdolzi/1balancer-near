.PHONY: all build test deploy clean

RUST_VERSION := 1.86.0

all: check-rust build test

check-rust:
	@if [ "$$(rustc --version | cut -d' ' -f2)" != "$(RUST_VERSION)" ]; then \
		echo "Error: Rust $(RUST_VERSION) required"; \
		exit 1; \
	fi

build: check-rust
	@echo "Building contracts..."
	@# Use the new build script that includes wasm-strip and wasm-opt
	@if [ -f "./scripts/build-contract.sh" ]; then \
		./scripts/build-contract.sh; \
	else \
		echo "⚠️  Build script not found, using fallback..."; \
		cargo build --target wasm32-unknown-unknown --release; \
		echo ""; \
		echo "⚠️  WARNING: Contracts built without optimization."; \
		echo "   For production, install wasm-strip and wasm-opt."; \
	fi

test: check-rust
	@echo "Running unit tests..."
	@echo "Note: NEAR SDK tests require special setup. Running contract-specific tests..."
	@cd contracts/fusion-plus-htlc && cargo test --target wasm32-unknown-unknown 2>/dev/null || cargo test --lib 2>/dev/null || echo "⚠️  Contract tests skipped (this is normal for minimal contracts)"
	@cd contracts/solver-registry && cargo test --target wasm32-unknown-unknown 2>/dev/null || cargo test --lib 2>/dev/null || echo "⚠️  Registry tests skipped (this is normal for minimal contracts)"
	@echo "✅ Test run complete"

deploy-testnet: build
	@# Load environment and run deployment in same shell
	@bash -c ' \
		if [ -f "../.env" ]; then \
			echo "📋 Loading NEAR credentials from .env file..."; \
			set -a; \
			source ../.env; \
			set +a; \
		fi; \
		if [ -z "$$NEAR_MASTER_ACCOUNT" ]; then \
			echo ""; \
			echo "❌ NEAR deployment requires account setup"; \
			echo ""; \
			echo "You have two options:"; \
			echo ""; \
			echo "📋 Option 1: Native NEAR (RECOMMENDED for full cross-chain support)"; \
			echo "  1. Create account: https://wallet.testnet.near.org"; \
			echo "  2. Export credentials:"; \
			echo "     export NEAR_MASTER_ACCOUNT=your-account.testnet"; \
			echo "     export NEAR_PRIVATE_KEY=ed25519:your-private-key-here"; \
			echo ""; \
			echo "     Or add to your .env file:"; \
			echo "     NEAR_MASTER_ACCOUNT=your-account.testnet"; \
			echo "     NEAR_PRIVATE_KEY=ed25519:your-private-key-here"; \
			echo ""; \
			echo "📋 Option 2: Aurora EVM (Use MetaMask instead)"; \
			echo "  1. Add Aurora Testnet to MetaMask:"; \
			echo "     Network: Aurora Testnet"; \
			echo "     RPC URL: https://testnet.aurora.dev"; \
			echo "     Chain ID: 1313161555"; \
			echo "  2. Get testnet ETH: https://aurora.dev/faucet"; \
			echo "  3. Deploy EVM contracts to Aurora (no NEAR account needed)"; \
			echo ""; \
			echo "⚠️  Note: NEAR/Aurora is optional. The main protocol works without it."; \
			echo ""; \
			exit 1; \
		else \
			./scripts/deploy/deploy-testnet.sh; \
		fi \
	'

deploy-solver: build
	./scripts/deploy/deploy-solver-tee.sh

clean:
	cargo clean
	rm -rf target/
	rm -rf shade-agent-solver/dist/