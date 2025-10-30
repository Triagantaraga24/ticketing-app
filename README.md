# Ticketing App - Rust, Rocket, & MongoDB

Aplikasi web ticketing sederhana yang dibangun dengan Rust (framework Rocket), MongoDB, dan mengintegrasikan Midtrans untuk pembayaran serta Resend untuk pengiriman email.

## 🎯 Fitur Utama

- **Dua Role Pengguna:**
  - **Guest (Publik):** Melihat daftar event, detail event, dan membeli tiket tanpa perlu login.
  - **Admin:** Login untuk mengelola event (CRUD), melihat semua pesanan, dan mengirim tiket ke pembeli.
- **Autentikasi Admin:** Login dengan email & password, password di-hash dengan bcrypt, dan dilindungi dengan JWT.
- **Pembayaran Terintegrasi:** Menggunakan Midtrans (sandbox) untuk proses pembayaran.
- **Notifikasi Webhook:** Menerima notifikasi status pembayaran dari Midtrans.
- **Pengiriman Tiket:** Admin dapat mengirim tiket secara manual ke email pembeli menggunakan Resend API.
- **Seeding Data:** Script otomatis untuk membuat akun admin dan event contoh.

## 🔄 Alur Pembelian Tiket (Guest Flow)

Alur ini dirancang agar pengguna dapat membeli tiket dengan cepat tanpa hambatan pendaftaran akun.

1.  **Memilih Event:** Pengguna membuka aplikasi, melihat daftar event di halaman utama (`/api/events`), dan memilih salah satu event untuk melihat detailnya (`/api/events/:id`).
2.  **Mengisi Data Diri:** Saat pengguna menekan tombol "Buy Ticket", sebuah form muncul memasukkan nama lengkap, email, dan nomor telepon.
3.  **Memproses Pembayaran:** Setelah form disubmit ke endpoint `/api/orders`, backend:
    - Membuat record pesanan baru dengan status `pending`.
    - Mengirimkan data pesanan ke Midtrans untuk membuat transaksi pembayaran.
    - Menerima token dan URL redirect dari Midtrans.
    - Mengembalikan token ke frontend untuk ditampilkan dalam Snap.js popup.
4.  **Pembayaran Sukses:** Pengguna menyelesaikan pembayaran di halaman Midtrans. Jika berhasil, Midtrans akan mengirimkan notifikasi (webhook) ke endpoint `/api/orders/notify` di backend kita.
5.  **Update Status:** Backend menerima webhook, memverifikasi, dan mengubah status pesanan menjadi `paid`.
6.  **Pengiriman Tiket (Manual):** Admin melihat pesanan yang sudah dibayar di panel admin (`/api/admin/orders`). Admin kemudian dapat menekan tombol "Send Ticket", yang akan memicu endpoint `/api/admin/orders/:id/send_ticket` untuk mengirimkan detail tiket ke email pembeli via Resend API.

## 🚀 Setup & Menjalankan Secara Lokal (Windows PowerShell)

### Prasyarat

-   **Rust:** [Install Rust](https://rustup.rs/)
-   **MongoDB:** [Install MongoDB Community Server](https://www.mongodb.com/try/download/community) dan pastikan service-nya berjalan.
-   **Node.js & NPM:** [Install Node.js](https://nodejs.org/) (diperlukan untuk testing).
-   **Akun Midtrans & Resend:** Dapatkan kunci API (Server Key, Client Key) dari dashboard Midtrans (mode Sandbox) dan kunci API dari Resend.

### Langkah-langkah

1.  **Clone Repository**
    ```powershell
    git clone https://github.com/Triagantaraga24/ticketing-app.git
    cd ticketing-app
    ```

2.  **Konfigurasi Environment Variable**
    -   Salin file `.env.example` menjadi `.env`.
    ```powershell
    copy .env.example .env
    ```
    -   Buka file `.env` dan isi dengan kredensial Anda:
    ```ini
    # Database
    MONGODB_URI="mongodb://localhost:27017/ticketing_db"

    # JWT
    JWT_SECRET="supersecretkeythatshouldbeinenvfile"

    # Admin Default
    ADMIN_EMAIL="admin@ticketing.local"
    ADMIN_PASSWORD="Admin123!"

    # Midtrans (Sandbox)
    MIDTRANS_SERVER_KEY="SB-Mid-server-..."
    MIDTRANS_CLIENT_KEY="SB-Mid-client-..."

    # Resend
    RESEND_API_KEY="re_..."
    RESEND_FROM_EMAIL="onboarding@resend.dev" # Gunakan email yang terverifikasi di Resend
    ```

3.  **Menjalankan Seed Script**
    Script ini akan membuat akun admin default dan beberapa event contoh di database Anda.
    ```powershell
    cargo run --bin seed
    ```
    Anda akan melihat output seperti ini:
    ```
    ✅ Admin user created:
       Email: admin@ticketing.local
       Password: Admin123!
    🎟️ Sample events added:
       - Jakarta Music Fest
       - Comedy Night
    🎉 Seeding complete!
    ```

4.  **Menjalankan Aplikasi**
    Jalankan server backend.
    ```powershell
    cargo run
    ```
    Server akan berjalan di `http://localhost:8000`.

## 📡 Daftar Endpoint API

| Endpoint                              | Method | Akses  | Deskripsi                                      |
| ------------------------------------- | ------ | ------ | ---------------------------------------------- |
| `/api/events`                         | GET    | Public | Mengambil daftar semua event                   |
| `/api/events/<id>`                    | GET    | Public | Melihat detail event berdasarkan ID            |
| `/api/orders`                         | POST   | Public | Checkout tiket & dapatkan token Midtrans       |
| `/api/orders/notify`                  | POST   | Public | Webhook Midtrans untuk update status pembayaran |
| `/api/admin/login`                    | POST   | Public | Login admin untuk mendapatkan token JWT        |
| `/api/admin/events`                   | GET    | Admin  | Melihat semua event (admin view)               |
| `/api/admin/events`                   | POST   | Admin  | Membuat event baru                             |
| `/api/admin/orders`                   | GET    | Admin  | Melihat semua pesanan                          |
| `/api/admin/orders/<id>/send_ticket`  | POST   | Admin  | Mengirim tiket ke email pembeli                |

*Semua endpoint `/api/admin/*` (kecuali `/login`) memerlukan header `Authorization: Bearer <JWT_TOKEN>`.*
