#!/bin/bash
# Test runner script for Terra Siaga
# Runs different types of tests with proper setup and teardown

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TEST_DB_URL="postgresql://postgres:postgres@localhost:5433/terrasiaga_test"

echo -e "${GREEN}🧪 Terra Siaga Test Runner${NC}"
echo "================================="

# Function to check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

    # Check if PostgreSQL is running
    if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
        echo -e "${RED}❌ PostgreSQL is not running${NC}"
        echo "Please start PostgreSQL: brew services start postgresql"
        exit 1
    fi

    # Check if Redis is running
    if ! redis-cli ping >/dev/null 2>&1; then
        echo -e "${RED}❌ Redis is not running${NC}"
        echo "Please start Redis: brew services start redis"
        exit 1
    fi

    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

# Function to setup test database
setup_test_db() {
    echo -e "${YELLOW}🗄️ Setting up test database...${NC}"

    # Drop and recreate test database
    dropdb --if-exists terrasiaga_test -h localhost -U postgres 2>/dev/null || true
    createdb terrasiaga_test -h localhost -U postgres

    # Run migrations
    DATABASE_URL=$TEST_DB_URL diesel migration run

    echo -e "${GREEN}✅ Test database ready${NC}"
}

# Function to cleanup test database
cleanup_test_db() {
    echo -e "${YELLOW}🧹 Cleaning up test database...${NC}"
    dropdb --if-exists terrasiaga_test -h localhost -U postgres 2>/dev/null || true
    echo -e "${GREEN}✅ Test database cleaned${NC}"
}

# Function to run unit tests
run_unit_tests() {
    echo -e "${YELLOW}🔧 Running unit tests...${NC}"
    cargo test --lib --tests unit -- --test-threads=1
    echo -e "${GREEN}✅ Unit tests completed${NC}"
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${YELLOW}🔗 Running integration tests...${NC}"
    cargo test --test integration_tests -- --test-threads=1
    echo -e "${GREEN}✅ Integration tests completed${NC}"
}

# Function to run performance tests
run_performance_tests() {
    echo -e "${YELLOW}⚡ Running performance tests...${NC}"
    cargo test --test performance_tests -- --test-threads=1 --ignored
    echo -e "${GREEN}✅ Performance tests completed${NC}"
}

# Function to run all tests
run_all_tests() {
    echo -e "${YELLOW}🎯 Running all tests...${NC}"
    cargo test -- --test-threads=1
    echo -e "${GREEN}✅ All tests completed${NC}"
}

# Function to generate test coverage
generate_coverage() {
    echo -e "${YELLOW}📊 Generating test coverage...${NC}"

    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin
    fi

    cargo tarpaulin --out Html --output-dir coverage
    echo -e "${GREEN}✅ Coverage report generated in coverage/tarpaulin-report.html${NC}"
}

# Function to run linting
run_linting() {
    echo -e "${YELLOW}🔍 Running code linting...${NC}"

    # Format check
    cargo fmt -- --check

    # Clippy check
    cargo clippy -- -D warnings

    echo -e "${GREEN}✅ Linting completed${NC}"
}

# Main function
main() {
    case "${1:-all}" in
        "prerequisites")
            check_prerequisites
            ;;
        "setup")
            check_prerequisites
            setup_test_db
            ;;
        "unit")
            check_prerequisites
            setup_test_db
            run_unit_tests
            cleanup_test_db
            ;;
        "integration")
            check_prerequisites
            setup_test_db
            run_integration_tests
            cleanup_test_db
            ;;
        "performance")
            check_prerequisites
            setup_test_db
            run_performance_tests
            cleanup_test_db
            ;;
        "coverage")
            check_prerequisites
            setup_test_db
            generate_coverage
            cleanup_test_db
            ;;
        "lint")
            run_linting
            ;;
        "all")
            check_prerequisites
            setup_test_db
            run_linting
            run_all_tests
            generate_coverage
            cleanup_test_db
            ;;
        "cleanup")
            cleanup_test_db
            ;;
        *)
            echo "Usage: $0 {prerequisites|setup|unit|integration|performance|coverage|lint|all|cleanup}"
            echo ""
            echo "Commands:"
            echo "  prerequisites - Check if required services are running"
            echo "  setup        - Setup test database"
            echo "  unit         - Run unit tests only"
            echo "  integration  - Run integration tests only"
            echo "  performance  - Run performance tests only"
            echo "  coverage     - Generate test coverage report"
            echo "  lint         - Run code linting (fmt + clippy)"
            echo "  all          - Run all tests and generate coverage"
            echo "  cleanup      - Clean up test database"
            exit 1
            ;;
    esac
}

# Set test environment
export ENVIRONMENT=test
export DATABASE_URL=$TEST_DB_URL

# Load test environment variables
if [ -f ".env.test" ]; then
    export $(grep -v '^#' .env.test | xargs)
fi

# Run main function
main "$@"
