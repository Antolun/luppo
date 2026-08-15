# 🛠️ Luppo Kod Geliştiricileri İçin Mimari & Modül Kılavuzu

Bu kılavuz, **Luppo** projesinin çekirdek kod tabanına (codebase) katkı sağlamak, yeni özellikler eklemek ve mevcut modülleri optimize etmek isteyen yazılımcılar için hazırlanmış teknik bir mimari belgedir.

---

## 🏗️ Genel Mimari ve Workspace Modülleri

Luppo, Rust'ın `workspace` (çalışma alanı) özelliğini kullanan modüler ve gevşek bağlı (loosely coupled) bir mimariye sahiptir. Kod tabanı 4 ana modüle ayrılmıştır:

```
luppo/ (Workspace Root)
├── luppo/               # 1. CLI Giriş Noktası & Parametre Yönetimi
├── luppo-core/          # 2. Veritabanı, Bağımlılık Çözücü & Ağ Katmanı
├── luppo-builder/       # 3. Derleme Motoru, ActionsAPI & Chroot Sandbox
└── luppo-spec/          # 4. Tarif XML/KDL Okuyucu & Model Doğrulayıcı
```

---

## 1. `luppo` Modülü (CLI Arabirimi)

Bu modül, son kullanıcının uçbirimde çalıştırdığı ana binary dosyasıdır. Sorumluluğu yalnızca parametreleri yakalamak, doğrulamak ve çekirdek kütüphaneyi tetiklemektir.

- **Ana Kütüphaneler:** `clap` (parametre ayrıştırma), `rust-i18n` (yerelleştirme).
- **Klasör Yapısı:**
  - `src/main.rs`: Giriş noktası (`fn main`). Parametreleri yakalar ve alt modüllere dağıtır.
  - `src/toolchain.rs`: Chroot chroot kurma (`--start`), chroot altında bootstrap güncelleme (`--update`) ve sanal dosya sistemi mount işlemlerini yönetir.

### Geliştirici İpucu: Yeni Alt Komut Ekleme

Parametre eklemek için `src/main.rs` içindeki `Commands` enum'ını genişletmeniz ve `match` bloğunda ilgili `luppo-core` veya `luppo-builder` fonksiyonunu çağırmanız gerekir.

---

## 2. `luppo-core` Modülü (Sistem Çekirdeği)

Luppo'in beynidir. İş mantığının (business logic), veritabanı işlemlerinin ve bağımlılık çözümlerinin tamamı burada döner.

### A. Veritabanı Yönetimi (`database.rs`)

- **Teknoloji:** SQLite (bağlantı yönetimi `rusqlite` kütüphanesiyle yapılır).
- **Veritabanı Yolu:** Varsayılan olarak `/var/lib/luppo/luppo.db` adresindedir.
- **Sorumluluklar:**
  - Kurulu paketlerin takibi (`installed_packages` tablosu).
  - Paket dosyalarının bütünlük takibi (`files` tablosu).
  - **İşlem Geçmişi (History Rollback):** Kullanıcı `luppo hs -t <no>` ile sistemi eski bir ana döndürdüğünde, geçmiş verileri silinmez. Sadece hedef durumla mevcut durum arasındaki farklar hesaplanıp paketler kurulur veya kaldırılır.

### B. Bağımlılık Çözücü (`resolver.rs`)

- **Algoritma:** Yönlü Çevrimsiz Graf (DAG) ve Kahn Algoritması (Topological Sort).
- **Döngü Yönetimi:** Paketler arasındaki döngüsel bağımlılıkları (circular dependencies) algılar, chroot Chroot bootstrap setinde tanımlı paketleri önceliklendirerek döngüleri güvenli şekilde kırar.

### C. Depo Yönetimi (`repo.rs`)

- **Sorumluluk:** `luppo-index.xml.xz` dosyalarını indirir, açar (`xz2`), ayrıştırır ve yerel SQLite veritabanına indeksler. Depo listeleme çıktısındaki aktif/pasif renklendirme mantığı buradaki `Repository` struct'ı üzerinden yönetilir.

---

## 3. `luppo-builder` Modülü (İnşa ve Derleme Motoru)

Luppo paketlerinin kaynak kodlardan çekilip chroot altında derlendiği yerdir.

### A. Chroot & Sandbox Katmanı (`sandbox.rs`)

- **Sorumluluk:** Derleme esnasında ana sisteme zarar gelmemesi veya ana sistemdeki kütüphanelerin derlemeyi kirletmemesi için derleme sürecini izole eder.
- **Teknoloji:** Linux `chroot`, `mount --bind` ve `namespaces` API'leri kullanılır.
- **Geliştirici Notu:** Chroot derlemelerinde ana sistem derleyicisine erişim gerektiğinde sandbox geçici olarak devre dışı bırakılabilir (`enable_sandbox: false` ayarıyla).

### B. ActionsAPI Köprüsü (`actionsapi.rs`)

- **Sorumluluk:** Python tabanlı `actions.py` dosyaları çalışırken, içlerindeki `autotools`, `cmake`, `cargo` gibi API çağrılarını Rust tarafındaki yerel komut koşturucularına bağlar.
- Python komutlarını okumak ve Rust tarafında paralel koşturmak için iç içe süreç yönetimi (process management) kullanılır.

---

## 4. `luppo-spec` Modülü (Tarif Ayrıştırıcı)

Luppo paket tanımlarını okuyan, doğrulayan ve belleğe yükleyen parser katmanıdır.

- **Ana Kütüphaneler:** `quick-xml` (XML okumak için ultra hızlı parser), `serde` (KDL ve XML serileştirme).
- **Model Yapıları (`models/`):**
  - `LuppoSpec`: `lopec.xml` belgesinin Rust'taki karşılığıdır.
  - `LuppoSpec`: Modern KDL tabanlı `paket.kdl` yapısını temsil eden struct.
- **Doğrulama (Validation):** XML içindeki tarih formatlarını (`YYYY-MM-DD`), versiyon numaralarını ve zorunlu alanları regex kurallarıyla denetler.

---

### 🛡️ XML İzolasyonu ve "Feature Flag" Stratejisi

XML desteğini ileride tamamen terk etmeyi kolaylaştırmak için, XML ile ilgili tüm ayrıştırma ve model mantığını **bağımsız alt modüllere** bölüyor ve bunları **Cargo Feature** bayrağıyla soyutluyoruz:

#### 1. Alt Modül Bölümlemesi

`luppo-spec` modülü içindeki XML parser kodları, KDL parser kodlarıyla asla karıştırılmamalıdır:

```
luppo-spec/src/
├── lib.rs              # Modül dışa aktarımları
├── kdl/                # Modern KDL parsing mantığı (Temiz & Kalıcı)
│   ├── mod.rs
│   └── models.rs
└── xml/                # Geçici XML parsing mantığı (Gelecekte Silinecek)
    ├── mod.rs
    └── models.rs
```

#### 2. Koşullu Derleme (Conditional Compilation)

`lib.rs` içerisinde XML desteği bir Cargo feature flag'ine bağlanır:

```rust
// luppo-spec/src/lib.rs

// Modern KDL desteği varsayılan ve kalıcıdır
pub mod kdl;

// Legacy XML desteği sadece feature aktifken derlenir
#[cfg(feature = "legacy-xml")]
pub mod xml;
```

#### 3. `Cargo.toml` Yapılandırması

```toml
# luppo-spec/Cargo.toml
[features]
default = ["legacy-xml"]
legacy-xml = ["dep:quick-xml"]

[dependencies]
quick-xml = { version = "0.31.0", optional = true }
serde = { version = "1.0", features = ["derive"] }
kdl = "6.0"
```

> [!TIP]
> **XML Desteğini Tamamen Kapatmak / Kaldırmak:**
> İlerleyen aşamalarda tüm paketler KDL formatına geçtiğinde, XML desteğini devre dışı bırakmak için tek yapılması gereken `Cargo.toml` altındaki `legacy-xml` feature'ını kapatmaktır. Böylece `quick-xml` bağımlılığı derleme sürecine dahi dahil edilmez ve binary boyutu anında küçülür! Tamamen silmek istediğimizde ise sadece `src/xml/` dizinini silmemiz yeterli olacaktır.

---

## 🔄 Modüller Arası Veri Akış Şeması

Bir paketin derlenip kurulması sürecindeki veri akışı şu şekildedir:

```mermaid
graph TD
    A[luppo CLI] -->|Komut parametreleri| B[luppo-core::Resolver]
    B -->|Bağımlılık Ağacı & Sıra| C[luppo-builder::PackageBuilder]
    C -->|Tarif Analizi| D[luppo-spec::LuppoSpec]
    D -->|Derleme Komutları| E[luppo-builder::Sandbox]
    E -->|Paket Paketleme (.luppo)| F[luppo-core::Installer]
    F -->|SQLite Durum Güncelleme| G[(luppo.db)]
```

---

## 🧪 Çekirdek Geliştirici İçin Test ve Kalite Standartları

Yazdığınız kodun çekirdek repoya kabul edilmesi için aşağıdaki kalite standartlarını karşılaması zorunludur:

### 1. Kod Formatı ve Stil Rehberi

Commit yapmadan önce kodunuzu her zaman Rust standart formatına getirin:

```bash
cargo fmt --all
```

### 2. Statik Analiz (Clippy)

Rust derleyicisinin en güçlü araçlarından biri olan Clippy, kodunuzdaki olası mantık hatalarını ve performans kayıplarını tespit eder. Hiçbir Clippy uyarısı kalmamalıdır:

```bash
cargo clippy --workspace -- -D warnings
```

### 3. Hata Yakalama (Error Handling)

Çekirdek modüllerde `unwrap()` veya `expect()` kullanmaktan kaçının. Bunun yerine `LuppoResult` ve `LuppoError` enum yapılarını kullanarak hataları üst katmana güvenle fırlatın:

```rust
// Yanlış:
let db = LuppoDatabase::open(path).unwrap();

// Doğru:
let db = LuppoDatabase::open(path).map_err(|e| LuppoError::DatabaseError(e.to_string()))?;
```

Bu rehberdeki standartlara uyarak Luppo çekirdeğini son derece kararlı, güvenli ve performanslı tutabiliriz. Katkılarınız için şimdiden teşekkürler!
