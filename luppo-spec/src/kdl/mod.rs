pub mod models;

use kdl::KdlDocument;
use models::*;
use std::fs;
use std::path::Path;

pub fn parse_kdl_spec<P: AsRef<Path>>(path: P) -> Result<LuppoSpec, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(rust_i18n::t!("spec_error_read", error = e).into()),
    };
    parse_kdl_spec_from_str(&content)
}

/// Workaround for kdl 6.0.0 bug: trailing whitespace after a closing `}`
/// causes parse failure when followed by sibling nodes.
fn strip_trailing_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_kdl_spec_from_str(content: &str) -> Result<LuppoSpec, String> {
    let content = strip_trailing_whitespace(content);
    let doc: KdlDocument = content
        .parse()
        .map_err(|e: kdl::KdlError| {
            let input = e.input.as_str();
            let mut details = String::new();
            for (i, diag) in e.diagnostics.iter().enumerate() {
                let offset = diag.span.offset();
                // Find line number by counting newlines before offset
                let line = input[..offset].chars().filter(|&c| c == '\n').count() + 1;
                // Find column on the same line
                let line_start = input[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let col = input[line_start..offset].chars().count() + 1;
                if i > 0 { details.push_str("; "); }
                details.push_str(&format!(
                    "line {}, col {}: {}",
                    line,
                    col,
                    diag.message.as_deref().unwrap_or("unknown error")
                ));
                if let Some(ref help) = diag.help {
                    details.push_str(&format!(" ({})", help));
                }
            }
            if details.is_empty() {
                format!("KDL parse error: {}", e)
            } else {
                format!("KDL parse error: {}", details)
            }
        })?;

    let mut spec = LuppoSpec::default();
    let nodes = if doc.nodes().len() == 1 && doc.nodes()[0].name().to_string() == "LuppoPackage" {
        if let Some(ch) = doc.nodes()[0].children() {
            ch.nodes()
        } else {
            return Ok(spec);
        }
    } else {
        doc.nodes()
    };

    for node in nodes {
        match node.name().to_string().to_lowercase().as_str() {
            "source" => spec.source = parse_source(node)?,
            "package" => {
                let pkg = parse_package_definition(node)?;
                spec.packages.push(pkg);
            }
            "history" => spec.history = Some(parse_history(node)?),
            _ => {}
        }
    }

    Ok(spec)
}

fn pos_str<'a>(node: &'a kdl::KdlNode, idx: usize) -> Option<&'a str> {
    node.entries().get(idx).and_then(|e| e.value().as_string())
}

fn pos_str_owned(node: &kdl::KdlNode, idx: usize) -> Option<String> {
    pos_str(node, idx).map(|s| s.to_string())
}

fn pos_str_default(node: &kdl::KdlNode, idx: usize) -> String {
    pos_str_owned(node, idx).unwrap_or_default()
}

fn get_prop(node: &kdl::KdlNode, name: &str) -> Option<String> {
    node.get(name).and_then(|v| v.as_string().map(|s| s.to_string()))
}

fn get_prop_default(node: &kdl::KdlNode, name: &str) -> String {
    get_prop(node, name).unwrap_or_default()
}

fn find_child<'a>(node: &'a kdl::KdlNode, name: &str) -> Option<&'a kdl::KdlNode> {
    node.children()?.nodes().iter().find(|n| n.name().to_string() == name)
}

fn try_child_text_owned(node: &kdl::KdlNode, pascal: &str, camel: &str) -> Option<String> {
    // New format: child node PascalCase with positional string
    if let Some(val) = child_text_owned(node, pascal) {
        if !val.is_empty() {
            return Some(val);
        }
    }
    // Old format: child node lowercase
    let lower = pascal.to_lowercase();
    if lower != pascal {
        if let Some(val) = child_text_owned(node, &lower) {
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    // Old format: property camelCase
    get_prop(node, camel)
}

fn child_text_owned(node: &kdl::KdlNode, name: &str) -> Option<String> {
    child_text(node, name).map(|s| s.to_string())
}

#[allow(dead_code)]
fn child_text_default(node: &kdl::KdlNode, name: &str) -> String {
    child_text_owned(node, name).unwrap_or_default()
}

fn child_text<'a>(node: &'a kdl::KdlNode, name: &str) -> Option<&'a str> {
    find_child(node, name).and_then(|c| pos_str(c, 0))
}

fn children_named<'a>(parent: &'a kdl::KdlNode, names: &[&str]) -> Vec<&'a kdl::KdlNode> {
    parent
        .children()
        .map(|ch| {
            ch.nodes()
                .iter()
                .filter(|n| names.contains(&n.name().to_string().as_str()))
                .collect()
        })
        .unwrap_or_default()
}

fn children_by_name<'a>(parent: &'a kdl::KdlNode, name: &str) -> Vec<&'a kdl::KdlNode> {
    children_named(parent, &[name])
}

/// Returns the first positional (unnamed) string value from a node, skipping properties.
fn first_pos_str_owned(node: &kdl::KdlNode) -> Option<String> {
    for entry in node.entries() {
        if entry.name().is_some() {
            continue;
        }
        if let Some(s) = entry.value().as_string() {
            return Some(s.to_string());
        }
    }
    None
}

/// Collects localized text nodes. Returns (default_text, lang→text map).
/// Supports: `Summary "text"` (default), `Summary lang="tr" "text"` (translated),
/// lowercase `summary "text"`, and property `summary="text"` fallback.
fn collect_localized(node: &kdl::KdlNode, pascal: &str) -> (Option<String>, std::collections::HashMap<String, String>) {
    let lower = pascal.to_lowercase();
    let children = children_named(node, &[pascal, &lower]);
    let mut default = None;
    let mut translations = std::collections::HashMap::new();

    for child in &children {
        if let Some(text) = first_pos_str_owned(child) {
            if !text.is_empty() {
                if let Some(lang) = get_prop(child, "lang") {
                    translations.insert(lang, text);
                } else if default.is_none() {
                    default = Some(text);
                }
            }
        }
    }

    // Fallback: property format (camelCase)
    if default.is_none() {
        if let Some(val) = get_prop(node, &lower) {
            if !val.is_empty() {
                default = Some(val);
            }
        }
    }

    (default, translations)
}

// ---------- Source ----------

fn parse_source(node: &kdl::KdlNode) -> Result<Source, String> {
    let mut s = Source::default();

    // New format: child nodes with PascalCase; Old format: properties
    s.name = try_child_text_owned(node, "Name", "name").unwrap_or_default();
    s.homepage = try_child_text_owned(node, "Homepage", "homepage");
    let (summary, summary_trans) = collect_localized(node, "Summary");
    s.summary = summary;
    for (lang, text) in summary_trans {
        s.translations.entry(lang).or_default().summary = Some(text);
    }
    let (description, desc_trans) = collect_localized(node, "Description");
    s.description = description;
    for (lang, text) in desc_trans {
        s.translations.entry(lang).or_default().description = Some(text);
    }
    s.part_of = try_child_text_owned(node, "PartOf", "partof");
    s.icon = try_child_text_owned(node, "Icon", "icon");
    s.screenshot = try_child_text_owned(node, "Screenshot", "screenshot");
    s.architecture = try_child_text_owned(node, "Architecture", "architecture");

    // Provides/Isa
    if let Some(prov_node) = find_child(node, "Provides") {
        let isa_list = children_by_name(prov_node, "Isa");
        s.provides = isa_list.iter()
            .filter_map(|n| pos_str_owned(n, 0))
            .collect();
    }
    if s.provides.is_empty() {
        if let Some(isa_str) = get_prop(node, "isa") {
            s.provides = isa_str.split(';').map(|s| s.trim().to_string()).collect();
        }
    }

    // License: new format child node or old format property
    if let Some(lic) = try_child_text_owned(node, "License", "license") {
        s.license = if lic.contains(';') {
            lic.split(';').map(|s| s.trim().to_string()).collect()
        } else {
            vec![lic]
        };
    }

    // Packager: new format child node or old format child node
    if let Some(pkgr) = find_child(node, "Packager").or_else(|| find_child(node, "packager")) {
        s.packager = Some(Packager {
            name: try_child_text_owned(pkgr, "Name", "name").unwrap_or_default(),
            email: try_child_text_owned(pkgr, "Email", "email").unwrap_or_default(),
        });
    }

    // Archives: new format child node "Archive" or old "archive"
    for child_node in children_by_name(node, "Archive") {
        s.archives.push(parse_archive(child_node)?);
    }
    if s.archives.is_empty() {
        for child_node in children_by_name(node, "archive") {
            s.archives.push(parse_archive_old(child_node)?);
        }
    }

    // BuildDependencies
    if let Some(bd_node) = find_child(node, "BuildDependencies").or_else(|| find_child(node, "build-dependencies").or_else(|| find_child(node, "build_dependencies"))) {
        let deps = parse_dependency_vec(bd_node);
        if !deps.is_empty() {
            s.build_dependencies = Some(BuildDeps { dependencies: deps });
        }
    }

    // Patches
    if let Some(p_node) = find_child(node, "Patches").or_else(|| find_child(node, "patches")) {
        let patches = parse_patches(p_node);
        if !patches.is_empty() {
            s.patches = Some(PatchesWrapper { patches });
        }
    }

    // BuildFlags
    if let Some(bf_node) = find_child(node, "BuildFlags").or_else(|| find_child(node, "build-flags").or_else(|| find_child(node, "build_flags"))) {
        let flags: Vec<String> = children_named(bf_node, &["Flag", "flag"])
            .iter()
            .filter_map(|n| pos_str_owned(n, 0))
            .collect();
        if !flags.is_empty() {
            s.build_flags = Some(BuildFlagsWrapper { flags });
        }
    }

    // AdditionalFiles
    if let Some(af_node) = find_child(node, "AdditionalFiles").or_else(|| find_child(node, "additional-files").or_else(|| find_child(node, "additional_files"))) {
        s.additional_files = parse_additional_files(af_node);
    }

    Ok(s)
}

// ---------- Archive ----------

fn parse_archive(node: &kdl::KdlNode) -> Result<Archive, String> {
    // Find first positional entry (not a property key=value)
    let mut url = String::new();
    for entry in node.entries() {
        if entry.name().is_some() {
            continue; // skip properties (key=value)
        }
        if let Some(s) = entry.value().as_string() {
            url = s.to_string();
            break;
        }
    }
    if url.is_empty() {
        // Try child node name: Archive sha1sum="..." { "url" }
        if let Some(ch) = node.children() {
            if let Some(url_node) = ch.nodes().first() {
                url = url_node.name().value().to_string();
            }
        }
    }
    Ok(Archive {
        url,
        sha1sum: get_prop(node, "sha1sum"),
        md5sum: get_prop(node, "md5sum"),
        hash: get_prop(node, "hash"),
        archive_type: get_prop(node, "type").unwrap_or_else(|| "targz".to_string()),
        target: get_prop(node, "target"),
    })
}

fn parse_archive_old(node: &kdl::KdlNode) -> Result<Archive, String> {
    let mut url = get_prop_default(node, "url");
    if url.is_empty() {
        // Try child node name as URL
        if let Some(ch) = node.children() {
            if let Some(url_node) = ch.nodes().first() {
                url = url_node.name().value().to_string();
            }
        }
    }
    Ok(Archive {
        url,
        sha1sum: get_prop(node, "sha1sum"),
        md5sum: get_prop(node, "md5sum"),
        hash: get_prop(node, "hash"),
        archive_type: get_prop(node, "type").unwrap_or_else(|| "targz".to_string()),
        target: get_prop(node, "target"),
    })
}

// ---------- Dependencies ----------

fn parse_dependency_vec(node: &kdl::KdlNode) -> Vec<Dependency> {
    children_named(node, &["Dependency", "dependency"])
        .into_iter()
        .map(|d| Dependency {
            name: pos_str_owned(d, 0)
                .or_else(|| get_prop(d, "name"))
                .unwrap_or_default(),
            release: get_prop(d, "release"),
            version_from: get_prop(d, "versionFrom").or_else(|| get_prop(d, "version_from")).or_else(|| get_prop(d, "version-from")),
        })
        .collect()
}

// ---------- Patches ----------

fn parse_patches(node: &kdl::KdlNode) -> Vec<Patch> {
    children_named(node, &["Patch", "patch"])
        .into_iter()
        .map(|p| Patch {
            file: pos_str_owned(p, 0)
                .or_else(|| get_prop(p, "file"))
                .unwrap_or_default(),
            level: p.get("level").and_then(|v| {
                v.as_integer()
                    .or_else(|| v.as_string().and_then(|s| s.parse::<i128>().ok()))
                    .map(|i| i as u8)
            }),
            compression_type: get_prop(p, "compressionType").or_else(|| get_prop(p, "compression-type")),
        })
        .collect()
}

// ---------- PackageDefinition ----------

fn parse_package_definition(node: &kdl::KdlNode) -> Result<PackageDefinition, String> {
    let mut pkg = PackageDefinition::default();

    pkg.name = try_child_text_owned(node, "Name", "name").unwrap_or_default();
    let (summary, summary_trans) = collect_localized(node, "Summary");
    pkg.summary = summary.unwrap_or_default();
    for (lang, text) in summary_trans {
        pkg.translations.entry(lang).or_default().summary = Some(text);
    }
    let (description, desc_trans) = collect_localized(node, "Description");
    pkg.description = description.unwrap_or_default();
    for (lang, text) in desc_trans {
        pkg.translations.entry(lang).or_default().description = Some(text);
    }
    pkg.version = try_child_text_owned(node, "Version", "version").unwrap_or_default();
    pkg.license = try_child_text_owned(node, "License", "license").unwrap_or_default();
    pkg.homepage = try_child_text_owned(node, "Homepage", "homepage");
    pkg.icon = try_child_text_owned(node, "Icon", "icon");
    pkg.screenshot = try_child_text_owned(node, "Screenshot", "screenshot");
    pkg.part_of = try_child_text_owned(node, "PartOf", "partof");
    pkg.build_type = try_child_text_owned(node, "BuildType", "build-type").or_else(|| try_child_text_owned(node, "BuildType", "build_type"));

    // BuildDependencies
    if let Some(bd) = find_child(node, "BuildDependencies").or_else(|| find_child(node, "build-dependencies").or_else(|| find_child(node, "build_dependencies"))) {
        let deps = parse_dependency_vec(bd);
        if !deps.is_empty() {
            pkg.build_dependencies = Some(BuildDeps { dependencies: deps });
        }
    }

    // RuntimeDependencies
    if let Some(rd) = find_child(node, "RuntimeDependencies").or_else(|| find_child(node, "runtime-dependencies").or_else(|| find_child(node, "runtime_dependencies"))) {
        let deps = parse_dependency_vec(rd);
        if !deps.is_empty() {
            let any_dep = find_child(rd, "AnyDependency")
                .or_else(|| find_child(rd, "any-dependency"))
                .or_else(|| find_child(rd, "any_dependency"))
                .map(|ad| AnyDependency { dependencies: parse_dependency_vec(ad) });
            pkg.runtime_dependencies = Some(RuntimeDeps { dependencies: deps, any_dependency: any_dep });
        }
    }

    // Files
    if let Some(files_node) = find_child(node, "Files").or_else(|| find_child(node, "files")) {
        pkg.files = parse_files(files_node);
    }

    // Actions: new (child nodes) or old (properties)
    if let Some(acts_node) = find_child(node, "Actions").or_else(|| find_child(node, "actions")) {
        pkg.actions = parse_actions(acts_node);
    } else {
        // Old format: actions on package node itself
        pkg.actions = parse_actions_inline(node);
    }

    // Provides
    if let Some(prov_node) = find_child(node, "Provides").or_else(|| find_child(node, "provides")) {
        pkg.provides = parse_provides(prov_node);
    }

    // Replaces
    if let Some(r_node) = find_child(node, "Replaces").or_else(|| find_child(node, "replaces")) {
        pkg.replaces = Some(ReplacesWrapper { packages: parse_plain_strings(r_node) });
    }

    // Conflicts
    if let Some(c_node) = find_child(node, "Conflicts").or_else(|| find_child(node, "conflicts")) {
        pkg.conflicts = Some(ConflictsWrapper { packages: parse_plain_strings(c_node) });
    }

    // AdditionalFiles
    if let Some(af_node) = find_child(node, "AdditionalFiles").or_else(|| find_child(node, "additional-files").or_else(|| find_child(node, "additional_files"))) {
        pkg.additional_files = parse_additional_files(af_node);
    }

    // Users
    if let Some(u_node) = find_child(node, "Users").or_else(|| find_child(node, "users")) {
        pkg.users = parse_users(u_node);
    }

    // Groups
    if let Some(g_node) = find_child(node, "Groups").or_else(|| find_child(node, "groups")) {
        pkg.groups = parse_groups(g_node);
    }

    Ok(pkg)
}

// ---------- Files ----------

fn parse_files(node: &kdl::KdlNode) -> Files {
    Files {
        paths: children_named(node, &["Path", "path"])
            .into_iter()
            .map(|p| PathDef {
                path: pos_str_owned(p, 0)
                    .or_else(|| get_prop(p, "path"))
                    .unwrap_or_default(),
                file_type: get_prop(p, "fileType").or_else(|| get_prop(p, "file-type")),
            })
            .collect(),
    }
}

// ---------- Actions ----------

fn parse_actions(node: &kdl::KdlNode) -> PackageActions {
    let mut steps = Vec::new();
    let mut step_types = Vec::new();
    let mut pre_install = None;
    let mut post_install = None;
    let mut pre_upgrade = None;
    let mut post_upgrade = None;
    let mut pre_remove = None;
    let mut post_remove = None;
    let mut install_filters = Vec::new();
    let mut no_strip = Vec::new();

    // New format: child nodes for each action
    if let Some(ch) = node.children() {
        for child in ch.nodes() {
            match child.name().to_string().as_str() {
                "setup" | "build" | "install" | "check" => {
                    steps.push(pos_str_default(child, 0));
                    step_types.push(child.name().to_string());
                }
                "pre-install" | "pre_install" => pre_install = pos_str_owned(child, 0),
                "post-install" | "post_install" => post_install = pos_str_owned(child, 0),
                "pre-upgrade" | "pre_upgrade" => pre_upgrade = pos_str_owned(child, 0),
                "post-upgrade" | "post_upgrade" => post_upgrade = pos_str_owned(child, 0),
                "pre-remove" | "pre_remove" => pre_remove = pos_str_owned(child, 0),
                "post-remove" | "post_remove" => post_remove = pos_str_owned(child, 0),
                "no-strip" | "NoStrip" | "nostrip" => {
                    for i in 0.. {
                        if let Some(val) = pos_str_owned(child, i) {
                            no_strip.push(val);
                        } else {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Fallback: collect repeated "steps" child nodes
    if steps.is_empty() {
        if let Some(ch) = node.children() {
            for child in ch.nodes() {
                if child.name().to_string() == "steps" || child.name().to_string() == "Steps" {
                    if let Some(val) = pos_str_owned(child, 0) {
                        steps.push(val);
                    }
                }
            }
        }
    }
    // Semicolon-separated "steps" child node or property (legacy)
    if steps.is_empty() {
        if let Some(s) = child_text_owned(node, "steps") {
            steps = s.split(';').map(|s| s.trim().to_string()).collect();
        } else if let Some(s) = get_prop(node, "steps") {
            steps = s.split(';').map(|s| s.trim().to_string()).collect();
        }
    }
    if pre_install.is_none() && get_prop(node, "pre-install").is_some() {
        pre_install = get_prop(node, "pre-install");
    }
    if post_install.is_none() && get_prop(node, "post-install").is_some() {
        post_install = get_prop(node, "post-install");
    }
    if pre_upgrade.is_none() && get_prop(node, "pre-upgrade").is_some() {
        pre_upgrade = get_prop(node, "pre-upgrade");
    }
    if post_upgrade.is_none() && get_prop(node, "post-upgrade").is_some() {
        post_upgrade = get_prop(node, "post-upgrade");
    }
    if pre_remove.is_none() && get_prop(node, "pre-remove").is_some() {
        pre_remove = get_prop(node, "pre-remove");
    }
    if post_remove.is_none() && get_prop(node, "post-remove").is_some() {
        post_remove = get_prop(node, "post-remove");
    }
    if let Some(f) = get_prop(node, "install-filters") {
        install_filters = f.split(';').map(|s| s.trim().to_string()).collect();
    }

    PackageActions {
        steps,
        step_types,
        configure: None,
        pre_install,
        post_install,
        pre_upgrade,
        post_upgrade,
        pre_remove,
        post_remove,
        install_filters,
        no_strip,
    }
}

fn parse_actions_inline(node: &kdl::KdlNode) -> PackageActions {
    // For old format where actions are properties directly on the package node
    let mut steps = Vec::new();
    let pre_install = get_prop(node, "pre-install");
    let post_install = get_prop(node, "post-install");
    let pre_upgrade = get_prop(node, "pre-upgrade");
    let post_upgrade = get_prop(node, "post-upgrade");
    let pre_remove = get_prop(node, "pre-remove");
    let post_remove = get_prop(node, "post-remove");
    let mut install_filters = Vec::new();
    let no_strip = Vec::new();

    if let Some(s) = get_prop(node, "steps") {
        steps = s.split(';').map(|s| s.trim().to_string()).collect();
    }
    if let Some(f) = get_prop(node, "install-filters") {
        install_filters = f.split(';').map(|s| s.trim().to_string()).collect();
    }

    PackageActions {
        steps,
        step_types: Vec::new(),
        configure: None,
        pre_install,
        post_install,
        pre_upgrade,
        post_upgrade,
        pre_remove,
        post_remove,
        install_filters,
        no_strip,
    }
}

// ---------- Provides ----------

fn parse_provides(node: &kdl::KdlNode) -> Option<ProvidesBlock> {
    let mut comar = Vec::new();
    let mut isa = Vec::new();

    // Old format: "isa" property on the provides node
    if let Some(isa_str) = get_prop(node, "isa") {
        for item in isa_str.split(';') {
            let item = item.trim().to_string();
            if !item.is_empty() {
                isa.push(item);
            }
        }
    }

    if let Some(ch) = node.children() {
        for child in ch.nodes() {
            match child.name().to_string().to_lowercase().as_str() {
                "comar" => {
                    comar.push(Comar {
                        provide: get_prop_default(child, "provide"),
                        script: get_prop_default(child, "script"),
                        name: get_prop(child, "name"),
                    });
                }
                "isa" => {
                    isa.push(pos_str_default(child, 0));
                }
                _ => {}
            }
        }
    }

    if comar.is_empty() && isa.is_empty() {
        None
    } else {
        Some(ProvidesBlock { comar, isa })
    }
}

// ---------- Replaces / Conflicts ----------

fn parse_plain_strings(node: &kdl::KdlNode) -> Vec<String> {
    // Old format: "package" property with semicolon-separated values
    if let Some(pkg_str) = get_prop(node, "package") {
        return pkg_str.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    // New format: child Package nodes
    children_named(node, &["Package", "package"])
        .into_iter()
        .map(|p| pos_str_default(p, 0))
        .filter(|s| !s.is_empty())
        .collect()
}

// ---------- AdditionalFiles ----------

fn parse_additional_files(node: &kdl::KdlNode) -> Option<AdditionalFilesWrapper> {
    let files: Vec<AdditionalFile> = children_named(node, &["AdditionalFile", "additional-file", "additional_file"])
        .into_iter()
        .map(|f| AdditionalFile {
            filename: pos_str_owned(f, 0).or_else(|| get_prop(f, "filename")).unwrap_or_default(),
            owner: get_prop(f, "owner"),
            permission: get_prop(f, "permission"),
            target: get_prop_default(f, "target"),
        })
        .collect();

    if files.is_empty() { None } else { Some(AdditionalFilesWrapper { files }) }
}

// ---------- Users ----------

fn parse_users(node: &kdl::KdlNode) -> Option<UsersWrapper> {
    let users: Vec<User> = children_named(node, &["User", "user"])
        .into_iter()
        .map(|u| User {
            name: pos_str_owned(u, 0).or_else(|| get_prop(u, "name")).unwrap_or_default(),
            uid: u.get("uid").and_then(|v| v.as_integer().map(|i| i as u32)),
            gid: u.get("gid").and_then(|v| v.as_integer().map(|i| i as u32)),
            home: get_prop(u, "home"),
            shell: get_prop(u, "shell"),
            system: u.get("system").and_then(|v| v.as_bool()),
        })
        .collect();

    if users.is_empty() { None } else { Some(UsersWrapper { users }) }
}

// ---------- Groups ----------

fn parse_groups(node: &kdl::KdlNode) -> Option<GroupsWrapper> {
    let groups: Vec<Group> = children_named(node, &["Group", "group"])
        .into_iter()
        .map(|g| Group {
            name: pos_str_owned(g, 0).or_else(|| get_prop(g, "name")).unwrap_or_default(),
            gid: g.get("gid").and_then(|v| v.as_integer().map(|i| i as u32)),
            system: g.get("system").and_then(|v| v.as_bool()),
        })
        .collect();

    if groups.is_empty() { None } else { Some(GroupsWrapper { groups }) }
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_kdl(input: &str) -> LuppoSpec {
        parse_kdl_spec_from_str(input).unwrap()
    }

    #[test]
    fn test_localized_summary_description() {
        let kdl = r#"
Source {
    Name "test-pkg"
    Summary "English summary"
    Summary lang="tr" "Türkçe özet"
    Summary lang="de" "Deutsche Zusammenfassung"
    Description "English description"
    Description lang="tr" "Türkçe açıklama"
    Description lang="fr" "Description française"
    Archive "http://test.com/test.tar.gz"
}
Package {
    Name "test-pkg"
    Summary "Package summary"
    Summary lang="tr" "Paket özeti"
    Description "Package description"
    Description lang="tr" "Paket açıklaması"
    Files {
        Path "/usr/bin"
    }
}
"#;
        let spec = parse_kdl(kdl);

        // Source default values
        assert_eq!(spec.source.summary.as_deref(), Some("English summary"));
        assert_eq!(spec.source.description.as_deref(), Some("English description"));

        // Source translations
        let tr_entry = spec.source.translations.get("tr").unwrap();
        assert_eq!(tr_entry.summary.as_deref(), Some("Türkçe özet"));
        assert_eq!(tr_entry.description.as_deref(), Some("Türkçe açıklama"));

        let de_entry = spec.source.translations.get("de").unwrap();
        assert_eq!(de_entry.summary.as_deref(), Some("Deutsche Zusammenfassung"));
        assert!(de_entry.description.is_none());

        let fr_entry = spec.source.translations.get("fr").unwrap();
        assert!(fr_entry.summary.is_none());
        assert_eq!(fr_entry.description.as_deref(), Some("Description française"));

        // Package default values
        let pkg = &spec.packages[0];
        assert_eq!(pkg.summary, "Package summary");
        assert_eq!(pkg.description, "Package description");

        // Package translations
        let pkg_tr = pkg.translations.get("tr").unwrap();
        assert_eq!(pkg_tr.summary.as_deref(), Some("Paket özeti"));
        assert_eq!(pkg_tr.description.as_deref(), Some("Paket açıklaması"));
    }

    #[test]
    fn test_localized_fallback_to_lowercase_child() {
        let kdl = r#"
Source {
    Name "test-pkg"
    summary "prop summary"
    description "prop desc"
    Archive "http://test.com/test.tar.gz"
}
Package {
    Name "test-pkg"
    summary "pkg prop summary"
    description "pkg prop desc"
    Files {
        Path "/usr/bin"
    }
}
"#;
        let spec = parse_kdl(kdl);
        assert_eq!(spec.source.summary.as_deref(), Some("prop summary"));
        assert_eq!(spec.source.description.as_deref(), Some("prop desc"));
        assert_eq!(spec.packages[0].summary, "pkg prop summary");
        assert_eq!(spec.packages[0].description, "pkg prop desc");
    }

    #[test]
    fn test_localized_only_translations_no_default() {
        let kdl = r#"
Source {
    Name "test-pkg"
    Summary lang="tr" "Türkçe özet"
    Description lang="tr" "Türkçe açıklama"
    Archive "http://test.com/test.tar.gz"
}
Package {
    Name "test-pkg"
    Summary lang="tr" "Paket özeti"
    Description lang="tr" "Paket açıklaması"
    Files {
        Path "/usr/bin"
    }
}
"#;
        let spec = parse_kdl(kdl);
        // No default summary/description
        assert!(spec.source.summary.is_none());
        assert!(spec.source.description.is_none());
        assert_eq!(spec.packages[0].summary, "");
        assert_eq!(spec.packages[0].description, "");

        // But translations should be present
        let tr_entry = spec.source.translations.get("tr").unwrap();
        assert_eq!(tr_entry.summary.as_deref(), Some("Türkçe özet"));
    }

    #[test]
    fn test_parse_error_shows_line_number() {
        let err = parse_kdl_spec_from_str(r#"LuppoPackage {
    Source { Name "test" }
    Package {
        Name "test"
        Actions {
            steps "unclosed
        }
        Files { Path "/usr" }
    }
}
"#).unwrap_err();
        assert!(err.contains("line "), "Error missing line info: {}", err);
        assert!(err.contains("col "), "Error missing col info: {}", err);
    }

    #[test]
    fn test_trailing_whitespace_after_brace() {
        // kdl 6.0.0 bug: trailing whitespace after `}` breaks parsing of
        // subsequent sibling nodes. strip_trailing_whitespace() must handle this.
        let kdl = r#"LuppoPackage {
    Source {
        Name "test"
        Archive "http://test.com/t.tar.gz"
    }   
    Package {
        Name "pkg1"
        Summary "test"
    }   
    Package {
        Name "pkg2"
        Summary "test"
    }
}
"#;
        let spec = parse_kdl_spec_from_str(kdl).unwrap();
        assert_eq!(spec.packages.len(), 2);
        assert_eq!(spec.packages[0].name, "pkg1");
        assert_eq!(spec.packages[1].name, "pkg2");
    }
}

// ---------- History ----------

fn parse_history(node: &kdl::KdlNode) -> Result<History, String> {
    let updates: Vec<Update> = children_named(node, &["Update", "update"])
        .into_iter()
        .map(|u| {
            let release = u.get("release").and_then(|v| v.as_integer()).map(|i| i as u32).unwrap_or(0);
            let date = get_prop_default(u, "date");
            // New format: child Version/Comment/Name/Email nodes
            let version = try_child_text_owned(u, "Version", "version").unwrap_or_default();
            let comment = try_child_text_owned(u, "Comment", "comment").unwrap_or_default();
            let committer = try_child_text_owned(u, "Name", "committer").or_else(|| {
                // Old format: child committer node with name property
                find_child(u, "committer")
                    .and_then(|c| get_prop(c, "name"))
            }).unwrap_or_default();
            let email = try_child_text_owned(u, "Email", "email").or_else(|| {
                find_child(u, "committer")
                    .and_then(|c| get_prop(c, "email"))
            }).unwrap_or_default();
            let type_ = try_child_text_owned(u, "Type", "type");
            let requires = try_child_text_owned(u, "Requires", "requires");
            Update { release, date, version, comment, committer, email, type_, requires }
        })
        .collect();

    Ok(History { updates })
}
