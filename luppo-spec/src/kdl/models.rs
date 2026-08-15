use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Patch {
    pub file: String,
    pub level: Option<u8>,
    pub compression_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PatchesWrapper {
    pub patches: Vec<Patch>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TranslationEntry {
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AdditionalFile {
    pub filename: String,
    pub owner: Option<String>,
    pub permission: Option<String>,
    pub target: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AdditionalFilesWrapper {
    pub files: Vec<AdditionalFile>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ReplacesWrapper {
    pub packages: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ConflictsWrapper {
    pub packages: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BuildFlagsWrapper {
    pub flags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnyDependency {
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Archive {
    pub url: String,
    pub sha1sum: Option<String>,
    pub md5sum: Option<String>,
    pub hash: Option<String>,
    pub archive_type: String,
    pub target: Option<String>,
}

impl Archive {
    pub fn get_hash(&self) -> (String, String) {
        if let Some(h) = &self.sha1sum {
            (h.clone(), "sha1".to_string())
        } else if let Some(h) = &self.md5sum {
            (h.clone(), "md5".to_string())
        } else if let Some(h) = &self.hash {
            (h.clone(), "sha256".to_string())
        } else {
            (String::new(), "unknown".to_string())
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Packager {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Dependency {
    pub name: String,
    pub release: Option<String>,
    pub version_from: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Source {
    pub name: String,
    pub homepage: Option<String>,
    pub packager: Option<Packager>,
    pub license: Vec<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub translations: std::collections::HashMap<String, TranslationEntry>,
    pub part_of: Option<String>,
    pub archives: Vec<Archive>,
    pub archive_url_kdl: Option<String>,
    pub archive_sha1_kdl: Option<String>,
    pub build_dependencies: Option<BuildDeps>,
    pub build_flags: Option<BuildFlagsWrapper>,
    pub patches: Option<PatchesWrapper>,
    pub additional_files: Option<AdditionalFilesWrapper>,
    pub history: Option<History>,
    pub history_kdl: Option<Vec<Update>>,
    pub icon: Option<String>,
    pub screenshot: Option<String>,
    pub provides: Vec<String>,
    pub architecture: Option<String>,
    pub environment: Option<EnvironmentVars>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct EnvironmentVar {
    pub name: String,
    pub value: Option<String>,
    pub force: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct EnvironmentVars {
    pub vars: Vec<EnvironmentVar>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BuildDeps {
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PathDef {
    pub path: String,
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Files {
    pub paths: Vec<PathDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Dependencies {
    pub runtime: Vec<String>,
    pub build: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Comar {
    pub provide: String,
    pub script: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ProvidesBlock {
    pub comar: Vec<Comar>,
    pub isa: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ConfigureAction {
    pub args: Option<String>,
    pub prefix: Option<String>,
    pub libdir: Option<String>,
    pub libexecdir: Option<String>,
    pub sysconfdir: Option<String>,
    pub localstatedir: Option<String>,
    pub datadir: Option<String>,
    pub mandir: Option<String>,
    pub infodir: Option<String>,
    pub with_systemd: Option<bool>,
    pub host: Option<String>,
    pub build: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageActions {
    pub steps: Vec<String>,
    /// Her adımın tipi: "setup", "build", "check", "install"
    pub step_types: Vec<String>,
    pub configure: Option<ConfigureAction>,
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_upgrade: Option<String>,
    pub post_upgrade: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
    pub install_filters: Vec<String>,
    pub no_strip: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct User {
    pub name: String,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub home: Option<String>,
    pub shell: Option<String>,
    pub system: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UsersWrapper {
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Group {
    pub name: String,
    pub gid: Option<u32>,
    pub system: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GroupsWrapper {
    pub groups: Vec<Group>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageDefinition {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub translations: std::collections::HashMap<String, TranslationEntry>,
    pub runtime_dependencies: Option<RuntimeDeps>,
    pub build_dependencies: Option<BuildDeps>,
    pub files: Files,
    pub homepage: Option<String>,
    pub icon: Option<String>,
    pub screenshot: Option<String>,
    pub provides: Option<ProvidesBlock>,
    pub additional_files: Option<AdditionalFilesWrapper>,
    pub part_of: Option<String>,
    pub replaces: Option<ReplacesWrapper>,
    pub conflicts: Option<ConflictsWrapper>,
    pub build_type: Option<String>,
    pub users: Option<UsersWrapper>,
    pub groups: Option<GroupsWrapper>,
    pub mirrors: Vec<String>,
    pub version: String,
    pub license: String,
    pub packager: Packager,
    pub deps: Dependencies,
    pub actions: PackageActions,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct RuntimeDeps {
    pub dependencies: Vec<Dependency>,
    pub any_dependency: Option<AnyDependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Update {
    pub release: u32,
    pub date: String,
    pub version: String,
    pub comment: String,
    pub committer: String,
    pub email: String,
    #[serde(rename = "type", alias = "Type", default)]
    pub type_: Option<String>,
    pub requires: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct History {
    pub updates: Vec<Update>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LuppoSpec {
    pub source: Source,
    pub packages: Vec<PackageDefinition>,
    pub history: Option<History>,
    pub history_kdl: Option<Vec<Update>>,
    pub main_package: Option<PackageDefinition>,
}

impl LuppoSpec {
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref).map_err(|e| e.to_string())?;

        #[allow(unused_mut)]
        let mut spec = if path_ref.extension().is_some_and(|ext| ext == "kdl") {
        let mut spec: Self = crate::kdl::parse_kdl_spec_from_str(&content)?;

            if let Some(main) = spec.main_package.take() {
                spec.packages.insert(0, main);
            }

            spec.populate_compat_fields();
            spec
        } else {
            #[cfg(feature = "legacy-xml")]
            {
                let xml_spec = crate::xml::parse_xml_spec(path_ref)?;
                let mut spec: Self = Self::from_xml_models(xml_spec);
                spec.populate_compat_fields();
                spec
            }
            #[cfg(not(feature = "legacy-xml"))]
            {
                return Err(format!(
                    "XML support is disabled (legacy-xml feature flag is off): {:?}",
                    path_ref
                ));
            }
        };

        #[cfg(feature = "legacy-xml")]
        {
            if path_ref.extension().is_none_or(|ext| ext != "xml") {
                if let Some(parent) = path_ref.parent() {
                    let trans_path = parent.join("translations.xml");
                    if trans_path.exists() {
                        let _ = spec.merge_translations_from_xml(&trans_path);
                    }
                }
            }
        }

        Ok(spec)
    }

    fn from_xml_models(xml_spec: crate::xml::models::LuppoSpec) -> Self {
        Self {
            source: Source {
                name: xml_spec.source.name.clone(),
                homepage: xml_spec.source.homepage.clone(),
                packager: xml_spec.source.packager.map(|p| Packager {
                    name: p.name,
                    email: p.email,
                }),
                license: xml_spec.source.license.clone(),
                summary: xml_spec.source.summary.clone(),
                description: xml_spec.source.description.clone(),
                translations: xml_spec.source.translations.iter().map(|(lang, t)| {
                    (lang.clone(), TranslationEntry {
                        summary: t.summary.clone(),
                        description: t.description.clone(),
                    })
                }).collect(),
                part_of: None,
                archives: xml_spec.source.archives.iter().map(|a| Archive {
                    url: a.url.clone(),
                    sha1sum: a.sha1sum.clone(),
                    md5sum: a.md5sum.clone(),
                    hash: a.hash.clone(),
                    archive_type: a.archive_type.clone(),
                    target: a.target.clone(),
                }).collect(),
                archive_url_kdl: None,
                archive_sha1_kdl: None,
                build_dependencies: {
                    let deps: Vec<Dependency> = xml_spec.source.build_dependencies.iter()
                        .flat_map(|bd| bd.dependencies.iter())
                        .map(|d| Dependency {
                            name: d.name.clone(),
                            release: d.release.clone(),
                            version_from: d.version_from.clone(),
                        })
                        .collect();
                    if deps.is_empty() { None } else { Some(BuildDeps { dependencies: deps }) }
                },
                build_flags: None,
                patches: {
                    let patches: Vec<Patch> = xml_spec.source.patches.iter()
                        .flat_map(|pw| pw.patches.iter())
                        .map(|p| Patch {
                            file: p.file.clone(),
                            level: p.level,
                            compression_type: p.compression_type.clone(),
                        })
                        .collect();
                    if patches.is_empty() { None } else { Some(PatchesWrapper { patches }) }
                },
                additional_files: None,
                history: None,
                history_kdl: None,
                icon: xml_spec.source.icon.clone(),
                screenshot: xml_spec.source.screenshot.clone(),
                provides: xml_spec.source.provides,
                architecture: None,
                environment: None,
            },
            packages: xml_spec.packages.into_iter().map(|p| PackageDefinition {
                name: p.name.clone(),
                summary: p.summary.clone(),
                description: p.description.clone(),
                translations: p.translations.iter().map(|(lang, t)| {
                    (lang.clone(), TranslationEntry {
                        summary: t.summary.clone(),
                        description: t.description.clone(),
                    })
                }).collect(),
                runtime_dependencies: p.runtime_dependencies.map(|rd| RuntimeDeps {
                    dependencies: rd.dependencies.into_iter().map(|d| Dependency {
                        name: d.name,
                        release: d.release,
                        version_from: d.version_from,
                    }).collect(),
                    any_dependency: None,
                }),
                build_dependencies: p.build_dependencies.map(|bd| BuildDeps {
                    dependencies: bd.dependencies.into_iter().map(|d| Dependency {
                        name: d.name,
                        release: d.release,
                        version_from: d.version_from,
                    }).collect(),
                }),
                files: Files {
                    paths: p.files.paths.into_iter().map(|path_def| PathDef {
                        path: path_def.path,
                        file_type: path_def.file_type,
                    }).collect(),
                },
                homepage: p.homepage.clone(),
                icon: p.icon.clone(),
                screenshot: p.screenshot.clone(),
                provides: p.provides.map(|pb| ProvidesBlock {
                    comar: pb.comar.into_iter().map(|c| Comar {
                        provide: c.provide,
                        script: c.script,
                        name: c.name,
                    }).collect(),
                    isa: pb.isa,
                }),
                additional_files: p.additional_files.map(|af| AdditionalFilesWrapper {
                    files: af.files.into_iter().map(|f| AdditionalFile {
                        filename: f.filename,
                        owner: f.owner,
                        permission: f.permission,
                        target: f.target,
                    }).collect(),
                }),
                part_of: p.part_of.clone(),
                replaces: p.replaces.map(|r| ReplacesWrapper { packages: r.packages }),
                conflicts: p.conflicts.map(|c| ConflictsWrapper { packages: c.packages }),
                build_type: p.build_type.clone(),
                users: p.users.map(|u| UsersWrapper { users: u.users.into_iter().map(|user| User {
                    name: user.name, uid: user.uid, gid: user.gid,
                    home: user.home, shell: user.shell, system: user.system,
                }).collect() }),
                groups: p.groups.map(|g| GroupsWrapper { groups: g.groups.into_iter().map(|group| Group {
                    name: group.name, gid: group.gid, system: group.system,
                }).collect() }),
                mirrors: p.mirrors,
                version: String::new(),
                license: String::new(),
                packager: Packager::default(),
                deps: Dependencies::default(),
                actions: PackageActions::default(),
            }).collect(),
            history: xml_spec.history.map(|h| History {
                updates: h.updates.into_iter().map(|u| Update {
                    release: u.release,
                    date: u.date,
                    version: u.version,
                    comment: u.comment,
                    committer: u.committer,
                    email: u.email,
                    type_: u.type_,
                    requires: u.requires.and_then(|r| r.actions.first().map(|a| a.action.clone())),
                }).collect(),
            }),
            history_kdl: None,
            main_package: None,
        }
    }

    fn populate_compat_fields(&mut self) {
        if self.history.is_none() {
            if let Some(updates) = self.history_kdl.take() {
                self.history = Some(History { updates });
            } else if let Some(updates) = self.source.history_kdl.take() {
                self.history = Some(History { updates });
            }
        }

        if self.history.is_none() && self.source.history.is_some() {
            self.history = self.source.history.take();
        }

        if self.source.archives.is_empty() {
            if let Some(url) = self.source.archive_url_kdl.clone() {
                self.source.archives.push(Archive {
                    url,
                    sha1sum: self.source.archive_sha1_kdl.clone(),
                    md5sum: None,
                    hash: None,
                    archive_type: "targz".to_string(),
                    target: None,
                });
            }
        }

        let version = self
            .history
            .as_ref()
            .and_then(|h| h.updates.first().map(|u| u.version.clone()))
            .unwrap_or_default();
        let license = self.source.license.join(", ");
        let packager = self.source.packager.clone().unwrap_or_default();
        let build_deps: Vec<String> = self
            .source
            .build_dependencies
            .as_ref()
            .map(|bd| bd.dependencies.iter().map(|d| d.name.clone()).collect())
            .unwrap_or_default();

        let homepage = self.source.homepage.clone();
        let icon = self.source.icon.clone();
        let screenshot = self.source.screenshot.clone();
        let provides = self.source.provides.clone();

        for pkg in &mut self.packages {
            if pkg.version.is_empty() {
                pkg.version = version.clone();
            }
            if pkg.license.is_empty() {
                pkg.license = license.clone();
            }
            if pkg.packager.name.is_empty() {
                pkg.packager = packager.clone();
            }
            if pkg.deps.build.is_empty() {
                if let Some(bd) = &pkg.build_dependencies {
                    pkg.deps.build = bd.dependencies.iter().map(|d| d.name.clone()).collect();
                }
            }
            if pkg.deps.build.is_empty() {
                pkg.deps.build = build_deps.clone();
            }
            if pkg.homepage.is_none() {
                pkg.homepage = homepage.clone();
            }
            if pkg.icon.is_none() {
                pkg.icon = icon.clone();
            }
            if pkg.screenshot.is_none() {
                pkg.screenshot = screenshot.clone();
            }
            if pkg.provides.is_none() && !provides.is_empty() {
                pkg.provides = Some(ProvidesBlock {
                    isa: provides.clone(),
                    comar: Vec::new(),
                });
            } else if let Some(ref mut p) = pkg.provides {
                if p.isa.is_empty() && !provides.is_empty() {
                    p.isa = provides.clone();
                }
            }

            if pkg.deps.runtime.is_empty() {
                pkg.deps.runtime = pkg
                    .runtime_dependencies
                    .as_ref()
                    .map(|rd| rd.dependencies.iter().map(|d| d.name.clone()).collect())
                    .unwrap_or_default();
            }
        }
    }

    #[cfg(feature = "legacy-xml")]
    pub fn merge_translations_from_xml<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let doc = roxmltree::Document::parse(&content).map_err(|e| e.to_string())?;

        if let Some(source_node) = doc.descendants().find(|n| n.has_tag_name("Source")) {
            for child in source_node.children() {
                if child.has_tag_name("Summary") {
                    if let Some(lang) = child
                        .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                        .or_else(|| child.attribute("lang"))
                    {
                        let text = child.text().unwrap_or("").trim().to_string();
                        let entry = self
                            .source
                            .translations
                            .entry(lang.to_string())
                            .or_default();
                        entry.summary = Some(text);
                    }
                } else if child.has_tag_name("Description") {
                    if let Some(lang) = child
                        .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                        .or_else(|| child.attribute("lang"))
                    {
                        let text = child.text().unwrap_or("").trim().to_string();
                        let entry = self
                            .source
                            .translations
                            .entry(lang.to_string())
                            .or_default();
                        entry.description = Some(text);
                    }
                }
            }
        }

        for pkg_node in doc.descendants().filter(|n| n.has_tag_name("Package")) {
            let name_node = pkg_node.children().find(|n| n.has_tag_name("Name"));
            if let Some(n_node) = name_node {
                let pkg_name = n_node.text().unwrap_or("").trim();
                if let Some(pkg) = self.packages.iter_mut().find(|p| p.name == pkg_name) {
                    for child in pkg_node.children() {
                        if child.has_tag_name("Summary") {
                            if let Some(lang) = child
                                .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                                .or_else(|| child.attribute("lang"))
                            {
                                let text = child.text().unwrap_or("").trim().to_string();
                                let entry = pkg.translations.entry(lang.to_string()).or_default();
                                entry.summary = Some(text);
                            }
                        } else if child.has_tag_name("Description") {
                            if let Some(lang) = child
                                .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                                .or_else(|| child.attribute("lang"))
                            {
                                let text = child.text().unwrap_or("").trim().to_string();
                                let entry = pkg.translations.entry(lang.to_string()).or_default();
                                entry.description = Some(text);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TriggerEntry {
    pub name: String,
    pub path: String,
    pub suffix: Option<String>,
    pub script: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TriggersConfig {
    pub triggers: Vec<TriggerEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryAction {
    pub trace_id: u64,
    pub operation: String,
    pub timestamp: String,
    pub details: String,
}

// ── KDL Serializer ──

fn kdl_escape(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    format!("\"{}\"", escaped)
}

#[allow(dead_code)]
fn as_kdl_val(v: &str) -> String {
    if v.is_empty() {
        return String::new();
    }
    if let Ok(_) = v.parse::<i64>() {
        return v.to_string();
    }
    if let Ok(_) = v.parse::<f64>() {
        return v.to_string();
    }
    if v == "true" || v == "false" {
        return v.to_string();
    }
    kdl_escape(v)
}

fn maybe_opt(s: &Option<String>) -> Option<&str> {
    s.as_ref().map(|x| x.as_str()).filter(|x| !x.is_empty())
}

impl Packager {
    pub fn to_kdl_indent(&self, indent: &str) -> String {
        let mut out = format!("{}Packager {{\n", indent);
        out.push_str(&format!("{}{}Name {}\n", indent, "    ", kdl_escape(&self.name)));
        out.push_str(&format!("{}{}Email {}\n", indent, "    ", kdl_escape(&self.email)));
        out.push_str(&format!("{}}}\n", indent));
        out
    }
}

impl Archive {
    pub fn to_kdl_indent(&self, indent: &str) -> String {
        let mut props = String::new();
        if self.archive_type != "targz" || self.url.is_empty() {
            props.push_str(&format!(" type={}", kdl_escape(&self.archive_type)));
        }
        if let Some(ref s) = self.sha1sum {
            props.push_str(&format!(" sha1sum={}", kdl_escape(s)));
        }
        if let Some(ref s) = self.md5sum {
            props.push_str(&format!(" md5sum={}", kdl_escape(s)));
        }
        if let Some(ref s) = self.hash {
            props.push_str(&format!(" hash={}", kdl_escape(s)));
        }
        if let Some(ref t) = self.target {
            props.push_str(&format!(" target={}", kdl_escape(t)));
        }
        if self.archive_type == "targz" && self.sha1sum.is_none() && self.md5sum.is_none() && self.hash.is_none() && self.target.is_none() && !self.url.is_empty() {
            format!("{}{}Archive {}\n", indent, "    ", kdl_escape(&self.url))
        } else if !self.url.is_empty() {
            format!("{}{}Archive{props} {{\n{}{}    {}\n{}{}}}\n", indent, "    ", indent, "    ", kdl_escape(&self.url), indent, "    ")
        } else {
            String::new()
        }
    }
}

impl Dependency {
    pub fn to_kdl_indent(&self, indent: &str, node_name: &str) -> String {
        let mut props = String::new();
        if let Some(ref r) = self.release {
            props.push_str(&format!(" release={}", kdl_escape(r)));
        }
        if let Some(ref v) = self.version_from {
            props.push_str(&format!(" version-from={}", kdl_escape(v)));
        }
        let name_val = if self.name.is_empty() { String::new() } else { kdl_escape(&self.name) };
        format!("{}{} {}{}\n", indent, node_name, name_val, props)
    }
}

impl Source {
    pub fn to_kdl_indent(&self, indent: &str) -> String {
        let mut out = format!("{}Source {{\n", indent);
        append_opt(&mut out, indent, "Name", Some(&self.name));
        append_opt(&mut out, indent, "Homepage", maybe_opt(&self.homepage));
        if let Some(ref p) = self.packager {
            out.push_str(&p.to_kdl_indent(&format!("{}    ", indent)));
        }
        for lic in &self.license {
            if !lic.is_empty() {
                out.push_str(&format!("{}{}License {}\n", indent, "    ", kdl_escape(lic)));
            }
        }
        append_opt(&mut out, indent, "Summary", maybe_opt(&self.summary));
        append_opt(&mut out, indent, "Description", maybe_opt(&self.description));
        // Translations with lang tags
        let mut lang_keys: Vec<&String> = self.translations.keys().collect();
        lang_keys.sort();
        for lang in lang_keys {
            if let Some(entry) = self.translations.get(lang) {
                if let Some(ref s) = entry.summary {
                    out.push_str(&format!("{}{}Summary lang={} {}\n", indent, "    ", kdl_escape(lang), kdl_escape(s)));
                }
                if let Some(ref d) = entry.description {
                    out.push_str(&format!("{}{}Description lang={} {}\n", indent, "    ", kdl_escape(lang), kdl_escape(d)));
                }
            }
        }
        append_opt(&mut out, indent, "PartOf", maybe_opt(&self.part_of));
        append_opt(&mut out, indent, "Icon", maybe_opt(&self.icon));
        append_opt(&mut out, indent, "Screenshot", maybe_opt(&self.screenshot));
        append_opt(&mut out, indent, "Architecture", maybe_opt(&self.architecture));

        // Provides (Isa)
        if !self.provides.is_empty() {
            out.push_str(&format!("{}{}Provides {{\n", indent, "    "));
            for isa in &self.provides {
                out.push_str(&format!("{}{}    Isa {}\n", indent, "    ", kdl_escape(isa)));
            }
            out.push_str(&format!("{}{}}}\n", indent, "    "));
        }

        // Archives
        for a in &self.archives {
            out.push_str(&a.to_kdl_indent(&format!("{}    ", indent)));
        }

        // BuildFlags
        if let Some(ref bf) = self.build_flags {
            if !bf.flags.is_empty() {
                out.push_str(&format!("{}{}BuildFlags {{\n", indent, "    "));
                for flag in &bf.flags {
                    out.push_str(&format!("{}{}    Flag {}\n", indent, "    ", kdl_escape(flag)));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // BuildDependencies
        if let Some(ref bd) = self.build_dependencies {
            if !bd.dependencies.is_empty() {
                out.push_str(&format!("{}{}BuildDependencies {{\n", indent, "    "));
                for d in &bd.dependencies {
                    out.push_str(&d.to_kdl_indent(&format!("{}    ", indent), "Dependency"));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // Patches
        if let Some(ref pw) = self.patches {
            if !pw.patches.is_empty() {
                out.push_str(&format!("{}{}Patches {{\n", indent, "    "));
                for p in &pw.patches {
                    let mut props = String::new();
                    if let Some(lvl) = p.level {
                        props.push_str(&format!(" level={}", lvl));
                    }
                    if let Some(ref ct) = p.compression_type {
                        props.push_str(&format!(" compression-type={}", kdl_escape(ct)));
                    }
                    let file_val = if p.file.is_empty() { String::new() } else { kdl_escape(&p.file) };
                    out.push_str(&format!("{}{}    Patch {file_val}{props}\n", indent, "    "));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        out.push_str(&format!("{}}}\n", indent));
        out
    }
}

fn append_opt(out: &mut String, indent: &str, name: &str, val: Option<&str>) {
    if let Some(v) = val {
        if !v.is_empty() {
            out.push_str(&format!("{}{}{} {}\n", indent, "    ", name, kdl_escape(v)));
        }
    }
}

impl PackageActions {
    pub fn to_kdl_indent(&self, indent: &str) -> Option<String> {
        let mut out = format!("{}Actions {{\n", indent);
        let mut has_any = false;

        let use_types = !self.step_types.is_empty() && self.step_types.len() == self.steps.len();
        for (i, step) in self.steps.iter().enumerate() {
            let node_name = if use_types {
                &self.step_types[i]
            } else {
                "steps"
            };
            if step.contains('\n') {
                out.push_str(&format!("{}{}    {node_name} \"\"\"\n{}\n{}{}    \"\"\"\n", indent, "    ", step, indent, "    "));
            } else {
                out.push_str(&format!("{}{}    {node_name} {}\n", indent, "    ", kdl_escape(step)));
            }
            has_any = true;
        }

        for (name, val) in [("pre-install", &self.pre_install), ("post-install", &self.post_install),
            ("pre-upgrade", &self.pre_upgrade), ("post-upgrade", &self.post_upgrade),
            ("pre-remove", &self.pre_remove), ("post-remove", &self.post_remove)] {
            if let Some(v) = val {
                if v.contains('\n') {
                    out.push_str(&format!("{}{}    {name} \"\"\"\n{v}\n{}{}    \"\"\"\n", indent, "    ", indent, "    "));
                } else {
                    out.push_str(&format!("{}{}    {name} {}\n", indent, "    ", kdl_escape(v)));
                }
                has_any = true;
            }
        }

        if !self.install_filters.is_empty() {
            out.push_str(&format!("{}{}    install-filters \"{}\"\n", indent, "    ", self.install_filters.join(";")));
            has_any = true;
        }

        if !self.no_strip.is_empty() {
            let paths: String = self.no_strip.iter().map(|p| format!(" {}", kdl_escape(p))).collect();
            out.push_str(&format!("{}{}    NoStrip{}\n", indent, "    ", paths));
            has_any = true;
        }

        out.push_str(&format!("{}{}}}\n", indent, "    "));
        if has_any { Some(out) } else { None }
    }
}

impl Files {
    pub fn to_kdl_indent(&self, indent: &str) -> Option<String> {
        if self.paths.is_empty() { return None; }
        let mut out = format!("{}Files {{\n", indent);
        for p in &self.paths {
            let mut props = String::new();
            if let Some(ref ft) = p.file_type {
                props.push_str(&format!(" file-type={}", kdl_escape(ft)));
            }
            out.push_str(&format!("{}{}    Path {}{}\n", indent, "    ", kdl_escape(&p.path), props));
        }
        out.push_str(&format!("{}{}}}\n", indent, "    "));
        Some(out)
    }
}

impl ProvidesBlock {
    pub fn to_kdl_indent(&self, indent: &str) -> Option<String> {
        if self.comar.is_empty() && self.isa.is_empty() { return None; }
        let mut out = format!("{}Provides {{\n", indent);
        for isa in &self.isa {
            out.push_str(&format!("{}{}    Isa {}\n", indent, "    ", kdl_escape(isa)));
        }
        for c in &self.comar {
            let mut props = String::new();
            props.push_str(&format!(" provide={}", kdl_escape(&c.provide)));
            props.push_str(&format!(" script={}", kdl_escape(&c.script)));
            if let Some(ref n) = c.name {
                props.push_str(&format!(" name={}", kdl_escape(n)));
            }
            out.push_str(&format!("{}{}    Comar{props}\n", indent, "    "));
        }
        out.push_str(&format!("{}{}}}\n", indent, "    "));
        Some(out)
    }
}

impl PackageDefinition {
    pub fn to_kdl_indent(&self, indent: &str) -> String {
        let mut out = format!("{}Package {{\n", indent);
        append_opt(&mut out, indent, "Name", Some(&self.name));
        append_opt(&mut out, indent, "Summary", Some(&self.summary));
        append_opt(&mut out, indent, "Description", Some(&self.description));
        // Translations with lang tags
        let mut lang_keys: Vec<&String> = self.translations.keys().collect();
        lang_keys.sort();
        for lang in lang_keys {
            if let Some(entry) = self.translations.get(lang) {
                if let Some(ref s) = entry.summary {
                    out.push_str(&format!("{}{}Summary lang={} {}\n", indent, "    ", kdl_escape(lang), kdl_escape(s)));
                }
                if let Some(ref d) = entry.description {
                    out.push_str(&format!("{}{}Description lang={} {}\n", indent, "    ", kdl_escape(lang), kdl_escape(d)));
                }
            }
        }
        append_opt(&mut out, indent, "Version", Some(&self.version));
        append_opt(&mut out, indent, "License", Some(&self.license));
        append_opt(&mut out, indent, "Homepage", maybe_opt(&self.homepage));
        append_opt(&mut out, indent, "PartOf", maybe_opt(&self.part_of));
        append_opt(&mut out, indent, "Icon", maybe_opt(&self.icon));
        append_opt(&mut out, indent, "Screenshot", maybe_opt(&self.screenshot));
        append_opt(&mut out, indent, "BuildType", maybe_opt(&self.build_type));

        // BuildDependencies
        if let Some(ref bd) = self.build_dependencies {
            if !bd.dependencies.is_empty() {
                out.push_str(&format!("{}{}BuildDependencies {{\n", indent, "    "));
                for d in &bd.dependencies {
                    out.push_str(&d.to_kdl_indent(&format!("{}    ", indent), "Dependency"));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // RuntimeDependencies
        if let Some(ref rd) = self.runtime_dependencies {
            if !rd.dependencies.is_empty() {
                out.push_str(&format!("{}{}RuntimeDependencies {{\n", indent, "    "));
                for d in &rd.dependencies {
                    out.push_str(&d.to_kdl_indent(&format!("{}    ", indent), "Dependency"));
                }
                if let Some(ref ad) = rd.any_dependency {
                    if !ad.dependencies.is_empty() {
                        out.push_str(&format!("{}{}    AnyDependency {{\n", indent, "    "));
                        for d in &ad.dependencies {
                            out.push_str(&d.to_kdl_indent(&format!("{}    ", indent), "Dependency"));
                        }
                        out.push_str(&format!("{}{}    }}\n", indent, "    "));
                    }
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // Files
        if let Some(f) = self.files.to_kdl_indent(&format!("{}    ", indent)) {
            out.push_str(&f);
        }

        // Actions
        if let Some(a) = self.actions.to_kdl_indent(&format!("{}    ", indent)) {
            out.push_str(&a);
        }

        // Provides
        if let Some(p) = self.provides.as_ref().and_then(|p| p.to_kdl_indent(&format!("{}    ", indent))) {
            out.push_str(&p);
        }

        // Replaces
        if let Some(ref r) = self.replaces {
            if !r.packages.is_empty() {
                out.push_str(&format!("{}{}Replaces {{\n", indent, "    "));
                for pkg in &r.packages {
                    out.push_str(&format!("{}{}    Package {}\n", indent, "    ", kdl_escape(pkg)));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // Conflicts
        if let Some(ref c) = self.conflicts {
            if !c.packages.is_empty() {
                out.push_str(&format!("{}{}Conflicts {{\n", indent, "    "));
                for pkg in &c.packages {
                    out.push_str(&format!("{}{}    Package {}\n", indent, "    ", kdl_escape(pkg)));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // AdditionalFiles
        if let Some(ref af) = self.additional_files {
            if !af.files.is_empty() {
                out.push_str(&format!("{}{}AdditionalFiles {{\n", indent, "    "));
                for f in &af.files {
                    let mut props = String::new();
                    props.push_str(&format!(" target={}", kdl_escape(&f.target)));
                    if let Some(ref o) = f.owner {
                        props.push_str(&format!(" owner={}", kdl_escape(o)));
                    }
                    if let Some(ref p) = f.permission {
                        props.push_str(&format!(" permission={}", kdl_escape(p)));
                    }
                    out.push_str(&format!("{}{}    AdditionalFile {}{props}\n", indent, "    ", kdl_escape(&f.filename)));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // Users
        if let Some(ref uw) = self.users {
            if !uw.users.is_empty() {
                out.push_str(&format!("{}{}Users {{\n", indent, "    "));
                for u in &uw.users {
                    let mut props = String::new();
                    if let Some(uid) = u.uid { props.push_str(&format!(" uid={}", uid)); }
                    if let Some(gid) = u.gid { props.push_str(&format!(" gid={}", gid)); }
                    if let Some(ref h) = u.home { props.push_str(&format!(" home={}", kdl_escape(h))); }
                    if let Some(ref s) = u.shell { props.push_str(&format!(" shell={}", kdl_escape(s))); }
                    if let Some(sys) = u.system { props.push_str(&format!(" system={}", sys)); }
                    out.push_str(&format!("{}{}    User {}{props}\n", indent, "    ", kdl_escape(&u.name)));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        // Groups
        if let Some(ref gw) = self.groups {
            if !gw.groups.is_empty() {
                out.push_str(&format!("{}{}Groups {{\n", indent, "    "));
                for g in &gw.groups {
                    let mut props = String::new();
                    if let Some(gid) = g.gid { props.push_str(&format!(" gid={}", gid)); }
                    if let Some(sys) = g.system { props.push_str(&format!(" system={}", sys)); }
                    out.push_str(&format!("{}{}    Group {}{props}\n", indent, "    ", kdl_escape(&g.name)));
                }
                out.push_str(&format!("{}{}}}\n", indent, "    "));
            }
        }

        out.push_str(&format!("{}}}\n", indent));
        out
    }
}

impl Update {
    pub fn to_kdl_indent(&self, indent: &str) -> String {
        let mut out = format!("{}Update release={} date={} {{\n", indent, self.release, kdl_escape(&self.date));
        out.push_str(&format!("{}{}Version {}\n", indent, "    ", kdl_escape(&self.version)));
        out.push_str(&format!("{}{}Comment {}\n", indent, "    ", kdl_escape(&self.comment)));
        out.push_str(&format!("{}{}Name {}\n", indent, "    ", kdl_escape(&self.committer)));
        out.push_str(&format!("{}{}Email {}\n", indent, "    ", kdl_escape(&self.email)));
        if let Some(ref t) = self.type_ {
            out.push_str(&format!("{}{}Type {}\n", indent, "    ", kdl_escape(t)));
        }
        if let Some(ref r) = self.requires {
            out.push_str(&format!("{}{}Requires {}\n", indent, "    ", kdl_escape(r)));
        }
        out.push_str(&format!("{}}}\n", indent));
        out
    }
}

impl History {
    pub fn to_kdl_indent(&self, indent: &str) -> String {
        let mut out = format!("{}History {{\n", indent);
        for u in &self.updates {
            out.push_str(&u.to_kdl_indent(&format!("{}    ", indent)));
        }
        out.push_str(&format!("{}}}\n", indent));
        out
    }
}

impl LuppoSpec {
    pub fn to_kdl_string(&self) -> String {
        let mut out = String::from("LuppoPackage {\n");
        out.push_str(&self.source.to_kdl_indent("    "));
        for pkg in &self.packages {
            out.push_str(&pkg.to_kdl_indent("    "));
        }
        if let Some(ref h) = self.history {
            out.push_str(&h.to_kdl_indent("    "));
        }
        out.push_str("}\n");
        out
    }
}
