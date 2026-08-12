use pisi_spec::models::*;

fn main() {
    let spec = PisiSpec::from_path("/home/pisicik/RUST/projeler/avahi/pspec.kdl").unwrap_or_else(|_| {
        PisiSpec::from_path("/home/pisicik/RUST/projeler/avahi/pspec.xml").unwrap()
    });
    if let Some(deps) = spec.source.build_dependencies {
        println!("deps count from pisi-spec: {:?}", deps.dependencies.len());
        for d in deps.dependencies {
            println!(" - {}", d.name);
        }
    } else {
        println!("deps count from pisi-spec: None");
    }
}
