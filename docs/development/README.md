# 🛠️ Development Guide - Terra Siaga

## 🚀 Getting Started

### Prerequisites

Pastikan system Anda memiliki tools berikut:

```bash
# Rust toolchain (minimum 1.70)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# PostgreSQL 14+
brew install postgresql@14  # macOS
sudo apt install postgresql-14  # Ubuntu

# Redis 6+
brew install redis  # macOS
sudo apt install redis-server  # Ubuntu

# Diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Optional: Docker for containerized development
brew install docker docker-compose  # macOS
```

### Environment Setup

1. **Clone dan Setup Project**
```bash
git clone https://github.com/your-org/terra-siaga.git
cd terra-siaga
cp .env.example .env
```

2. **Setup Database**
```bash
# Start PostgreSQL
brew services start postgresql@14  # macOS
sudo systemctl start postgresql    # Ubuntu

# Create database
createdb terrasiaga
createuser --interactive terrasiaga_user

# Setup Diesel
diesel setup
diesel migration run
```

3. **Setup Redis**
```bash
# Start Redis
brew services start redis  # macOS
sudo systemctl start redis # Ubuntu

# Verify Redis is running
redis-cli ping
```

4. **Configure Environment Variables**
```bash
# Edit .env file
nano .env

# Minimum required configuration
DATABASE_URL=postgresql://terrasiaga_user:password@localhost:5432/terrasiaga
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-super-secret-jwt-key-minimum-32-characters
```

## 🏃‍♂️ Running the Application

### Development Mode
```bash
# Run with auto-reload
cargo watch -x run

# Or standard run
cargo run

# Run with specific log level
RUST_LOG=debug cargo run
```

### Production Mode
```bash
# Build optimized binary
cargo build --release

# Run production binary
./target/release/terra-siaga
```

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_user_creation

# Run tests in specific module
cargo test user::tests
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration_tests

# Run with test database
DATABASE_URL=postgresql://localhost/terrasiaga_test cargo test
```

### Coverage
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html
open tarpaulin-report.html
```

## 📁 Code Organization

### Adding New Features

1. **Domain-First Approach**
```bash
# 1. Start with domain entities
touch src/domain/entities/new_entity.rs

# 2. Define repositories and services
touch src/domain/ports/new_repository.rs

# 3. Implement use cases
touch src/application/use_cases/new_feature.rs

# 4. Add infrastructure
touch src/infrastructure/repository/new_repository.rs

# 5. Add API endpoints
touch src/presentation/api/v1/new_endpoints.rs
```

2. **File Naming Conventions**
```
snake_case untuk files dan modules
PascalCase untuk structs dan enums
camelCase untuk JSON fields
SCREAMING_SNAKE_CASE untuk constants
```

### Code Style

```bash
# Install rustfmt
rustup component add rustfmt

# Format code
cargo fmt

# Install clippy
rustup component add clippy

# Lint code
cargo clippy -- -D warnings
```

## 🔧 Database Development

### Migrations

```bash
# Create new migration
diesel migration generate add_new_table

# Edit migration files
# - migrations/*/up.sql (apply changes)
# - migrations/*/down.sql (rollback changes)

# Run migrations
diesel migration run

# Rollback migration
diesel migration revert

# Reset database
diesel database reset
```

### Schema Updates

```bash
# After migration, update schema.rs
diesel print-schema > src/schema.rs

# Or use automatic generation
echo 'print_schema = true' >> diesel.toml
```

### Sample Migration Example

```sql
-- up.sql
CREATE TABLE disasters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    description TEXT,
    disaster_type VARCHAR NOT NULL,
    severity VARCHAR NOT NULL,
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    status VARCHAR DEFAULT 'reported',
    reporter_id UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- down.sql
DROP TABLE disasters;
```

## 🐳 Docker Development

### Development with Docker

```bash
# Build development image
docker build -t terra-siaga:dev .

# Run with docker-compose
docker-compose -f docker-compose.dev.yml up

# Run specific services
docker-compose up postgres redis
```

### Docker Compose Configuration
```yaml
# docker-compose.dev.yml
version: '3.8'
services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@postgres:5432/terrasiaga
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    volumes:
      - .:/app
      - target:/app/target

  postgres:
    image: postgres:14
    environment:
      POSTGRES_DB: terrasiaga
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:6-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_data:
  target:
```

## 🔍 Debugging

### Logging
```rust
// Add to Cargo.toml
tracing = "0.1"
tracing-subscriber = "0.3"

// In code
#[tracing::instrument]
async fn create_disaster(req: CreateDisasterRequest) -> AppResult<Disaster> {
    tracing::info!("Creating disaster: {}", req.title);
    tracing::debug!("Request data: {:?}", req);
    // Implementation
}
```

### Debug Tools

```bash
# Install useful tools
cargo install cargo-watch    # Auto-reload
cargo install cargo-edit     # Manage dependencies
cargo install cargo-audit    # Security audit
cargo install cargo-outdated # Check outdated deps

# Use tools
cargo watch -x test         # Auto-run tests
cargo add serde            # Add dependency
cargo audit                # Security check
cargo outdated             # Check updates
```

### VSCode Setup

`.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "files.watcherExclude": {
    "**/target/**": true
  }
}
```

`.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Terra Siaga",
      "cargo": {
        "args": ["build", "--bin=terra-siaga"],
        "filter": {
          "name": "terra-siaga",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

## 🔧 Performance Profiling

### Flamegraph
```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin terra-siaga

# View flamegraph.svg
open flamegraph.svg
```

### Benchmarking
```bash
# Install criterion
cargo install cargo-criterion

# Add benchmark
mkdir benches
touch benches/disaster_benchmarks.rs

# Run benchmarks
cargo bench
```

## 📋 Development Workflow

### Feature Development
```bash
# 1. Create feature branch
git checkout -b feature/disaster-analytics

# 2. Implement feature (TDD approach)
cargo test test_analytics_feature  # Write test first
# Implement feature
cargo test                         # Ensure tests pass

# 3. Run full validation
cargo fmt
cargo clippy
cargo test
cargo audit

# 4. Commit and push
git add .
git commit -m "feat: add disaster analytics"
git push origin feature/disaster-analytics
```

### Code Review Checklist

- [ ] Tests written and passing
- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Migration files included (if applicable)
- [ ] Error handling implemented
- [ ] Performance considered
- [ ] Security reviewed

### Release Process
```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Create release branch
git checkout -b release/v1.2.0

# 4. Final validation
cargo test --release
cargo build --release

# 5. Tag and push
git tag v1.2.0
git push origin v1.2.0
```

## 🛡️ Security Development

### Secret Management
```bash
# Never commit secrets to git
echo ".env" >> .gitignore
echo "*.key" >> .gitignore

# Use environment variables
export JWT_SECRET=$(openssl rand -base64 32)
export DATABASE_PASSWORD=$(openssl rand -base64 16)
```

### Dependency Audit
```bash
# Regular security audits
cargo audit

# Update dependencies
cargo update
cargo outdated
```

## 🚨 Troubleshooting

### Common Issues

1. **Database Connection Error**
```bash
# Check PostgreSQL status
pg_isready -h localhost -p 5432

# Check connection string
psql $DATABASE_URL
```

2. **Migration Errors**
```bash
# Reset database
diesel database reset

# Check migration status
diesel migration list
```

3. **Port Already in Use**
```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

4. **Redis Connection Error**
```bash
# Check Redis status
redis-cli ping

# Restart Redis
brew services restart redis
```

### Debug Environment
```bash
# Enable debug logging
export RUST_LOG=debug,sqlx=warn,hyper=warn

# Enable backtrace
export RUST_BACKTRACE=1

# Run with debug info
cargo run
```

## 📚 Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Actix Web Documentation](https://actix.rs/)
- [Diesel Documentation](https://diesel.rs/guides/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

**Happy Coding!** 🦀
