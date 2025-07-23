# 📁 Terra Siaga - Project Structure Documentation

## 🗂️ Complete Project Structure

```
terra-siaga/
├── 📄 Configuration Files
│   ├── Cargo.toml              # Rust project configuration & dependencies
│   ├── Cargo.lock              # Dependency lock file
│   ├── diesel.toml             # Database ORM configuration
│   ├── Makefile                # Build & development automation
│   ├── .env                    # Environment variables (development)
│   ├── .env.example            # Environment template
│   └── .env.test               # Test environment variables
│
├── 📚 Documentation
│   ├── README.md               # Main project documentation
│   ├── CHANGELOG.md            # Version history & release notes
│   ├── CONTRIBUTING.md         # Contribution guidelines
│   ├── SECURITY.md             # Security policy & vulnerability reporting
│   └── docs/                   # Detailed documentation
│       ├── README.md           # Comprehensive documentation index
│       ├── api/                # API documentation
│       │   └── README.md       # REST API specifications
│       ├── architecture/       # Architecture documentation
│       │   └── README.md       # Clean Architecture implementation
│       ├── deployment/         # Deployment guides
│       │   └── README.md       # Production deployment instructions
│       ├── development/        # Development guides
│       │   └── README.md       # Local development setup
│       └── examples/           # Code examples & tutorials
│
├── 🗄️ Database
│   └── migrations/             # Database schema migrations
│       ├── 00000000000000_diesel_initial_setup/
│       ├── 2025-07-21-002327_installing_extention/
│       ├── 2025-07-21-125402_initial_structure/
│       ├── 2025-07-21-211500_sample_data/
│       └── 2025-07-21-215000_additional_sample_data/
│
├── 🛠️ Scripts & Tools
│   └── scripts/
│       └── test.sh             # Comprehensive test runner
│
├── 🧪 Testing
│   └── tests/
│       ├── common/             # Shared test utilities
│       │   ├── mod.rs          # Test helpers & mock implementations
│       │   └── test_config.rs  # Test configuration & setup
│       ├── unit/               # Unit tests
│       │   ├── user_tests.rs           # User entity tests
│       │   ├── disaster_tests.rs       # Disaster logic tests
│       │   └── auth_use_case_tests.rs  # Authentication tests
│       └── integration/        # Integration tests
│           ├── api_tests.rs            # API endpoint tests
│           └── performance_tests.rs    # Performance & load tests
│
└── 💻 Source Code
    └── src/
        ├── lib.rs              # Library root & public API
        ├── main.rs             # Application entry point
        ├── schema.rs           # Generated database schema (Diesel)
        ├── uuid.rs             # UUID utilities & helpers
        │
        ├── 📋 Application Layer (Use Cases & Orchestration)
        │   └── application/
        │       ├── mod.rs              # Application layer exports
        │       ├── use_cases/          # Business use cases
        │       │   ├── mod.rs          # Use case trait definitions
        │       │   └── auth.rs         # Authentication workflows
        │       ├── commands/           # CQRS Command handlers
        │       │   └── mod.rs
        │       ├── queries/            # CQRS Query handlers
        │       │   └── mod.rs
        │       ├── dto/                # Data Transfer Objects
        │       │   └── mod.rs
        │       ├── services/           # Application services
        │       │   └── mod.rs
        │       └── handlers/           # Event handlers
        │           ├── mod.rs
        │           ├── auth.rs         # Authentication events
        │           ├── emergency.rs    # Emergency response events
        │           ├── notification.rs # Notification events
        │           ├── user.rs         # User management events
        │           ├── analytics.rs    # Analytics events
        │           └── map.rs          # Mapping events
        │
        ├── 🏛️ Domain Layer (Business Logic)
        │   └── domain/
        │       ├── mod.rs              # Domain layer exports
        │       ├── entities/           # Business entities
        │       │   ├── mod.rs
        │       │   ├── user.rs         # User entity with business rules
        │       │   ├── disaster.rs     # Disaster management entity
        │       │   ├── location.rs     # Geographic location entity
        │       │   └── notification.rs # Notification system entity
        │       ├── value_objects/      # Domain value objects
        │       │   └── mod.rs          # Email, coordinates, etc.
        │       ├── events/             # Domain events
        │       │   └── mod.rs          # Event definitions
        │       ├── services/           # Domain services
        │       │   └── mod.rs          # Business logic services
        │       └── ports/              # Interfaces (Dependency Inversion)
        │           ├── mod.rs
        │           ├── repositories.rs # Repository contracts
        │           ├── services.rs     # Service contracts
        │           └── events.rs       # Event publisher contracts
        │
        ├── 🔧 Infrastructure Layer (External Concerns)
        │   └── infrastructure/
        │       ├── mod.rs              # Infrastructure exports
        │       ├── container.rs        # Dependency injection container
        │       ├── monitoring.rs       # Health checks & metrics
        │       ├── messaging.rs        # Message queue integration
        │       ├── logging.rs          # Structured logging setup
        │       ├── database/           # Database implementations
        │       │   └── mod.rs          # Connection pool & health checks
        │       ├── cache/              # Redis cache implementation
        │       │   └── mod.rs          # Cache service with fallback
        │       ├── repository/         # Repository implementations
        │       │   ├── mod.rs
        │       │   ├── user_repository.rs       # User data access
        │       │   ├── disaster_repository.rs   # Disaster data access
        │       │   └── location_repository.rs   # Location data access
        │       ├── external_services/  # Third-party service integrations
        │       │   ├── mod.rs
        │       │   ├── whatsapp.rs     # WhatsApp Business API
        │       │   ├── email.rs        # Email service (SMTP/SendGrid)
        │       │   ├── sms.rs          # SMS service (Twilio/AWS SNS)
        │       │   ├── weather.rs      # Weather API integration
        │       │   ├── geolocation.rs  # Google Maps/geolocation
        │       │   └── notification.rs # Push notification service
        │       └── external_api/       # Alternative API implementations
        │           ├── mod.rs
        │           ├── whatsapp.rs
        │           ├── email.rs
        │           ├── geo_service.rs
        │           └── weather_api.rs
        │
        ├── 🌐 Presentation Layer (HTTP Interface)
        │   └── presentation/
        │       ├── mod.rs              # Presentation layer exports
        │       └── api/                # REST API implementation
        │           ├── mod.rs          # API configuration
        │           └── v1/             # API version 1
        │               ├── mod.rs      # Route configuration
        │               ├── auth.rs     # Authentication endpoints
        │               ├── users.rs    # User management endpoints
        │               ├── disasters.rs # Disaster management endpoints
        │               ├── locations.rs # Location service endpoints
        │               ├── notifications.rs # Notification endpoints
        │               ├── analytics.rs # Analytics & reporting endpoints
        │               └── emergency.rs # Emergency response endpoints
        │
        ├── 🔗 Middleware (Cross-cutting Concerns)
        │   └── middleware/
        │       ├── mod.rs              # Middleware exports
        │       ├── auth.rs             # JWT authentication middleware
        │       ├── cors.rs             # CORS configuration
        │       └── logging.rs          # Request/response logging
        │
        ├── 🛠️ Shared Utilities
        │   └── shared/
        │       ├── mod.rs              # Shared utilities exports
        │       ├── error.rs            # Error types & handling
        │       ├── types.rs            # Common type definitions
        │       ├── response.rs         # API response wrappers
        │       ├── validation.rs       # Input validation utilities
        │       ├── jwt.rs              # JWT token utilities
        │       └── database.rs         # Database utilities
        │
        └── ⚙️ Configuration
            └── config/
                ├── mod.rs              # Configuration exports
                ├── env.rs              # Environment variable handling
                └── database.rs         # Database configuration
```

## 🔄 Data Flow Architecture

### Request Processing Flow
```
1. HTTP Request
   ↓
2. Middleware (auth, cors, logging)
   ↓
3. Presentation Layer (API controllers)
   ↓
4. Application Layer (use cases)
   ↓
5. Domain Layer (business logic)
   ↓
6. Infrastructure Layer (repositories)
   ↓
7. Database/External Services
```

### Layer Dependencies
```
Presentation → Application → Domain ← Infrastructure
     ↓              ↓          ↑         ↑
  HTTP Logic    Use Cases  Entities  Database/APIs
```

## 📂 Key File Responsibilities

### 🏛️ Domain Layer Files

#### `domain/entities/user.rs`
- User business entity with role-based permissions
- User lifecycle management (activation, suspension)
- Authentication state and email verification

#### `domain/entities/disaster.rs`
- Disaster reporting and classification
- Status transitions (reported → verified → responding → resolved)
- Severity levels and response priority logic

#### `domain/entities/location.rs`
- Geographic location management
- Coordinate validation and geocoding
- Emergency shelter and facility management

#### `domain/entities/notification.rs`
- Multi-channel notification system
- Delivery status tracking
- Priority-based notification routing

### 📋 Application Layer Files

#### `application/use_cases/auth.rs`
- User registration and login workflows
- Token generation and validation
- Password reset and email verification

#### `application/handlers/*.rs`
- Event-driven architecture implementations
- Cross-cutting concern handlers
- Integration between different domains

### 🔧 Infrastructure Layer Files

#### `infrastructure/repository/*.rs`
- PostgreSQL database implementations
- Query optimization and connection pooling
- Data mapping between database and domain entities

#### `infrastructure/external_services/*.rs`
- Third-party API integrations
- Circuit breaker and retry logic
- Service health monitoring

### 🌐 Presentation Layer Files

#### `presentation/api/v1/*.rs`
- RESTful API endpoint implementations
- Request/response serialization
- HTTP status code and error handling

## 🔧 Development Workflow

### 1. Adding New Feature

```bash
# 1. Domain entity (if needed)
touch src/domain/entities/new_feature.rs

# 2. Repository interface
# Add to src/domain/ports/repositories.rs

# 3. Use case implementation
touch src/application/use_cases/new_feature.rs

# 4. Repository implementation
touch src/infrastructure/repository/new_feature_repository.rs

# 5. API endpoints
touch src/presentation/api/v1/new_feature.rs

# 6. Tests
touch tests/unit/new_feature_tests.rs
touch tests/integration/new_feature_api_tests.rs
```

### 2. Database Changes

```bash
# Create migration
diesel migration generate feature_name

# Edit migration files
# - migrations/*/up.sql
# - migrations/*/down.sql

# Apply migration
diesel migration run

# Update schema
diesel print-schema > src/schema.rs
```

### 3. Testing Workflow

```bash
# Unit tests (domain logic)
make test-unit

# Integration tests (API endpoints)
make test-integration

# Performance tests
make test-performance

# Full test suite with coverage
make test-coverage
```

## 📊 Architecture Benefits

### ✅ **Clean Architecture Advantages**
- **Testability**: Easy to mock dependencies
- **Maintainability**: Clear separation of concerns
- **Flexibility**: Framework-independent business logic
- **Scalability**: Easy to add new features

### ✅ **Project Structure Benefits**
- **Developer Experience**: Intuitive file organization
- **Code Reusability**: Shared utilities and common patterns
- **Documentation**: Comprehensive guides and examples
- **Quality Assurance**: Automated testing and linting

### ✅ **Operational Excellence**
- **Monitoring**: Built-in health checks and metrics
- **Deployment**: Docker and automation support
- **Security**: JWT authentication and input validation
- **Performance**: Caching and database optimization

---

*This structure supports a maintainable, scalable, and robust emergency response system built with Rust and Clean Architecture principles.*
