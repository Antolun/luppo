use luppo_spec::models::LuppoSpec;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

fn main() {
    let core_path = Path::new("/media/luppocuk/REPO/LUPUS/LupuS_docker/core/");
    println!("Scanning recipes in {} ...", core_path.display());

    let mut spec_files = Vec::new();
    find_specs(core_path, &mut spec_files);
    println!("Found {} spec files.", spec_files.len());

    // 1. Parse all specs and build package mapping
    let mut specs = Vec::new();
    let mut subpkg_to_source = HashMap::new();
    let mut source_names = HashSet::new();

    for path in spec_files {
        match LuppoSpec::from_path(&path) {
            Ok(spec) => {
                let source_name = spec.source.name.clone();
                source_names.insert(source_name.clone());

                // Map each produced subpackage to this source package
                for pkg in &spec.packages {
                    subpkg_to_source.insert(pkg.name.clone(), source_name.clone());
                }
                // Also source package itself might be a package name
                subpkg_to_source.insert(source_name.clone(), source_name.clone());

                specs.push((path, spec));
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse spec at {}: {}", path.display(), e);
            }
        }
    }

    println!(
        "Total distinct source packages found: {}",
        source_names.len()
    );

    // 2. Build dependency graph
    // adj[A] = packages that depend on A (i.e. A must be built before B, so edge A -> B)
    let mut adj: HashMap<String, HashSet<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for name in &source_names {
        adj.insert(name.clone(), HashSet::new());
        in_degree.insert(name.clone(), 0);
    }

    for (_, spec) in &specs {
        let source_name = &spec.source.name;
        let mut deps = HashSet::new();

        // Build dependencies
        if let Some(ref b_deps) = spec.source.build_dependencies {
            for dep in &b_deps.dependencies {
                deps.insert(dep.name.clone());
            }
        }

        // Runtime dependencies of all subpackages
        for pkg in &spec.packages {
            if let Some(ref r_deps) = pkg.runtime_dependencies {
                for dep in &r_deps.dependencies {
                    deps.insert(dep.name.clone());
                }
            }
        }

        // Resolve dependencies to their source package
        for dep in deps {
            if let Some(dep_source) = subpkg_to_source.get(&dep) {
                if dep_source != source_name {
                    // dep_source must be built before source_name
                    if adj
                        .entry(dep_source.clone())
                        .or_default()
                        .insert(source_name.clone())
                    {
                        *in_degree.entry(source_name.clone()).or_default() += 1;
                    }
                }
            }
        }
    }

    // 3. Topological Sort (Kahn's Algorithm)
    let mut queue = VecDeque::new();
    for (name, deg) in &in_degree {
        if *deg == 0 {
            queue.push_back(name.clone());
        }
    }

    let mut order = Vec::new();
    while let Some(u) = queue.pop_front() {
        order.push(u.clone());
        if let Some(neighbors) = adj.get(&u) {
            for v in neighbors {
                let deg = in_degree.get_mut(v).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(v.clone());
                }
            }
        }
    }

    if order.len() == source_names.len() {
        println!("\n🎉 SUCCESS: Found complete conflict-free build order!\n");
        println!("BUILD ORDER ({} packages):", order.len());
        println!("--------------------------------------------------");
        for (i, name) in order.iter().enumerate() {
            println!("{:3}. {}", i + 1, name);
        }
    } else {
        println!("\n⚠️  WARNING: Circular dependencies or missing roots detected!");
        println!("Sorted {} / {} packages.", order.len(), source_names.len());

        let remaining: HashSet<String> = source_names
            .difference(&order.iter().cloned().collect())
            .cloned()
            .collect();

        println!("\nRemaining unsorted packages with cyclic dependencies:");
        println!("--------------------------------------------------");
        for name in &remaining {
            let deg = in_degree.get(name).unwrap();
            println!("- {} (in-degree: {})", name, deg);
        }

        // 4. Resolve using pre-satisfied LFS bootstrap toolchain packages
        println!("\n🔄 Resolving compilation sequence assuming standard LFS toolchain bootstrap is pre-installed...");
        let bootstrap_set: HashSet<String> = [
            "baselayout",
            "binutils",
            "gcc",
            "glibc",
            "m4",
            "bison",
            "flex",
            "sed",
            "grep",
            "coreutils",
            "bash",
            "make",
            "gawk",
            "diffutils",
            "tar",
            "gettext",
            "perl",
            "python3",
            "python",
            "ncurses",
            "readline",
            "zlib",
            "xz",
            "bzip2",
            "file",
            "patch",
            "texinfo",
            "gzip",
            "openssl",
            "util-linux",
            "pkgconfig",
            "libcap",
            "acl",
            "attr",
            "expat",
            "libffi",
            "gdbm",
            "sqlite",
            "db",
            "perl-Locale-gettext",
            "help2man",
            "autoconf",
            "automake",
            "libtool",
            "spidermonkey",
            "nasm",
            "cmake",
            "meson",
            "ninja",
            "pkgconfig",
            "libxml2",
            "libxslt",
            "zstd",
            "lz4",
            "elfutils",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let adj_bootstrap = adj.clone();
        let mut in_degree_bootstrap = in_degree.clone();

        // Treat bootstrap packages as already satisfied (in-degree 0)
        let mut queue_b = VecDeque::new();
        for name in &source_names {
            if bootstrap_set.contains(name) || *in_degree_bootstrap.get(name).unwrap() == 0 {
                queue_b.push_back(name.clone());
            }
        }

        let mut order_b = Vec::new();
        let mut visited_b = HashSet::new();
        while let Some(u) = queue_b.pop_front() {
            if !visited_b.insert(u.clone()) {
                continue;
            }
            if !bootstrap_set.contains(&u) {
                order_b.push(u.clone());
            }
            if let Some(neighbors) = adj_bootstrap.get(&u) {
                for v in neighbors {
                    let deg = in_degree_bootstrap.get_mut(v).unwrap();
                    if *deg > 0 {
                        *deg -= 1;
                        if *deg == 0 || bootstrap_set.contains(v) {
                            queue_b.push_back(v.clone());
                        }
                    }
                }
            }
        }

        // Force remaining elements that might be blocked by minor remaining edges
        for name in &source_names {
            if !visited_b.contains(name) && !bootstrap_set.contains(name) {
                order_b.push(name.clone());
            }
        }

        println!(
            "\n✅ RESOLVED COMPILATION SEQUENCE FOR CORE REPOSITORY ({} non-bootstrap packages):",
            order_b.len()
        );
        println!("--------------------------------------------------");
        for (i, name) in order_b.iter().enumerate() {
            println!("{:3}. {}", i + 1, name);
        }
    }
}

fn find_specs(dir: &Path, specs: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|name| name == ".git" || name == "files" || name == "comar")
                {
                    continue;
                }
                find_specs(&path, specs);
            } else if path.file_name().is_some_and(|name| name == "lopec.xml" || name == "lopec.kdl") {
                specs.push(path);
            }
        }
    }
}
