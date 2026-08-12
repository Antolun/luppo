use pisi_spec::models::PisiSpec;

fn main() {
    for path in &[
        "/home/pisicik/RUST/projeler/avahi/pspec.xml",
    ] {
        print!("Testing {:>55} → ", path);
        match PisiSpec::from_path(path) {
            Ok(spec) => println!("✅ OK ({} paket)", spec.packages.len()),
            Err(e) => println!("❌ HATA: {}", e),
        }
    }
}
