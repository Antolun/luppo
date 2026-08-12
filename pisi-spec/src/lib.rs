pub mod kdl;

#[cfg(feature = "legacy-xml")]
pub mod xml;

pub use kdl::models;

rust_i18n::i18n!("../locales", fallback = "tr");

use std::path::Path;
use kdl::models::PisiSpec;

/// Belirtilen yoldaki 'package.kdl' dosyasını ayrıştırır ve bir PisiSpec yapısı döndürür.
pub fn parse_kdl_spec<P: AsRef<Path>>(path: P) -> Result<PisiSpec, String> {
    kdl::parse_kdl_spec(path)
}
