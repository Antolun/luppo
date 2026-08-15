use luppo_core::package::Package;

fn main() {
    println!("=== DB read test ===");
    // Try reading the actual database
    let db = sled::open("/var/lib/luppo/db").expect("failed to open db");
    let packages = db.open_tree("repo_packages").unwrap();
    
    for item in packages.iter().take(5) {
        let (k, v) = item.unwrap();
        let name = String::from_utf8_lossy(&k);
        match zstd::decode_all(&v[..]) {
            Err(e) => {
                println!("[{}] zstd FAILED: {}", name, e);
            }
            Ok(decompressed) => {
                let hex: String = decompressed.iter().take(32).map(|b| format!("{:02x} ", b)).collect();
                println!("[{}] zstd OK, {} bytes. First bytes: {}", name, decompressed.len(), hex);
                match bincode::deserialize::<Package>(&decompressed) {
                    Ok(p) => {
                        println!("  bincode OK: name={}, summaries={}", p.name, p.summaries.len());
                    }
                    Err(e) => {
                        println!("  bincode FAILED: {:?}", e);
                    }
                }
            }
        }
    }
}
