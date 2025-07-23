# 📝 CONTRIBUTING Guide - Terra Siaga

## 🎯 Welcome Contributors!

Terima kasih atas minat Anda untuk berkontribusi pada Terra Siaga! Panduan ini akan membantu Anda memahami cara berkontribusi secara efektif.

## 🤝 Code of Conduct

### Our Pledge
Kami berkomitmen untuk menciptakan lingkungan yang terbuka dan ramah bagi semua kontributor, tanpa memandang:
- Pengalaman dan tingkat keahlian
- Latar belakang etnis, ras, atau agama
- Identitas gender dan orientasi seksual
- Kemampuan fisik atau mental

### Expected Behavior
- Gunakan bahasa yang inklusif dan ramah
- Hormati pandangan dan pengalaman yang berbeda
- Terima kritik konstruktif dengan terbuka
- Fokus pada yang terbaik untuk komunitas
- Tunjukkan empati terhadap anggota komunitas lain

## 🚀 Getting Started

### 1. Fork Repository
```bash
# Fork di GitHub, kemudian clone
git clone https://github.com/YOUR_USERNAME/terra-siaga.git
cd terra-siaga

# Add upstream remote
git remote add upstream https://github.com/original-org/terra-siaga.git
```

### 2. Setup Development Environment
```bash
# Install dependencies
cargo build

# Setup database
diesel setup
diesel migration run

# Run tests
cargo test

# Start development server
cargo run
```

### 3. Keep Fork Updated
```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

## 🛠️ Development Workflow

### Branch Naming Convention
```
feature/disaster-analytics     # New features
bugfix/user-login-error       # Bug fixes
hotfix/security-vulnerability # Critical fixes
docs/api-documentation        # Documentation
refactor/clean-architecture   # Code refactoring
```

### Commit Message Format
```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(auth): add two-factor authentication

Implement TOTP-based 2FA for enhanced security.
- Add TOTP generation and verification
- Update user model to include 2FA settings
- Add API endpoints for 2FA management

Closes #123
```

### Pull Request Process

1. **Create Feature Branch**
```bash
git checkout -b feature/your-feature-name
```

2. **Make Changes**
- Write tests first (TDD approach)
- Implement feature
- Ensure all tests pass
- Update documentation

3. **Pre-submission Checklist**
```bash
# Format code
cargo fmt

# Check for linting issues
cargo clippy -- -D warnings

# Run all tests
cargo test

# Check test coverage
cargo tarpaulin

# Security audit
cargo audit
```

4. **Submit Pull Request**
- Fill out PR template completely
- Link related issues
- Add screenshots/demos if applicable
- Request review from maintainers

## 🧪 Testing Guidelines

### Writing Tests

1. **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange
        let repo = MockUserRepository::new();
        let use_case = CreateUserUseCase::new(Arc::new(repo));
        
        // Act
        let result = use_case.execute(valid_request()).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

2. **Integration Tests**
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_disaster_creation_workflow() {
    let container = test_container().await;
    let client = test_client(&container).await;
    
    // Test complete workflow
    let response = client
        .post("/api/v1/disasters")
        .json(&disaster_request())
        .send()
        .await;
        
    assert_eq!(response.status(), 201);
}
```

### Test Coverage Requirements
- **Minimum 80% code coverage**
- **100% coverage for business logic**
- **Integration tests for critical workflows**

## 📝 Documentation Standards

### Code Documentation
```rust
/// Creates a new disaster report with location validation
/// 
/// # Arguments
/// * `request` - The disaster creation request containing title, location, etc.
/// 
/// # Returns
/// * `Ok(Disaster)` - Successfully created disaster
/// * `Err(AppError)` - Validation or persistence error
/// 
/// # Examples
/// ```
/// let request = CreateDisasterRequest { ... };
/// let disaster = use_case.execute(request).await?;
/// ```
pub async fn create_disaster(&self, request: CreateDisasterRequest) -> AppResult<Disaster> {
    // Implementation
}
```

### API Documentation
- Update OpenAPI/Swagger specs
- Include request/response examples
- Document error codes and scenarios

### README Updates
- Update feature list for new capabilities
- Add configuration examples
- Include migration guides for breaking changes

## 🏗️ Architecture Guidelines

### Clean Architecture Principles
1. **Domain Independence**: Core business logic must not depend on external frameworks
2. **Dependency Inversion**: High-level modules should not depend on low-level modules
3. **Single Responsibility**: Each module should have one reason to change

### Code Organization
```
src/
├── domain/              # Business entities and rules
├── application/         # Use cases and app services  
├── infrastructure/      # External integrations
└── presentation/        # HTTP/API layer
```

### Adding New Features

1. **Start with Domain**
```rust
// 1. Define entity
pub struct NewEntity {
    // Business data and rules
}

// 2. Define repository interface
pub trait NewEntityRepository {
    async fn save(&self, entity: &NewEntity) -> AppResult<NewEntity>;
}
```

2. **Add Use Cases**
```rust
pub struct CreateNewEntityUseCase {
    repository: Arc<dyn NewEntityRepository>,
}
```

3. **Implement Infrastructure**
```rust
pub struct PostgresNewEntityRepository {
    // Database implementation
}
```

4. **Add API Endpoints**
```rust
pub async fn create_new_entity(req: web::Json<CreateRequest>) -> HttpResponse {
    // HTTP handler
}
```

## 🐛 Bug Reports

### Before Reporting
1. Search existing issues
2. Test with latest version
3. Reproduce in clean environment

### Bug Report Template
```markdown
**Bug Description**
Clear description of the bug

**Steps to Reproduce**
1. Go to '...'
2. Click on '....'
3. See error

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: [e.g. macOS 12.0]
- Rust version: [e.g. 1.70.0]
- Terra Siaga version: [e.g. 1.2.0]

**Additional Context**
- Error logs
- Screenshots
- Related issues
```

## 💡 Feature Requests

### Feature Request Template
```markdown
**Feature Description**
Clear description of the proposed feature

**Problem Statement**
What problem does this solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
Other approaches you've considered

**Additional Context**
- User stories
- Mockups/wireframes
- Related features
```

### Evaluation Criteria
Features are evaluated based on:
- **Impact**: How many users benefit?
- **Effort**: Development complexity and time
- **Alignment**: Fits product vision and roadmap
- **Maintenance**: Long-term support requirements

## 🎨 UI/UX Contributions

### Design Guidelines
- Follow accessibility standards (WCAG 2.1)
- Ensure mobile responsiveness
- Use consistent color scheme and typography
- Provide clear error messages and feedback

### Design Assets
- Provide Figma/Sketch files
- Include different screen sizes
- Consider dark/light themes
- Document design decisions

## 📊 Performance Contributions

### Performance Standards
- API response time < 200ms (95th percentile)
- Database queries < 100ms
- Memory usage < 512MB per instance
- CPU usage < 50% under normal load

### Benchmarking
```bash
# Run performance tests
cargo bench

# Profile with flamegraph
cargo flamegraph --bin terra-siaga

# Load testing
wrk -t12 -c400 -d30s http://localhost:8080/api/v1/health
```

## 🔐 Security Contributions

### Security Guidelines
- Follow OWASP Top 10
- Use parameterized queries
- Validate all inputs
- Implement proper authentication/authorization
- Log security events

### Reporting Security Issues
**DO NOT** open public issues for security vulnerabilities.

Email: security@terrasiaga.id
- Include detailed description
- Provide reproduction steps
- Suggest potential fixes

## 🏆 Recognition

### Contributor Types
- **Code Contributors**: Bug fixes, features, refactoring
- **Documentation Contributors**: Docs, tutorials, examples
- **Community Contributors**: Issue triage, user support
- **Security Contributors**: Vulnerability reports, security improvements

### Hall of Fame
Contributors will be recognized in:
- README.md contributors section
- Release notes
- Community newsletter
- Annual contributor awards

## 📞 Getting Help

### Communication Channels
- **GitHub Discussions**: General questions and ideas
- **Slack**: #terra-siaga-dev (invite required)
- **Email**: contributors@terrasiaga.id

### Weekly Office Hours
- **Time**: Every Wednesday 10:00 AM UTC
- **Platform**: Google Meet
- **Purpose**: Q&A, design discussions, community sync

### Mentorship Program
New contributors can request a mentor for:
- Codebase orientation
- First contribution guidance
- Architecture discussions
- Career development advice

## 📋 Review Process

### Review Criteria
- **Functionality**: Does it work as intended?
- **Code Quality**: Clean, readable, maintainable
- **Tests**: Adequate coverage and quality
- **Documentation**: Updated and clear
- **Performance**: No significant degradation
- **Security**: No vulnerabilities introduced

### Review Timeline
- **First Response**: Within 2 business days
- **Full Review**: Within 1 week
- **Merge Decision**: Within 2 weeks

### Reviewer Assignment
- Automatic assignment based on expertise
- Multiple reviewers for significant changes
- Maintainer approval required for merge

---

**Thank you for contributing to Terra Siaga! Together we're building a safer Indonesia.** 🇮🇩
