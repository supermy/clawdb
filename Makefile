.PHONY: all build test run clean bench install fmt check doc help \
        build-dev run-dev test-all clippy release ci \
        download-data generate-data benchmark quick-bench

all: build

build:
	@echo "🔨 Building project (release mode)..."
	cargo build --release

build-dev:
	@echo "🔨 Building project (debug mode)..."
	cargo build

test:
	@echo "🧪 Running unit tests..."
	cargo test --lib

test-all:
	@echo "🧪 Running all tests..."
	cargo test

run:
	@echo "🚀 Running example..."
	cargo run --release

run-dev:
	@echo "🚀 Running example (debug mode)..."
	cargo run

bench:
	@echo "📊 Running Criterion benchmarks..."
	cargo bench

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf ./vector_db
	rm -rf ./data

fmt:
	@echo "✨ Formatting code..."
	cargo fmt

check:
	@echo "🔍 Checking code..."
	cargo check --all-targets

clippy:
	@echo "🔍 Running clippy..."
	cargo clippy -- -D warnings

doc:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --open

install:
	@echo "📦 Installing dependencies..."
	cargo build

update:
	@echo "📦 Updating dependencies..."
	cargo update

download-data:
	@echo "📥 Downloading SIFT1M dataset..."
	@chmod +x scripts/download_sift1m.sh
	./scripts/download_sift1m.sh

generate-data:
	@echo "🔧 Generating synthetic test data..."
	cargo run --release --bin generate_test_data 100000

benchmark:
	@echo "📊 Running SIFT1M benchmark..."
	cargo run --release --bin sift_benchmark

quick-bench:
	@echo "⚡ Running quick benchmark..."
	cargo run --release --bin quick_benchmark

comprehensive-bench:
	@echo "🔍 Running comprehensive benchmark..."
	cargo run --release --bin comprehensive_benchmark

real-bench:
	@echo "🎯 Running real end-to-end benchmark..."
	cargo run --release --bin real_benchmark

profile:
	@echo "📈 Profiling..."
	cargo run --release --features profiling

release: clean test build
	@echo "✅ Release build ready!"

ci: fmt check clippy test
	@echo "✅ All CI checks passed!"

git-init:
	@echo "📝 Initializing git repository..."
	git init
	git add .
	git commit -m "Initial commit: ClawDB v0.4.0"

git-add:
	@echo "📝 Adding files to git..."
	git add .

git-commit:
	@echo "📝 Committing changes..."
	git commit -m "Update: Documentation and optimizations"

git-push:
	@echo "📤 Pushing to GitHub..."
	git push origin main

help:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║              ClawDB - High-Performance Vector Database        ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "📦 Build Commands:"
	@echo "  build          Build the project (release mode)"
	@echo "  build-dev      Build the project (debug mode)"
	@echo "  clean          Remove build artifacts and data"
	@echo "  install        Install dependencies"
	@echo "  update         Update dependencies"
	@echo ""
	@echo "🧪 Testing Commands:"
	@echo "  test           Run unit tests"
	@echo "  test-all       Run all tests including doc tests"
	@echo "  bench          Run Criterion benchmarks"
	@echo ""
	@echo "📊 Benchmark Commands:"
	@echo "  download-data  Download SIFT1M dataset"
	@echo "  generate-data  Generate synthetic test data (100K vectors)"
	@echo "  benchmark      Run full SIFT1M benchmark"
	@echo "  quick-bench    Run quick benchmark (faster)"
	@echo "  comprehensive-bench Run comprehensive benchmark (all features)"
	@echo "  real-bench     Run real end-to-end benchmark (data->optimization->rocksdb)"
	@echo ""
	@echo "🔍 Code Quality:"
	@echo "  fmt            Format code using rustfmt"
	@echo "  check          Check code for errors"
	@echo "  clippy         Run clippy linter"
	@echo "  ci             Run all CI checks (fmt, check, clippy, test)"
	@echo ""
	@echo "📚 Documentation:"
	@echo "  doc            Generate and open documentation"
	@echo ""
	@echo "🚀 Release:"
	@echo "  release        Prepare release build (clean + test + build)"
	@echo ""
	@echo "📝 Git Commands:"
	@echo "  git-init       Initialize git repository"
	@echo "  git-add        Add all files to git"
	@echo "  git-commit     Commit changes"
	@echo "  git-push       Push to GitHub"
	@echo ""
	@echo "💡 Examples:"
	@echo "  make build              # Build release version"
	@echo "  make test               # Run tests"
	@echo "  make generate-data      # Generate test data"
	@echo "  make benchmark          # Run performance benchmark"
	@echo "  make ci                 # Run all checks before commit"
	@echo ""
	@echo "📖 For more information, see README.md"
