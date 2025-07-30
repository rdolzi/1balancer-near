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
	./scripts/build/build-contracts.sh
	@echo "Building solver..."
	./scripts/build/build-solver.sh

test: check-rust
	@echo "Running unit tests..."
	cargo test
	@echo "Running integration tests..."
	./scripts/test/run-integration.sh

deploy-testnet: build
	./scripts/deploy/deploy-testnet.sh

deploy-solver: build
	./scripts/deploy/deploy-solver-tee.sh

clean:
	cargo clean
	rm -rf target/
	rm -rf shade-agent-solver/dist/