use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub local_names: Vec<(String, String)>,
    pub summaries: Vec<(String, String)>,
    pub descriptions: Vec<(String, String)>,
    pub group: String,
    pub maintainer_name: String,
    pub maintainer_email: String,
}

impl Component {
    pub fn empty(name: &str) -> Self {
        Component {
            name: name.to_string(),
            local_names: vec![("en".to_string(), "FIXME".to_string())],
            summaries: vec![("en".to_string(), "FIXME".to_string())],
            descriptions: vec![("en".to_string(), "FIXME".to_string())],
            group: "FIXME".to_string(),
            maintainer_name: "PisiLinux Community".to_string(),
            maintainer_email: "admins@pisilinux.org".to_string(),
        }
    }
}

// ── Serde helper: Vec<(String,String)> ↔ HashMap ──

#[allow(dead_code)]
fn vec_to_map(items: &[(String, String)]) -> HashMap<String, String> {
    items.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

#[allow(dead_code)]
fn map_to_vec(map: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

// ── File-format serde struct ──

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct ComponentsFile {
    #[serde(rename = "component")]
    components: Vec<ComponentEntry>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct ComponentEntry {
    name: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    local_name: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    summary: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    description: HashMap<String, String>,
    #[serde(default)]
    group: String,
    #[serde(default = "default_mname")]
    maintainer_name: String,
    #[serde(default = "default_memail")]
    maintainer_email: String,
}

fn default_mname() -> String {
    "PisiLinux Community".to_string()
}
fn default_memail() -> String {
    "admins@pisilinux.org".to_string()
}

#[allow(dead_code)]
impl ComponentEntry {
    fn from_component(c: &Component) -> Self {
        ComponentEntry {
            name: c.name.clone(),
            local_name: vec_to_map(&c.local_names),
            summary: vec_to_map(&c.summaries),
            description: vec_to_map(&c.descriptions),
            group: c.group.clone(),
            maintainer_name: c.maintainer_name.clone(),
            maintainer_email: c.maintainer_email.clone(),
        }
    }

    fn into_component(self) -> Component {
        Component {
            name: self.name,
            local_names: map_to_vec(&self.local_name),
            summaries: map_to_vec(&self.summary),
            descriptions: map_to_vec(&self.description),
            group: self.group,
            maintainer_name: self.maintainer_name,
            maintainer_email: self.maintainer_email,
        }
    }
}

// ── Format-agnostic backend enum ──

pub enum ComponentsBackend {
    Xml(XmlComponents),
    Kdl(KdlComponents),
}

impl ComponentsBackend {
    pub fn detect(base: &Path) -> Result<Self, String> {
        let xml = XmlComponents::new(base);
        if xml.exists() {
            return Ok(ComponentsBackend::Xml(xml));
        }

        let kdl = KdlComponents::new(base);
        if kdl.exists() {
            return Ok(ComponentsBackend::Kdl(kdl));
        }

        Err(format!(
            "No components file found in {}. Tried: components.xml, components.kdl",
            base.display()
        ))
    }

    pub fn path(&self) -> &Path {
        match self {
            ComponentsBackend::Xml(b) => &b.path,
            ComponentsBackend::Kdl(b) => &b.path,
        }
    }

    pub fn read(&self) -> Result<Vec<Component>, String> {
        match self {
            ComponentsBackend::Xml(b) => b.read(),
            ComponentsBackend::Kdl(b) => b.read(),
        }
    }

    pub fn write(&self, components: &[Component]) -> Result<(), String> {
        match self {
            ComponentsBackend::Xml(b) => b.write(components),
            ComponentsBackend::Kdl(b) => b.write(components),
        }
    }

    pub fn insert_missing(&self, names: &[String]) -> Result<(), String> {
        match self {
            ComponentsBackend::Xml(b) => {
                let content = fs::read_to_string(&b.path).map_err(|e| e.to_string())?;
                b.insert_missing(&content, names)
            }
            ComponentsBackend::Kdl(b) => {
                let mut comps = b.read()?;
                let existing: HashSet<String> = comps.iter().map(|c| c.name.clone()).collect();
                for name in names {
                    if !existing.contains(name) {
                        comps.push(Component::empty(name));
                    }
                }
                b.write(&comps)
            }
        }
    }
}

// ── XML backend ──

pub struct XmlComponents {
    path: PathBuf,
}

impl XmlComponents {
    fn new(base: &Path) -> Self {
        XmlComponents {
            path: base.join("components.xml"),
        }
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn read(&self) -> Result<Vec<Component>, String> {
        let content = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        Ok(parse_components_xml(&content))
    }

    fn write(&self, components: &[Component]) -> Result<(), String> {
        let mut sorted = components.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        if self.path.exists() {
            numbered_backup(&self.path)?;
        }

        let mut content = String::from("<PISI>\n    <Components>\n");
        for comp in &sorted {
            content.push_str(&component_xml_block(comp));
            content.push('\n');
        }
        content.push_str("    </Components>\n</PISI>\n");

        fs::write(&self.path, content).map_err(|e| e.to_string())?;
        println!(
            "✓ {} components written to {}",
            sorted.len(),
            self.path.display()
        );
        Ok(())
    }

    fn insert_missing(&self, content: &str, names: &[String]) -> Result<(), String> {
        let csend_re = Regex::new(r"</Components>").unwrap();
        let mut new_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            if csend_re.is_match(line) {
                for name in names {
                    new_lines.push(component_xml_block(&Component::empty(name)));
                }
            }
            new_lines.push(line.to_string());
        }

        let new_content = new_lines.join("\n");
        numbered_backup(&self.path)?;
        fs::write(&self.path, new_content).map_err(|e| e.to_string())?;
        println!("components.xml updated.");
        Ok(())
    }
}

// ── KDL backend ──

#[allow(dead_code)]
fn kdl_str(val: &str) -> String {
    if val.contains('"') || val.contains('\\') {
        format!("r#\"{}\"#", val)
    } else {
        format!("\"{}\"", val)
    }
}

fn component_kdl_block(comp: &Component) -> String {
    let mut kdl = format!("component \"{}\" {{\n", comp.name);
    for (lang, val) in &comp.local_names {
        kdl.push_str(&format!("    local-name lang=\"{}\" \"{}\"\n", lang, val));
    }
    for (lang, val) in &comp.summaries {
        kdl.push_str(&format!("    summary lang=\"{}\" \"{}\"\n", lang, val));
    }
    for (lang, val) in &comp.descriptions {
        kdl.push_str(&format!("    description lang=\"{}\" \"{}\"\n", lang, val));
    }
    kdl.push_str(&format!("    group \"{}\"\n", comp.group));
    kdl.push_str(&format!("    maintainer-name \"{}\"\n", comp.maintainer_name));
    kdl.push_str(&format!("    maintainer-email \"{}\"\n", comp.maintainer_email));
    kdl.push_str("}\n");
    kdl
}

fn parse_kdl_components(content: &str) -> Result<Vec<Component>, String> {
    let doc: kdl::KdlDocument = content.parse().map_err(|e| format!("KDL parse error: {}", e))?;
    let mut components = Vec::new();

    for node in doc.nodes() {
        if node.name().to_string().as_str() == "component" {
            let name = node.entries().first()
                .and_then(|e| e.value().as_string())
                .map(|s| s.to_string())
                .or_else(|| node.get("name").and_then(|v| v.as_string()).map(|s| s.to_string()))
                .ok_or_else(|| "Component missing name".to_string())?;

            let mut local_names = Vec::new();
            let mut summaries = Vec::new();
            let mut descriptions = Vec::new();

            if let Some(children) = node.children() {
                for child in children.nodes() {
                    let cname = child.name().to_string();
                    let lang = child.get("lang").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_default();
                    let val = child.entries().first().and_then(|e| e.value().as_string()).map(|s| s.to_string()).unwrap_or_default();
                    match cname.as_str() {
                        "local-name" | "local_name" => local_names.push((lang, val)),
                        "summary" => summaries.push((lang, val)),
                        "description" | "desc" => descriptions.push((lang, val)),
                        _ => {}
                    }
                }
            }

            // Also check inline properties
            let entries = node.entries();
            for i in (0..entries.len()).step_by(3) {
                if i + 2 < entries.len() {
                    if let Some(key) = entries[i].value().as_string() {
                        if let Some(lang_val) = entries.get(i + 1).and_then(|e| e.value().as_string()) {
                            if let Some(text_val) = entries.get(i + 2).and_then(|e| e.value().as_string()) {
                                match key {
                                    "local-name" | "local_name" => local_names.push((lang_val.to_string(), text_val.to_string())),
                                    "summary" => summaries.push((lang_val.to_string(), text_val.to_string())),
                                    "description" | "desc" => descriptions.push((lang_val.to_string(), text_val.to_string())),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            components.push(Component {
                name,
                local_names,
                summaries,
                descriptions,
                group: node.get("group").and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_default(),
                maintainer_name: node.get("maintainer-name").or_else(|| node.get("maintainer_name")).and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_else(default_mname),
                maintainer_email: node.get("maintainer-email").or_else(|| node.get("maintainer_email")).and_then(|v| v.as_string()).map(|s| s.to_string()).unwrap_or_else(default_memail),
            });
        }
    }

    Ok(components)
}

pub struct KdlComponents {
    path: PathBuf,
}

impl KdlComponents {
    fn new(base: &Path) -> Self {
        KdlComponents {
            path: base.join("components.kdl"),
        }
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn read(&self) -> Result<Vec<Component>, String> {
        let content = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        parse_kdl_components(&content)
    }

    fn write(&self, components: &[Component]) -> Result<(), String> {
        let mut sorted = components.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        if self.path.exists() {
            numbered_backup(&self.path)?;
        }

        let mut content = String::from("// PisiLinux Components\n");
        for comp in &sorted {
            content.push_str(&component_kdl_block(comp));
            content.push('\n');
        }

        fs::write(&self.path, content).map_err(|e| e.to_string())?;
        println!(
            "✓ {} components written to {}",
            sorted.len(),
            self.path.display()
        );
        Ok(())
    }
}

// ── Shared helpers ──

fn parse_components_xml(content: &str) -> Vec<Component> {
    let name_re = Regex::new(r"<Name>(.+?)</Name>").unwrap();
    let lname_re =
        Regex::new(r#"<LocalName\s+xml:lang\s*=\s*["'](\w\w.*?)["']\s*>(.+?)</LocalName>"#)
            .unwrap();
    let summary_re =
        Regex::new(r#"<Summary\s+xml:lang\s*=\s*["'](\w\w.*?)["']\s*>(.+?)</Summary>"#).unwrap();
    let desc_re =
        Regex::new(r#"<Description\s+xml:lang\s*=\s*["'](\w\w.*?)["']\s*>(.+?)</Description>"#)
            .unwrap();
    let group_re = Regex::new(r"<Group>(.+?)</Group>").unwrap();
    let email_re = Regex::new(r"<Email>(.+?)</Email>").unwrap();
    let cbgn_re = Regex::new(r"<Component>").unwrap();
    let cend_re = Regex::new(r"</Component>").unwrap();
    let mbgn_re = Regex::new(r"<Maintainer>").unwrap();
    let mend_re = Regex::new(r"</Maintainer>").unwrap();

    let mut components = Vec::new();
    let mut in_component = false;
    let mut in_maintainer = false;
    let mut comp: Option<Component> = None;

    for line in content.lines() {
        if cbgn_re.is_match(line) {
            in_component = true;
            comp = Some(Component {
                name: String::new(),
                local_names: Vec::new(),
                summaries: Vec::new(),
                descriptions: Vec::new(),
                group: String::new(),
                maintainer_name: String::new(),
                maintainer_email: String::new(),
            });
            continue;
        }
        if cend_re.is_match(line) {
            in_component = false;
            if let Some(c) = comp.take() {
                if !c.name.is_empty() {
                    components.push(c);
                }
            }
            continue;
        }
        if !in_component {
            continue;
        }
        let comp = comp.as_mut().unwrap();

        if mbgn_re.is_match(line) {
            in_maintainer = true;
            continue;
        }
        if mend_re.is_match(line) {
            in_maintainer = false;
            continue;
        }

        if let Some(caps) = name_re.captures(line) {
            let val = caps.get(1).unwrap().as_str().to_string();
            if in_maintainer {
                comp.maintainer_name = val;
            } else {
                comp.name = val;
            }
        } else if let Some(caps) = email_re.captures(line) {
            comp.maintainer_email = caps.get(1).unwrap().as_str().to_string();
        } else if let Some(caps) = lname_re.captures(line) {
            comp.local_names.push((
                caps.get(1).unwrap().as_str().to_string(),
                caps.get(2).unwrap().as_str().to_string(),
            ));
        } else if let Some(caps) = summary_re.captures(line) {
            comp.summaries.push((
                caps.get(1).unwrap().as_str().to_string(),
                caps.get(2).unwrap().as_str().to_string(),
            ));
        } else if let Some(caps) = desc_re.captures(line) {
            comp.descriptions.push((
                caps.get(1).unwrap().as_str().to_string(),
                caps.get(2).unwrap().as_str().to_string(),
            ));
        } else if let Some(caps) = group_re.captures(line) {
            comp.group = caps.get(1).unwrap().as_str().to_string();
        }
    }

    components
}

fn component_xml_block(comp: &Component) -> String {
    let mut xml = format!(
        "        <Component>\n            <Name>{}</Name>\n",
        comp.name
    );
    for (lang, val) in &comp.local_names {
        xml.push_str(&format!(
            "            <LocalName xml:lang=\"{lang}\">{val}</LocalName>\n"
        ));
    }
    for (lang, val) in &comp.summaries {
        xml.push_str(&format!(
            "            <Summary xml:lang=\"{lang}\">{val}</Summary>\n"
        ));
    }
    for (lang, val) in &comp.descriptions {
        xml.push_str(&format!(
            "            <Description xml:lang=\"{lang}\">{val}</Description>\n"
        ));
    }
    xml.push_str(&format!(
        "            <Group>{}</Group>\n            <Maintainer>\n                <Name>{}</Name>\n                <Email>{}</Email>\n            </Maintainer>\n        </Component>",
        comp.group, comp.maintainer_name, comp.maintainer_email
    ));
    xml
}

fn numbered_backup(path: &Path) -> Result<(), String> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    let mut last: u32 = 0;
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if let Some(suffix) = fname.strip_prefix(&format!("{filename}.")) {
                if let Ok(num) = suffix.parse::<u32>() {
                    if num > last {
                        last = num;
                    }
                }
            }
        }
    }
    let backup_name = format!("{filename}.{:03}", last + 1);
    let backup_path = parent.join(&backup_name);
    fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
    println!("Backed up to {}", backup_path.display());
    Ok(())
}

// ── Public API ──

pub fn check_components(base_path: &Path, fix: bool) -> Result<(), String> {
    let mut component_dirs = Vec::new();
    find_component_dirs(base_path, base_path, &mut component_dirs, fix)?;

    let backend = ComponentsBackend::detect(base_path)?;

    let existing: HashSet<String> = backend.read()?.into_iter().map(|c| c.name).collect();

    let mut missing_components: Vec<String> = Vec::new();
    for (comp_name, _) in &component_dirs {
        if !existing.contains(comp_name) {
            missing_components.push(comp_name.clone());
        }
    }

    if !missing_components.is_empty() {
        if fix {
            println!(
                "Fixing {} - adding missing components: {:?}",
                backend
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                missing_components
            );
            backend.insert_missing(&missing_components)?;
            println!("{} updated.", backend.path().display());
        } else {
            println!(
                "WARNING: Components missing in {}:",
                backend.path().display()
            );
            for m in missing_components {
                println!("  - {}", m);
            }
            println!("Run with --fix to automatically add them.");
        }
    } else {
        println!("All components are in sync!");
    }

    Ok(())
}

pub fn edit_components(base_path: &Path) -> Result<(), String> {
    let backend = ComponentsBackend::detect(base_path)?;
    let mut components = backend.read()?;

    if components.is_empty() {
        return Err(format!(
            "No components found in {}",
            backend.path().display()
        ));
    }

    let format_label = match backend {
        ComponentsBackend::Xml(_) => "XML",
        ComponentsBackend::Kdl(_) => "KDL",
    };

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        println!("\n── Main Menu [{}] ──", format_label);
        println!("  l [pattern]  - list components (optional regex filter)");
        println!("  c <name/#>   - choose component to edit");
        println!("  w            - write (sorted + numbered backup)");
        println!("  q            - quit");
        println!();
        print!("> ");
        std::io::stdout().flush().map_err(|e| e.to_string())?;

        input.clear();
        stdin.read_line(&mut input).map_err(|e| e.to_string())?;
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "q" | "quit" => break,
            "w" | "write" => {
                backend.write(&components)?;
            }
            "l" | "list" => {
                let pattern = if arg.is_empty() { ".*" } else { arg };
                let re = Regex::new(pattern).map_err(|e| format!("Invalid regex: {e}"))?;
                let matched: Vec<&Component> =
                    components.iter().filter(|c| re.is_match(&c.name)).collect();
                if matched.is_empty() {
                    println!("No components match pattern.");
                } else {
                    for (i, comp) in matched.iter().enumerate() {
                        println!("  {}  {}", i, comp.name);
                    }
                }
            }
            "c" | "choose" => {
                if arg.is_empty() {
                    println!("Usage: c <name or #>");
                    continue;
                }
                let component: Option<usize> = if let Ok(idx) = arg.parse::<usize>() {
                    if idx < components.len() {
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    components.iter().position(|c| c.name == arg)
                };
                match component {
                    None => println!("Component not found: {arg}"),
                    Some(idx) => edit_component_loop(&mut components, idx, &stdin)?,
                }
            }
            "h" | "help" => {
                println!("Commands:");
                println!("  l [pattern]  - list components");
                println!("  c <name>     - choose component to edit");
                println!("  w            - write sorted + backup");
                println!("  q            - quit");
            }
            _ => println!("Unknown command: {cmd}"),
        }
    }

    Ok(())
}

fn edit_component_loop(
    components: &mut [Component],
    idx: usize,
    stdin: &std::io::Stdin,
) -> Result<(), String> {
    loop {
        let comp = &components[idx];
        println!("\n── Editing: {} ──", comp.name);
        println!("  LocalNames:");
        for (lang, val) in &comp.local_names {
            println!("    {lang}:{val}");
        }
        println!("  Summaries:");
        for (lang, val) in &comp.summaries {
            println!("    {lang}:{val}");
        }
        println!("  Descriptions:");
        for (lang, val) in &comp.descriptions {
            println!("    {lang}:{val}");
        }
        println!("  Group: {}", comp.group);
        println!("  Maintainer Name: {}", comp.maintainer_name);
        println!("  Maintainer Email: {}", comp.maintainer_email);
        println!();
        println!("  Commands:");
        println!("    ln <lang>:<name>  - set add/update LocalName");
        println!("    s  <lang>:<text>  - set add/update Summary");
        println!("    d  <lang>:<text>  - set add/update Description");
        println!("    g  <group>        - set Group");
        println!("    mn <name>         - set Maintainer Name");
        println!("    me <email>        - set Maintainer Email");
        println!("    m                 - main menu");
        println!("    h                 - help");
        println!();
        print!("> ");
        std::io::stdout().flush().map_err(|e| e.to_string())?;

        let mut input = String::new();
        stdin.read_line(&mut input).map_err(|e| e.to_string())?;
        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "m" => break,
            "h" => {
                println!("Editor commands:");
                println!("  ln <lang>:<name>  - add/update LocalName");
                println!("  s  <lang>:<text>  - add/update Summary");
                println!("  d  <lang>:<text>  - add/update Description");
                println!("  g  <group>        - set Group");
                println!("  mn <name>         - set Maintainer Name");
                println!("  me <email>        - set Maintainer Email");
                println!("  m                 - main menu");
            }
            "ln" | "s" | "d" => {
                if arg.is_empty() || !arg.contains(':') {
                    println!("Usage: {cmd} <lang>:<value>");
                    continue;
                }
                let colon_pos = arg.find(':').unwrap();
                let lang = &arg[..colon_pos];
                let value = &arg[colon_pos + 1..];
                let target = match cmd {
                    "ln" => &mut components[idx].local_names,
                    "s" => &mut components[idx].summaries,
                    "d" => &mut components[idx].descriptions,
                    _ => unreachable!(),
                };
                let mut found = false;
                for pair in target.iter_mut() {
                    if pair.0 == lang {
                        pair.1 = value.to_string();
                        found = true;
                        break;
                    }
                }
                if !found {
                    target.push((lang.to_string(), value.to_string()));
                }
            }
            "g" => {
                if arg.is_empty() {
                    println!("Usage: g <group>");
                    continue;
                }
                components[idx].group = arg.to_string();
            }
            "mn" => {
                if arg.is_empty() {
                    println!("Usage: mn <name>");
                    continue;
                }
                components[idx].maintainer_name = arg.to_string();
            }
            "me" => {
                if arg.is_empty() {
                    println!("Usage: me <email>");
                    continue;
                }
                components[idx].maintainer_email = arg.to_string();
            }
            _ => println!("Unknown command: {cmd}"),
        }
    }

    Ok(())
}

fn find_component_dirs(
    dir: &Path,
    base_path: &Path,
    component_dirs: &mut Vec<(String, PathBuf)>,
    fix: bool,
) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut has_pspec = false;
    let mut has_component = false;
    let mut has_actions = false;

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "pspec.xml" {
            has_pspec = true;
        } else if name == "component.xml" {
            has_component = true;
        } else if name == "actions.py" {
            has_actions = true;
        }
    }

    let rel = dir.strip_prefix(base_path).unwrap_or(dir);
    let rel_parts: Vec<&str> = rel
        .iter()
        .map(|s| s.to_str().unwrap_or(""))
        .filter(|s| !s.is_empty())
        .collect();

    if !rel_parts.contains(&"files") && !rel_parts.contains(&"comar") {
        let comp_name = rel_parts.join(".");

        if !comp_name.is_empty() {
            if !has_component && !has_pspec {
                if has_actions {
                    println!("WARNING: {}/pspec.xml not exists", dir.display());
                } else {
                    let mut has_inner_pspec = false;
                    check_inner_pspec(dir, &mut has_inner_pspec).map_err(|e| e.to_string())?;
                    if has_inner_pspec {
                        if fix {
                            let component_xml_path = dir.join("component.xml");
                            let content =
                                format!("<PISI>\n    <Name>{}</Name>\n</PISI>\n", comp_name);
                            fs::write(&component_xml_path, content).map_err(|e| e.to_string())?;
                            println!("Created component.xml for {}", comp_name);
                        } else {
                            println!("Missing component.xml in {}", dir.display());
                        }
                        component_dirs.push((comp_name.clone(), dir.to_path_buf()));
                    }
                }
            } else if has_component {
                component_dirs.push((comp_name.clone(), dir.to_path_buf()));
            }
        }

        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_component_dirs(&path, base_path, component_dirs, fix)?;
            }
        }
    }

    Ok(())
}

fn check_inner_pspec(dir: &Path, found: &mut bool) -> std::io::Result<()> {
    if *found {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            check_inner_pspec(&path, found)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("pspec.xml") {
            *found = true;
            return Ok(());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_roundtrip() {
        let comps = vec![Component::empty("test.a"), Component::empty("test.b")];
        let xml_backend = XmlComponents {
            path: PathBuf::from("/tmp/_test_components.xml"),
        };
        xml_backend.write(&comps).unwrap();
        let read = xml_backend.read().unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].name, "test.a");
        assert_eq!(read[1].name, "test.b");
        let _ = std::fs::remove_file("/tmp/_test_components.xml");
    }

    #[test]
    fn test_kdl_roundtrip() {
        let comps = vec![Component::empty("test.a"), Component::empty("test.b")];
        let kdl_backend = KdlComponents {
            path: PathBuf::from("/tmp/_test_components.kdl"),
        };
        kdl_backend.write(&comps).unwrap();
        let read = kdl_backend.read().unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].name, "test.a");
        assert_eq!(read[1].name, "test.b");
        let _ = std::fs::remove_file("/tmp/_test_components.kdl");
    }
}
