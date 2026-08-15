// luppo-core/src/error.rs

// Hata yönetimini otomatikleştiren crate'leri kullanıyoruz.
use std::io;
use thiserror::Error; // Bu crate'in Cargo.toml'da [dependencies] altında tanımlı olduğundan emin olun.

/// Projedeki tüm hata tiplerini sarmalayan sonuç tipi.
pub type LuppoResult<T> = Result<T, LuppoError>;

// `thiserror::Error` derive'ı, From trait'ini ve `std::error::Error` trait'ini otomatik olarak uygular.
#[derive(Debug, Error)]
pub enum LuppoError {
    /// Çalışma zamanında karşılaşılan mantıksal hatalar.
    #[error("Runtime Error: {0}")]
    RuntimeError(String),

    /// Paket spesifikasyonunu okuma veya ayrıştırma hataları.
    #[error("Spec Parsing Error: {0}")]
    SpecError(String),

    /// Dosya sistemi ve I/O hataları.
    #[error("I/O Error: {0}")]
    IoError(#[from] io::Error), // io::Error'dan dönüşümü otomatikleştirir

    /// Veritabanı hataları (sled).
    #[error("Database Error: {0}")]
    DatabaseError(#[from] sled::Error), // 👈 1. Düzeltme: sled::Error'dan dönüşümü otomatikleştirir

    /// Serileştirme/Deserileştirme hataları (bincode).
    #[error("Serialization Error: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>), // 👈 2. Düzeltme: bincode hatasından dönüşümü otomatikleştirir

    #[error("Cycle Dependency Error:\n{0}")]
    CycleDependency(String),

    /// Kurulu bir paketle çakışma durumu
    #[error("Installed package conflict: {package} conflicts with {conflicting_package} (installed).")]
    InstalledConflict {
        package: String,
        conflicting_package: String,
    },

    /// Kurulum planındaki başka bir paketle çakışma durumu
    #[error("Plan conflict: {package} conflicts with {conflicting_package} (in installation plan).")]
    PlannedConflict {
        package: String,
        conflicting_package: String,
    },
}

impl From<String> for LuppoError {
    fn from(s: String) -> Self {
        LuppoError::RuntimeError(s) // String'den gelen hataları genel Runtime hatası olarak sarmala
    }
}
