rust_i18n::i18n!("../../locales", fallback = "tr");
fn main() {
    println!("Locale: {}", rust_i18n::locale());
}
