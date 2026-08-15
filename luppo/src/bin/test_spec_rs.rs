use luppo_spec::models::LuppoSpec;

fn main() {
    for path in &[
        "/home/luppocik/RUST/projeler/avahi/lopec.xml",
    ] {
        print!("Testing {:>55} → ", path);
        match LuppoSpec::from_path(path) {
            Ok(spec) => println!("✅ OK ({} paket)", spec.packages.len()),
            Err(e) => println!("❌ HATA: {}", e),
        }
    }
}
