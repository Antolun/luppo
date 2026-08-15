use crate::database::LuppoDatabase;
use crate::version::LuppoVersion;
use crate::{LuppoError, LuppoResult};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::{
    visit::{Bfs, Reversed},
    Direction,
};
use luppo_spec::models::{Dependencies, PackageActions, PackageDefinition, Packager};
use pubgrub::{resolve, DefaultStringReporter, OfflineDependencyProvider, Ranges};
use rust_i18n::t;
use std::collections::{HashMap, HashSet};

rust_i18n::i18n!("../locales", fallback = "tr");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DependencyType {
    Runtime,
    #[allow(dead_code)]
    Build,
}

/// Gerçek veritabanı verilerini kullanarak çözümleme yapan Repo yapısı.
#[derive(Clone)]
pub struct LuppoRepo {
    pub packages: HashMap<String, PackageDefinition>,
}

impl LuppoRepo {
    /// Veritabanındaki tüm paketleri çözümleyici için hazırlar.
    pub fn new(db: LuppoDatabase) -> Self {
        let mut packages = HashMap::new();

        if let Ok(all_packages) = db.list_available_packages() {
            for pkg in all_packages {
                let runtime_deps_list: Vec<String> = pkg
                    .runtime_dependencies
                    .as_ref()
                    .map(|r| r.dependencies.clone())
                    .unwrap_or_default();
                let pkg_def = PackageDefinition {
                    name: pkg.name.clone(),
                    version: pkg.latest_version(),
                    summary: pkg.get_summary(),
                    description: pkg.get_description(),
                    homepage: pkg.homepage.clone(),
                    icon: pkg.icon.clone(),
                    screenshot: pkg.screenshot.clone(),
                    provides: Some(luppo_spec::models::ProvidesBlock {
                        isa: pkg.provides.clone(),
                        comar: Vec::new(),
                    }),
                    additional_files: None,
                    build_type: None,
                    license: "GPL".to_string(),
                    packager: Packager {
                        name: "Luppo Community".to_string(),
                        email: "info@antolun.com".to_string(),
                    },
                    deps: Dependencies {
                        runtime: runtime_deps_list.clone(),
                        conflicts: pkg
                            .conflicts
                            .as_ref()
                            .map(|c| c.packages.clone())
                            .unwrap_or_default(),
                        build: pkg
                            .build_dependencies
                            .as_ref()
                            .map(|b| b.dependencies.clone())
                            .unwrap_or_default(),
                    },
                    actions: PackageActions {
                        steps: Vec::new(),
                        step_types: Vec::new(),
                        configure: None,
                        pre_install: None,
                        post_install: None,
                        pre_upgrade: None,
                        post_upgrade: None,
                        pre_remove: None,
                        post_remove: None,
                        install_filters: Vec::new(),
                        no_strip: Vec::new(),
                    },
                    files: luppo_spec::models::Files::default(),
                    runtime_dependencies: Some(luppo_spec::models::RuntimeDeps {
                        dependencies: runtime_deps_list
                            .into_iter()
                            .map(|d| luppo_spec::models::Dependency {
                                name: d,
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                packages.insert(pkg.name.clone(), pkg_def);
            }
        }

        LuppoRepo { packages }
    }

    /// EKSİK OLAN METOD: Bir paketi adına göre depodan getirir.
    pub fn get_package(&self, name: &str) -> Option<&PackageDefinition> {
        self.packages.get(name)
    }
}

/// Bağımlılık çözümleme mantığını içerir.
pub struct PackageResolver {
    db: LuppoDatabase,
    repo: LuppoRepo,
    resolved_versions: HashMap<String, String>, // Paket adı -> Karar verilen versiyon
    pub ignore_package_conflict: bool,
    pub reinstall: bool,
    pub ignore_dependency: bool,
    pubgrub_provider: Option<OfflineDependencyProvider<String, Ranges<LuppoVersion>>>,
}

impl PackageResolver {
    pub fn new(db: LuppoDatabase, repo: LuppoRepo) -> Self {
        PackageResolver {
            db,
            repo,
            resolved_versions: HashMap::new(),
            ignore_package_conflict: false,
            reinstall: false,
            ignore_dependency: false,
            pubgrub_provider: None,
        }
    }

    fn get_pubgrub_provider(
        &mut self,
    ) -> LuppoResult<&OfflineDependencyProvider<String, Ranges<LuppoVersion>>> {
        if self.pubgrub_provider.is_none() {
            self.pubgrub_provider = Some(self.build_pubgrub_provider()?);
        }
        Ok(self.pubgrub_provider.as_ref().unwrap())
    }

    fn build_pubgrub_provider(
        &self,
    ) -> LuppoResult<OfflineDependencyProvider<String, Ranges<LuppoVersion>>> {
        let mut provider = OfflineDependencyProvider::new();

        for (name, pkg_def) in &self.repo.packages {
            let version = LuppoVersion::new(&pkg_def.version);

            let mut pubgrub_deps = Vec::new();
            for dep in &pkg_def.deps.runtime {
                pubgrub_deps.push((dep.clone(), Ranges::full()));
            }

            provider.add_dependencies(name.clone(), version, pubgrub_deps);
        }

        Ok(provider)
    }

    pub fn resolve_deps(&mut self, package_names: &[String]) -> LuppoResult<Vec<PackageDefinition>> {
        // This is for runtime
        if self.ignore_dependency {
            let mut result = Vec::new();
            for pkg_name in package_names {
                let package_def = match self.repo.get_package(pkg_name) {
                    Some(pkg) => pkg.clone(),
                    None => {
                        return Err(LuppoError::RuntimeError(
                            t!("resolver_error_not_found", name = pkg_name).into(),
                        ))
                    }
                };
                result.push(package_def);
            }
            return Ok(result);
        }

        let installed_packages = self.db.list_installed_packages()?;
        let installed_set: HashSet<String> = installed_packages
            .into_iter()
            .map(|p| p.name.clone())
            .collect();

        // Use PubGrub first to find a valid combination of versions (cached)
        let mut provider = self.get_pubgrub_provider()?.clone();

        let root_pkg = "__root__".to_string();
        let root_version = LuppoVersion::new("1.0.0");
        let mut root_deps = Vec::new();

        for pkg_name in package_names {
            root_deps.push((pkg_name.clone(), Ranges::full()));
        }

        provider.add_dependencies(root_pkg.clone(), root_version.clone(), root_deps);

        let resolution = resolve(&provider, root_pkg.clone(), root_version.clone());

        match resolution {
            Ok(solution) => {
                let mut graph = DiGraph::<PackageDefinition, ()>::new();
                let mut nodes = HashMap::<String, NodeIndex>::new();

                // Add all resolved packages to build_graph, except root
                for (pkg_name, version) in solution {
                    if pkg_name == root_pkg {
                        continue;
                    }

                    if self.reinstall || !installed_set.contains(&pkg_name) {
                        self.resolved_versions
                            .insert(pkg_name.clone(), version.to_string());
                        self.build_graph(
                            &pkg_name,
                            &installed_set,
                            &mut graph,
                            &mut nodes,
                            DependencyType::Runtime,
                        )?;
                    }
                }

                match toposort(&graph, None) {
                    Ok(order) => {
                        let mut result: Vec<PackageDefinition> =
                            order.into_iter().map(|idx| graph[idx].clone()).collect();
                        result.reverse();
                        Ok(result)
                    }
                    Err(cycle) => {
                        let node_name = &graph[cycle.node_id()].name;
                        let mut visual_graph = String::new();
                        visual_graph.push_str(&format!("  ┌──▶ {}\n", node_name));
                        visual_graph.push_str(&t!("resolver_cycle_detected"));
                        visual_graph.push_str("  └────┘");
                        Err(LuppoError::CycleDependency(visual_graph))
                    }
                }
            }
            Err(pubgrub_err) => {
                use pubgrub::Reporter;
                match pubgrub_err {
                    pubgrub::PubGrubError::NoSolution(tree) => {
                        let report = DefaultStringReporter::report(&tree);
                        Err(LuppoError::RuntimeError(format!(
                            "PubGrub Resolution Error:\n{}",
                            report
                        )))
                    }
                    err => Err(LuppoError::RuntimeError(format!("PubGrub Error: {:?}", err))),
                }
            }
        }
    }

    /// Belirtilen paketlerin build bağımlılıklarını çözümler ve sıralı bir liste döndürür.
    /// `build_env_installed` burada build ortamında (chroot) zaten var olan paketleri temsil eder.
    /// Basit DFS + visited set yaklaşımı (Python luppo benzeri) - çok daha hızlı.
    /// `LuppoRepo` kullanmaz, doğrudan DB üzerinden tekil paket sorguları yapar (tüm paketleri yüklemez).
    pub fn resolve_build_deps(
        &mut self,
        package_names: &[String],
        build_env_installed: &HashSet<String>,
    ) -> LuppoResult<Vec<PackageDefinition>> {
        // Bu method özellikle testler için self.repo kullanır.
        // build.rs'de doğrudan resolve_build_deps_static çağrılır (DB üzerinden).
        resolve_build_deps_from_repo(&self.repo, package_names, build_env_installed)
    }

    fn build_graph(
        &mut self,
        current_pkg_name: &str,
        installed_set: &HashSet<String>,
        graph: &mut DiGraph<PackageDefinition, ()>,
        nodes: &mut HashMap<String, NodeIndex>,
        dep_type: DependencyType,
    ) -> LuppoResult<NodeIndex> {
        if let Some(&idx) = nodes.get(current_pkg_name) {
            return Ok(idx);
        }

        // Burada artık get_package metodunu bulabilecek
        let package_def = match self.repo.get_package(current_pkg_name) {
            Some(pkg) => pkg.clone(),
            None => {
                return Err(LuppoError::RuntimeError(
                    t!("resolver_error_not_found", name = current_pkg_name).into(),
                ))
            }
        };

        // --- PAKET ÇAKIŞMA KONTROLÜ ---
        if !self.ignore_package_conflict {
            for conflict in &package_def.deps.conflicts {
                if installed_set.contains(conflict) {
                    return Err(LuppoError::InstalledConflict {
                        package: current_pkg_name.to_string(),
                        conflicting_package: conflict.clone(),
                    });
                }
                if nodes.contains_key(conflict) {
                    return Err(LuppoError::PlannedConflict {
                        package: current_pkg_name.to_string(),
                        conflicting_package: conflict.clone(),
                    });
                }
            }
        }

        // --- VERSİYON ÇAKIŞMA KONTROLÜ ---
        if let Some(existing_version) = self.resolved_versions.get(&package_def.name) {
            if existing_version != &package_def.version {
                return Err(LuppoError::RuntimeError(
                    t!(
                        "resolver_error_version_conflict",
                        name = package_def.name,
                        v1 = existing_version,
                        v2 = package_def.version
                    )
                    .into(),
                ));
            }
        }
        self.resolved_versions
            .insert(package_def.name.clone(), package_def.version.clone());

        let current_node = graph.add_node(package_def.clone());
        nodes.insert(current_pkg_name.to_string(), current_node);

        let deps_to_consider = match dep_type {
            DependencyType::Runtime => &package_def.deps.runtime,
            DependencyType::Build => &package_def.deps.build,
        };

        for dep_name in deps_to_consider {
            if !installed_set.contains(dep_name) {
                let dep_node = self.build_graph(dep_name, installed_set, graph, nodes, dep_type)?;
                // A -> B (A paketi B'ye bağımlı)
                graph.add_edge(current_node, dep_node, ());
            }
        }

        Ok(current_node)
    }

    /// Petgraph kullanarak ters bağımlılıkları (istenen pakete bağımlı olanlar) bulur.
    /// Manuel döngü yerine graf yapısını kullanarak daha verimli sorgulama sağlar.
    pub fn check_reverse_deps(&self, package_name: &str) -> LuppoResult<Vec<String>> {
        let (graph, nodes) = self.build_installed_graph()?;

        if let Some(&target_idx) = nodes.get(package_name) {
            // Incoming yönü, bu düğüme işaret eden (yani ona bağımlı olan) paketleri döner.
            Ok(graph
                .neighbors_directed(target_idx, Direction::Incoming)
                .map(|idx| graph[idx].clone())
                .collect())
        } else {
            Ok(vec![])
        }
    }

    /// Petgraph ve BFS kullanarak tüm (dolaylı) ters bağımlılıkları bulur.
    /// Bir paket kaldırıldığında etkilenecek tüm paketleri listelemek için idealdir.
    pub fn check_transitive_reverse_deps(&self, package_name: &str) -> LuppoResult<Vec<String>> {
        let (graph, nodes) = self.build_installed_graph()?;

        if let Some(&target_idx) = nodes.get(package_name) {
            // Grafı tersine çeviriyoruz (bağımlılık -> paket yönü için)
            let rev_graph = Reversed(&graph);
            let mut bfs = Bfs::new(&rev_graph, target_idx);
            let mut dependents = Vec::new();

            let _ = bfs.next(&rev_graph); // Paketin kendisini listeden atla
            while let Some(nx) = bfs.next(&rev_graph) {
                dependents.push(graph[nx].clone());
            }
            Ok(dependents)
        } else {
            Ok(vec![])
        }
    }

    /// Tüm kurulu paketleri ve bağımlılıklarını içeren bir petgraph grafı oluşturur.
    fn build_installed_graph(
        &self,
    ) -> LuppoResult<(DiGraph<String, ()>, HashMap<String, NodeIndex>)> {
        let mut graph = DiGraph::new();
        let mut nodes = HashMap::new();
        let installed = self.db.list_installed_packages()?;

        for pkg in &installed {
            nodes.insert(pkg.name.clone(), graph.add_node(pkg.name.clone()));
        }

        for pkg in &installed {
            if let Some(pkg_def) = self.repo.get_package(&pkg.name) {
                if let Some(&u) = nodes.get(&pkg.name) {
                    for dep in &pkg_def.deps.runtime {
                        if let Some(&v) = nodes.get(dep) {
                            graph.add_edge(u, v, ());
                        }
                    }
                }
            }
        }
        Ok((graph, nodes))
    }
}

/// Package (luppo-core) -> PackageDefinition (luppo-spec) dönüşümü
fn package_to_definition(pkg: &crate::package::Package) -> PackageDefinition {
    let runtime_deps_list: Vec<String> = pkg
        .runtime_dependencies
        .as_ref()
        .map(|r| r.dependencies.clone())
        .unwrap_or_default();
    PackageDefinition {
        name: pkg.name.clone(),
        version: pkg.latest_version(),
        summary: pkg.get_summary(),
        description: pkg.get_description(),
        homepage: pkg.homepage.clone(),
        icon: pkg.icon.clone(),
        screenshot: pkg.screenshot.clone(),
        provides: Some(luppo_spec::models::ProvidesBlock {
            isa: pkg.provides.clone(),
            comar: Vec::new(),
        }),
        additional_files: None,
        build_type: None,
        license: "GPL".to_string(),
        packager: luppo_spec::models::Packager {
            name: "Luppo Community".to_string(),
            email: "info@antolun.com".to_string(),
        },
        deps: luppo_spec::models::Dependencies {
            runtime: runtime_deps_list.clone(),
            conflicts: pkg
                .conflicts
                .as_ref()
                .map(|c| c.packages.clone())
                .unwrap_or_default(),
            build: pkg
                .build_dependencies
                .as_ref()
                .map(|b| b.dependencies.clone())
                .unwrap_or_default(),
        },
        files: luppo_spec::models::Files::default(),
        runtime_dependencies: Some(luppo_spec::models::RuntimeDeps {
            dependencies: runtime_deps_list
                .into_iter()
                .map(|d| luppo_spec::models::Dependency {
                    name: d,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }),
        actions: luppo_spec::models::PackageActions {
            steps: Vec::new(),
            step_types: Vec::new(),
            configure: None,
            pre_install: None,
            post_install: None,
            pre_upgrade: None,
            post_upgrade: None,
            pre_remove: None,
            post_remove: None,
            install_filters: Vec::new(),
            no_strip: Vec::new(),
        },
        mirrors: pkg.mirrors.clone(),
        ..Default::default()
    }
}

/// PackageResolver::resolve_build_deps ve build.rs tarafından kullanılır (LuppoRepo üzerinden).
pub fn resolve_build_deps_from_repo(
    repo: &LuppoRepo,
    package_names: &[String],
    build_env_installed: &HashSet<String>,
) -> LuppoResult<Vec<PackageDefinition>> {
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut in_stack = HashSet::new();

    let mut process_stack: Vec<(String, bool)> = Vec::new();
    for pkg_name in package_names {
        if !build_env_installed.contains(pkg_name) {
            process_stack.push((pkg_name.clone(), false));
        }
    }

    while let Some((current, processed_children)) = process_stack.pop() {
        if processed_children {
            if visited.insert(current.clone()) {
                if let Some(pkg_def) = repo.get_package(&current) {
                    result.push(pkg_def.clone());
                }
            }
            in_stack.remove(&current);
            continue;
        }

        if visited.contains(&current) || in_stack.contains(&current) {
            if in_stack.contains(&current) {
                return Err(LuppoError::CycleDependency(format!(
                    "Circular dependency detected involving: {}",
                    current
                )));
            }
            continue;
        }

        in_stack.insert(current.clone());
        process_stack.push((current.clone(), true));

        if let Some(pkg_def) = repo.get_package(&current) {
            for dep_name in &pkg_def.deps.runtime {
                if !build_env_installed.contains(dep_name) && !visited.contains(dep_name) {
                    process_stack.push((dep_name.clone(), false));
                }
            }
        }
    }

    Ok(result)
}

/// Bağımlılık çözümleme işlemini tamamlar (varsayılan çözümleme).
/// Free function: `PackageResolver` veya `LuppoRepo` gerektirmez, doğrudan DB kullanır.
/// TEK SEFERLİK DB iterasyonu: tüm runtime deps'leri yükle, sonra in-memory DFS.
pub fn resolve_build_deps_static(
    db: &LuppoDatabase,
    package_names: &[String],
    build_env_installed: &HashSet<String>,
) -> LuppoResult<Vec<PackageDefinition>> {
    // Cache'lenmiş runtime deps'leri getir (ilk seferde oluşturur, sonraki çağrılarda anlık)
    let runtime_deps_map = db.get_or_build_runtime_deps_cache()?;

    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut in_stack = HashSet::new();
    let mut pkg_cache: HashMap<String, Option<PackageDefinition>> = HashMap::new();

    let mut process_stack: Vec<(String, bool)> = Vec::new();
    for pkg_name in package_names {
        if !build_env_installed.contains(pkg_name) && runtime_deps_map.contains_key(pkg_name) {
            process_stack.push((pkg_name.clone(), false));
        }
    }

    while let Some((current, processed_children)) = process_stack.pop() {
        if processed_children {
            if visited.insert(current.clone()) {
                if let Some(pkg_def) = pkg_cache.get(&current).and_then(|o| o.as_ref()) {
                    result.push(pkg_def.clone());
                }
            }
            in_stack.remove(&current);
            continue;
        }

        if visited.contains(&current) || in_stack.contains(&current) {
            if in_stack.contains(&current) {
                return Err(LuppoError::CycleDependency(format!(
                    "Circular dependency detected involving: {}",
                    current
                )));
            }
            continue;
        }

        in_stack.insert(current.clone());
        process_stack.push((current.clone(), true));

        if !runtime_deps_map.contains_key(&current) {
            continue;
        }

        if !pkg_cache.contains_key(&current) {
            let def = db
                .get_available_package(&current)
                .ok()
                .flatten()
                .map(|p| package_to_definition(&p));
            pkg_cache.insert(current.clone(), def.clone());
        }

        if let Some(deps) = runtime_deps_map.get(&current) {
            for dep_name in deps {
                let dep_ref: &String = dep_name;
                if !build_env_installed.contains(dep_ref)
                    && !visited.contains(dep_ref)
                    && runtime_deps_map.contains_key(dep_ref)
                {
                    process_stack.push((dep_name.clone(), false));
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::InstalledPackage;
    use luppo_spec::models::{Dependencies, PackageActions, PackageDefinition, Packager};
    use tempfile::tempdir;

    // Testler için yardımcı paket oluşturma fonksiyonu
    fn create_mock_pkg(
        name: &str,
        runtime_deps: Vec<&str>,
        build_deps: Vec<&str>,
        conflicts: Vec<&str>,
    ) -> PackageDefinition {
        PackageDefinition {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            summary: "Test Summary".to_string(),
            description: "Test Description".to_string(),
            homepage: None,
            icon: None,
            screenshot: None,
            provides: None,
            additional_files: None,
            build_type: None,
            license: "GPL".to_string(),
            packager: Packager {
                name: "Luppo Test".to_string(),
                email: "test@antolun.com".to_string(),
            },
            deps: Dependencies {
                runtime: runtime_deps.iter().map(|s| s.to_string()).collect(),
                build: build_deps.iter().map(|s| s.to_string()).collect(),
                conflicts: conflicts.iter().map(|s| s.to_string()).collect(),
            },
            actions: PackageActions {
                steps: Vec::new(),
                step_types: Vec::new(),
                configure: None,
                pre_install: None,
                post_install: None,
                pre_upgrade: None,
                post_upgrade: None,
                pre_remove: None,
                post_remove: None,
                install_filters: Vec::new(),
                no_strip: Vec::new(),
            },
            files: luppo_spec::models::Files::default(),
            runtime_dependencies: Some(luppo_spec::models::RuntimeDeps {
                dependencies: runtime_deps
                    .iter()
                    .map(|&s| luppo_spec::models::Dependency {
                        name: s.to_string(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    // Testler için yardımcı kurulu paket oluşturma fonksiyonu
    fn create_mock_installed_pkg(name: &str) -> InstalledPackage {
        InstalledPackage {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Already Installed".to_string(),
            install_date: "2023-10-27 10:00:00".to_string(),
            installed_files: HashMap::new(),
            total_size: 0,
            package_hash: "oldhash".to_string(),
            release: 1,
            distribution_release: "1".to_string(),
            licenses: Vec::new(),
            provides: Vec::new(),
            post_remove: None,
            pre_remove: None,
            homepage: None,
            icon: None,
            screenshot: None,
            packager: None,
            install_tar_hash: None,
            package_format: None,
            build_host: None,
            distribution: None,
            configured: true,
            signature_verified: true,
        }
    }

    #[test]
    fn test_simple_circular_dependency() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // A -> B, B -> A döngüsü
        repo_map.insert(
            "pkg-a".to_string(),
            create_mock_pkg("pkg-a", vec!["pkg-b"], vec![], vec![]),
        );
        repo_map.insert(
            "pkg-b".to_string(),
            create_mock_pkg("pkg-b", vec!["pkg-a"], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["pkg-a".to_string()]);

        assert!(result.is_err(), "Döngüsel bağımlılık hata döndürmeliydi");
        if let Err(LuppoError::CycleDependency(visual)) = result {
            assert!(
                visual.contains("pkg-a") || visual.contains("pkg-b"),
                "Görsel grafik paket adını içermeli"
            );
            assert!(
                visual.contains("┌──▶"),
                "Görsel grafik başlangıç karakterlerini içermeli"
            );
        } else {
            panic!("Beklenen CycleDependency tipi alınamadı");
        }
    }

    #[test]
    fn test_complex_circular_dependency() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // X -> Y -> Z -> X döngüsü
        repo_map.insert(
            "X".to_string(),
            create_mock_pkg("X", vec!["Y"], vec![], vec![]),
        );
        repo_map.insert(
            "Y".to_string(),
            create_mock_pkg("Y", vec!["Z"], vec![], vec![]),
        );
        repo_map.insert(
            "Z".to_string(),
            create_mock_pkg("Z", vec!["X"], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["X".to_string()]);
        assert!(result.is_err());
        if let Err(LuppoError::CycleDependency(visual)) = result {
            assert!(visual.contains("X") || visual.contains("Y") || visual.contains("Z"));
        } else {
            panic!("Beklenen CycleDependency tipi alınamadı");
        }
    }

    #[test]
    fn test_no_circular_dependency() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // A -> B -> C (Düz zincir, döngü yok)
        repo_map.insert(
            "A".to_string(),
            create_mock_pkg("A", vec!["B"], vec![], vec![]),
        );
        repo_map.insert(
            "B".to_string(),
            create_mock_pkg("B", vec!["C"], vec![], vec![]),
        );
        repo_map.insert(
            "C".to_string(),
            create_mock_pkg("C", vec![], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["A".to_string()]);
        assert!(result.is_ok(), "Geçerli bağımlılık zinciri hata vermemeli");
        let plan = result.unwrap();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].name, "C"); // En temel paket en başta olmalı
    }

    #[test]
    fn test_dependencies_already_installed() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        // Senaryo: A kurulmak isteniyor. A -> B bağımlılığı var ama B zaten kurulu.
        let installed_pkg = create_mock_installed_pkg("pkg-b");
        db.install_package(&installed_pkg).unwrap();

        let mut repo_map = HashMap::new();
        repo_map.insert(
            "pkg-a".to_string(),
            create_mock_pkg("pkg-a", vec!["pkg-b"], vec![], vec![]),
        );
        repo_map.insert(
            "pkg-b".to_string(),
            create_mock_pkg("pkg-b", vec![], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["pkg-a".to_string()]);
        assert!(result.is_ok());
        let plan = result.unwrap();

        // Sadece A kurulmalı çünkü B zaten sistemde var.
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].name, "pkg-a");
    }

    #[test]
    fn test_nested_dependencies_installed() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        // Senaryo: A -> B -> C zinciri var. C zaten kurulu.
        // A istendiğinde plan: [B, A] olmalı.
        let installed_pkg = create_mock_installed_pkg("pkg-c");
        db.install_package(&installed_pkg).unwrap();

        let mut repo_map = HashMap::new();
        repo_map.insert(
            "pkg-a".to_string(),
            create_mock_pkg("pkg-a", vec!["pkg-b"], vec![], vec![]),
        );
        repo_map.insert(
            "pkg-b".to_string(),
            create_mock_pkg("pkg-b", vec!["pkg-c"], vec![], vec![]),
        );
        repo_map.insert(
            "pkg-c".to_string(),
            create_mock_pkg("pkg-c", vec![], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["pkg-a".to_string()]);
        assert!(result.is_ok());
        let plan = result.unwrap();

        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].name, "pkg-b");
        assert_eq!(plan[1].name, "pkg-a");
    }

    #[test]
    fn test_version_conflict() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();

        // lib-common v1.0.0
        let mut lib_v1 = create_mock_pkg("lib-common", vec![], vec![], vec![]);
        lib_v1.version = "1.0.0".to_string();

        // pkg-a, lib-common v1.0.0 istiyor
        repo_map.insert(
            "pkg-a".to_string(),
            create_mock_pkg("pkg-a", vec!["lib-common"], vec![], vec![]),
        );
        repo_map.insert("lib-common".to_string(), lib_v1);

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        // İlk sürümü kayda geçir
        resolver
            .resolved_versions
            .insert("lib-common".to_string(), "2.0.0".to_string());

        let result = resolver.resolve_deps(&["pkg-a".to_string()]);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result);
        assert!(err_msg.contains("Versiyon çakışması") || err_msg.contains("Version conflict"));
    }

    #[test]
    fn test_diamond_dependency() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // Senaryo: A -> [B, C], B -> D, C -> D
        // Sonuçta D sadece 1 kez ve en başta kurulmalı.
        repo_map.insert(
            "A".to_string(),
            create_mock_pkg("A", vec!["B", "C"], vec![], vec![]),
        );
        repo_map.insert(
            "B".to_string(),
            create_mock_pkg("B", vec!["D"], vec![], vec![]),
        );
        repo_map.insert(
            "C".to_string(),
            create_mock_pkg("C", vec!["D"], vec![], vec![]),
        );
        repo_map.insert(
            "D".to_string(),
            create_mock_pkg("D", vec![], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["A".to_string()]);
        assert!(result.is_ok());
        let plan = result.unwrap();

        assert_eq!(plan.len(), 4, "Toplam 4 paket olmalı");
        assert_eq!(plan[0].name, "D", "En temel bağımlılık (D) en başta olmalı");
        assert_eq!(plan[3].name, "A", "Ana hedef paket (A) en sonda olmalı");

        // B ve C'nin varlığını kontrol et
        let mid_names: Vec<String> = plan[1..3].iter().map(|p| p.name.clone()).collect();
        assert!(mid_names.contains(&"B".to_string()));
        assert!(mid_names.contains(&"C".to_string()));
    }

    #[test]
    fn test_multiple_roots() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // Senaryo: [A, B] kurulmak isteniyor. Her ikisi de C'ye bağımlı.
        repo_map.insert(
            "A".to_string(),
            create_mock_pkg("A", vec!["C"], vec![], vec![]),
        );
        repo_map.insert(
            "B".to_string(),
            create_mock_pkg("B", vec!["C"], vec![], vec![]),
        );
        repo_map.insert(
            "C".to_string(),
            create_mock_pkg("C", vec![], vec![], vec![]),
        );

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["A".to_string(), "B".to_string()]);
        assert!(result.is_ok());
        let plan = result.unwrap();

        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].name, "C"); // Ortak bağımlılık en önce
    }

    #[test]
    fn test_missing_package_in_repo() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let repo = LuppoRepo {
            packages: HashMap::new(),
        };
        let mut resolver = PackageResolver::new(db, repo);

        let result = resolver.resolve_deps(&["non-existent".to_string()]);
        assert!(result.is_err(), "Depoda olmayan paket hata döndürmeli");
    }

    #[test]
    fn test_resolve_build_deps_follows_runtime_deps() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // pkg-app (runtime-dep: runtime-lib)
        let app_pkg = create_mock_pkg("pkg-app", vec!["runtime-lib"], vec![], vec![]);
        // runtime-lib (no deps)
        let runtime_lib = create_mock_pkg("runtime-lib", vec![], vec![], vec![]);

        repo_map.insert("pkg-app".to_string(), app_pkg);
        repo_map.insert("runtime-lib".to_string(), runtime_lib);

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let build_env_installed = HashSet::new();
        let result = resolver.resolve_build_deps(&["pkg-app".to_string()], &build_env_installed);

        assert!(result.is_ok());
        let plan = result.unwrap();

        // build dep'in runtime bagimliligi da cozulmeli
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].name, "runtime-lib");
        assert_eq!(plan[1].name, "pkg-app");
    }

    #[test]
    fn test_resolve_build_deps_with_installed_in_env() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // pkg-app (runtime-dep: runtime-lib)
        let app_pkg = create_mock_pkg("pkg-app", vec!["runtime-lib"], vec![], vec![]);
        let runtime_lib = create_mock_pkg("runtime-lib", vec![], vec![], vec![]);
        repo_map.insert("pkg-app".to_string(), app_pkg);
        repo_map.insert("runtime-lib".to_string(), runtime_lib);

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);

        let mut build_env_installed = HashSet::new();
        build_env_installed.insert("runtime-lib".to_string());

        let result = resolver.resolve_build_deps(&["pkg-app".to_string()], &build_env_installed);

        assert!(result.is_ok());
        let plan = result.unwrap();

        assert_eq!(plan.len(), 1); // Sadece pkg-app, runtime-lib zaten kurulu
        assert_eq!(plan[0].name, "pkg-app");
    }

    #[test]
    fn test_ignore_dependency_resolution() {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("DB açılamadı");

        let mut repo_map = HashMap::new();
        // pkg-app (runtime-dep: dep-pkg)
        let app_pkg = create_mock_pkg("pkg-app", vec!["dep-pkg"], vec![], vec![]);
        let dep_pkg = create_mock_pkg("dep-pkg", vec![], vec![], vec![]);

        repo_map.insert("pkg-app".to_string(), app_pkg);
        repo_map.insert("dep-pkg".to_string(), dep_pkg);

        let repo = LuppoRepo { packages: repo_map };
        let mut resolver = PackageResolver::new(db, repo);
        resolver.ignore_dependency = true;

        let result = resolver.resolve_deps(&["pkg-app".to_string()]);
        assert!(result.is_ok());
        let plan = result.unwrap();

        assert_eq!(plan.len(), 1); // dep-pkg should be ignored and not present in the plan
        assert_eq!(plan[0].name, "pkg-app");
    }
}
