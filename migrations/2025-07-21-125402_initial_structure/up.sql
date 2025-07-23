-- This migration creates the initial structure for the disaster reporting system.

-- Ensure extensions are installed
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

CREATE TABLE roles
(
    id   SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE, -- admin, relawan, pelapor, dll
    description TEXT,                 -- deskripsi peran
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE users
(
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username          TEXT    NOT NULL UNIQUE,
    password_hash     TEXT    NOT NULL,
    email             TEXT    NOT NULL UNIQUE,
    phone             TEXT UNIQUE,                                      -- opsional
    role_id           INTEGER REFERENCES roles (id) ON DELETE SET NULL, -- relasi ke roles

    -- Informasi tambahan
    full_name         TEXT,
    address           TEXT,
    profile_photo_url TEXT,
    bio               TEXT,                                             -- biografi singkat
    date_of_birth     DATE,                                             -- tanggal lahir
    gender            VARCHAR(20),                                      -- jenis kelamin

    is_verified       BOOLEAN          DEFAULT FALSE,                   -- apakah email/telepon sudah diverifikasi
    is_active         BOOLEAN          DEFAULT TRUE,                    -- apakah akun aktif
    last_login        TIMESTAMP,                                        -- waktu login terakhir
    created_at        TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP        DEFAULT CURRENT_TIMESTAMP
);

-- User Roles (many-to-many relationship)
CREATE TABLE user_roles
(
    user_id UUID REFERENCES users (id) ON DELETE CASCADE,
    role_id INTEGER REFERENCES roles (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Verification Codes for Email/Phone
CREATE TABLE verification_codes
(
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID REFERENCES users (id) ON DELETE CASCADE,
    code       TEXT NOT NULL,                           -- kode verifikasi
    type       VARCHAR(20) NOT NULL,                    -- email, phone
    expires_at TIMESTAMP NOT NULL,                      -- waktu kadaluarsa
    is_used    BOOLEAN DEFAULT FALSE,                   -- apakah sudah digunakan
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Auth Sessions
CREATE TABLE auth_sessions
(
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID REFERENCES users (id) ON DELETE CASCADE,
    token      TEXT      NOT NULL UNIQUE, -- token otentikasi
    user_agent TEXT,                      -- informasi browser/device
    ip_address TEXT,                      -- alamat IP
    expires_at TIMESTAMP NOT NULL,        -- waktu kadaluarsa token
    created_at TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP        DEFAULT CURRENT_TIMESTAMP
);

-- Refresh Tokens Table
CREATE TABLE refresh_tokens
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID REFERENCES users (id) ON DELETE CASCADE,
    token       TEXT      NOT NULL UNIQUE,     -- token refresh
    is_valid    BOOLEAN          DEFAULT TRUE, -- apakah token masih valid
    revoked_at  TIMESTAMP,                     -- waktu token di-revoke
    expires_at  TIMESTAMP NOT NULL,            -- waktu kadaluarsa token
    created_at  TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP        DEFAULT CURRENT_TIMESTAMP
);

-- Disaster Type Table
CREATE TABLE disaster_types
(
    id          SERIAL PRIMARY KEY,
    name        TEXT    NOT NULL UNIQUE,    -- nama jenis bencana (misal: "banjir", "gempa")
    description TEXT,                       -- deskripsi jenis bencana
    severity    INTEGER NOT NULL DEFAULT 1, -- tingkat keparahan (1-5)
    icon_url    TEXT,                       -- URL ikon untuk jenis bencana
    color_code  VARCHAR(7),                 -- kode warna untuk visualisasi (hex: #RRGGBB)
    created_at  TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP        DEFAULT CURRENT_TIMESTAMP
);

-- Location Table
CREATE TABLE locations
(
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       TEXT NOT NULL,                -- nama lokasi
    region     TEXT NOT NULL,                -- nama wilayah (kecamatan, desa, dll)
    province   TEXT,                         -- provinsi
    city       TEXT,                         -- kota/kabupaten
    postal_code TEXT,                        -- kode pos
    geometry   GEOGRAPHY(Point, 4326),       -- posisi lat/lon
    address    TEXT,                         -- alamat lengkap
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Reports Table
CREATE TABLE reports
(
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reporter_id       UUID REFERENCES users (id),                      -- NULL jika anonim
    anonymous_name    TEXT,                                            -- nama untuk anonim
    anonymous_phone   TEXT,                                            -- opsional
    is_anonymous      BOOLEAN   DEFAULT FALSE,

    -- Informasi bencana
    disaster_type_id  INTEGER REFERENCES disaster_types (id),          -- referensi ke jenis bencana
    title             TEXT NOT NULL,                                   -- judul laporan
    description       TEXT,                                            -- deskripsi bencana
    location_id       UUID REFERENCES locations (id),                  -- lokasi bencana
    address           TEXT,                                            -- alamat detail
    impact_radius     FLOAT,                                           -- perkiraan radius dampak (meter)
    estimated_severity INTEGER DEFAULT 1,                              -- perkiraan tingkat keparahan (1-5)
    
    -- Informasi korban
    casualties        INTEGER,                                         -- jumlah korban jiwa
    injuries          INTEGER,                                         -- jumlah korban luka
    missing           INTEGER,                                         -- jumlah orang hilang
    affected_people   INTEGER,                                         -- jumlah orang terdampak

    -- Validasi & status
    status            TEXT      DEFAULT 'pending',                     -- pending, valid, invalid, resolved
    credibility_score FLOAT     DEFAULT 0.0,                           -- skor kredibilitas (0.0-1.0)
    validated_by      UUID REFERENCES users (id),                      -- relawan/admin yang validasi
    validation_notes  TEXT,                                            -- catatan validasi
    validation_date   TIMESTAMP,                                       -- tanggal validasi

    created_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Report History Table (for tracking changes)
CREATE TABLE report_history
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id   UUID REFERENCES reports (id) ON DELETE CASCADE,
    changed_by  UUID REFERENCES users (id),
    status_from TEXT,
    status_to   TEXT,
    notes       TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Disaster Report Media Table
CREATE TABLE report_media
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id   UUID REFERENCES reports (id) ON DELETE CASCADE,
    media_type  TEXT NOT NULL,                                -- image, video, document, dll
    media_url   TEXT NOT NULL,                                -- URL media
    caption     TEXT,                                         -- keterangan media
    is_primary  BOOLEAN DEFAULT FALSE,                        -- apakah media utama
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Comments on Reports
CREATE TABLE report_comments
(
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id  UUID REFERENCES reports (id) ON DELETE CASCADE,
    user_id    UUID REFERENCES users (id) ON DELETE SET NULL,
    content    TEXT NOT NULL,
    parent_id  UUID REFERENCES report_comments (id) ON DELETE CASCADE, -- untuk komentar bersarang
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Notification Table
CREATE TABLE notifications
(
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID REFERENCES users (id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    message    TEXT NOT NULL,
    channel    TEXT NOT NULL,                                -- email, sms, app
    status     VARCHAR(20) DEFAULT 'queued',                 -- queued, sent, failed
    is_read    BOOLEAN DEFAULT FALSE,                        -- apakah sudah dibaca
    send_at    TIMESTAMP,                                    -- waktu pengiriman
    read_at    TIMESTAMP,                                    -- waktu dibaca
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Volunteers Table
CREATE TABLE volunteers
(
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id           UUID REFERENCES users (id) ON DELETE CASCADE,
    skills            TEXT[],                                         -- array keterampilan
    certifications    TEXT[],                                         -- array sertifikasi
    availability      BOOLEAN DEFAULT TRUE,                           -- ketersediaan
    availability_notes TEXT,                                          -- catatan ketersediaan
    experience_years  INTEGER,                                        -- tahun pengalaman
    specialization    TEXT,                                           -- spesialisasi
    current_location_id UUID REFERENCES locations (id),               -- lokasi saat ini
    created_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Volunteer Tracking
CREATE TABLE volunteer_tracking
(
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    volunteer_id UUID REFERENCES volunteers (id) ON DELETE CASCADE,
    report_id    UUID REFERENCES reports (id) ON DELETE CASCADE,
    status       TEXT DEFAULT 'assigned',                          -- assigned, en_route, on_site, completed
    notes        TEXT,                                             -- catatan
    assigned_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    arrived_at   TIMESTAMP,                                        -- waktu tiba di lokasi
    completed_at TIMESTAMP,                                        -- waktu selesai
    created_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Volunteer Location History
CREATE TABLE volunteer_locations
(
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    volunteer_id UUID REFERENCES volunteers (id) ON DELETE CASCADE,
    geometry     GEOGRAPHY(Point, 4326),                          -- posisi lat/lon
    recorded_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Organizations Table (for relief organizations, government agencies, etc.)
CREATE TABLE organizations
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    description TEXT,
    logo_url    TEXT,
    website     TEXT,
    phone       TEXT,
    email       TEXT,
    address     TEXT,
    location_id UUID REFERENCES locations (id),
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Organization Members
CREATE TABLE organization_members
(
    organization_id UUID REFERENCES organizations (id) ON DELETE CASCADE,
    user_id         UUID REFERENCES users (id) ON DELETE CASCADE,
    role            TEXT NOT NULL,                                      -- admin, member, etc.
    PRIMARY KEY (organization_id, user_id),
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Disaster Table (verified disaster events)
CREATE TABLE disasters
(
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    disaster_type_id  INTEGER REFERENCES disaster_types (id),
    name              TEXT NOT NULL,                                    -- nama bencana (e.g., "Banjir Jakarta 2023")
    description       TEXT,
    severity          INTEGER CHECK (severity BETWEEN 1 AND 5),
    status            TEXT DEFAULT 'active',                            -- active, contained, resolved
    start_time        TIMESTAMP,                                        -- waktu mulai bencana
    end_time          TIMESTAMP,                                        -- waktu berakhir bencana
    primary_location_id UUID REFERENCES locations (id),                 -- lokasi utama
    created_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Link Reports to Disasters
CREATE TABLE disaster_reports
(
    disaster_id UUID REFERENCES disasters (id) ON DELETE CASCADE,
    report_id   UUID REFERENCES reports (id) ON DELETE CASCADE,
    PRIMARY KEY (disaster_id, report_id),
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Disaster Analytics Table
CREATE TABLE disaster_analytics
(
    id                       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    disaster_id              UUID REFERENCES disasters (id) ON DELETE CASCADE,
    affected_population      INTEGER,
    economic_loss_estimation BIGINT,
    infrastructure_damage    TEXT,                                      -- deskripsi kerusakan infrastruktur
    updated_at               TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Disaster Movement Tracking
CREATE TABLE disaster_movements
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    disaster_id UUID REFERENCES disasters (id) ON DELETE CASCADE,
    geometry    GEOMETRY(Point, 4326),
    speed       FLOAT,                                                  -- kecepatan pergerakan (km/h)
    direction   FLOAT,                                                  -- arah pergerakan (derajat)
    description TEXT,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Disaster Zone Mapping
CREATE TABLE disaster_zones
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    disaster_id UUID REFERENCES disasters (id) ON DELETE CASCADE,
    area        GEOMETRY(Polygon, 4326),
    zone_type   VARCHAR(50),                                            -- affected, danger, safe
    description TEXT,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Evacuation Centers
CREATE TABLE evacuation_centers
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    description TEXT,
    capacity    INTEGER,                                                -- kapasitas orang
    current_occupancy INTEGER DEFAULT 0,                                -- jumlah orang saat ini
    location_id UUID REFERENCES locations (id),
    status      TEXT DEFAULT 'operational',                             -- operational, full, closed
    contact_person TEXT,
    contact_phone  TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Evacuation Center Facilities
CREATE TABLE evacuation_center_facilities
(
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    evacuation_center_id UUID REFERENCES evacuation_centers (id) ON DELETE CASCADE,
    facility_name       TEXT NOT NULL,                                  -- nama fasilitas
    quantity            INTEGER,                                        -- jumlah
    status              TEXT DEFAULT 'available',                       -- available, limited, unavailable
    created_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Emergency Resources
CREATE TABLE emergency_resources
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,                                          -- food, medical, shelter, etc.
    quantity    INTEGER NOT NULL,
    unit        TEXT NOT NULL,                                          -- kg, box, piece, etc.
    location_id UUID REFERENCES locations (id),
    organization_id UUID REFERENCES organizations (id),
    expiry_date DATE,                                                   -- tanggal kadaluarsa (untuk makanan/obat)
    status      TEXT DEFAULT 'available',                               -- available, reserved, depleted
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Resource Allocation
CREATE TABLE resource_allocations
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_id UUID REFERENCES emergency_resources (id),
    disaster_id UUID REFERENCES disasters (id),
    quantity    INTEGER NOT NULL,
    allocated_by UUID REFERENCES users (id),
    status      TEXT DEFAULT 'allocated',                               -- allocated, in_transit, delivered
    notes       TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Weather Data
CREATE TABLE weather_data
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    location_id UUID REFERENCES locations (id),
    temperature FLOAT,                                                  -- suhu (Celsius)
    humidity    FLOAT,                                                  -- kelembaban (%)
    wind_speed  FLOAT,                                                  -- kecepatan angin (km/h)
    wind_direction FLOAT,                                               -- arah angin (derajat)
    precipitation FLOAT,                                                -- curah hujan (mm)
    pressure    FLOAT,                                                  -- tekanan udara (hPa)
    weather_condition TEXT,                                             -- kondisi cuaca (cerah, hujan, dll)
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for all tables with updated_at column
CREATE TRIGGER update_roles_timestamp
BEFORE UPDATE ON roles
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_users_timestamp
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_auth_sessions_timestamp
BEFORE UPDATE ON auth_sessions
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_refresh_tokens_timestamp
BEFORE UPDATE ON refresh_tokens
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_disaster_types_timestamp
BEFORE UPDATE ON disaster_types
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_locations_timestamp
BEFORE UPDATE ON locations
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_reports_timestamp
BEFORE UPDATE ON reports
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_report_media_timestamp
BEFORE UPDATE ON report_media
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_notifications_timestamp
BEFORE UPDATE ON notifications
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_volunteers_timestamp
BEFORE UPDATE ON volunteers
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_volunteer_tracking_timestamp
BEFORE UPDATE ON volunteer_tracking
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_organizations_timestamp
BEFORE UPDATE ON organizations
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_organization_members_timestamp
BEFORE UPDATE ON organization_members
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_disasters_timestamp
BEFORE UPDATE ON disasters
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_evacuation_centers_timestamp
BEFORE UPDATE ON evacuation_centers
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_evacuation_center_facilities_timestamp
BEFORE UPDATE ON evacuation_center_facilities
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_emergency_resources_timestamp
BEFORE UPDATE ON emergency_resources
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_resource_allocations_timestamp
BEFORE UPDATE ON resource_allocations
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_report_comments_timestamp
BEFORE UPDATE ON report_comments
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

-- Insert default roles
INSERT INTO roles (name, description) VALUES 
('admin', 'Administrator with full access'),
('volunteer', 'Volunteer who can validate reports and assist in disaster response'),
('reporter', 'Regular user who can submit disaster reports'),
('analyst', 'User who can analyze disaster data and create reports'),
('organization', 'Representative of an organization involved in disaster response');