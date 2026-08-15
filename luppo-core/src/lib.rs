pub mod builder;
pub mod comar;
pub mod safe_env;
pub mod util;
pub mod config;
pub mod database;
pub mod deb;
pub mod error;
pub mod installer;
pub mod package;
pub mod packager;
pub mod progress;
pub mod query;
pub mod repo;
pub mod resolver;
pub mod version;

rust_i18n::i18n!("../locales", fallback = "tr");

// Hata tipini dışarıya açın
pub use error::LuppoError;

use database::LuppoDatabase;
use std::path::Path;

// LuppoResult'i tüm modüller için kökte tanımlıyoruz.
pub type LuppoResult<T> = Result<T, LuppoError>;

use rust_i18n::t;
// Luppo çekirdek kütüphanesi başlatıcısı.
pub fn initialize_luppo_core<P: AsRef<Path>>(db_path: P) -> Result<LuppoDatabase, String> {
    // db_path'i (P) önce Path referansına (.as_ref()) çeviriyoruz,
    // ardından PathBuf'a (.to_path_buf()) klonluyoruz.
    let path_buf = db_path.as_ref().to_path_buf(); // <<< DÖNÜŞÜM BURADA

    match LuppoDatabase::open(path_buf) {
        // <<< PathBuf kullanıldı
        Ok(db) => Ok(db),
        Err(e) => Err(t!("lib_err_critical", error = e).to_string()),
    }
}

const COLORS: &[(&str, &str)] = &[
    ("black", "\x1b[30m"),
    ("red", "\x1b[31m"),
    ("green", "\x1b[32m"),
    ("yellow", "\x1b[33m"),
    ("blue", "\x1b[34m"),
    ("purple", "\x1b[35m"),
    ("cyan", "\x1b[36m"),
    ("white", "\x1b[37m"),
    ("brightblack", "\x1b[01;30m"),
    ("brightred", "\x1b[01;31m"),
    ("brightgreen", "\x1b[01;32m"),
    ("brightyellow", "\x1b[01;33m"),
    ("brightblue", "\x1b[01;34m"),
    ("brightmagenta", "\x1b[01;35m"),
    ("brightcyan", "\x1b[01;36m"),
    ("brightwhite", "\x1b[01;37m"),
    ("gray", "\x1b[02;37m"),
];

pub fn colorize(text: &str, color: &str) -> String {
    if std::env::var("NO_COLOR").is_ok() {
        text.to_string()
    } else {
        let code = COLORS
            .iter()
            .find(|(name, _)| *name == color)
            .map(|(_, c)| *c)
            .unwrap_or("");
        format!("{}{}\x1b[0m", code, text)
    }
}

pub fn print_in_columns<S: AsRef<str>>(items: &[S]) {
    if items.is_empty() {
        return;
    }

    let term_width = 120; // Default fallback width
    let max_len = items.iter().map(|s| s.as_ref().len()).max().unwrap_or(0);
    let col_width = max_len + 2;
    let num_cols = std::cmp::max(1, term_width / col_width);

    for (i, item) in items.iter().enumerate() {
        print!("{:<width$}", item.as_ref(), width = col_width);
        if (i + 1) % num_cols == 0 {
            println!();
        }
    }
    if items.len() % num_cols != 0 {
        println!();
    }
}
