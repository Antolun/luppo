use std::fs::File;
use std::io::Read;

fn main() {
    let mut file = File::open("pisi-index.xml").unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    
    match serde_xml_rs::from_str::<pisi_core::package::PisiIndex>(&xml) {
        Ok(idx) => {
            println!("Parsed {} packages", idx.packages.len());
            if let Some(pkg) = idx.packages.iter().find(|p| p.name == "nano") {
                println!("Found nano: {:?}", pkg.name);
            } else {
                println!("Nano NOT FOUND!");
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
