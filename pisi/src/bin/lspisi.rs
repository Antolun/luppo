use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lspisi <package.pisi>");
        exit(1);
    }

    let pkg_path = &args[1];
    if !Path::new(pkg_path).exists() {
        eprintln!("File not found: {}", pkg_path);
        exit(1);
    }

    match pisi_core::packager::Packager::read_package(pkg_path) {
        Ok(pkg_data) => {
            let mut paths: Vec<String> = pkg_data.files.iter().map(|f| f.path.clone()).collect();
            paths.sort();
            for path in paths {
                println!("/{}", path);
            }
        }
        Err(e) => {
            eprintln!("Error reading package: {}", e);
            exit(1);
        }
    }
}
