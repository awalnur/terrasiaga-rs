# 🏗️ Terra Siaga Architecture Documentation

## 📋 Overview

Terra Siaga dibangun menggunakan **Clean Architecture** dengan prinsip Dependency Inversion, memastikan separation of concerns dan maintainability yang tinggi.

## 🎯 Architectural Principles

### 1. Clean Architecture Layers
```
┌─────────────────────────────────────────┐
│            Presentation Layer           │  ← HTTP Handlers, Controllers
├─────────────────────────────────────────┤
│            Application Layer            │  ← Use Cases, Services
├─────────────────────────────────────────┤
│              Domain Layer               │  ← Business Logic, Entities
├─────────────────────────────────────────┤
│           Infrastructure Layer          │  ← Database, External APIs
└─────────────────────────────────────────┘
```

### 2. Dependency Direction
- **Outer layers depend on inner layers**
- **Inner layers never depend on outer layers**
- **Domain layer is completely independent**

## 📁 Project Structure

```
src/
├── domain/                 # 🏛️ Core Business Logic
│   ├── entities/          # Business entities
│   ├── value_objects/     # Domain value objects
│   ├── events/           # Domain events
│   ├── ports/            # Interfaces (repositories, services)
│   └── services/         # Domain services
│
├── application/           # 📋 Application Logic
│   ├── use_cases/        # Business use cases
│   ├── services/         # Application services
│   ├── dto/              # Data Transfer Objects
│   ├── commands/         # Command handlers
│   └── queries/          # Query handlers
│
├── infrastructure/       # 🔧 External Concerns
│   ├── database/         # Database implementation
│   ├── cache/           # Caching implementation
│   ├── external_services/ # Third-party integrations
│   ├── repository/       # Repository implementations
│   ├── messaging/        # Message queue
│   └── monitoring/       # Health checks, metrics
│
├── presentation/         # 🌐 HTTP Interface
│   ├── api/             # REST API endpoints
│   ├── middleware/      # HTTP middleware
│   └── handlers/        # Request handlers
│
└── shared/              # 🛠️ Shared Utilities
    ├── error.rs         # Error handling
    ├── types.rs         # Common types
    └── validation.rs    # Input validation
```

## 🏛️ Domain Layer

### Entities
Core business objects yang mengandung business logic:

```rust
// User entity dengan business rules
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub role: UserRole,
    // ... business methods
}

impl User {
    pub fn can_approve_disaster(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Responder)
    }
}
```

### Value Objects
Objek immutable yang merepresentasikan nilai domain:

```rust
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        // Validation logic
        if is_valid_email(&email) {
            Ok(Email(email))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }
}
```

### Ports (Interfaces)
Contract untuk external dependencies:

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;
    async fn save(&self, user: &User) -> AppResult<User>;
    // ...
}
```

## 📋 Application Layer

### Use Cases
Implementasi business scenarios:

```rust
pub struct RegisterUserUseCase {
    user_repository: Arc<dyn UserRepository>,
    auth_service: Arc<dyn AuthService>,
}

impl RegisterUserUseCase {
    pub async fn execute(&self, request: RegisterRequest) -> AppResult<UserResponse> {
        // 1. Validate input
        self.validate(&request).await?;
        
        // 2. Create user entity
        let user = User::new(request.email, request.username)?;
        
        // 3. Save to repository
        let saved_user = self.user_repository.save(&user).await?;
        
        // 4. Return response
        Ok(UserResponse::from(saved_user))
    }
}
```

### Application Services
Koordinasi antara multiple use cases:

```rust
pub struct DisasterManagementService {
    disaster_use_case: Arc<CreateDisasterUseCase>,
    notification_service: Arc<NotificationService>,
    team_dispatch_service: Arc<TeamDispatchService>,
}
```

## 🔧 Infrastructure Layer

### Repository Implementation
Implementasi concrete dari domain ports:

```rust
pub struct PostgresUserRepository {
    pool: DbPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        // Database implementation
    }
}
```

### External Services
Integrasi dengan third-party services:

```rust
pub struct WhatsAppService {
    client: reqwest::Client,
    api_key: String,
}

#[async_trait]
impl NotificationService for WhatsAppService {
    async fn send_message(&self, message: &Message) -> AppResult<()> {
        // WhatsApp API implementation
    }
}
```

## 🌐 Presentation Layer

### API Controllers
HTTP request handlers:

```rust
pub async fn create_disaster(
    req: web::Json<CreateDisasterRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let use_case = &container.create_disaster_use_case;
    let result = use_case.execute(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(result))
}
```

## 🔄 Data Flow

### 1. Request Flow
```
HTTP Request → Controller → Use Case → Domain Service → Repository → Database
```

### 2. Response Flow
```
Database → Repository → Domain Entity → Use Case → DTO → Controller → HTTP Response
```

### 3. Event Flow
```
Domain Event → Event Handler → External Service → Notification
```

## 🏗️ Design Patterns

### 1. Repository Pattern
Abstraksi akses data dari domain logic:

```rust
pub trait DisasterRepository {
    async fn find_by_location(&self, lat: f64, lng: f64, radius: f64) -> AppResult<Vec<Disaster>>;
}
```

### 2. Dependency Injection
Container pattern untuk managing dependencies:

```rust
pub struct AppContainer {
    pub user_repository: Arc<dyn UserRepository>,
    pub disaster_repository: Arc<dyn DisasterRepository>,
    // ...
}
```

### 3. CQRS (Command Query Responsibility Segregation)
Pemisahan antara command dan query operations:

```rust
// Command
pub struct CreateDisasterCommand {
    pub title: String,
    pub location: Coordinates,
}

// Query
pub struct GetNearbyDisastersQuery {
    pub location: Coordinates,
    pub radius: f64,
}
```

### 4. Event Sourcing
Menyimpan state changes sebagai sequence of events:

```rust
pub enum DisasterEvent {
    DisasterReported { id: DisasterId, title: String },
    DisasterVerified { id: DisasterId, verified_by: UserId },
    ResponderAssigned { disaster_id: DisasterId, responder_id: UserId },
}
```

## 🚀 Performance Considerations

### 1. Caching Strategy
```rust
// Repository with cache
impl CachedUserRepository {
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        // 1. Check cache first
        if let Some(user) = self.cache.get(&id).await? {
            return Ok(Some(user));
        }
        
        // 2. Fallback to database
        let user = self.db_repository.find_by_id(id).await?;
        
        // 3. Update cache
        if let Some(ref u) = user {
            self.cache.set(&id, u).await?;
        }
        
        Ok(user)
    }
}
```

### 2. Database Optimization
- Connection pooling dengan r2d2
- Read replicas untuk query operations
- Indexing strategy untuk geospatial queries

### 3. Async/Await
- Non-blocking I/O operations
- Concurrent request handling
- Efficient resource utilization

## 🔐 Security Architecture

### 1. Authentication Flow
```
Client → JWT Token → Middleware → Use Case → Business Logic
```

### 2. Authorization
Role-based access control di domain layer:

```rust
impl Disaster {
    pub fn can_be_updated_by(&self, user: &User) -> bool {
        match user.role {
            UserRole::Admin => true,
            UserRole::Responder => self.assigned_responders.contains(&user.id),
            UserRole::Citizen => self.reporter_id == user.id,
        }
    }
}
```

## 📊 Monitoring & Observability

### 1. Health Checks
```rust
pub struct HealthMonitor {
    database: Arc<DatabaseService>,
    cache: Arc<CacheService>,
    external_services: Arc<ExternalServicesManager>,
}
```

### 2. Metrics Collection
- Response time monitoring
- Error rate tracking
- Resource utilization metrics

### 3. Logging
Structured logging dengan tracing:

```rust
#[tracing::instrument(skip(self))]
pub async fn create_disaster(&self, request: CreateDisasterRequest) -> AppResult<Disaster> {
    tracing::info!("Creating new disaster report");
    // Implementation
}
```

## 🧪 Testing Strategy

### 1. Unit Tests
Testing individual components in isolation:

```rust
#[tokio::test]
async fn test_create_user_use_case() {
    let mock_repository = MockUserRepository::new();
    let use_case = CreateUserUseCase::new(Arc::new(mock_repository));
    // Test implementation
}
```

### 2. Integration Tests
Testing component interactions:

```rust
#[tokio::test]
async fn test_disaster_workflow() {
    let container = test_container().await;
    // Test full workflow
}
```

### 3. Contract Tests
Testing external service integrations with mocks.

## 🔮 Future Architecture Considerations

### 1. Microservices Migration
- Service decomposition strategy
- Inter-service communication
- Data consistency patterns

### 2. Event-Driven Architecture
- Message queues (Redis Streams, Apache Kafka)
- Event sourcing implementation
- CQRS with separate read/write models

### 3. Scalability Patterns
- Horizontal scaling strategies
- Database sharding
- Caching layers (Redis Cluster)

---

**Architecture Goals**: Maintainability, Testability, Scalability, Performance
