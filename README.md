# RedToken - Honeytoken Injector & Intrusion Detector

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.60%2B-orange" alt="Rust 1.60+">
  <img src="https://img.shields.io/badge/License-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/Status-Production Ready-green" alt="Status">
</p>

RedToken adalah alat keamanan yang dirancang untuk profesional keamanan dalam menyisipkan honeytokens (kredensial palsu) ke berbagai jenis file dan memantau penggunaannya yang tidak sah. Ketika token tersebut diakses atau digunakan, peringatan akan dikirim melalui saluran notifikasi yang dapat dikonfigurasi, membantu mendeteksi dan menganalisis upaya intrusi.

## 🔍 Fungsi RedToken

RedToken memiliki beberapa fungsi utama:

1. **Penyisipan Honeytokens**:

   - Menyisipkan kredensial palsu (API keys, tokens, passwords) ke file konfigurasi
   - Token ini berfungsi sebagai "jebakan" untuk mendeteksi akses tidak sah pada sistem
   - Jika penyerang mendapatkan akses ke sistem dan mencoba menggunakan token, aktivitas tersebut dapat terdeteksi

2. **Deteksi Intrusi**:

   - Ketika token palsu digunakan (misalnya untuk API call atau akses database)
   - Server web memantau penggunaan token dan memicu alert
   - Memungkinkan deteksi dini pada fase awal serangan

3. **Notifikasi Otomatis**:
   - Mengirim alert real-time ketika token terdeteksi digunakan
   - Support untuk multiple channel notifikasi (Telegram, Discord, Email)
   - Memberikan informasi detail tentang token yang digunakan dan waktunya

## ⚙️ Cara Kerja RedToken

RedToken bekerja dengan cara sebagai berikut:

### 1. Fase Penyisipan Token

1. RedToken menganalisis tipe file target (.env, JSON, YAML, bash_history)
2. Membuat backup file asli untuk keamanan
3. Menghasilkan token yang tampak legitimate dengan format sesuai konteks
4. Menyisipkan token ke file dengan cara yang tidak merusak struktur file
5. Menyimpan metadata token (ID, nilai, path file) ke database lokal

```
Contoh Token dalam .env:
API_TOKEN_342=RT_a7bX9cD45eF2gH3i
```

### 2. Fase Pemantauan

1. RedToken menjalankan server web (default di port 8080)
2. Server berfungsi sebagai endpoint untuk memeriksa token
3. Ketika token digunakan (misalnya oleh penyerang yang menemukan kredensial), permintaan biasanya akan dikirim ke endpoint ini
4. Server mengidentifikasi token yang digunakan dan mencatatnya

```
Endpoint Pemantauan: http://localhost:8080/api/check?token=RT_a7bX9cD45eF2gH3i
```

### 3. Fase Peringatan

1. Setelah token terdeteksi digunakan, RedToken memeriksa database untuk validasi
2. Jika token ditemukan, RedToken menandainya sebagai "triggered"
3. Alert segera dikirimkan ke semua saluran notifikasi yang dikonfigurasi
4. Alert berisi informasi detail (ID token, file sumber, waktu)

```
🚨 ALERT: Honeytoken triggered!
Token ID: 550e8400-e29b-41d4-a716-446655440000
File Path: /home/user/.env
Triggered: 2023-07-15 14:30:22
```

## ✨ Keunggulan Fitur

- **Penyisipan Token Cerdas**:

  - Otomatis menyisipkan kredensial palsu yang terlihat realistis ke file konfigurasi
  - Menghormati format dan struktur file
  - Support multi-format: .env, JSON, YAML, bash_history, dan tipe file kustom

- **Deteksi Intrusi Real-time**:

  - Monitoring HTTP ping atau API untuk penggunaan token
  - Alert langsung ketika token digunakan
  - Tracking untuk semua token aktif

- **Sistem Notifikasi Terpadu**:

  - Telegram: notifikasi instant messaging
  - Discord: webhook untuk channel server
  - Email: notifikasi ke alamat email tujuan

- **Sistem Backup Otomatis**:

  - Backup file sebelum modifikasi untuk mencegah kehilangan data
  - Format nama file backup dengan timestamp

- **Logging Komprehensif**:

  - Audit trail dan logging aktivitas
  - Detail tentang kapan token disuntikkan dan diakses

- **CLI & API Professional**:
  - Antarmuka command-line lengkap
  - API HTTP untuk integrasi dengan tools lain

## 🚀 Memulai

### Instalasi

```bash
# Clone repository
git clone https://github.com/kepinserius/RedToken.git
cd RedToken

# Build versi release yang optimal
cargo build --release

# Install secara global (opsional)
cargo install --path .
```

### Penggunaan Dasar

#### Penyisipan Token

```bash
# Menyisipkan token ke file .env
redtoken inject --file .env

# Menyisipkan dengan nilai token kustom
redtoken inject --file config.json --value "my-secret-token" --file-type json

# Menyisipkan ke file konfigurasi YAML
redtoken inject --file config.yml
```

#### Pemantauan & Manajemen

```bash
# Memulai server web pemantauan
redtoken serve --port 8080

# Menampilkan semua token aktif
redtoken list

# Menghapus token tertentu
redtoken remove --id <token-id>
```

#### Konfigurasi Notifikasi

```bash
# Setup notifikasi Telegram
redtoken configure --telegram "https://api.telegram.org/bot<token>/sendMessage?chat_id=<chat_id>"

# Setup webhook Discord
redtoken configure --discord "https://discord.com/api/webhooks/<webhook-id>/<token>"

# Setup notifikasi Email
redtoken configure --email "smtp://user:pass@smtp.example.com:587/from@example.com/to@example.com"
```

## 🔧 Arsitektur

RedToken dibangun dengan arsitektur Clean Architecture yang terdiri dari:

1. **Core Layer** (Domain):

   - `token.rs`: Model inti untuk honeytokens
   - `notification.rs`: Interface untuk notifikasi
   - `injection.rs`: Interface untuk penyisipan file
   - `error.rs`: Handling error terpusat

2. **Application Layer**:

   - `service.rs`: Layanan aplikasi utama
   - `config.rs`: Manajemen konfigurasi

3. **Infrastructure Layer**:

   - `repository.rs`: Penyimpanan token
   - `notification.rs`: Implementasi notifikasi
   - `injection.rs`: Implementasi penyisipan file

4. **Interface Layer**:
   - `cli.rs`: Command-line interface
   - `web.rs`: Web API dan server

## 📋 Penggunaan yang Aman

RedToken dirancang khusus untuk tujuan pengujian keamanan dan edukasi. Penggunaan alat ini pada sistem tanpa otorisasi yang tepat adalah ilegal dan tidak etis.

- **Selalu** gunakan di lingkungan terkontrol
- **Jangan pernah** menyisipkan token ke sistem produksi tanpa manajemen perubahan yang tepat
- **Dokumentasikan** semua penempatan token untuk referensi di masa depan

## 📄 Lisensi

MIT License. Lihat file [LICENSE](LICENSE) untuk detail.
