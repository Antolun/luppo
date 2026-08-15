use crate::database::LuppoDatabase;
use crate::installer::Installer;
use crate::package::Package;
use crate::packager::Packager;
use crate::repo::Repository;
use crate::LuppoError;
use luppo_spec::models::PackageDefinition;
use rayon::prelude::*;
use rust_i18n::t;
use std::collections::{HashMap, HashSet};
use std::{fs, path::Path};
rust_i18n::i18n!("../locales", fallback = "tr");

type LuppoResult<T> = Result<T, LuppoError>;

pub struct QueryManager {
    pub db: LuppoDatabase,
    config: crate::config::Config,
}

impl QueryManager {
    pub fn new(db: LuppoDatabase, config: crate::config::Config) -> Self {
        QueryManager { db, config }
    }

    pub fn perform_list_available(&self) -> LuppoResult<()> {
        let available = self.db.list_available_packages()?;
        if available.is_empty() {
            println!("{}", t!("query_available_empty"));
        } else {
            println!("{}", t!("query_available_total", count = available.len()));
            for pkg in available {
                println!("🌐 {:<20} v{}", pkg.name, pkg.latest_version());
            }
        }
        Ok(())
    }

    pub fn perform_graph(
        &self,
        package_names: Vec<String>,
        installed: bool,
        reverse: bool,
        output: &str,
    ) -> LuppoResult<()> {
        println!("{}", t!("query_graph_starting"));
        let mut dot_content = String::from("digraph G {\n");
        dot_content.push_str(
            "  node [shape=box, style=filled, fillcolor=lightblue, fontname=\"Arial\"];\n",
        );
        dot_content.push_str("  edge [color=gray50];\n");
        let mut processed_edges = HashSet::new();
        let mut packages_to_scan = Vec::new();

        if installed {
            for pkg in self.db.list_installed_packages()? {
                if package_names.is_empty() || package_names.contains(&pkg.name) {
                    if let Ok(Some(repo_pkg)) = self.db.get_available_package(&pkg.name) {
                        packages_to_scan.push(repo_pkg);
                    }
                }
            }
        } else {
            for pkg in self.db.list_available_packages()? {
                if package_names.is_empty() || package_names.contains(&pkg.name) {
                    packages_to_scan.push(pkg);
                }
            }
        }

        for pkg in packages_to_scan {
            if let Some(runtime) = &pkg.runtime_dependencies {
                for dep in &runtime.dependencies {
                    let (from, to) = if reverse {
                        (dep, &pkg.name)
                    } else {
                        (&pkg.name, dep)
                    };
                    if processed_edges.insert((from.clone(), to.clone())) {
                        dot_content.push_str(&format!("  \"{}\" -> \"{}\";\n", from, to));
                    }
                }
            }
        }
        dot_content.push_str("}\n");
        fs::write(output, dot_content).map_err(LuppoError::IoError)?;
        println!("{}", t!("query_graph_saved", path = output));
        println!("{}", t!("query_graph_info", path = output));
        Ok(())
    }

    pub fn perform_search(&self, query: String) -> LuppoResult<()> {
        let available = self.db.search_package(&query)?;
        if available.is_empty() {
            println!("{}", t!("query_no_results", query = query));
        }
        for pkg in available {
            let pkg_name_colored = &pkg.name;
            if pkg.repo_name.is_empty() {
                println!("🌐 {} v{}", pkg_name_colored, pkg.latest_version());
            } else {
                let repo_name_colored = crate::colorize(&pkg.repo_name, "cyan");
                println!(
                    "🌐 [{}] {} v{}",
                    repo_name_colored,
                    pkg_name_colored,
                    pkg.latest_version()
                );
            }
        }
        Ok(())
    }

    pub fn perform_search_file(&self, query: &str) -> LuppoResult<()> {
        let results = self.search_file(query)?;
        if results.is_empty() {
            println!("{}", t!("query_file_not_found", query = query));
        } else {
            println!("{}", t!("query_file_searching", query = query));
            for (pkg_name, path) in results {
                println!(
                    "{}",
                    t!("query_file_match", package = pkg_name, path = path)
                );
            }
        }
        Ok(())
    }

    pub fn perform_list_installed(&self) -> LuppoResult<()> {
        let packages = self.db.list_installed_packages()?;
        for pkg in packages {
                let summary = self
                    .db
                    .get_available_package(&pkg.name)
                    .ok()
                    .flatten()
                    .map(|p| p.get_summary())
                    .unwrap_or_default();
                if summary.is_empty() {
                    println!("{:<38}- {}", pkg.name, pkg.description);
                } else {
                    println!("{:<38}- {}", pkg.name, summary);
                }
        }
        Ok(())
    }

    pub fn perform_list_orphaned(&self, installer: &Installer) -> LuppoResult<()> {
        let orphans = installer.find_orphaned_packages()?;
        if orphans.is_empty() {
            println!("{}", t!("query_orphans_empty"));
        } else {
            println!("{}", t!("query_orphans_title"));
            println!("{:-<40}", "");
            for pkg in &orphans {
                println!("  • {}", pkg);
            }
            println!("{:-<40}", "");
            println!("{}", t!("query_orphans_count", count = orphans.len()));
            println!("{}", t!("query_orphans_info"));
        }
        Ok(())
    }

    pub fn perform_list_pending(&self) -> LuppoResult<()> {
        let installed = self.db.list_installed_packages()?;
        let pending: Vec<_> = installed.into_iter().filter(|p| !p.configured).collect();
        if pending.is_empty() {
            println!("{}", t!("query_pending_empty"));
        } else {
            println!("{}", t!("query_pending_title"));
            for pkg in pending {
                println!("  • {} v{}", pkg.name, pkg.version);
            }
        }
        Ok(())
    }

    pub fn perform_list_sources(&self) -> LuppoResult<()> {
        let available = self.db.list_available_packages()?;
        if available.is_empty() {
            println!("{}", t!("query_sources_empty"));
        } else {
            println!("{}", t!("query_sources_title"));
            for pkg in available {
                println!(
                    "  • {:<25} [repo: {}]",
                    pkg.name,
                    if pkg.repo_name.is_empty() {
                        "stable"
                    } else {
                        &pkg.repo_name
                    }
                );
            }
        }
        Ok(())
    }

    pub fn perform_blame(
        &self,
        package_name: &str,
        release: Option<u32>,
        all: bool,
    ) -> LuppoResult<()> {
        let pkg = match self.db.get_available_package(package_name)? {
            Some(p) => p,
            None => {
                println!(
                    "{}",
                    t!("query_error_pkg_not_found", package = package_name)
                );
                return Ok(());
            }
        };
        let updates = &pkg.history.updates;
        if all {
            for update in updates {
                self.display_update_blame(&pkg.name, update);
            }
        } else if let Some(rel) = release {
            if let Some(update) = updates.iter().find(|u| u.release == rel) {
                self.display_update_blame(&pkg.name, update);
            } else {
                println!(
                    "{}",
                    t!(
                        "query_error_release_not_found",
                        package = package_name,
                        release = rel
                    )
                );
            }
        } else if let Some(update) = updates.first() {
            self.display_update_blame(&pkg.name, update);
        }
        Ok(())
    }

    fn display_update_blame(&self, name: &str, update: &crate::package::Update) {
        println!(
            "{}: {}, {}: {}, {}: {}",
            t!("query_label_name"),
            name,
            t!("query_label_version"),
            update.version,
            t!("query_label_release"),
            update.release
        );
        println!(
            "{}: {} <{}>",
            t!("query_label_blame_updater"),
            update.name,
            update.email.as_deref().unwrap_or("")
        );
        println!("{}: {}", t!("query_label_date"), update.date);
        println!("\n{}\n", update.comment);
        println!("{}", "-".repeat(50));
    }

    pub fn perform_list_components(&self) -> LuppoResult<()> {
        let components = self.db.list_components()?;

        if components.is_empty() {
            let available = self.db.list_available_packages()?;
            let mut comp_map: HashMap<String, usize> = HashMap::new();
            for pkg in available {
                let comp = if pkg.partof.is_empty() {
                    t!("query_label_description_none").to_string()
                } else {
                    pkg.partof.clone()
                };
                *comp_map.entry(comp).or_insert(0) += 1;
            }
            let mut sorted_keys: Vec<_> = comp_map.keys().collect();
            sorted_keys.sort();
            println!("{}", t!("query_components_empty"));
            println!("{:-<45}", "");
            for comp in sorted_keys {
                println!("  • {:<30} {}", comp, t!("query_package_count", count = comp_map[comp]));
            }
        } else {
            println!("{}", t!("query_components_hierarchical"));
            println!("{:-<70}", "");
            for comp in components {
                let summary = comp
                    .summaries
                    .first()
                    .map(|s| s.text.clone())
                    .unwrap_or_else(|| t!("query_label_description_none").to_string());
                println!("  • {:<20} - {}", comp.name, summary);
            }
        }
        Ok(())
    }

    pub fn get_packages_for_component(&self, comp_name: &str) -> LuppoResult<Vec<String>> {
        let available = self.db.list_available_packages()?;

        let mut packages = std::collections::HashSet::new();
        let mut visited_components = std::collections::HashSet::new();
        let mut component_queue = vec![comp_name.to_string()];

        while let Some(current_comp) = component_queue.pop() {
            if !visited_components.insert(current_comp.clone()) {
                continue; // Already processed, avoid infinite loop
            }

            // 1. Find all packages that are part of this component (or its subcomponents)
            let comp_prefix = format!("{}.", current_comp);
            for pkg in &available {
                if pkg.partof == current_comp || pkg.partof.starts_with(&comp_prefix) {
                    packages.insert(pkg.name.clone());
                }
            }

            // 2. Fetch the component itself to see if it has dependencies
            if let Ok(Some(comp_meta)) = self.db.get_component(&current_comp) {
                if let Some(deps) = comp_meta.dependencies {
                    for dep in deps.dependencies {
                        // If the dependency exists as a component, add to queue.
                        // Otherwise, treat it as a direct package dependency.
                        if let Ok(Some(_)) = self.db.get_component(&dep) {
                            component_queue.push(dep);
                        } else {
                            packages.insert(dep);
                        }
                    }
                }
            }
        }

        let mut sorted_packages: Vec<String> = packages.into_iter().collect();
        sorted_packages.sort();
        Ok(sorted_packages)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perform_check_install(
        &self,
        installer: &Installer,
        names: Vec<String>,
        reinstall: bool,
        yes_all: bool,
        trace_id: u64,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
    ) -> LuppoResult<()> {
        let packages = if names.is_empty() {
            self.db.list_installed_packages()?
        } else {
            let mut pkgs = Vec::new();
            for name in names {
                if let Ok(Some(pkg)) = self.db.get_installed_package(&name) {
                    pkgs.push(pkg);
                } else {
                    println!("{}", t!("query_check_error_not_installed", package = name));
                }
            }
            pkgs
        };
        if packages.is_empty() {
            println!("{}", t!("query_check_empty"));
            return Ok(());
        }
        let mut total_corrupted = 0;
        let mut corrupted_packages = Vec::new();
        for pkg in packages {
            println!(
                "{}",
                t!(
                    "query_check_checking",
                    package = pkg.name,
                    version = pkg.version
                )
            );
            let missing_files: Vec<&String> = pkg
                .installed_files
                .par_iter()
                .filter(|(file_path, _meta)| {
                    let full_path = self
                        .config
                        .general
                        .destination_directory
                        .join(file_path.trim_start_matches('/'));
                    !full_path.exists()
                })
                .map(|(path, _meta)| path)
                .collect();
            if missing_files.is_empty() {
                println!("{}", t!("query_check_success"));
            } else {
                println!(
                    "{}",
                    t!("query_check_corrupted", count = missing_files.len())
                );
                for missing in missing_files {
                    println!("{}", t!("query_check_missing", path = missing));
                }
                corrupted_packages.push(pkg.name.clone());
                total_corrupted += 1;
            }
        }
        if total_corrupted > 0 {
            println!("{}", t!("query_check_summary", count = total_corrupted));
            if reinstall {
                println!("{}", t!("query_check_reinstalling"));
                installer.perform_install(
                    corrupted_packages,
                    trace_id,
                    true,
                    yes_all,
                    limit_kb,
                    auth,
                    false,
                    false,
                    false,
                    false,
                    false,
                    true,  // reinstall
                    false, // ignore_dependency
                    None,
                )?;
            }
        } else {
            println!("{}", t!("query_check_all_success"));
        }
        Ok(())
    }

    pub fn perform_list_newest(&self, limit: usize) -> LuppoResult<()> {
        let mut available = self.db.list_available_packages()?;
        available.sort_by(|a, b| {
            let date_a = a
                .history
                .updates
                .first()
                .map(|u| &u.date)
                .cloned()
                .unwrap_or_default();
            let date_b = b
                .history
                .updates
                .first()
                .map(|u| &u.date)
                .cloned()
                .unwrap_or_default();
            date_b.cmp(&date_a)
        });
        let limited_packages: Vec<&Package> = available.iter().take(limit).collect();
        println!("{}", t!("query_newest_title", limit = limit));
        println!("{:-<65}", "");
        println!(
            "{:<25} {:<15} {:<15}",
            t!("query_label_package_name"),
            t!("query_label_version"),
            t!("query_label_date")
        );
        println!("{:-<65}", "");
        for pkg in &limited_packages {
            let date = pkg
                .history
                .updates
                .first()
                .map(|u| &u.date)
                .cloned()
                .unwrap_or_else(|| t!("query_label_history_unknown").to_string());
            println!("{:<25} {:<15} {:<15}", pkg.name, pkg.latest_version(), date);
        }
        if limited_packages.is_empty() {
            println!("{}", t!("query_newest_empty"));
        }
        Ok(())
    }

    pub fn perform_list_files(&self, package_name: &str) -> LuppoResult<()> {
        if package_name.ends_with(".luppo") && Path::new(package_name).exists() {
            let package_data = Packager::read_package(package_name)?;
            println!(
                "{}",
                t!(
                    "query_files_archive_title",
                    package = package_data.metadata.name,
                    version = package_data.metadata.version
                )
            );
            let mut files: Vec<_> = package_data.files.iter().map(|f| f.path.clone()).collect();
            files.sort();
            for file in files {
                println!("  /{}", file.trim_start_matches('/'));
            }
        } else {
            match self.db.get_installed_package(package_name)? {
                Some(pkg) => {
                    println!(
                        "{}",
                        t!(
                            "query_files_installed_title",
                            package = pkg.name,
                            version = pkg.version
                        )
                    );
                    let mut files: Vec<_> = pkg.installed_files.into_keys().collect();
                    files.sort();
                    for file in files {
                        println!("  {}", file);
                    }
                }
                None => {
                    println!(
                        "{}",
                        t!("query_error_files_not_found", package = package_name)
                    );
                }
            }
        }
        Ok(())
    }

    pub fn perform_list_upgrades(
        &self,
        verbose: bool,
        compare_hashes: bool,
    ) -> LuppoResult<()> {
        let repo_manager = Repository::new(self.db.clone(), self.config.clone());
        let updates = repo_manager.find_updates(&self.db, compare_hashes)?;
        if updates.is_empty() {
            println!("{}", t!("query_upgrades_empty"));
        } else {
            println!("{}", t!("query_upgrades_title"));
            if verbose {
                println!("{:-<115}", "");
                println!(
                    "{:<35} {:<15} {:<30} {:<15} {:<15}",
                    t!("query_label_package_name"),
                    t!("query_label_current"),
                    t!("query_label_new_status"),
                    t!("query_label_repo"),
                    t!("query_label_arch")
                );
                println!("{:-<115}", "");
            } else {
                println!("{:-<85}", "");
                println!(
                    "{:<35} {:<20} {:<30}",
                    t!("query_label_package_name"),
                    t!("query_label_current_version"),
                    t!("query_label_new_status")
                );
                println!("{:-<85}", "");
            }
            for (name, old_ver, reason) in &updates {
                if verbose {
                    if let Ok(Some(remote)) = self.db.get_available_package(name) {
                        println!(
                            "{:<35} {:<15} {:<30} {:<15} {:<15}",
                            name,
                            old_ver,
                            reason,
                            if remote.repo_name.is_empty() {
                                "stable"
                            } else {
                                &remote.repo_name
                            },
                            remote.architecture
                        );
                        println!(
                            "   ┗━ {}: {} [{}: {}]",
                            t!("query_info_summary"),
                            remote.get_summary(),
                            t!("query_info_package_size"),
                            self.format_size(remote.package_size)
                        );
                    } else {
                        println!("{:<35} {:<15} {:<30}", name, old_ver, reason);
                    }
                } else {
                    println!("{:<35} {:<20} {:<30}", name, old_ver, reason);
                }
            }
            let sep = if verbose {
                "-".repeat(115)
            } else {
                "-".repeat(85)
            };
            println!("{}", sep);
            println!("{}", t!("query_upgrades_count", count = updates.len()));
            println!("{}", t!("query_upgrades_info"));
        }
        Ok(())
    }

    pub fn perform_list_history(
        &self,
        from: Option<String>,
        to: Option<String>,
    ) -> LuppoResult<()> {
        let mut actions = self.db.list_history(None)?;
        if actions.is_empty() {
            println!("{}", t!("query_history_empty"));
            return Ok(());
        }
        if from.is_some() || to.is_some() {
            actions.retain(|action| {
                let action_date = action.timestamp.split_whitespace().next().unwrap_or("");
                let is_after = from.as_deref().is_none_or(|f| action_date >= f);
                let is_before = to.as_deref().is_none_or(|t| action_date <= t);
                is_after && is_before
            });
            if actions.is_empty() {
                println!("{}", t!("query_history_range_empty"));
                return Ok(());
            }
        }
        actions.sort_by(|a, b| b.trace_id.cmp(&a.trace_id));
        println!("{}", t!("query_history_title"));
        println!("{:-<100}", "");
        println!(
            "{:<5} | {:<19} | {:<15} | {}",
            t!("query_label_id"),
            t!("query_label_date"),
            t!("query_label_operation"),
            t!("query_label_detail")
        );
        println!("{:-<100}", "");
        struct GroupedAction {
            trace_id: u64,
            date: String,
            operation: String,
            details: Vec<String>,
        }

        let mut grouped: Vec<GroupedAction> = Vec::new();
        for action in actions {
            let op_display = match action.operation.as_str() {
                "install" => t!("query_op_install").to_string(),
                "remove" => t!("query_op_remove").to_string(),
                "update" => t!("query_op_update").to_string(),
                "repo_add" => t!("query_op_repo_add").to_string(),
                "repo_remove" => t!("query_op_repo_remove").to_string(),
                "repo_enable" => t!("query_op_repo_enable").to_string(),
                "repo_disable" => t!("query_op_repo_disable").to_string(),
                "rollback" => t!("query_op_rollback").to_string(),
                "autoremove" => t!("query_op_autoremove").to_string(),
                _ => action.operation.clone(),
            };
            let clean_date = action
                .timestamp
                .split('.')
                .next()
                .unwrap_or(&action.timestamp)
                .to_string();
            let details = match action.operation.as_str() {
                "install" => t!("query_detail_installed", details = action.details).to_string(),
                "remove" => t!("query_detail_removed", details = action.details).to_string(),
                _ => action.details.clone(),
            };

            if let Some(last) = grouped.last_mut() {
                if last.trace_id == action.trace_id {
                    last.details.push(details);
                    continue;
                }
            }
            grouped.push(GroupedAction {
                trace_id: action.trace_id,
                date: clean_date,
                operation: op_display,
                details: vec![details],
            });
        }

        for group in grouped {
            if group.details.len() == 1 {
                println!(
                    "{:<5} | {:<19} | {:<15} | {}",
                    group.trace_id, group.date, group.operation, group.details[0]
                );
            } else {
                println!(
                    "{:<5} | {:<19} | {:<15} | {}",
                    group.trace_id, group.date, group.operation, group.details[0]
                );
                for detail in &group.details[1..] {
                    println!(
                        "{:<5} | {:<19} | {:<15} | {}",
                        "", "", "", detail
                    );
                }
            }
        }
        println!("{:-<100}", "");
        Ok(())
    }

    pub fn perform_info(&self, package_name: &str) -> LuppoResult<()> {
        let installed = self.db.get_installed_package(package_name).ok().flatten();
        let available = self.db.get_available_package(package_name).ok().flatten();
        let reverse_deps = self.get_reverse_deps(package_name)?;

        // 1. Kurulu paket bölümü
        if let Some(ref inst) = installed {
            println!("{}", t!("query_info_installed_title"));

            let version = &inst.version;
            let release = inst.release;
            println!(
                "{:<20}: {}, {}: {}, {}: {}",
                t!("query_label_name"),
                inst.name,
                t!("query_label_version"),
                version,
                t!("query_label_release"),
                release
            );

            let summary = available
                .as_ref()
                .map(|p| p.get_summary())
                .unwrap_or_else(|| t!("query_label_description_none").to_string());
            println!("{:<20}: {}", t!("query_info_summary"), summary);

            let description = available
                .as_ref()
                .map(|p| p.get_description())
                .unwrap_or_else(|| inst.description.clone());
            let wrapped_desc = self.wrap_text(&description, 22);
            println!("{:<20}: {}", t!("query_info_description"), wrapped_desc);

            let licenses = if !inst.licenses.is_empty() {
                inst.licenses.join(" ")
            } else {
                available
                    .as_ref()
                    .map(|p| p.licenses.join(" "))
                    .unwrap_or_default()
            };
            println!("{:<20}: {}", t!("query_info_licenses"), licenses);

            println!(
                "{:<20}: {}",
                t!("query_info_component"),
                available
                    .as_ref()
                    .map(|p| p.partof.clone())
                    .unwrap_or_default()
            );

            let provides = if !inst.provides.is_empty() {
                inst.provides.join(" ")
            } else {
                available
                    .as_ref()
                    .map(|p| p.provides.join(" "))
                    .unwrap_or_default()
            };
            println!("{:<20}: {} ", t!("query_info_provides"), provides);

            let deps = available
                .as_ref()
                .and_then(|p| p.runtime_dependencies.as_ref())
                .map(|d| d.dependencies.join(" "))
                .unwrap_or_default();
            println!("{:<20}: {} ", t!("query_info_dependencies"), deps);

            println!(
                "{:<20}: LupuS, {}: {}",
                t!("query_info_distribution"),
                t!("query_info_dist_release"),
                inst.distribution_release
            );

            let arch = available
                .as_ref()
                .map(|p| p.architecture.as_str())
                .unwrap_or("x86_64");
            println!(
                "{:<20}: {}, {}: {}, install.tar.xz sha1sum: {}",
                t!("query_label_arch"),
                arch,
                t!("query_info_installed_size"),
                self.format_size(inst.total_size),
                inst.package_hash
            );

            println!(
                "{:<20}: {} ",
                t!("query_info_reverse_deps"),
                reverse_deps.join(" ")
            );
        } else {
            println!("{}", t!("query_info_not_installed", package = package_name));
        }

        // 2. Depo paketi bölümü
        if let Some(ref repo_pkg) = available {
            let repo_name = if repo_pkg.repo_name.is_empty() {
                "Stable"
            } else {
                &repo_pkg.repo_name
            };
            println!("{}", t!("query_info_remote_title", repo = repo_name));

            println!(
                "{:<20}: {}, {}: {}, {}: {}",
                t!("query_label_name"),
                repo_pkg.name,
                t!("query_label_version"),
                repo_pkg.latest_version(),
                t!("query_label_release"),
                repo_pkg.release
            );

            println!(
                "{:<20}: {}",
                t!("query_info_summary"),
                repo_pkg.get_summary()
            );

            let wrapped_desc = self.wrap_text(&repo_pkg.get_description(), 22);
            println!("{:<20}: {}", t!("query_info_description"), wrapped_desc);

            println!(
                "{:<20}: {}",
                t!("query_info_licenses"),
                repo_pkg.licenses.join(" ")
            );

            println!("{:<20}: {}", t!("query_info_component"), repo_pkg.partof);

            println!(
                "{:<20}: {} ",
                t!("query_info_provides"),
                repo_pkg.provides.join(" ")
            );

            let deps = repo_pkg
                .runtime_dependencies
                .as_ref()
                .map(|d| d.dependencies.join(" "))
                .unwrap_or_default();
            println!("{:<20}: {} ", t!("query_info_dependencies"), deps);

            println!(
                "{:<20}: LupuS, {}: {}",
                t!("query_info_distribution"),
                t!("query_info_dist_release"),
                repo_pkg.distribution_release
            );

            println!(
                "{:<20}: {}, {}: {}, {}: {}, install.tar.xz sha1sum: {}",
                t!("query_label_arch"),
                repo_pkg.architecture,
                t!("query_info_installed_size"),
                self.format_size(repo_pkg.installed_size),
                t!("query_info_package_size"),
                self.format_size(repo_pkg.package_size),
                repo_pkg.package_hash
            );

            println!(
                "{:<20}: {} ",
                t!("query_info_reverse_deps"),
                reverse_deps.join(" ")
            );
        }

        // 3. Kaynak depo mesajı (her zaman göster)
        println!(
            "{}",
            t!("query_info_source_not_found", package = package_name)
        );

        if installed.is_none() && available.is_none() {
            println!("{}", t!("query_error_info_not_found"));
        }

        Ok(())
    }

    /// Uzun metinleri belirli bir girinti ile satır sonlarında kelime sınırına göre sarar.
    fn wrap_text(&self, text: &str, indent: usize) -> String {
        let term_width = 160; // Geniş terminal genişliği
        let text_width = term_width - indent;
        if text.len() <= text_width {
            return text.to_string();
        }

        let indent_str = " ".repeat(indent);
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() > text_width {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                current_line.push(' ');
                current_line.push_str(word);
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines.join(&format!("\n{}", indent_str))
    }

    fn get_reverse_deps(&self, package_name: &str) -> LuppoResult<Vec<String>> {
        let mut dependents = Vec::new();
        let available = self.db.list_available_packages()?;
        for pkg in available {
            if let Some(runtime) = pkg.runtime_dependencies {
                if runtime.dependencies.contains(&package_name.to_string()) {
                    dependents.push(pkg.name);
                }
            }
        }
        Ok(dependents)
    }

    fn format_size(&self, bytes: u64) -> String {
        let kb = 1024.0;
        let mb = kb * 1024.0;
        let gb = mb * 1024.0;
        let b = bytes as f64;

        if b >= gb {
            format!("{:.2} GB", b / gb)
        } else if b >= mb {
            format!("{:.2} MB", b / mb)
        } else if b >= kb {
            format!("{:.2} KB", b / kb)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn perform_info_hybrid(&self, name: &str) -> LuppoResult<()> {
        println!("{}", t!("query_hybrid_searching", name = name));
        let mut found = false;
        if let Ok(Some(pkg)) = self.db.get_installed_package(name) {
            println!(
                "{}",
                t!(
                    "query_hybrid_installed",
                    package = pkg.name,
                    version = pkg.version
                )
            );
            println!("📝 {}", pkg.description);
            println!(
                "{}",
                t!("query_hybrid_install_date", date = pkg.install_date)
            );
            found = true;
        }
        if let Ok(Some(pkg)) = self.db.get_available_package(name) {
            if found {
                println!("---");
            }
            println!(
                "{}",
                t!(
                    "query_hybrid_available",
                    package = pkg.name,
                    version = pkg.latest_version()
                )
            );
            println!("📝 {}", pkg.get_summary());
            found = true;
        }
        if !found {
            println!("{}", t!("query_hybrid_not_found", name = name));
        }
        Ok(())
    }

    pub fn search_file(&self, query: &str) -> LuppoResult<Vec<(String, String)>> {
        self.db.search_file(query)
    }

    pub fn get_package_info(&self, name: &str) -> LuppoResult<Option<PackageDefinition>> {
        let pkg_res = self.db.get_package(name)?;

        Ok(pkg_res.map(|pkg| {
            let current_version = pkg
                .history
                .updates
                .first()
                .map(|u| u.version.clone())
                .unwrap_or_else(|| "0.0.0".to_string());

            let summary_text = pkg
                .summaries
                .first()
                .map(|s| s.text.clone())
                .unwrap_or_default();

            let desc_text = pkg
                .descriptions
                .first()
                .map(|d| d.text.clone())
                .unwrap_or_default();

            let runtime_deps = pkg
                .runtime_dependencies
                .clone()
                .map(|r| r.dependencies)
                .unwrap_or_default();
            let build_deps = pkg
                .build_dependencies
                .clone()
                .map(|b| b.dependencies)
                .unwrap_or_default();

            PackageDefinition {
                name: pkg.name.clone(),
                version: current_version,
                summary: summary_text,
                description: desc_text,
                homepage: pkg.homepage.clone(),
                icon: pkg.icon.clone(),
                screenshot: pkg.screenshot.clone(),
                provides: Some(luppo_spec::models::ProvidesBlock {
                    isa: pkg.provides.clone(),
                    comar: Vec::new(),
                }),
                additional_files: None,
                build_type: None,
                license: "GPLv3".to_string(),
                packager: luppo_spec::models::Packager {
                    name: "Luppo Team".to_string(),
                    email: "admin@antolun.com".to_string(),
                },
                deps: luppo_spec::models::Dependencies {
                    runtime: runtime_deps.clone(),
                    build: build_deps,
                    conflicts: Vec::new(),
                },
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
                files: luppo_spec::models::Files::default(),
                runtime_dependencies: Some(luppo_spec::models::RuntimeDeps {
                    dependencies: runtime_deps
                        .into_iter()
                        .map(|d| luppo_spec::models::Dependency {
                            name: d,
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }))
    }

    // Eski hali ActionEntry idi, HistoryAction olarak güncelliyoruz
    pub fn list_history(&self) -> LuppoResult<Vec<luppo_spec::models::HistoryAction>> {
        self.db.list_history(None)
    }

    pub fn find_orphaned_packages(&self) -> LuppoResult<Vec<String>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::database::LuppoDatabase;
    use crate::package::{Component, ComponentDependencies};
    use tempfile::tempdir;

    fn make_db() -> (LuppoDatabase, tempfile::TempDir) {
        let dir = tempdir().expect("Failed to create tempdir");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("Failed to open DB");
        (db, dir)
    }

    fn make_component(name: &str, deps: Vec<&str>) -> Component {
        Component {
            name: name.to_string(),
            local_names: vec![],
            summaries: vec![],
            descriptions: vec![],
            group: None,
            maintainer: None,
            dependencies: if deps.is_empty() {
                None
            } else {
                Some(ComponentDependencies {
                    dependencies: deps.into_iter().map(|s| s.to_string()).collect(),
                })
            },
        }
    }

    /// Bir bileşenin sahip olduğu paketlerin doğru listelendiğini test eder.
    #[test]
    fn test_component_basic_packages() {
        let (db, _dir) = make_db();
        let config = Config::load(None);

        // system.base bileşenine ait iki paket ekle
        let pkg_json = |name: &str, partof: &str| -> String {
            format!(
                r#"{{
                "Name": "{name}",
                "Summary": [],
                "Description": [],
                "History": {{"Update": []}},
                "Architecture": "x86_64",
                "InstalledSize": 0,
                "PartOf": "{partof}"
            }}"#
            )
        };
        let p1: Package = serde_json::from_str(&pkg_json("glibc", "system.base")).unwrap();
        let p2: Package = serde_json::from_str(&pkg_json("bash", "system.base")).unwrap();
        let p3: Package = serde_json::from_str(&pkg_json("gcc", "system.devel")).unwrap();
        db.save_package(&p1).unwrap();
        db.save_package(&p2).unwrap();
        db.save_package(&p3).unwrap();

        let query = QueryManager::new(db, config);
        let mut pkgs = query.get_packages_for_component("system.base").unwrap();
        pkgs.sort();
        assert_eq!(pkgs, vec!["bash", "glibc"]);
    }

    /// Bileşen bağımlılıklarının (component → component) özyinelemeli çözümlendiğini test eder.
    #[test]
    fn test_component_dependency_resolution() {
        let (db, _dir) = make_db();
        let config = Config::load(None);

        // system.devel bileşeni system.base bileşenine bağımlı olacak
        let comp_base = make_component("system.base", vec![]);
        let comp_devel = make_component("system.devel", vec!["system.base"]);
        db.save_component(&comp_base).unwrap();
        db.save_component(&comp_devel).unwrap();

        let full_pkg = |name: &str, partof: &str| -> Package {
            serde_json::from_str(&format!(
                r#"{{
                "Name": "{name}",
                "Summary": [],
                "Description": [],
                "History": {{"Update": []}},
                "Architecture": "x86_64",
                "InstalledSize": 0,
                "PartOf": "{partof}"
            }}"#
            ))
            .unwrap()
        };

        db.save_package(&full_pkg("glibc", "system.base")).unwrap();
        db.save_package(&full_pkg("gcc", "system.devel")).unwrap();

        let query = QueryManager::new(db, config);

        // system.devel kurulduğunda system.base'deki glibc de gelmeli
        let mut pkgs = query.get_packages_for_component("system.devel").unwrap();
        pkgs.sort();
        assert!(
            pkgs.contains(&"glibc".to_string()),
            "system.base bağımlılığındaki glibc eksik!"
        );
        assert!(
            pkgs.contains(&"gcc".to_string()),
            "system.devel paketi gcc eksik!"
        );
    }

    /// Döngüsel bağımlılık (A → B → A) durumunda sonsuz döngü oluşmadığını test eder.
    #[test]
    fn test_component_circular_dependency_protection() {
        let (db, _dir) = make_db();
        let config = Config::load(None);

        // Döngüsel bağımlılık: comp-a ↔ comp-b
        let comp_a = make_component("comp-a", vec!["comp-b"]);
        let comp_b = make_component("comp-b", vec!["comp-a"]);
        db.save_component(&comp_a).unwrap();
        db.save_component(&comp_b).unwrap();

        // comp-a'ya ait bir paket
        let p: Package = serde_json::from_str(
            r#"{
            "Name": "pkg-a",
            "Summary": [],
            "Description": [],
            "History": {"Update": []},
            "Architecture": "x86_64",
            "InstalledSize": 0,
            "PartOf": "comp-a"
        }"#,
        )
        .unwrap();
        db.save_package(&p).unwrap();

        let query = QueryManager::new(db, config);

        // Sonsuz döngü oluşmamalı, sonuç dönmeli
        let result = query.get_packages_for_component("comp-a");
        assert!(
            result.is_ok(),
            "Döngüsel bağımlılık sonsuz döngüye yol açtı!"
        );
    }

    /// Bileşen bağımlılığında doğrudan paket referansı (bileşen olmayan) desteklendiğini test eder.
    #[test]
    fn test_component_direct_package_dependency() {
        let (db, _dir) = make_db();
        let config = Config::load(None);

        // comp-with-dep, doğrudan "extra-pkg" paketine bağımlı (bu bir bileşen değil)
        let comp = make_component("comp-with-dep", vec!["extra-pkg"]);
        db.save_component(&comp).unwrap();

        let full_pkg = |name: &str, partof: &str| -> Package {
            serde_json::from_str(&format!(
                r#"{{
                "Name": "{name}",
                "Summary": [],
                "Description": [],
                "History": {{"Update": []}},
                "Architecture": "x86_64",
                "InstalledSize": 0,
                "PartOf": "{partof}"
            }}"#
            ))
            .unwrap()
        };

        db.save_package(&full_pkg("main-pkg", "comp-with-dep"))
            .unwrap();
        db.save_package(&full_pkg("extra-pkg", "other")).unwrap();

        let query = QueryManager::new(db, config);
        let mut pkgs = query.get_packages_for_component("comp-with-dep").unwrap();
        pkgs.sort();
        assert!(pkgs.contains(&"main-pkg".to_string()));
        assert!(
            pkgs.contains(&"extra-pkg".to_string()),
            "Doğrudan paket bağımlılığı eksik!"
        );
    }
}
