-- This migration adds sample data for development and testing purposes
-- Created on: 2025-07-21

-- Note: This is sample data only and should not be used in production

-- Sample Users
-- Note: In a real application, password_hash would be properly hashed values
-- These are placeholder hashes for development only
INSERT INTO users (id, username, password_hash, email, phone, role_id, full_name, address, profile_photo_url, bio, date_of_birth, gender, is_verified, is_active) VALUES
('11111111-1111-1111-1111-111111111111', 'admin', '$2a$12$1234567890123456789012', 'admin@terrasiaga.org', '+6281234567890', 1, 'Admin Utama', 'Jl. Sudirman No. 123, Jakarta', 'https://terrasiaga.org/profiles/admin.jpg', 'Administrator sistem Terra Siaga', '1985-05-15', 'Laki-laki', TRUE, TRUE),
('22222222-2222-2222-2222-222222222222', 'budi_relawan', '$2a$12$2345678901234567890123', 'budi@relawan.org', '+6282345678901', 2, 'Budi Santoso', 'Jl. Thamrin No. 45, Jakarta', 'https://terrasiaga.org/profiles/budi.jpg', 'Relawan berpengalaman dalam penanganan bencana banjir', '1990-03-20', 'Laki-laki', TRUE, TRUE),
('33333333-3333-3333-3333-333333333333', 'siti_pelapor', '$2a$12$3456789012345678901234', 'siti@gmail.com', '+6283456789012', 3, 'Siti Rahayu', 'Jl. Gatot Subroto No. 67, Jakarta', 'https://terrasiaga.org/profiles/siti.jpg', 'Warga peduli lingkungan', '1995-07-10', 'Perempuan', TRUE, TRUE),
('44444444-4444-4444-4444-444444444444', 'dian_analis', '$2a$12$4567890123456789012345', 'dian@terrasiaga.org', '+6284567890123', 4, 'Dian Pratiwi', 'Jl. Kuningan No. 89, Jakarta', 'https://terrasiaga.org/profiles/dian.jpg', 'Analis data bencana dengan pengalaman 5 tahun', '1988-11-25', 'Perempuan', TRUE, TRUE),
('55555555-5555-5555-5555-555555555555', 'joko_bnpb', '$2a$12$5678901234567890123456', 'joko@bnpb.go.id', '+6285678901234', 5, 'Joko Widodo', 'Jl. Merdeka No. 10, Jakarta', 'https://terrasiaga.org/profiles/joko.jpg', 'Perwakilan BNPB untuk koordinasi penanganan bencana', '1980-09-05', 'Laki-laki', TRUE, TRUE),
('66666666-6666-6666-6666-666666666666', 'ani_relawan', '$2a$12$6789012345678901234567', 'ani@relawan.org', '+6286789012345', 2, 'Ani Kusuma', 'Jl. Diponegoro No. 32, Bandung', 'https://terrasiaga.org/profiles/ani.jpg', 'Relawan medis untuk penanganan korban bencana', '1992-04-18', 'Perempuan', TRUE, TRUE),
('77777777-7777-7777-7777-777777777777', 'rudi_pelapor', '$2a$12$7890123456789012345678', 'rudi@yahoo.com', '+6287890123456', 3, 'Rudi Hartono', 'Jl. Ahmad Yani No. 54, Surabaya', 'https://terrasiaga.org/profiles/rudi.jpg', 'Warga aktif dalam pelaporan bencana', '1993-08-30', 'Laki-laki', FALSE, TRUE),
('88888888-8888-8888-8888-888888888888', 'maya_pmi', '$2a$12$8901234567890123456789', 'maya@pmi.or.id', '+6288901234567', 5, 'Maya Indah', 'Jl. Pahlawan No. 76, Semarang', 'https://terrasiaga.org/profiles/maya.jpg', 'Koordinator PMI untuk bantuan bencana', '1987-12-12', 'Perempuan', TRUE, TRUE),
('99999999-9999-9999-9999-999999999999', 'agus_pemda', '$2a$12$9012345678901234567890', 'agus@jakarta.go.id', '+6289012345678', 5, 'Agus Hermawan', 'Jl. Cendrawasih No. 21, Jakarta', 'https://terrasiaga.org/profiles/agus.jpg', 'Perwakilan Pemda DKI Jakarta', '1983-06-22', 'Laki-laki', TRUE, TRUE),
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'dewi_analis', '$2a$12$0123456789012345678901', 'dewi@terrasiaga.org', '+6280123456789', 4, 'Dewi Safitri', 'Jl. Hayam Wuruk No. 43, Jakarta', 'https://terrasiaga.org/profiles/dewi.jpg', 'Spesialis analisis dampak ekonomi bencana', '1991-02-14', 'Perempuan', TRUE, TRUE);

-- Sample Disaster Types
INSERT INTO disaster_types (id, name, description, severity, icon_url, color_code) VALUES
(1, 'Banjir', 'Bencana akibat meluapnya air yang menggenangi daratan', 3, 'https://terrasiaga.org/icons/flood.png', '#3498DB'),
(2, 'Gempa Bumi', 'Getaran atau guncangan yang terjadi di permukaan bumi', 4, 'https://terrasiaga.org/icons/earthquake.png', '#E74C3C'),
(3, 'Kebakaran Hutan', 'Kebakaran yang terjadi di kawasan hutan', 3, 'https://terrasiaga.org/icons/forest_fire.png', '#E67E22'),
(4, 'Tanah Longsor', 'Perpindahan material pembentuk lereng berupa batuan, tanah, atau material campuran', 3, 'https://terrasiaga.org/icons/landslide.png', '#795548'),
(5, 'Tsunami', 'Gelombang air laut yang sangat besar yang disebabkan oleh gangguan di dasar laut', 5, 'https://terrasiaga.org/icons/tsunami.png', '#1A237E'),
(6, 'Letusan Gunung Berapi', 'Pelepasan magma, gas, dan abu dari gunung berapi', 4, 'https://terrasiaga.org/icons/volcano.png', '#F44336'),
(7, 'Angin Topan', 'Angin kencang yang berputar dan bergerak dengan kecepatan tinggi', 3, 'https://terrasiaga.org/icons/hurricane.png', '#607D8B'),
(8, 'Kekeringan', 'Kondisi kekurangan air dalam jangka waktu yang lama', 2, 'https://terrasiaga.org/icons/drought.png', '#FF9800'),
(9, 'Wabah Penyakit', 'Penyebaran penyakit menular dalam skala luas', 3, 'https://terrasiaga.org/icons/epidemic.png', '#9C27B0'),
(10, 'Kecelakaan Industri', 'Bencana yang terjadi di kawasan industri seperti kebocoran bahan kimia', 3, 'https://terrasiaga.org/icons/industrial.png', '#607D8B');

-- Sample Locations
-- Note: Using ST_GeographyFromText to create GEOGRAPHY(Point) from WKT
INSERT INTO locations (id, name, region, province, city, postal_code, geometry, address) VALUES
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Kelurahan Kampung Melayu', 'Jatinegara', 'DKI Jakarta', 'Jakarta Timur', '13320', ST_GeographyFromText('SRID=4326;POINT(106.8674 -6.2147)'), 'Jl. Kampung Melayu Besar, Jakarta Timur'),
('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Kelurahan Bukit Duri', 'Tebet', 'DKI Jakarta', 'Jakarta Selatan', '12840', ST_GeographyFromText('SRID=4326;POINT(106.8584 -6.2280)'), 'Jl. Bukit Duri Utara, Jakarta Selatan'),
('dddddddd-dddd-dddd-dddd-dddddddddddd', 'Kelurahan Bidara Cina', 'Jatinegara', 'DKI Jakarta', 'Jakarta Timur', '13330', ST_GeographyFromText('SRID=4326;POINT(106.8731 -6.2175)'), 'Jl. Bidara Cina, Jakarta Timur'),
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'Desa Cangkringan', 'Cangkringan', 'DI Yogyakarta', 'Sleman', '55583', ST_GeographyFromText('SRID=4326;POINT(110.4462 -7.6407)'), 'Cangkringan, Sleman, Yogyakarta'),
('ffffffff-ffff-ffff-ffff-ffffffffffff', 'Desa Umbulharjo', 'Cangkringan', 'DI Yogyakarta', 'Sleman', '55583', ST_GeographyFromText('SRID=4326;POINT(110.4664 -7.6306)'), 'Umbulharjo, Cangkringan, Sleman'),
('aaaaaaaa-1111-1111-1111-111111111111', 'Kelurahan Pangandaran', 'Pangandaran', 'Jawa Barat', 'Pangandaran', '46396', ST_GeographyFromText('SRID=4326;POINT(108.6571 -7.6709)'), 'Jl. Pantai Barat, Pangandaran'),
('eeeeeeee-9090-9090-9090-909090909090', 'Kelurahan Pemenang Barat', 'Pemenang', 'Nusa Tenggara Barat', 'Lombok Utara', '83352', ST_GeographyFromText('SRID=4326;POINT(116.0400 -8.4058)'), 'Pemenang Barat, Lombok Utara'),
('ffffffff-1111-2222-ffff-121212121212', 'Kelurahan Kayangan', 'Kayangan', 'Nusa Tenggara Barat', 'Lombok Utara', '83353', ST_GeographyFromText('SRID=4326;POINT(116.1511 -8.3135)'), 'Kayangan, Lombok Utara'),
('eaeaeaea-aaaa-cccc-dddd-c12232312312', 'Kelurahan Sembalun Bumbung', 'Sembalun', 'Nusa Tenggara Barat', 'Lombok Timur', '83656', ST_GeographyFromText('SRID=4326;POINT(116.5043 -8.3451)'), 'Sembalun Bumbung, Lombok Timur'),
('abababaa-abab-2123-2134-123123123123', 'Kelurahan Petobo', 'Palu Selatan', 'Sulawesi Tengah', 'Palu', '94231', ST_GeographyFromText('SRID=4326;POINT(119.8867 -0.9294)'), 'Petobo, Palu Selatan, Palu'),
('adeadead-adee-1234-aaa0-98888aaaa999', 'Kelurahan Balaroa', 'Palu Barat', 'Sulawesi Tengah', 'Palu', '94221', ST_GeographyFromText('SRID=4326;POINT(119.8558 -0.9017)'), 'Balaroa, Palu Barat, Palu'),
('111123ad-acda-cccc-ffff-facafafafafa', 'Kelurahan Talise', 'Mantikulore', 'Sulawesi Tengah', 'Palu', '94118', ST_GeographyFromText('SRID=4326;POINT(119.8783 -0.8561)'), 'Talise, Mantikulore, Palu'),
('aeaeaeae-cccc-dddd-eeee-bbbbbbaabbaa', 'Desa Sumber Wuluh', 'Sumber Wuluh', 'Jawa Timur', 'Lumajang', '67361', ST_GeographyFromText('SRID=4326;POINT(113.0576 -8.0883)'), 'Sumber Wuluh, Lumajang'),
('12344aaa-aaaa-2231-eeaa-cacacacacaca', 'Desa Curah Kobokan', 'Pronojiwo', 'Jawa Timur', 'Lumajang', '67374', ST_GeographyFromText('SRID=4326;POINT(113.0367 -8.1308)'), 'Curah Kobokan, Pronojiwo, Lumajang'),
('ccddccaa-1233-ccac-2233-8948744747cc', 'Kelurahan Cigobang', 'Pasaleman', 'Jawa Barat', 'Cirebon', '45187', ST_GeographyFromText('SRID=4326;POINT(108.5500 -6.8431)'), 'Cigobang, Pasaleman, Cirebon');

-- Sample Organizations
INSERT INTO organizations (id, name, description, logo_url, website, phone, email, address, location_id) VALUES
('f47ac10b-58cc-4372-a567-0e02b2c3d4e5', 'BNPB', 'Badan Nasional Penanggulangan Bencana', 'https://terrasiaga.org/logos/bnpb.png', 'https://bnpb.go.id', '+62215703957', 'contact@bnpb.go.id', 'Jl. Pramuka No. 38, Jakarta Timur', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
('1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', 'PMI', 'Palang Merah Indonesia', 'https://terrasiaga.org/logos/pmi.png', 'https://pmi.or.id', '+62213906005', 'info@pmi.or.id', 'Jl. Jend. Gatot Subroto Kav. 96, Jakarta Selatan', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', 'BASARNAS', 'Badan SAR Nasional', 'https://terrasiaga.org/logos/basarnas.png', 'https://basarnas.go.id', '+62216345765', 'humas@basarnas.go.id', 'Jl. Angkasa Blok B-15 Kav. 2-3, Jakarta Pusat', 'dddddddd-dddd-dddd-dddd-dddddddddddd'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1a', 'BMKG', 'Badan Meteorologi, Klimatologi, dan Geofisika', 'https://terrasiaga.org/logos/bmkg.png', 'https://bmkg.go.id', '+62216546316', 'info@bmkg.go.id', 'Jl. Angkasa I No. 2, Jakarta Pusat', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e12', 'Dompet Dhuafa', 'Lembaga Filantropi Islam', 'https://terrasiaga.org/logos/dompet_dhuafa.png', 'https://dompetdhuafa.org', '+62217863364', 'info@dompetdhuafa.org', 'Jl. Ir. H. Juanda No. 50, Ciputat, Tangerang Selatan', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0111', 'ACT', 'Aksi Cepat Tanggap', 'https://terrasiaga.org/logos/act.png', 'https://act.id', '+62217388455', 'info@act.id', 'Jl. TB Simatupang No. 2, Jakarta Selatan', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0112', 'WALHI', 'Wahana Lingkungan Hidup Indonesia', 'https://terrasiaga.org/logos/walhi.png', 'https://walhi.or.id', '+62215794125', 'info@walhi.or.id', 'Jl. Tegal Parang Utara No. 14, Jakarta Selatan', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0113', 'PVMBG', 'Pusat Vulkanologi dan Mitigasi Bencana Geologi', 'https://terrasiaga.org/logos/pvmbg.png', 'https://vsi.esdm.go.id', '+62227272606', 'pvmbg@esdm.go.id', 'Jl. Diponegoro No. 57, Bandung', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0115', 'BPBD DKI Jakarta', 'Badan Penanggulangan Bencana Daerah DKI Jakarta', 'https://terrasiaga.org/logos/bpbd_jakarta.png', 'https://bpbd.jakarta.go.id', '+62123456789', 'bpbd@jakarta.go.id', 'Jl. Kyai Haji Zainul Arifin No. 71, Jakarta Barat', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
('3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0116', 'Mercy Corps Indonesia', 'Organisasi Kemanusiaan Internasional', 'https://terrasiaga.org/logos/mercy_corps.png', 'https://indonesia.mercycorps.org', '+62217237979', 'info@id.mercycorps.org', 'Jl. H.R. Rasuna Said Kav. C-17, Jakarta Selatan', 'cccccccc-cccc-cccc-cccc-cccccccccccc');

-- Sample User Roles (many-to-many relationship)
INSERT INTO user_roles (user_id, role_id) VALUES
('11111111-1111-1111-1111-111111111111', 1), -- Admin Utama as admin
('22222222-2222-2222-2222-222222222222', 2), -- Budi Santoso as volunteer
('33333333-3333-3333-3333-333333333333', 3), -- Siti Rahayu as reporter
('44444444-4444-4444-4444-444444444444', 4), -- Dian Pratiwi as analyst
('55555555-5555-5555-5555-555555555555', 5), -- Joko Widodo as organization
('66666666-6666-6666-6666-666666666666', 2), -- Ani Kusuma as volunteer
('77777777-7777-7777-7777-777777777777', 3), -- Rudi Hartono as reporter
('88888888-8888-8888-8888-888888888888', 5), -- Maya Indah as organization
('99999999-9999-9999-9999-999999999999', 5), -- Agus Hermawan as organization
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 4), -- Dewi Safitri as analyst
-- Some users with multiple roles
('11111111-1111-1111-1111-111111111111', 4), -- Admin also as analyst
('22222222-2222-2222-2222-222222222222', 3), -- Volunteer also as reporter
('44444444-4444-4444-4444-444444444444', 2), -- Analyst also as volunteer
('55555555-5555-5555-5555-555555555555', 2), -- Organization rep also as volunteer
('88888888-8888-8888-8888-888888888888', 2); -- Organization rep also as volunteer

-- Sample Volunteers
INSERT INTO volunteers (id, user_id, skills, certifications, availability, availability_notes, experience_years, specialization, current_location_id) VALUES
('aabbccdd-eeff-0011-2233-445566778899', '22222222-2222-2222-2222-222222222222', ARRAY['Pertolongan Pertama', 'Evakuasi', 'Penyelamatan Air'], ARRAY['Sertifikat Pertolongan Pertama PMI', 'Pelatihan SAR Dasar'], TRUE, 'Tersedia 24/7 untuk keadaan darurat', 5, 'Penanganan banjir', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
('aabbccdd-eeff-0011-2233-44556677889a', '66666666-6666-6666-6666-666666666666', ARRAY['Medis Darurat', 'Triase', 'Perawatan Luka'], ARRAY['Sertifikat Perawat', 'Pelatihan Medis Darurat'], TRUE, 'Tersedia untuk shift malam dan akhir pekan', 3, 'Bantuan medis', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
('aabbccdd-eeff-0011-2233-44556677889b', '44444444-4444-4444-4444-444444444444', ARRAY['Analisis Data', 'Pemetaan', 'Koordinasi Tim'], ARRAY['GIS Specialist', 'Manajemen Bencana'], FALSE, 'Sedang dalam penugasan hingga akhir bulan', 4, 'Analisis dan koordinasi', 'dddddddd-dddd-dddd-dddd-dddddddddddd'),
('aabbccdd-eeff-0011-2233-44556677889c', '55555555-5555-5555-5555-555555555555', ARRAY['Koordinasi Lembaga', 'Manajemen Logistik', 'Komunikasi Publik'], ARRAY['Manajemen Bencana Tingkat Lanjut', 'Koordinator Lapangan'], TRUE, 'Tersedia untuk koordinasi antar lembaga', 8, 'Koordinasi antar lembaga', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
('aabbccdd-eeff-0011-2233-44556677889d', '88888888-8888-8888-8888-888888888888', ARRAY['Distribusi Bantuan', 'Manajemen Pengungsian', 'Konseling Trauma'], ARRAY['Sertifikat Manajemen Logistik', 'Pelatihan Konseling Trauma'], TRUE, 'Tersedia untuk penugasan jangka panjang', 6, 'Manajemen bantuan kemanusiaan', 'cccccccc-cccc-cccc-cccc-cccccccccccc');

-- Sample Reports
INSERT INTO reports (id, reporter_id, anonymous_name, anonymous_phone, is_anonymous, disaster_type_id, title, description, location_id, address, impact_radius, estimated_severity, casualties, injuries, missing, affected_people, status, credibility_score, validated_by, validation_notes, validation_date) VALUES
('11223344-5566-7788-99aa-bbccddeeff00', '33333333-3333-3333-3333-333333333333', NULL, NULL, FALSE, 1, 'Banjir di Kampung Melayu', 'Banjir setinggi 1 meter telah menggenangi area perumahan di Kampung Melayu. Warga terpaksa mengungsi ke tempat yang lebih tinggi.', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Jl. Kampung Melayu Besar RT 05/RW 03', 500.0, 3, 0, 5, 0, 200, 'valid', 0.9, '22222222-2222-2222-2222-222222222222', 'Laporan telah diverifikasi dengan kunjungan langsung ke lokasi', '2025-07-20 15:30:00'),
('11223344-5566-7788-99aa-bbccddeeff01', '77777777-7777-7777-7777-777777777777', NULL, NULL, FALSE, 2, 'Gempa di Lombok Utara', 'Gempa berkekuatan 6.2 SR telah mengguncang Lombok Utara. Beberapa bangunan rusak dan warga panik keluar rumah.', 'eeeeeeee-9090-9090-9090-909090909090', 'Desa Pemenang Barat, Kecamatan Pemenang', 10000.0, 4, 2, 15, 1, 500, 'valid', 0.95, '22222222-2222-2222-2222-222222222222', 'Laporan sesuai dengan data BMKG dan telah diverifikasi oleh tim di lapangan', '2025-07-19 08:45:00'),
('11223344-5566-7788-99aa-bbccddeeff02', NULL, 'Ahmad', '+6281234567890', TRUE, 3, 'Kebakaran Hutan di Lumajang', 'Kebakaran hutan telah terjadi di area hutan dekat Desa Sumber Wuluh. Asap tebal membumbung tinggi dan terlihat dari jarak jauh.', 'aeaeaeae-cccc-dddd-eeee-bbbbbbaabbaa', 'Hutan di sebelah timur Desa Sumber Wuluh', 2000.0, 3, 0, 0, 0, 100, 'pending', 0.6, NULL, NULL, NULL),
('11223344-5566-7788-99aa-bbccddeeff03', '33333333-3333-3333-3333-333333333333', NULL, NULL, FALSE, 4, 'Tanah Longsor di Cianjur', 'Tanah longsor telah terjadi di area perbukitan setelah hujan deras selama 3 hari. Beberapa rumah tertimbun dan akses jalan terputus.', 'ccddccaa-1233-ccac-2233-8948744747cc', 'Desa Cigobang, Kecamatan Pasaleman', 300.0, 4, 3, 7, 5, 50, 'valid', 0.85, '66666666-6666-6666-6666-666666666666', 'Laporan telah diverifikasi oleh tim SAR yang diterjunkan ke lokasi', '2025-07-18 14:20:00'),
('11223344-5566-7788-99aa-bbccddeeff04', NULL, 'Budi Warga', '+6285678901234', TRUE, 6, 'Erupsi Gunung Merapi', 'Gunung Merapi mengeluarkan abu vulkanik dan terdengar suara gemuruh. Warga di sekitar mulai bersiap untuk evakuasi.', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'Desa Cangkringan, Sleman', 5000.0, 3, 0, 0, 0, 1000, 'valid', 0.8, '55555555-5555-5555-5555-555555555555', 'Laporan sesuai dengan data PVMBG dan telah diverifikasi', '2025-07-17 23:10:00'),
('11223344-5566-7788-99aa-bbccddeeff05', '77777777-7777-7777-7777-777777777777', NULL, NULL, FALSE, 5, 'Potensi Tsunami di Pangandaran', 'Setelah gempa 7.0 SR di laut, warga di pesisir Pangandaran dihimbau untuk waspada tsunami. Beberapa warga telah mengungsi ke tempat yang lebih tinggi.', 'aaaaaaaa-1111-1111-1111-111111111111', 'Pantai Pangandaran, Kecamatan Pangandaran', 5000.0, 5, 0, 0, 0, 2000, 'invalid', 0.2, '44444444-4444-4444-4444-444444444444', 'Laporan tidak sesuai dengan data BMKG. Tidak ada potensi tsunami dari gempa tersebut.', '2025-07-16 10:30:00'),
('11223344-5566-7788-99aa-bbccddeeff06', '33333333-3333-3333-3333-333333333333', NULL, NULL, FALSE, 1, 'Banjir di Bidara Cina', 'Banjir setinggi 1.5 meter telah menggenangi area perumahan di Bidara Cina akibat luapan Kali Ciliwung. Warga terpaksa mengungsi.', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 'Jl. Bidara Cina RT 08/RW 04', 600.0, 4, 0, 3, 0, 300, 'valid', 0.9, '22222222-2222-2222-2222-222222222222', 'Laporan telah diverifikasi dengan kunjungan langsung ke lokasi', '2025-07-20 16:45:00'),
('11223344-5566-7788-99aa-bbccddeeff07', NULL, 'Warga Peduli', '+6287654321098', TRUE, 7, 'Angin Puting Beliung di Cirebon', 'Angin puting beliung telah merusak puluhan rumah di Kelurahan Cigobang. Atap rumah beterbangan dan beberapa pohon tumbang.', 'ccddccaa-1233-ccac-2233-8948744747cc', 'Kelurahan Cigobang, Kecamatan Pasaleman', 400.0, 3, 0, 8, 0, 100, 'pending', 0.5, NULL, NULL, NULL),
('11223344-5566-7788-99aa-bbccddeeff08', '77777777-7777-7777-7777-777777777777', NULL, NULL, FALSE, 2, 'Gempa Susulan di Palu', 'Gempa susulan berkekuatan 5.6 SR telah mengguncang Palu. Warga masih trauma dan banyak yang memilih tidur di luar rumah.', 'abababaa-abab-2123-2134-123123123123', 'Kelurahan Petobo, Palu Selatan', 8000.0, 3, 0, 5, 0, 1000, 'valid', 0.85, '66666666-6666-6666-6666-666666666666', 'Laporan sesuai dengan data BMKG dan telah diverifikasi', '2025-07-15 22:15:00'),
('11223344-5566-7788-99aa-bbccddeeff09', '33333333-3333-3333-3333-333333333333', NULL, NULL, FALSE, 10, 'Kebocoran Gas di Kawasan Industri', 'Kebocoran gas telah terjadi di salah satu pabrik di kawasan industri. Beberapa pekerja mengalami gangguan pernapasan.', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Kawasan Industri Pulogadung, Jakarta Timur', 200.0, 2, 0, 12, 0, 50, 'resolved', 0.9, '55555555-5555-5555-5555-555555555555', 'Kebocoran telah berhasil diatasi oleh tim tanggap darurat pabrik', '2025-07-14 09:20:00');

-- Sample Disasters (verified disaster events)
INSERT INTO disasters (id, disaster_type_id, name, description, severity, status, start_time, end_time, primary_location_id, created_at, updated_at) VALUES
('aabbccdd-1111-2222-3333-445566778899', 1, 'Banjir Jakarta 2025', 'Banjir besar yang melanda Jakarta akibat curah hujan tinggi dan luapan sungai Ciliwung', 4, 'active', '2025-07-20 05:00:00', NULL, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '2025-07-20 07:30:00', '2025-07-20 07:30:00'),
('aabbccdd-1111-2222-3333-44556677889a', 2, 'Gempa Lombok 2025', 'Gempa berkekuatan 6.2 SR yang mengguncang Lombok Utara', 4, 'active', '2025-07-19 06:15:00', NULL, 'eeeeeeee-9090-9090-9090-909090909090', '2025-07-19 07:00:00', '2025-07-19 07:00:00'),
('aabbccdd-1111-2222-3333-44556677889b', 4, 'Tanah Longsor Cianjur', 'Tanah longsor yang terjadi di area perbukitan Cianjur setelah hujan deras', 4, 'contained', '2025-07-18 10:30:00', NULL, 'ccddccaa-1233-ccac-2233-8948744747cc', '2025-07-18 12:00:00', '2025-07-18 18:30:00'),
('aabbccdd-1111-2222-3333-44556677889c', 6, 'Erupsi Merapi 2025', 'Erupsi Gunung Merapi yang mengeluarkan abu vulkanik dan material piroklastik', 3, 'active', '2025-07-17 22:00:00', NULL, 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', '2025-07-17 22:30:00', '2025-07-17 22:30:00'),
('aabbccdd-1111-2222-3333-44556677889d', 2, 'Gempa Susulan Palu', 'Gempa susulan berkekuatan 5.6 SR yang mengguncang Palu', 3, 'contained', '2025-07-15 20:45:00', '2025-07-15 21:00:00', 'abababaa-abab-2123-2134-123123123123', '2025-07-15 21:30:00', '2025-07-16 08:00:00'),
('aabbccdd-1111-2222-3333-44556677889e', 10, 'Kebocoran Gas Pulogadung', 'Kebocoran gas di salah satu pabrik di kawasan industri Pulogadung', 2, 'resolved', '2025-07-14 08:30:00', '2025-07-14 10:15:00', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '2025-07-14 09:00:00', '2025-07-14 11:00:00');

-- Link Reports to Disasters
INSERT INTO disaster_reports (disaster_id, report_id, created_at) VALUES
('aabbccdd-1111-2222-3333-445566778899', '11223344-5566-7788-99aa-bbccddeeff00', '2025-07-20 07:45:00'),
('aabbccdd-1111-2222-3333-445566778899', '11223344-5566-7788-99aa-bbccddeeff06', '2025-07-20 17:00:00'),
('aabbccdd-1111-2222-3333-44556677889a', '11223344-5566-7788-99aa-bbccddeeff01', '2025-07-19 09:00:00'),
('aabbccdd-1111-2222-3333-44556677889b', '11223344-5566-7788-99aa-bbccddeeff03', '2025-07-18 14:30:00'),
('aabbccdd-1111-2222-3333-44556677889c', '11223344-5566-7788-99aa-bbccddeeff04', '2025-07-17 23:15:00'),
('aabbccdd-1111-2222-3333-44556677889d', '11223344-5566-7788-99aa-bbccddeeff08', '2025-07-15 22:30:00'),
('aabbccdd-1111-2222-3333-44556677889e', '11223344-5566-7788-99aa-bbccddeeff09', '2025-07-14 09:30:00');

-- Sample Report Media
INSERT INTO report_media (id, report_id, media_type, media_url, caption, is_primary) VALUES
('a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d', '11223344-5566-7788-99aa-bbccddeeff00', 'image', 'https://terrasiaga.org/media/banjir_kampung_melayu_1.jpg', 'Kondisi banjir di Jl. Kampung Melayu Besar', TRUE),
('abce0002-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff00', 'image', 'https://terrasiaga.org/media/banjir_kampung_melayu_2.jpg', 'Warga mengungsi ke tempat yang lebih tinggi', FALSE),
('abde0003-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff00', 'video', 'https://terrasiaga.org/media/banjir_kampung_melayu_video.mp4', 'Video situasi banjir dan evakuasi warga', FALSE),
('acde0004-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff01', 'image', 'https://terrasiaga.org/media/gempa_lombok_1.jpg', 'Kerusakan bangunan akibat gempa di Lombok Utara', TRUE),
('acde0005-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff01', 'image', 'https://terrasiaga.org/media/gempa_lombok_2.jpg', 'Warga berkumpul di tempat pengungsian sementara', FALSE),
('acde0006-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff02', 'image', 'https://terrasiaga.org/media/kebakaran_hutan_lumajang.jpg', 'Asap tebal dari kebakaran hutan di Lumajang', TRUE),
('acde0007-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff03', 'image', 'https://terrasiaga.org/media/longsor_cianjur_1.jpg', 'Tanah longsor yang menutupi jalan di Cianjur', TRUE),
('acde0008-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff03', 'image', 'https://terrasiaga.org/media/longsor_cianjur_2.jpg', 'Tim SAR melakukan pencarian korban', FALSE),
('acde0009-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff03', 'video', 'https://terrasiaga.org/media/longsor_cianjur_video.mp4', 'Video proses evakuasi korban longsor', FALSE),
('acde0010-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff04', 'image', 'https://terrasiaga.org/media/erupsi_merapi.jpg', 'Erupsi Gunung Merapi terlihat dari Desa Cangkringan', TRUE),
('acde0011-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff06', 'image', 'https://terrasiaga.org/media/banjir_bidara_cina.jpg', 'Banjir di kawasan Bidara Cina', TRUE),
('acde0012-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff07', 'image', 'https://terrasiaga.org/media/angin_puting_beliung.jpg', 'Kerusakan akibat angin puting beliung di Cirebon', TRUE),
('acde0013-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff08', 'image', 'https://terrasiaga.org/media/gempa_palu.jpg', 'Warga Palu tidur di luar rumah pasca gempa susulan', TRUE),
('acde0014-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff09', 'image', 'https://terrasiaga.org/media/kebocoran_gas.jpg', 'Tim tanggap darurat menangani kebocoran gas', TRUE),
('acde0015-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff09', 'document', 'https://terrasiaga.org/media/laporan_kebocoran_gas.pdf', 'Laporan penanganan kebocoran gas', FALSE);

-- Sample Report Comments
INSERT INTO report_comments (id, report_id, user_id, content, parent_id) VALUES
('abcd1001-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff00', '22222222-2222-2222-2222-222222222222', 'Saya sudah tiba di lokasi dan sedang melakukan assessment. Banjir memang cukup parah, ketinggian air mencapai 1 meter.', NULL),
('abcd1002-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff00', '55555555-5555-5555-5555-555555555555', 'BNPB sudah mengirimkan bantuan logistik berupa makanan, selimut, dan obat-obatan. Akan tiba dalam 2 jam.', NULL),
('abcd1003-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff00', '33333333-3333-3333-3333-333333333333', 'Terima kasih atas respon cepatnya. Warga sangat membutuhkan bantuan segera.', 'abcd1002-aaaa-bbbb-cccc-ddddeeeeeeee'),
('abcd1004-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff01', '66666666-6666-6666-6666-666666666666', 'Tim medis sudah tiba di lokasi dan sedang memberikan pertolongan pertama kepada korban luka-luka.', NULL),
('abcd1005-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff01', '77777777-7777-7777-7777-777777777777', 'Apakah ada informasi mengenai potensi gempa susulan?', NULL),
('abcd1006-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff01', '44444444-4444-4444-4444-444444444444', 'Menurut data BMKG, masih ada potensi gempa susulan dalam 24 jam ke depan. Warga diharapkan tetap waspada.', 'abcd1005-aaaa-bbbb-cccc-ddddeeeeeeee'),
('abcd1007-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff03', '66666666-6666-6666-6666-666666666666', 'Tim SAR masih melakukan pencarian korban yang tertimbun longsor. Sudah ditemukan 3 korban meninggal dan 7 luka-luka.', NULL),
('abcd1008-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff03', '55555555-5555-5555-5555-555555555555', 'BNPB sudah mengirimkan alat berat untuk membantu proses evakuasi. Diperkirakan tiba dalam 3 jam.', NULL),
('abcd1009-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff04', '88888888-8888-8888-8888-888888888888', 'PMI sudah mendirikan posko pengungsian di beberapa titik. Warga yang membutuhkan bantuan bisa menuju ke posko terdekat.', NULL),
('cffff010-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff04', '44444444-4444-4444-4444-444444444444', 'Berdasarkan data PVMBG, aktivitas Gunung Merapi masih dalam status Siaga. Warga di radius 5 km diharapkan mengungsi.', NULL),
('cffff011-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff06', '22222222-2222-2222-2222-222222222222', 'Saya sedang di lokasi banjir Bidara Cina. Ketinggian air mencapai 1.5 meter dan masih berpotensi naik karena hujan masih turun.', NULL),
('cffff012-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff06', '99999999-9999-9999-9999-999999999999', 'Pemda DKI Jakarta sudah mengerahkan pompa air untuk mempercepat surut. Juga sedang menyiapkan bantuan logistik untuk warga terdampak.', NULL),
('cffff013-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff08', '66666666-6666-6666-6666-666666666666', 'Tim medis sedang memberikan dukungan psikologis kepada warga yang trauma akibat gempa susulan.', NULL),
('cffff014-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff09', '55555555-5555-5555-5555-555555555555', 'Kebocoran gas sudah berhasil diatasi. Tim dari BNPB dan Damkar sudah memastikan area aman.', NULL),
('cffff015-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff09', '33333333-3333-3333-3333-333333333333', 'Apakah ada korban yang masih dirawat di rumah sakit?', 'cffff014-aaaa-bbbb-cccc-ddddeeeeeeee'),
('cffff016-aaaa-bbbb-cccc-ddddeeeeeeee', '11223344-5566-7788-99aa-bbccddeeff09', '55555555-5555-5555-5555-555555555555', 'Ada 5 orang yang masih dirawat di RS Persahabatan, tapi kondisinya sudah membaik.', 'cffff015-aaaa-bbbb-cccc-ddddeeeeeeee');

-- Sample Evacuation Centers
INSERT INTO evacuation_centers (id, name, description, capacity, current_occupancy, location_id, status, contact_person, contact_phone) VALUES
('12345671-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Kampung Melayu', 'Posko pengungsian untuk korban banjir di Kampung Melayu', 300, 180, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'operational', 'Budi Santoso', '+6282345678901'),
('12345672-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Bidara Cina', 'Posko pengungsian untuk korban banjir di Bidara Cina', 250, 200, 'dddddddd-dddd-dddd-dddd-dddddddddddd', 'operational', 'Ani Kusuma', '+6286789012345'),
('12345673-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Lombok Utara', 'Posko pengungsian untuk korban gempa di Lombok Utara', 500, 450, 'eeeeeeee-9090-9090-9090-909090909090', 'full', 'Maya Indah', '+6288901234567'),
('12345674-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Kayangan', 'Posko pengungsian tambahan untuk korban gempa di Lombok Utara', 300, 250, 'ffffffff-1111-2222-ffff-121212121212', 'operational', 'Koordinator PMI Lombok', '+6281234567891'),
('12345675-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Cangkringan', 'Posko pengungsian untuk warga terdampak erupsi Merapi', 400, 350, 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'operational', 'Koordinator BPBD Sleman', '+6281234567892'),
('12345676-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Umbulharjo', 'Posko pengungsian tambahan untuk warga terdampak erupsi Merapi', 350, 300, 'ffffffff-ffff-ffff-ffff-ffffffffffff', 'operational', 'Koordinator PMI Sleman', '+6281234567893'),
('12345677-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Cigobang', 'Posko pengungsian untuk korban tanah longsor di Cigobang', 200, 150, 'ccddccaa-1233-ccac-2233-8948744747cc', 'operational', 'Koordinator BPBD Cirebon', '+6281234567894'),
('12345678-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Pengungsian Petobo', 'Posko pengungsian untuk korban gempa di Palu', 400, 350, 'abababaa-abab-2123-2134-123123123123', 'operational', 'Koordinator BNPB Palu', '+6281234567895');

-- Sample Evacuation Center Facilities
INSERT INTO evacuation_center_facilities (id, evacuation_center_id, facility_name, quantity, status) VALUES
('facaab01-aaaa-bbbb-cccc-ddddeeeeeeee', '12345671-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 300, 'available'),
('facaab02-aaaa-bbbb-cccc-ddddeeeeeeee', '12345671-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 20, 'available'),
('facaab03-aaaa-bbbb-cccc-ddddeeeeeeee', '12345671-aaaa-bbbb-cccc-ddddeeeeeeee', 'Dapur Umum', 2, 'available'),
('facaab04-aaaa-bbbb-cccc-ddddeeeeeeee', '12345671-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Kesehatan', 1, 'available'),
('facaab05-aaaa-bbbb-cccc-ddddeeeeeeee', '12345672-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 250, 'limited'),
('facaab06-aaaa-bbbb-cccc-ddddeeeeeeee', '12345672-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 15, 'available'),
('facaab07-aaaa-bbbb-cccc-ddddeeeeeeee', '12345672-aaaa-bbbb-cccc-ddddeeeeeeee', 'Dapur Umum', 2, 'available'),
('facaab08-aaaa-bbbb-cccc-ddddeeeeeeee', '12345672-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Kesehatan', 1, 'available'),
('facaab09-aaaa-bbbb-cccc-ddddeeeeeeee', '12345673-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 500, 'limited'),
('facaab10-aaaa-bbbb-cccc-ddddeeeeeeee', '12345673-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 30, 'limited'),
('facaab11-aaaa-bbbb-cccc-ddddeeeeeeee', '12345673-aaaa-bbbb-cccc-ddddeeeeeeee', 'Dapur Umum', 3, 'available'),
('facaab12-aaaa-bbbb-cccc-ddddeeeeeeee', '12345673-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Kesehatan', 2, 'available'),
('facaab13-aaaa-bbbb-cccc-ddddeeeeeeee', '12345673-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tenda Pengungsian', 50, 'limited'),
('facaab14-aaaa-bbbb-cccc-ddddeeeeeeee', '12345674-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 300, 'limited'),
('facaab15-aaaa-bbbb-cccc-ddddeeeeeeee', '12345674-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 20, 'available'),
('facaab16-aaaa-bbbb-cccc-ddddeeeeeeee', '12345674-aaaa-bbbb-cccc-ddddeeeeeeee', 'Dapur Umum', 2, 'available'),
('facaab17-aaaa-bbbb-cccc-ddddeeeeeeee', '12345675-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 400, 'limited'),
('facaab18-aaaa-bbbb-cccc-ddddeeeeeeee', '12345675-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 25, 'available'),
('facaab19-aaaa-bbbb-cccc-ddddeeeeeeee', '12345675-aaaa-bbbb-cccc-ddddeeeeeeee', 'Dapur Umum', 3, 'available'),
('facaab20-aaaa-bbbb-cccc-ddddeeeeeeee', '12345675-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Kesehatan', 2, 'available'),
('facaab21-aaaa-bbbb-cccc-ddddeeeeeeee', '12345676-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 350, 'limited'),
('facaab22-aaaa-bbbb-cccc-ddddeeeeeeee', '12345676-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 20, 'available'),
('facaab23-aaaa-bbbb-cccc-ddddeeeeeeee', '12345677-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 200, 'available'),
('facaab24-aaaa-bbbb-cccc-ddddeeeeeeee', '12345677-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 15, 'available'),
('facaab25-aaaa-bbbb-cccc-ddddeeeeeeee', '12345678-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tempat Tidur', 400, 'limited'),
('facaab26-aaaa-bbbb-cccc-ddddeeeeeeee', '12345678-aaaa-bbbb-cccc-ddddeeeeeeee', 'Toilet', 25, 'available'),
('facaab27-aaaa-bbbb-cccc-ddddeeeeeeee', '12345678-aaaa-bbbb-cccc-ddddeeeeeeee', 'Dapur Umum', 3, 'available'),
('facaab28-aaaa-bbbb-cccc-ddddeeeeeeee', '12345678-aaaa-bbbb-cccc-ddddeeeeeeee', 'Posko Kesehatan', 2, 'available'),
('facaab29-aaaa-bbbb-cccc-ddddeeeeeeee', '12345678-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tenda Pengungsian', 40, 'available');

-- Sample Emergency Resources
INSERT INTO emergency_resources (id, name, category, quantity, unit, location_id, organization_id, expiry_date, status) VALUES
('facaacb1-aaaa-bbbb-cccc-ddddeeeeeeee', 'Beras', 'food', 5000, 'kg', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-07-21', 'available'),
('facaacb2-aaaa-bbbb-cccc-ddddeeeeeeee', 'Mie Instan', 'food', 10000, 'box', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-01-15', 'available'),
('facaacb3-aaaa-bbbb-cccc-ddddeeeeeeee', 'Air Mineral', 'food', 20000, 'bottle', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-07-21', 'available'),
('facaacb4-aaaa-bbbb-cccc-ddddeeeeeeee', 'Selimut', 'shelter', 2000, 'piece', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', NULL, 'available'),
('facaacb5-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tenda Pengungsian', 'shelter', 500, 'unit', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', NULL, 'available'),
('facaacb6-aaaa-bbbb-cccc-ddddeeeeeeee', 'Pakaian Layak Pakai', 'clothing', 5000, 'piece', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e12', NULL, 'available'),
('facaacb7-aaaa-bbbb-cccc-ddddeeeeeeee', 'Obat-obatan Dasar', 'medical', 1000, 'box', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', '2025-12-31', 'available'),
('facaacb8-aaaa-bbbb-cccc-ddddeeeeeeee', 'Masker', 'medical', 10000, 'piece', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', '2026-06-30', 'available'),
('facaacb9-aaaa-bbbb-cccc-ddddeeeeeeee', 'Perahu Karet', 'equipment', 20, 'unit', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '3c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', NULL, 'available'),
('facaacc0-aaaa-bbbb-cccc-ddddeeeeeeee', 'Generator Listrik', 'equipment', 50, 'unit', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', NULL, 'available'),
('facaacc1-aaaa-bbbb-cccc-ddddeeeeeeee', 'Beras', 'food', 3000, 'kg', 'eeeeeeee-9090-9090-9090-909090909090', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-07-21', 'available'),
('facaacc2-aaaa-bbbb-cccc-ddddeeeeeeee', 'Mie Instan', 'food', 5000, 'box', 'eeeeeeee-9090-9090-9090-909090909090', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-01-15', 'available'),
('facaacc3-aaaa-bbbb-cccc-ddddeeeeeeee', 'Air Mineral', 'food', 10000, 'bottle', 'eeeeeeee-9090-9090-9090-909090909090', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-07-21', 'available'),
('facaacc4-aaaa-bbbb-cccc-ddddeeeeeeee', 'Selimut', 'shelter', 1000, 'piece', 'eeeeeeee-9090-9090-9090-909090909090', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', NULL, 'available'),
('facaacc5-aaaa-bbbb-cccc-ddddeeeeeeee', 'Tenda Pengungsian', 'shelter', 200, 'unit', 'eeeeeeee-9090-9090-9090-909090909090', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', NULL, 'available'),
('facaacc6-aaaa-bbbb-cccc-ddddeeeeeeee', 'Obat-obatan Dasar', 'medical', 500, 'box', 'eeeeeeee-9090-9090-9090-909090909090', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', '2025-12-31', 'available'),
('facaacc7-aaaa-bbbb-cccc-ddddeeeeeeee', 'Beras', 'food', 2000, 'kg', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-07-21', 'available'),
('facaacc8-aaaa-bbbb-cccc-ddddeeeeeeee', 'Mie Instan', 'food', 4000, 'box', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-01-15', 'available'),
('facaacc9-aaaa-bbbb-cccc-ddddeeeeeeee', 'Air Mineral', 'food', 8000, 'bottle', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'f47ac10b-58cc-4372-a567-0e02b2c3d4e5', '2026-07-21', 'available'),
('facaacd0-aaaa-bbbb-cccc-ddddeeeeeeee', 'Masker Anti Abu Vulkanik', 'medical', 5000, 'piece', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', '1c3d5e7f-9a1b-2c3d-4e5f-6a7b8c9d0e1f', '2026-06-30', 'available');

-- Sample Resource Allocations
INSERT INTO resource_allocations (id, resource_id, disaster_id, quantity, allocated_by, status, notes) VALUES
('acced001-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacb1-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-445566778899', 1000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan beras untuk korban banjir di Kampung Melayu'),
('acced002-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacb2-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-445566778899', 2000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan mie instan untuk korban banjir di Kampung Melayu'),
('acced003-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacb3-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-445566778899', 5000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan air mineral untuk korban banjir di Kampung Melayu'),
('acced004-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacb4-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-445566778899', 500, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan selimut untuk korban banjir di Kampung Melayu'),
('acced005-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacb9-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-445566778899', 10, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan perahu karet untuk evakuasi korban banjir di Kampung Melayu'),
('acced006-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc0-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-445566778899', 10, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan generator listrik untuk posko pengungsian di Kampung Melayu'),
('acced007-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc1-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889a', 1000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan beras untuk korban gempa di Lombok Utara'),
('acced008-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc2-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889a', 2000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan mie instan untuk korban gempa di Lombok Utara'),
('acced009-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc3-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889a', 5000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan air mineral untuk korban gempa di Lombok Utara'),
('acced010-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc4-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889a', 500, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan selimut untuk korban gempa di Lombok Utara'),
('acced011-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc5-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889a', 100, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan tenda pengungsian untuk korban gempa di Lombok Utara'),
('acced012-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc6-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889a', 200, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan obat-obatan untuk korban gempa di Lombok Utara'),
('acced013-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc7-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889c', 1000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan beras untuk korban erupsi Merapi'),
('acced014-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc8-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889c', 2000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan mie instan untuk korban erupsi Merapi'),
('acced015-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacc9-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889c', 4000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan air mineral untuk korban erupsi Merapi'),
('acced016-aaaa-bbbb-cccc-ddddeeeeeeee', 'facaacd0-aaaa-bbbb-cccc-ddddeeeeeeee', 'aabbccdd-1111-2222-3333-44556677889c', 3000, '55555555-5555-5555-5555-555555555555', 'delivered', 'Bantuan masker anti abu vulkanik untuk korban erupsi Merapi');
