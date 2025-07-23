-- This migration adds additional sample data for development and testing purposes
-- Created on: 2025-07-21

-- Sample Verification Codes
INSERT INTO verification_codes (id, user_id, code, type, expires_at, is_used) VALUES
('a1de0001-aaaa-bbbb-cccc-ddddeeeeeeee', '77777777-7777-7777-7777-777777777777', '123456', 'email', '2025-07-22 12:00:00', FALSE),
('a2de0002-aaaa-bbbb-cccc-ddddeeeeeeee', '77777777-7777-7777-7777-777777777777', '654321', 'phone', '2025-07-22 12:00:00', FALSE),
('a3de0003-aaaa-bbbb-cccc-ddddeeeeeeee', '33333333-3333-3333-3333-333333333333', '987654', 'email', '2025-07-20 12:00:00', TRUE),
('a4de0004-aaaa-bbbb-cccc-ddddeeeeeeee', '66666666-6666-6666-6666-666666666666', '456789', 'email', '2025-07-23 12:00:00', FALSE),
('a5de0005-aaaa-bbbb-cccc-ddddeeeeeeee', '44444444-4444-4444-4444-444444444444', '789456', 'email', '2025-07-19 12:00:00', TRUE);

-- Sample Auth Sessions
INSERT INTO auth_sessions (id, user_id, token, user_agent, ip_address, expires_at) VALUES
('b1ac0011-aaaa-bbbb-cccc-ddddeeeeeeee', '11111111-1111-1111-1111-111111111111', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiMTExMTExMTEtMTExMS0xMTExLTExMTEtMTExMTExMTExMTExIn0.aaaabbbbccccdddd', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36', '192.168.1.1', '2025-07-22 12:00:00'),
('b2ac0012-aaaa-bbbb-cccc-ddddeeeeeeee', '22222222-2222-2222-2222-222222222222', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiMjIyMjIyMjItMjIyMi0yMjIyLTIyMjItMjIyMjIyMjIyMjIyIn0.aaaabbbbccccdddd', 'Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X)', '192.168.1.2', '2025-07-22 14:00:00'),
('b3ac0013-aaaa-bbbb-cccc-ddddeeeeeeee', '33333333-3333-3333-3333-333333333333', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiMzMzMzMzMzMtMzMzMy0zMzMzLTMzMzMtMzMzMzMzMzMzMzMzIn0.aaaabbbbccccdddd', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', '192.168.1.3', '2025-07-22 16:00:00'),
('b4ac0014-aaaa-bbbb-cccc-ddddeeeeeeee', '44444444-4444-4444-4444-444444444444', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiNDQ0NDQ0NDQtNDQ0NC00NDQ0LTQ0NDQtNDQ0NDQ0NDQ0NDQ0In0.aaaabbbbccccdddd', 'Mozilla/5.0 (Linux; Android 11; Pixel 5)', '192.168.1.4', '2025-07-22 18:00:00'),
('b5ac0015-aaaa-bbbb-cccc-ddddeeeeeeee', '55555555-5555-5555-5555-555555555555', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiNTU1NTU1NTUtNTU1NS01NTU1LTU1NTUtNTU1NTU1NTU1NTU1In0.aaaabbbbccccdddd', 'Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)', '192.168.1.5', '2025-07-22 20:00:00');

-- Sample Refresh Tokens
INSERT INTO refresh_tokens (id, user_id, token, is_valid, revoked_at, expires_at) VALUES
('c1aaddd1-aaaa-bbbb-cccc-ddddeeeeeeee', '11111111-1111-1111-1111-111111111111', 'rtok_aaaabbbbccccddddeeeeffffgggghhhhiiiijjjjkkkkllllmmmmnnnnoooopppp_1', TRUE, NULL, '2025-08-21 12:00:00'),
('c2aaddd2-aaaa-bbbb-cccc-ddddeeeeeeee', '22222222-2222-2222-2222-222222222222', 'rtok_aaaabbbbccccddddeeeeffffgggghhhhiiiijjjjkkkkllllmmmmnnnnoooopppp_2', TRUE, NULL, '2025-08-21 14:00:00'),
('c3aaddd3-aaaa-bbbb-cccc-ddddeeeeeeee', '33333333-3333-3333-3333-333333333333', 'rtok_aaaabbbbccccddddeeeeffffgggghhhhiiiijjjjkkkkllllmmmmnnnnoooopppp_3', TRUE, NULL, '2025-08-21 16:00:00'),
('c4aaddd4-aaaa-bbbb-cccc-ddddeeeeeeee', '44444444-4444-4444-4444-444444444444', 'rtok_aaaabbbbccccddddeeeeffffgggghhhhiiiijjjjkkkkllllmmmmnnnnoooopppp_4', FALSE, '2025-07-15 10:00:00', '2025-08-21 18:00:00'),
('c5aaddd5-aaaa-bbbb-cccc-ddddeeeeeeee', '55555555-5555-5555-5555-555555555555', 'rtok_aaaabbbbccccddddeeeeffffgggghhhhiiiijjjjkkkkllllmmmmnnnnoooopppp_5', TRUE, NULL, '2025-08-21 20:00:00');

-- Sample Notifications
INSERT INTO notifications (id, user_id, title, message, channel, status, is_read, send_at, read_at) VALUES
('d1ca0001-aaaa-bbbb-cccc-ddddeeeeeeee', '33333333-3333-3333-3333-333333333333', 'Laporan Banjir Divalidasi', 'Laporan banjir di Kampung Melayu telah divalidasi oleh relawan.', 'app', 'sent', TRUE, '2025-07-20 16:00:00', '2025-07-20 16:30:00'),
('d2ca0002-aaaa-bbbb-cccc-ddddeeeeeeee', '33333333-3333-3333-3333-333333333333', 'Bantuan Telah Tiba', 'Bantuan untuk korban banjir di Kampung Melayu telah tiba di lokasi.', 'email', 'sent', TRUE, '2025-07-20 18:00:00', '2025-07-20 18:15:00'),
('d3ca0003-aaaa-bbbb-cccc-ddddeeeeeeee', '77777777-7777-7777-7777-777777777777', 'Laporan Gempa Divalidasi', 'Laporan gempa di Lombok Utara telah divalidasi oleh relawan.', 'app', 'sent', TRUE, '2025-07-19 09:30:00', '2025-07-19 10:00:00'),
('d4ca0004-aaaa-bbbb-cccc-ddddeeeeeeee', '77777777-7777-7777-7777-777777777777', 'Komentar Baru', 'Ada komentar baru pada laporan gempa Anda.', 'app', 'sent', FALSE, '2025-07-19 15:00:00', NULL),
('d5ca0005-aaaa-bbbb-cccc-ddddeeeeeeee', '22222222-2222-2222-2222-222222222222', 'Penugasan Baru', 'Anda telah ditugaskan untuk memvalidasi laporan banjir di Bidara Cina.', 'email', 'sent', TRUE, '2025-07-20 14:00:00', '2025-07-20 14:10:00'),
('d6ca0006-aaaa-bbbb-cccc-ddddeeeeeeee', '66666666-6666-6666-6666-666666666666', 'Penugasan Baru', 'Anda telah ditugaskan untuk memvalidasi laporan gempa susulan di Palu.', 'email', 'sent', TRUE, '2025-07-15 21:00:00', '2025-07-15 21:05:00'),
('d7ca0007-aaaa-bbbb-cccc-ddddeeeeeeee', '55555555-5555-5555-5555-555555555555', 'Laporan Bencana Baru', 'Ada laporan banjir baru di Jakarta yang memerlukan verifikasi.', 'app', 'sent', TRUE, '2025-07-20 08:00:00', '2025-07-20 08:30:00'),
('d8ca0008-aaaa-bbbb-cccc-ddddeeeeeeee', '44444444-4444-4444-4444-444444444444', 'Analisis Diperlukan', 'Mohon lakukan analisis dampak gempa di Lombok Utara.', 'email', 'sent', TRUE, '2025-07-19 10:00:00', '2025-07-19 10:15:00'),
('d9ca0009-aaaa-bbbb-cccc-ddddeeeeeeee', '33333333-3333-3333-3333-333333333333', 'Update Status Bencana', 'Status banjir di Kampung Melayu telah diperbarui menjadi "contained".', 'app', 'queued', FALSE, '2025-07-21 08:00:00', NULL),
('daca0010-aaaa-bbbb-cccc-ddddeeeeeeee', '11111111-1111-1111-1111-111111111111', 'Laporan Sistem', 'Terdapat 5 laporan baru dalam 24 jam terakhir yang memerlukan verifikasi.', 'app', 'sent', FALSE, '2025-07-21 07:00:00', NULL);

-- Sample Resources (jika belum ada)
-- Diasumsikan tabel resources sudah memiliki data sampel yang lengkap sesuai snippet yang diberikan
