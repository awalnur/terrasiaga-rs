# 📚 Terra Siaga API Documentation

Base URL: `http://localhost:8080` (Development) | `https://api.terrasiaga.id` (Production)

## 🔐 Authentication

Terra Siaga menggunakan JWT (JSON Web Token) untuk autentikasi. Token harus disertakan dalam header untuk setiap request yang memerlukan autentikasi.

```http
Authorization: Bearer <your-jwt-token>
```

## 📋 Response Format

Semua response menggunakan format JSON dengan struktur standar:

### Success Response
```json
{
  "success": true,
  "data": { ... },
  "message": "Operation successful",
  "timestamp": "2025-07-23T10:00:00Z"
}
```

### Error Response
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": { ... }
  },
  "timestamp": "2025-07-23T10:00:00Z"
}
```

## 📖 API Endpoints

### 🔑 Authentication

#### POST /api/v1/auth/login
Login dengan email dan password.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user_id": "uuid",
    "access_token": "jwt-token",
    "refresh_token": "refresh-token",
    "expires_in": 86400
  }
}
```

#### POST /api/v1/auth/register
Registrasi user baru.

**Request Body:**
```json
{
  "email": "newuser@example.com",
  "username": "newuser",
  "password": "securepassword",
  "full_name": "New User",
  "phone": "+6281234567890"
}
```

---

### 🚨 Disaster Management

#### GET /api/v1/disasters
Mendapatkan daftar laporan bencana dengan filter.

**Query Parameters:**
- `status` (optional): reported, verified, responding, resolved
- `severity` (optional): low, medium, high, critical
- `disaster_type` (optional): earthquake, flood, fire, landslide
- `lat`, `lng`, `radius` (optional): Filter berdasarkan lokasi
- `page`, `limit` (optional): Pagination

**Response:**
```json
{
  "success": true,
  "data": {
    "disasters": [
      {
        "id": "uuid",
        "title": "Banjir di Jakarta Barat",
        "description": "Banjir setinggi 1.5 meter...",
        "disaster_type": "flood",
        "severity": "high",
        "status": "verified",
        "location": {
          "latitude": -6.2088,
          "longitude": 106.8456,
          "address": "Jakarta Barat"
        },
        "reporter": {
          "id": "uuid",
          "name": "John Doe"
        },
        "created_at": "2025-07-23T08:00:00Z",
        "updated_at": "2025-07-23T09:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 100,
      "total_pages": 5
    }
  }
}
```

#### POST /api/v1/disasters
Membuat laporan bencana baru.

**Request Body:**
```json
{
  "title": "Gempa Bumi 5.2 SR",
  "description": "Gempa bumi dengan kekuatan 5.2 SR...",
  "disaster_type": "earthquake",
  "severity": "medium",
  "latitude": -7.2575,
  "longitude": 112.7521,
  "address": "Surabaya, Jawa Timur",
  "affected_population": 1000,
  "contact_info": "+6281234567890"
}
```

---

### 🚑 Emergency Response

#### POST /api/v1/emergency/response
Memulai operasi tanggap darurat.

**Request Body:**
```json
{
  "disaster_id": "uuid",
  "response_type": "rescue",
  "priority": "high",
  "estimated_duration": 120,
  "required_resources": ["ambulance", "rescue_team"],
  "team_size": 8
}
```

#### GET /api/v1/emergency/teams/available
Mendapatkan daftar tim respons yang tersedia.

**Response:**
```json
{
  "success": true,
  "data": {
    "teams": [
      {
        "id": "team-001",
        "name": "Alpha Rescue Team",
        "type": "rescue",
        "status": "available",
        "location": {
          "latitude": -6.2088,
          "longitude": 106.8456
        },
        "members": 8,
        "specialization": ["water_rescue", "medical_aid"]
      }
    ]
  }
}
```

---

### 📍 Location Services

#### GET /api/v1/locations/geocode
Konversi alamat menjadi koordinat.

**Query Parameters:**
- `address`: Alamat yang akan dikonversi

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "Monas, Jakarta",
    "coordinates": {
      "latitude": -6.1751,
      "longitude": 106.8270
    },
    "formatted_address": "Monumen Nasional, Jakarta Pusat, DKI Jakarta"
  }
}
```

#### GET /api/v1/locations/shelters
Mendapatkan daftar shelter terdekat.

**Query Parameters:**
- `lat`, `lng`: Koordinat referensi
- `radius`: Radius pencarian (km)

---

### 📢 Notifications

#### POST /api/v1/notifications/broadcast/emergency
Broadcast peringatan darurat.

**Request Body:**
```json
{
  "title": "SIAGA DARURAT",
  "message": "Tsunami Warning - Segera evakuasi ke dataran tinggi",
  "priority": "critical",
  "target_audience": "location_based",
  "location_filter": {
    "latitude": -8.3405,
    "longitude": 115.0920,
    "radius": 50
  },
  "channels": ["push", "sms", "whatsapp"]
}
```

---

### 📊 Analytics

#### GET /api/v1/analytics/dashboard
Mendapatkan data dashboard analytics.

**Response:**
```json
{
  "success": true,
  "data": {
    "summary": {
      "total_disasters": 150,
      "active_disasters": 8,
      "resolved_disasters": 142,
      "response_teams": 25,
      "avg_response_time": "12 minutes"
    },
    "trends": {
      "disasters_this_month": 12,
      "trend_percentage": 8.5
    }
  }
}
```

## 📝 Status Codes

- `200` - Success
- `201` - Created
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `422` - Validation Error
- `500` - Internal Server Error

## 🔍 Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Input validation failed |
| `UNAUTHORIZED` | Authentication required |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `DUPLICATE_ENTRY` | Resource already exists |
| `EXTERNAL_SERVICE_ERROR` | Third-party service error |

## 📋 Rate Limiting

- **Standard endpoints**: 100 requests per minute per IP
- **Authentication endpoints**: 10 requests per minute per IP
- **Emergency endpoints**: 1000 requests per minute per IP

## 🧪 Testing

### Postman Collection
Import collection dari: `docs/api/postman_collection.json`

### cURL Examples

```bash
# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# Get disasters
curl -X GET http://localhost:8080/api/v1/disasters \
  -H "Authorization: Bearer <token>"

# Create disaster report
curl -X POST http://localhost:8080/api/v1/disasters \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Disaster","disaster_type":"flood",...}'
```

## 📞 Support

Untuk pertanyaan terkait API, hubungi:
- Email: api-support@terrasiaga.id
- Slack: #api-support
- GitHub Issues: [API Issues](https://github.com/your-org/terra-siaga/labels/api)
