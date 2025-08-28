# Analisis Arsitektur Terra Siaga (Clean Architecture + PASETO)

Tanggal: 2025-08-12
Penulis: Junie (JetBrains Autonomous Programmer)

## Ringkasan Eksekutif
Proyek telah mengadopsi pola Clean Architecture dengan pemisahan layer yang cukup jelas (domain, application/use-cases, infrastructure, presentation, shared). Integrasi keamanan menggunakan PASETO (pasetors) sudah tersedia, namun terduplikasi pada dua modul berbeda sehingga berpotensi menimbulkan inkonsistensi API dan kebingungan dependensi. Modul Monitoring/Health telah berevolusi menjadi layanan asinkron, tetapi sebagian kode masih mengasumsikan API sinkron.

Perubahan kecil berdampak tinggi yang telah dilakukan pada sesi ini:
- Menyelaraskan endpoint Health (presentation/api/health.rs) agar menggunakan API HealthService asinkron terkini dan menambahkan tracing pada health_check.

Rekomendasi strategis inti:
- Konsolidasikan otentikasi pada satu jalur: gunakan infrastructure/security/PasetoSecurityService sebagai standar (sesuai pilihan Anda: Opsi B), dan deprekasi atau jadikan shared/paseto_auth.rs sebagai adaptor/DTO minimal.
- Normalkan penggunaan Monitoring: gunakan re-exports dari `infrastructure::` dan API asinkron secara konsisten di main.rs, container, dan routes.
- Selaraskan kontrak domain ports (AuthService, Repository) dengan implementasi nyata di infrastructure untuk menghilangkan error kompilasi.

## Pemetaan Lapisan (Clean Architecture)
- Domain
  - ports/: kontrak layanan dan repositori (services.rs, repositories.rs, dll.)
  - entities/, value_objects/: tipe inti domain
- Application
  - use_cases/: orkestrasi kasus penggunaan; bergantung pada domain ports
- Infrastructure
  - database/, cache/, external_services/, repository/: implementasi ports
  - monitoring/: HealthService asinkron terkini
  - security/: PasetoSecurityService sebagai implementasi otentikasi dan hashing
  - container.rs: komposisi dependency
- Presentation
  - api/: handler Actix-Web, termasuk health.rs
- Shared
  - error.rs, types.rs, cache, rate_limiter, auth_middleware, paseto_auth.rs (duplikasi jalur PASETO)

Struktur ini umumnya sesuai Clean Architecture: domain bersih, application tipis, infrastructure sarat implementasi, presentation untuk I/O HTTP. Area yang perlu dirapikan: duplikasi security dan inkonsistensi kontrak.

## Keamanan & Otentikasi (PASETO)
Terdapat dua implementasi:
1) shared/paseto_auth.rs (PasetoService, TokenClaims, TokenType, TokenPair) + shared/auth_middleware.rs
   - Mengelola pembuatan dan verifikasi token PASETO (v4.local), termasuk claim yang kaya (role, permissions, session_id, device_fingerprint, ip_address).
   - Digunakan langsung oleh middleware untuk validasi token dan kontrol izin/role.
2) infrastructure/security/paseto_service.rs (PasetoSecurityService)
   - Mengimplementasi domain::ports::services::AuthService (hash/verify password, generate/validate/revoke token), terintegrasi dengan cache untuk manajemen sesi, dan di-wire dalam AppContainer.

Masalah:
- Duplikasi fungsi dan tipe (TokenPair/Claims) pada dua modul berbeda.
- Perbedaan tanggung jawab: versi infrastructure mencakup hashing (argon2) dan manajemen sesi; versi shared fokus token + middleware.
- Ketidakselarasan kontrak: AuthService di domain berisi sejumlah metode (generate_tokens, refresh_token) sementara implementasi di infrastructure baru meng-impl sebagian (generate_token, validate_token, revoke_token). Hal ini menimbulkan error kompilasi saat `cargo test`.

Keputusan Strategis (disepakati: Opsi B):
- Standarisasi ke infrastructure/security/PasetoSecurityService sebagai satu-satunya sumber kebenaran untuk otentikasi.
- shared/paseto_auth.rs:
  - Jadikan adaptor/DTO tipis bila masih diperlukan oleh middleware; atau deprekasi bertahap.
  - Middleware sebaiknya berinteraksi melalui port domain (AuthService) sehingga tetap testable dan terlepas dari detail pasetors.
- Selaraskan kontrak AuthService di domain agar sesuai kapabilitas yang diinginkan:
  - Tentukan dan finalkan signature berikut (contoh):
    - hash_password(password) -> AppResult<String>
    - verify_password(password, hash) -> AppResult<bool>
    - generate_tokens(user_id: UserId) -> AppResult<TokenPair>
    - refresh_token(refresh_token: &str) -> AppResult<TokenPair>
    - validate_token(token: &str) -> AppResult<UserId atau Claims>
    - revoke_token(token/session_id) -> AppResult<()>
  - Pastikan implementasi PasetoSecurityService melengkapi semua metode di atas.
- Satukan tipe TokenPair/TokenClaims agar hanya ada satu definisi (disarankan didefinisikan di domain ports atau shared types, bukan ganda).

Catatan Keamanan:
- Pastikan kunci simetris PASETO minimal 32 byte (sudah dicek pada PasetoService shared). Terapkan validasi serupa/lebih kuat di PasetoSecurityService.
- Gunakan expiry berbeda untuk access vs refresh token; simpan session_id di cache untuk revocation.
- Hindari kebocoran informasi sensitif pada endpoint health/readiness.

## Monitoring & Kesehatan Layanan
- Infrastruktur memiliki HealthService asinkron modern (check_health().await, quick_check().await, readiness_check().await).
- Re-exports pada infrastructure/mod.rs menyediakan type alias yang lebih sederhana untuk diimpor.
- Sebagian kode (sebelum perbaikan) masih mengasumsikan metode sinkron dan path modul lama.

Rekomendasi:
- Gunakan import dari `crate::infrastructure::{HealthService, HealthStatus, ...}` untuk konsistensi.
- Gunakan factory `create_health_service(...)` untuk menyusun health checks berdasarkan konfigurasi.
- Pastikan presentation/api/health.rs memanggil metode asinkron. (Sudah disesuaikan pada sesi ini.)
- Tambahkan tracing ringan pada endpoint health (sudah ditambahkan pada health_check). Pertimbangkan menambahkan tracing di readiness/liveness bila diperlukan.

## Repositori & Port Domain
- Hasil `cargo test` menunjukkan ketidaksesuaian antara trait repositori domain dan implementasi di infrastructure (contoh: method find_by_id/save tidak ada dalam trait atau namanya berbeda).
- Tindakan yang disarankan:
  - Audit domain/ports/repositories.rs dan implementasi di infrastructure/repository/* untuk menyelaraskan method (nama, signature, tipe return).
  - Untuk sementara, pilih salah satu: (a) perluas trait agar mencakup method yang dibutuhkan implementasi, atau (b) sesuaikan implementasi agar mengikuti trait yang ada.
  - Tambahkan test kontrak (mockall) untuk menjaga kesesuaian ke depan.

## Observability
- Logging/tracing telah digunakan (tracing + tracing-subscriber), dan Prometheus exporter disiapkan di main.
- Rekomendasi tambahan ringan:
  - Tambahkan tracing span di jalur autentikasi (validasi token) dan jalur kritikal (dispatch/emergency). 
  - Simpan metric sederhana untuk health overall_status per permintaan.

## Backlog Perbaikan Bertahap (Rencana Aksi)
1) Konsolidasi Security (Opsi B) [Prioritas Tinggi]
   - Selaraskan domain AuthService dengan fitur yang dibutuhkan (generate/refresh/validate/revoke token, hash/verify password).
   - Lengkapi PasetoSecurityService agar memenuhi kontrak yang sama.
   - Adaptasi shared/auth_middleware agar bergantung pada port AuthService, bukan langsung ke shared/paseto_auth.
   - Deprekasi shared/paseto_auth atau ubah menjadi adaptor/DTO tipis.
2) Normalisasi Monitoring
   - Perbarui main.rs dan container agar memakai re-exports dari infrastructure dan API health asinkron.
   - Gunakan `create_health_service` untuk konfigurasi yang lebih bersih.
3) Selaraskan Repositori
   - Audit trait dan implementasi; samakan method dan kontrak error.
   - Tambahkan test kontrak.
4) Kualitas Kode & Keamanan
   - clippy + rustfmt pipeline
   - Audit penggunaan zeroize untuk material sensitif
   - Pastikan konfigurasi kunci PASETO diambil aman dari env/secret manager
5) Dokumentasi
   - Update README/CHANGELOG setelah konsolidasi security & monitoring.

## Risiko & Mitigasi
- Risiko regresi saat konsolidasi security: mitigasi dengan test unit untuk sign/verify token dan test middleware.
- Risiko perubahan luas pada repositori: mitigasi dengan test kontrak dan perubahan bertahap.
- Perbedaan API health lama vs baru: mitigasi dengan type alias dan re-exports yang konsisten.

## Penutup
Proyek telah berada di arah yang benar dengan Clean Architecture. Fokus berikutnya adalah menghilangkan duplikasi implementasi PASETO, menormalkan API Monitoring asinkron di seluruh aplikasi, dan menyelaraskan kontrak domain dengan implementasi sehingga kompilasi stabil serta memudahkan pengembangan selanjutnya.
