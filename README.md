# 🌍 Terra Siaga - Emergency Response Management System

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/postgres-%23316192.svg?style=for-the-badge&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Redis](https://img.shields.io/badge/redis-%23DD0031.svg?style=for-the-badge&logo=redis&logoColor=white)](https://redis.io/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)

Terra Siaga adalah sistem manajemen tanggap darurat yang dibangun dengan Rust, menggunakan arsitektur Clean Architecture untuk mengelola pelaporan bencana, koordinasi tim respons, dan komunikasi multi-channel dalam situasi darurat.

## 🎯 Fitur Utama

### 🚨 Manajemen Bencana
- **Pelaporan Real-time**: Pelaporan bencana dengan geolokasi otomatis
- **Tracking Status**: Monitoring status dari laporan hingga penyelesaian
- **Klasifikasi Bencana**: Kategorisasi berdasarkan jenis dan tingkat keparahan
- **Timeline Kejadian**: Histori lengkap timeline respons bencana

### 👥 Koordinasi Tim Respons
- **Dispatch Tim**: Assignment otomatis tim respons berdasarkan lokasi dan keahlian
- **Live Tracking**: Pelacakan lokasi tim respons secara real-time
- **Resource Management**: Manajemen sumber daya dan peralatan darurat
- **Communication Hub**: Komunikasi terpusat untuk koordinasi operasi

### 📍 Location Intelligence
- **Pemetaan Interaktif**: Visualisasi bencana dan tim respons di peta
- **Geocoding Services**: Konversi alamat ke koordinat dan sebaliknya
- **Radius Search**: Pencarian lokasi dalam radius tertentu
- **Evacuation Routes**: Perhitungan rute evakuasi optimal

### 📢 Multi-Channel Notifications
- **Push Notifications**: Notifikasi real-time via aplikasi
- **SMS Alert**: Peringatan darurat via SMS
- **Email Notifications**: Laporan detail via email
- **WhatsApp Integration**: Broadcast peringatan via WhatsApp

### 📊 Analytics & Reporting
- **Dashboard Analytics**: Visualisasi KPI dan metrik utama
- **Trend Analysis**: Analisis pola dan tren bencana
- **Performance Metrics**: Monitoring waktu respons dan efektivitas
- **Custom Reports**: Laporan yang dapat disesuaikan dalam berbagai format

## 🏗️ Arsitektur

Terra Siaga menggunakan **Clean Architecture** dengan pemisahan yang jelas antara layer:

```
src/
├── domain/          # Business logic dan entities
├── application/     # Use cases dan services
├── infrastructure/  # External concerns (database, APIs)
└── presentation/    # HTTP handlers dan controllers
```

### 🔧 Tech Stack

- **Backend**: Rust with Actix-web
- **Database**: PostgreSQL with Diesel ORM
- **Cache**: Redis
- **Authentication**: JWT tokens
- **External APIs**: WhatsApp, Email SMTP, Weather, Geolocation
- **Monitoring**: Built-in health checks dan metrics

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ 
- PostgreSQL 14+
- Redis 6+
- Docker & Docker Compose (optional)

### 1. Clone Repository

```bash
git clone https://github.com/your-org/terra-siaga.git
cd terra-siaga
```

### 2. Setup Environment

```bash
cp .env.example .env
# Edit .env dengan konfigurasi yang sesuai
```

### 3. Setup Database

```bash
# Install Diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Run migrations
diesel setup
diesel migration run
```

### 4. Run Development Server

```bash
cargo run
```

Server akan berjalan di `http://localhost:8080`

## 🔧 Konfigurasi

### Environment Variables

Salin file `.env.example` ke `.env` dan sesuaikan konfigurasi:

```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
ENVIRONMENT=development

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/terrasiaga
DATABASE_MAX_CONNECTIONS=10

# Authentication
JWT_SECRET=your-super-secret-jwt-key-here
JWT_EXPIRY=86400

# External Services
WHATSAPP_API_KEY=your_whatsapp_api_key
EMAIL_SMTP_HOST=smtp.gmail.com
EMAIL_SMTP_PORT=587
GEO_API_KEY=your_google_maps_api_key
WEATHER_API_KEY=your_weather_api_key
```

### Database Migration

```bash
# Create new migration
diesel migration generate migration_name

# Run migrations
diesel migration run

# Rollback migration
diesel migration revert
```

## 📖 API Documentation

### Base URL
```
Production: https://api.terrasiaga.id
Development: http://localhost:8080
```

### Authentication
Semua endpoint yang memerlukan autentikasi menggunakan JWT token di header:

```http
Authorization: Bearer <your-jwt-token>
```

### Core Endpoints

#### Authentication
```http
POST /api/v1/auth/login
POST /api/v1/auth/register
POST /api/v1/auth/refresh
GET  /api/v1/auth/me
```

#### Disaster Management
```http
GET    /api/v1/disasters
POST   /api/v1/disasters
GET    /api/v1/disasters/{id}
PUT    /api/v1/disasters/{id}
GET    /api/v1/disasters/nearby?lat={lat}&lng={lng}&radius={km}
POST   /api/v1/disasters/{id}/assign
```

#### Emergency Response
```http
POST /api/v1/emergency/response
GET  /api/v1/emergency/active
POST /api/v1/emergency/{id}/dispatch
GET  /api/v1/emergency/teams/available
```

#### Notifications
```http
GET  /api/v1/notifications
POST /api/v1/notifications
POST /api/v1/notifications/broadcast/emergency
GET  /api/v1/notifications/unread-count
```

Lihat [API Documentation](docs/api/README.md) untuk detail lengkap.

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with coverage
cargo test --coverage

# Integration tests
cargo test --test integration_tests
```

## 🚢 Deployment

### Docker Deployment

```bash
# Build image
docker build -t terra-siaga .

# Run with docker-compose
docker-compose up -d
```

### Manual Deployment

```bash
# Build release
cargo build --release

# Run binary
./target/release/terra-siaga
```

## 📁 Project Structure

```
terra-siaga/
├── docs/                    # Documentation
├── migrations/              # Database migrations
├── src/
│   ├── application/         # Use cases & application services
│   ├── domain/              # Business logic & entities
│   ├── infrastructure/      # External integrations
│   ├── presentation/        # HTTP API layer
│   ├── shared/             # Shared utilities
│   └── middleware/         # HTTP middleware
├── tests/                  # Test files
├── .env.example           # Environment template
├── Cargo.toml            # Rust dependencies
├── diesel.toml           # Database configuration
└── README.md            # This file
```

## 🤝 Contributing

1. Fork repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

Lihat [CONTRIBUTING.md](docs/CONTRIBUTING.md) untuk panduan lengkap.

## 📋 Development Roadmap

### Phase 1 - Core Features ✅
- [x] User authentication & authorization
- [x] Disaster reporting system
- [x] Basic notification system
- [x] Location management

### Phase 2 - Advanced Features 🚧
- [ ] Real-time team tracking
- [ ] Advanced analytics dashboard
- [ ] Mobile app integration
- [ ] ML-based disaster prediction

### Phase 3 - Enterprise Features 📝
- [ ] Multi-tenant support
- [ ] Advanced reporting
- [ ] Third-party integrations
- [ ] High availability setup

## 🐛 Known Issues

- WhatsApp Business API requires approval untuk production use
- Geolocation services membutuhkan API key yang valid
- Real-time features memerlukan WebSocket implementation

## 📄 License

Project ini dilisensikan under [MIT License](LICENSE).

## 👥 Team

- **Tech Lead**: [Your Name](mailto:your.email@domain.com)
- **Backend Developer**: [Developer Name](mailto:dev@domain.com)
- **DevOps Engineer**: [DevOps Name](mailto:devops@domain.com)

## 📞 Support

- **Documentation**: [docs.terrasiaga.id](https://docs.terrasiaga.id)
- **Issue Tracker**: [GitHub Issues](https://github.com/your-org/terra-siaga/issues)
- **Email**: support@terrasiaga.id
- **Slack**: #terra-siaga

## 🙏 Acknowledgments

- [Actix Web](https://actix.rs/) - Web framework
- [Diesel](https://diesel.rs/) - ORM
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization
- OpenStreetMap untuk data geografis

---

**Terra Siaga** - *Siaga untuk Indonesia yang Lebih Aman* 🇮🇩
