# Makefile for Terra Siaga - Test Automation and Development Tasks

.PHONY: help test test-unit test-integration test-performance test-coverage clean setup lint format check build dev docker-test

# Default target
help:
	@echo "🚀 Terra Siaga - Available Commands"
	@echo "=================================="
	@echo ""
	@echo "🧪 Testing:"
	@echo "  test                Run all tests"
	@echo "  test-unit          Run unit tests only"
	@echo "  test-integration   Run integration tests only"
	@echo "  test-performance   Run performance tests only"
	@echo "  test-coverage      Generate test coverage report"
	@echo ""
	@echo "🔧 Development:"
	@echo "  setup              Setup development environment"
	@echo "  build              Build the project"
	@echo "  dev                Run development server with hot reload"
	@echo "  format             Format code with rustfmt"
	@echo "  lint               Run linting (clippy + fmt check)"
	@echo "  check              Run cargo check"
	@echo ""
	@echo "🐳 Docker:"
	@echo "  docker-test        Run tests in Docker container"
	@echo "  docker-build       Build Docker image"
	@echo ""
	@echo "🧹 Cleanup:"
	@echo "  clean              Clean build artifacts and test data"

# Setup development environment
setup:
	@echo "🔧 Setting up development environment..."
	@scripts/test.sh prerequisites
	@scripts/test.sh setup
	@echo "✅ Development environment ready!"

# Build project
build:
	@echo "🏗️ Building Terra Siaga..."
	@cargo build
	@echo "✅ Build completed!"

# Build release
build-release:
	@echo "🏗️ Building Terra Siaga (release)..."
	@cargo build --release
	@echo "✅ Release build completed!"

# Run development server with hot reload
dev:
	@echo "🚀 Starting development server..."
	@cargo watch -x run

# Format code
format:
	@echo "🎨 Formatting code..."
	@cargo fmt
	@echo "✅ Code formatted!"

# Run linting
lint:
	@echo "🔍 Running linting..."
	@scripts/test.sh lint
	@echo "✅ Linting completed!"

# Run cargo check
check:
	@echo "🔍 Running cargo check..."
	@cargo check
	@echo "✅ Check completed!"

# Run all tests
test:
	@echo "🧪 Running all tests..."
	@scripts/test.sh all
	@echo "✅ All tests completed!"

# Run unit tests only
test-unit:
	@echo "🔧 Running unit tests..."
	@scripts/test.sh unit
	@echo "✅ Unit tests completed!"

# Run integration tests only
test-integration:
	@echo "🔗 Running integration tests..."
	@scripts/test.sh integration
	@echo "✅ Integration tests completed!"

# Run performance tests only
test-performance:
	@echo "⚡ Running performance tests..."
	@scripts/test.sh performance
	@echo "✅ Performance tests completed!"

# Generate test coverage
test-coverage:
	@echo "📊 Generating test coverage..."
	@scripts/test.sh coverage
	@echo "✅ Coverage report generated!"
	@echo "📖 Open coverage/tarpaulin-report.html to view results"

# Clean build artifacts and test data
clean:
	@echo "🧹 Cleaning up..."
	@cargo clean
	@scripts/test.sh cleanup
	@rm -rf coverage/
	@echo "✅ Cleanup completed!"

# Docker targets
docker-build:
	@echo "🐳 Building Docker image..."
	@docker build -t terra-siaga:latest .
	@echo "✅ Docker image built!"

docker-test:
	@echo "🐳 Running tests in Docker..."
	@docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit
	@docker-compose -f docker-compose.test.yml down
	@echo "✅ Docker tests completed!"

# Database operations
db-setup:
	@echo "🗄️ Setting up database..."
	@diesel setup
	@diesel migration run
	@echo "✅ Database setup completed!"

db-reset:
	@echo "🗄️ Resetting database..."
	@diesel database reset
	@echo "✅ Database reset completed!"

db-migrate:
	@echo "🗄️ Running database migrations..."
	@diesel migration run
	@echo "✅ Migrations completed!"

# Security audit
audit:
	@echo "🔒 Running security audit..."
	@cargo audit
	@echo "✅ Security audit completed!"

# Update dependencies
update:
	@echo "📦 Updating dependencies..."
	@cargo update
	@cargo outdated
	@echo "✅ Dependencies updated!"

# Install development tools
install-tools:
	@echo "🛠️ Installing development tools..."
	@cargo install diesel_cli --no-default-features --features postgres
	@cargo install cargo-watch
	@cargo install cargo-tarpaulin
	@cargo install cargo-audit
	@cargo install cargo-outdated
	@echo "✅ Development tools installed!"

# CI/CD simulation
ci:
	@echo "🔄 Running CI/CD pipeline simulation..."
	@make lint
	@make test
	@make test-coverage
	@make audit
	@echo "✅ CI/CD pipeline completed!"

# Development workflow
dev-workflow: format lint test
	@echo "✅ Development workflow completed!"

# Pre-commit hook
pre-commit: format lint test-unit
	@echo "✅ Pre-commit checks passed!"
