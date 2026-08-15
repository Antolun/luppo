use luppo_spec::models::*;

fn main() {
    let spec = LuppoSpec::from_path("/home/luppocik/RUST/projeler/avahi/lopec.kdl").unwrap_or_else(|_| {
        LuppoSpec::from_path("/home/luppocik/RUST/projeler/avahi/lopec.xml").unwrap()
    });
    if let Some(deps) = spec.source.build_dependencies {
        println!("deps count from luppo-spec: {:?}", deps.dependencies.len());
        for d in deps.dependencies {
            println!(" - {}", d.name);
        }
    } else {
        println!("deps count from luppo-spec: None");
    }
}
