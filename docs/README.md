# 📚 Terra Siaga - Comprehensive Documentation

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [API Documentation](#api-documentation)
4. [Development Guide](#development-guide)
5. [Deployment Guide](#deployment-guide)
6. [Testing Framework](#testing-framework)
7. [Contributing Guidelines](#contributing-guidelines)

---

## 🌍 Project Overview

Terra Siaga adalah sistem manajemen tanggap darurat yang dibangun dengan Rust menggunakan Clean Architecture. Sistem ini dirancang untuk menangani pelaporan bencana, koordinasi tim respons, dan komunikasi multi-channel dalam situasi darurat.

### 🎯 Key Features

- **🚨 Real-time Disaster Reporting**: Pelaporan bencana dengan geolokasi dan kategori risiko
- **👥 Emergency Response Coordination**: Koordinasi tim respons dengan tracking real-time
- **📱 Multi-channel Notifications**: WhatsApp, SMS, Email, dan Push notifications
- **📊 Analytics & Reporting**: Dashboard analitik dan laporan komprehensif
- **🗺️ GIS Integration**: Pemetaan interaktif dengan layanan geolokasi
- **🔐 Role-based Access Control**: Sistem otentikasi dengan role Admin, Responder, Citizen

### 🏗️ Architecture Overview

```
┌─────────────────────────────────────────┐
│            Presentation Layer           │  ← API Controllers, HTTP Handlers
├─────────────────────────────────────────┤
│            Application Layer            │  ← Use Cases, Commands, Queries
├─────────────────────────────────────────┤
│              Domain Layer               │  ← Entities, Value Objects, Events
├─────────────────────────────────────────┤
│           Infrastructure Layer          │  ← Database, Cache, External APIs
└─────────────────────────────────────────┘
```

### 🛠️ Tech Stack

- **Backend**: Rust with Actix-web
- **Database**: PostgreSQL with Diesel ORM
- **Cache**: Redis
- **Authentication**: JWT tokens
- **External APIs**: WhatsApp Business, Google Maps, Weather APIs
- **Testing**: Comprehensive test suite with 80%+ coverage

---

## 🏗️ Architecture

### Clean Architecture Layers

#### 1. **Domain Layer** (`src/domain/`)
Core business logic yang independen dari framework dan teknologi eksternal.

```
domain/
├── entities/           # Business entities
│   ├── user.rs        # User entity dengan business rules
│   ├── disaster.rs    # Disaster management entity
│   ├── location.rs    # Location dan geographic data
│   └── notification.rs # Notification system entity
├── value_objects/     # Domain value objects
├── events/           # Domain events
├── services/         # Domain services
└── ports/           # Interfaces untuk external dependencies
    ├── repositories.rs # Repository contracts
    └── services.rs    # Service contracts
```

#### 2. **Application Layer** (`src/application/`)
Orchestration layer yang menggunakan domain entities untuk implementasi use cases.

```
application/
├── use_cases/        # Business use cases
│   ├── auth.rs      # Authentication flows
│   └── mod.rs
├── commands/         # Command handlers (CQRS)
├── queries/          # Query handlers (CQRS)
├── dto/             # Data Transfer Objects
├── services/        # Application services
└── handlers/        # Event handlers
    ├── auth.rs
    ├── emergency.rs
    ├── notification.rs
    └── analytics.rs
```

#### 3. **Infrastructure Layer** (`src/infrastructure/`)
Implementation detail untuk external concerns.

```
infrastructure/
├── database/         # Database implementations
├── cache/           # Redis cache implementation
├── repository/      # Repository implementations
│   ├── user_repository.rs
│   ├── disaster_repository.rs
│   └── location_repository.rs
├── external_services/ # Third-party integrations
│   ├── whatsapp.rs
│   ├── email.rs
│   ├── weather.rs
│   └── geolocation.rs
├── monitoring.rs    # Health checks & metrics
└── container.rs     # Dependency injection
```

#### 4. **Presentation Layer** (`src/presentation/`)
HTTP interface dan API endpoints.

```
presentation/
└── api/
    └── v1/
        ├── auth.rs        # Authentication endpoints
        ├── users.rs       # User management
        ├── disasters.rs   # Disaster management
        ├── locations.rs   # Location services
        ├── notifications.rs # Notification system
        ├── analytics.rs   # Analytics & reporting
        └── emergency.rs   # Emergency response
```

### 📊 Data Flow

```
HTTP Request → Middleware → Controller → Use Case → Domain Service → Repository → Database
     ↓              ↓           ↓           ↓            ↓           ↓         ↓
HTTP Response ← Serialization ← DTO ← Domain Entity ← Business Logic ← Query ← Result
```

---

## 📖 API Documentation

### Base URL
- **Development**: `http://localhost:8080`
- **Production**: `https://api.terrasiaga.id`

### Authentication
Menggunakan JWT Bearer token:
```http
Authorization: Bearer <your-jwt-token>
```

### Core API Endpoints

#### 🔐 Authentication
```http
POST /api/v1/auth/login        # User login
POST /api/v1/auth/register     # User registration
POST /api/v1/auth/refresh      # Token refresh
GET  /api/v1/auth/me          # Current user info
POST /api/v1/auth/logout      # User logout
```

#### 🚨 Disaster Management
```http
GET    /api/v1/disasters                    # List disasters
POST   /api/v1/disasters                    # Create disaster report
GET    /api/v1/disasters/{id}              # Get disaster details
PUT    /api/v1/disasters/{id}              # Update disaster
DELETE /api/v1/disasters/{id}              # Delete disaster
POST   /api/v1/disasters/{id}/assign       # Assign responder
GET    /api/v1/disasters/nearby            # Get nearby disasters
```

#### 🚑 Emergency Response
```http
POST /api/v1/emergency/response            # Initiate emergency response
GET  /api/v1/emergency/active             # Get active emergencies
POST /api/v1/emergency/{id}/dispatch      # Dispatch teams
GET  /api/v1/emergency/teams/available    # Available response teams
```

#### 📢 Notifications
```http
GET  /api/v1/notifications                # User notifications
POST /api/v1/notifications                # Create notification
POST /api/v1/notifications/broadcast      # Emergency broadcast
GET  /api/v1/notifications/unread-count   # Unread count
```

#### 📊 Analytics
```http
GET /api/v1/analytics/dashboard           # Dashboard data
GET /api/v1/analytics/disasters/trends    # Disaster trends
GET /api/v1/analytics/response-times      # Response analytics
GET /api/v1/analytics/geographic          # Geographic analysis
```

### Request/Response Examples

#### Create Disaster Report
```http
POST /api/v1/disasters
Content-Type: application/json
Authorization: Bearer <token>

{
  "title": "Gempa Bumi 6.2 SR",
  "description": "Gempa bumi dengan kekuatan 6.2 SR di wilayah Yogyakarta",
  "disaster_type": "earthquake",
  "severity": "high",
  "latitude": -7.7956,
  "longitude": 110.3695,
  "address": "Yogyakarta, Indonesia",
  "affected_population": 5000
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "title": "Gempa Bumi 6.2 SR",
    "status": "reported",
    "severity": "high",
    "created_at": "2025-07-23T10:00:00Z"
  }
}
```

---

## 🛠️ Development Guide

### Prerequisites

```bash
# Install Rust (minimum 1.70)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL
brew install postgresql@14  # macOS
sudo apt install postgresql-14  # Ubuntu

# Install Redis
brew install redis  # macOS
sudo apt install redis-server  # Ubuntu

# Install Diesel CLI
cargo install diesel_cli --no-default-features --features postgres
```

### Quick Start

1. **Clone & Setup**
```bash
git clone https://github.com/terra-siaga/terra-siaga.git
cd terra-siaga
cp .env.example .env
```

2. **Database Setup**
```bash
# Start services
brew services start postgresql redis

# Create database
createdb terrasiaga
diesel setup
diesel migration run
```

3. **Run Application**
```bash
# Development mode
make dev

# Or manual
cargo run
```

### Development Commands

```bash
# Available commands
make help

# Common workflows
make setup          # Setup development environment
make dev            # Run with hot reload
make test           # Run all tests
make test-coverage  # Generate coverage report
make lint           # Run linting
make format         # Format code
```

### Project Structure

```
src/
├── lib.rs              # Library root
├── main.rs             # Application entry point
├── schema.rs           # Database schema (generated)
├── uuid.rs             # UUID utilities
├── application/        # Application layer
├── domain/             # Domain layer
├── infrastructure/     # Infrastructure layer
├── presentation/       # Presentation layer
├── middleware/         # HTTP middleware
├── shared/            # Shared utilities
└── config/           # Configuration management
```

### Adding New Features

1. **Domain First**: Start with domain entities and business rules
2. **Use Cases**: Implement application use cases
3. **Infrastructure**: Add repository implementations
4. **API**: Create HTTP endpoints
5. **Tests**: Write comprehensive tests

Example workflow:
```bash
# 1. Create domain entity
touch src/domain/entities/new_feature.rs

# 2. Add use case
touch src/application/use_cases/new_feature.rs

# 3. Implement repository
touch src/infrastructure/repository/new_feature_repository.rs

# 4. Add API endpoints
touch src/presentation/api/v1/new_feature.rs

# 5. Write tests
touch tests/unit/new_feature_tests.rs
```

---

## 🚀 Deployment Guide

### Docker Deployment

1. **Build Image**
```bash
make docker-build
```

2. **Production Setup**
```bash
docker-compose -f docker-compose.prod.yml up -d
```

### Manual Deployment

1. **Build Release**
```bash
make build-release
```

2. **Setup Production Environment**
```bash
# Configure environment variables
cp .env.example .env.production

# Setup database
diesel migration run

# Start application
./target/release/terra-siaga
```

### Environment Variables

```env
# Core Configuration
DATABASE_URL=postgresql://user:pass@localhost:5432/terrasiaga
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-super-secret-jwt-key-32-chars-min

# External Services
WHATSAPP_API_KEY=your_whatsapp_api_key
GOOGLE_MAPS_API_KEY=your_google_maps_api_key
WEATHER_API_KEY=your_weather_api_key

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
ENVIRONMENT=production
```

---

## 🧪 Testing Framework

### Test Structure

```
tests/
├── common/             # Test utilities & mocks
├── unit/              # Unit tests
│   ├── user_tests.rs
│   ├── disaster_tests.rs
│   └── auth_use_case_tests.rs
└── integration/       # Integration tests
    ├── api_tests.rs
    └── performance_tests.rs
```

### Running Tests

```bash
# All tests
make test

# Specific test types
make test-unit
make test-integration
make test-performance

# With coverage
make test-coverage
```

### Test Categories

#### Unit Tests
- Domain entities business logic
- Value objects validation
- Use cases behavior
- Service implementations

#### Integration Tests
- Complete API workflows
- Database operations
- Authentication flows
- External service integrations

#### Performance Tests
- Concurrent operations
- Load testing
- Memory usage
- Response time benchmarks

### Test Examples

```rust
#[tokio::test]
async fn test_disaster_creation() {
    let container = create_test_container().await;
    let disaster_data = TestFixtures::create_test_disaster();
    
    let result = container.disaster_repository
        .save(&disaster_data)
        .await;
    
    assert!(result.is_ok());
}
```

---

## 🤝 Contributing Guidelines

### Code Standards

- **Clean Architecture**: Follow layer separation
- **Test Coverage**: Minimum 80% coverage
- **Documentation**: Document public APIs
- **Performance**: Consider performance implications

### Development Workflow

1. **Fork & Branch**
```bash
git checkout -b feature/your-feature-name
```

2. **Develop & Test**
```bash
make pre-commit  # Format, lint, test
```

3. **Submit PR**
- Fill out PR template
- Ensure tests pass
- Add documentation

### Commit Messages

```
feat(disaster): add severity classification
fix(auth): resolve token expiration issue
docs(api): update endpoint documentation
test(integration): add disaster workflow tests
```

---

## 📞 Support & Resources

### Documentation Links
- [API Documentation](docs/api/README.md)
- [Architecture Guide](docs/architecture/README.md)
- [Deployment Guide](docs/deployment/README.md)
- [Development Guide](docs/development/README.md)

### Community
- **GitHub**: [terra-siaga/terra-siaga](https://github.com/terra-siaga/terra-siaga)
- **Issues**: [GitHub Issues](https://github.com/terra-siaga/terra-siaga/issues)
- **Discussions**: [GitHub Discussions](https://github.com/terra-siaga/terra-siaga/discussions)

### Support
- **Email**: support@terrasiaga.id
- **Documentation**: [docs.terrasiaga.id](https://docs.terrasiaga.id)

---

**Terra Siaga** - *Building safer communities through technology* 🌍

*Last updated: July 23, 2025*
