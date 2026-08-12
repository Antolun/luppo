use std::fs;
use std::path::Path;

fn visit_kdl(dir: &Path) {
    if !dir.is_dir() { return; }
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            visit_kdl(&path);
        } else if path.is_file() {
            let fname = path.file_name().unwrap().to_str().unwrap();
            if !matches!(fname, "pspec.kdl" | "pisi.kdl" | "pisi_template.kdl" | "comar.kdl" | "mudur.kdl" | "freetype.kdl") {
                continue;
            }
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => { eprintln!("❌ {}: read error: {}", path.display(), e); continue; }
            };
            match pisi_spec::kdl::parse_kdl_spec_from_str(&content) {
                Ok(spec) => {
                    let kdl = spec.to_kdl_string();
                    fs::write(&path, &kdl).unwrap_or_else(|e| panic!("Failed to write {}: {}", path.display(), e));
                    println!("✅ {} ({} pkgs, {} history)", path.display(), spec.packages.len(),
                        spec.history.as_ref().map(|h| h.updates.len()).unwrap_or(0));
                }
                Err(e) => eprintln!("❌ {}: {}", path.display(), e),
            }
        }
    }
}

fn main() {
    visit_kdl(Path::new("/media/pisicik/DEPO/PISILINUX/PisiLinux_docker/core/system/base"));
    visit_kdl(Path::new("/media/pisicik/DEPO/RUST/projeler/pisi"));
}
