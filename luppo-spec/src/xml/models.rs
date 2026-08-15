use serde::Deserialize;
use serde::Serialize;


#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Patch {
    #[serde(rename = "$value", alias = "file")]
    pub file: String,
    #[serde(
        alias = "@level",
        alias = "level",
        default
    )]
    pub level: Option<u8>,
    #[serde(rename = "@compressionType", alias = "compressionType", default)]
    pub compression_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PatchesWrapper {
    #[serde(rename = "Patch", alias = "patch", default)]
    pub patches: Vec<Patch>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TranslationEntry {
    #[serde(rename = "Summary", alias = "summary", default)]
    pub summary: Option<String>,
    #[serde(rename = "Description", alias = "description", default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AdditionalFile {
    #[serde(rename = "@target", alias = "target", default)]
    pub target: String,
    #[serde(rename = "$value", alias = "filename", default)]
    pub filename: String,
    #[serde(rename = "@owner", alias = "owner", default)]
    pub owner: Option<String>,
    #[serde(rename = "@permission", alias = "permission", default)]
    pub permission: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AdditionalFilesWrapper {
    #[serde(rename = "AdditionalFile", alias = "additional-file", default)]
    pub files: Vec<AdditionalFile>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ReplacesWrapper {
    #[serde(rename = "Package", alias = "package", default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ConflictsWrapper {
    #[serde(rename = "Package", alias = "package", default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BuildFlagsWrapper {
    #[serde(rename = "Flag", alias = "flag", default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnyDependency {
    #[serde(rename = "Dependency", default)]
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Archive {
    #[serde(rename = "$value", alias = "url")]
    pub url: String,
    #[serde(rename = "@sha1sum", alias = "sha1sum")]
    pub sha1sum: Option<String>,
    #[serde(rename = "@md5sum", alias = "md5sum")]
    pub md5sum: Option<String>,
    #[serde(rename = "@hash", alias = "hash")]
    pub hash: Option<String>,
    #[serde(rename = "@type", alias = "type")]
    pub archive_type: String,
    #[serde(rename = "@target", alias = "target", default)]
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
    #[serde(rename = "Name", alias = "name", default)]
    pub name: String,
    #[serde(rename = "Email", alias = "email", default)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Dependency {
    #[serde(rename = "$value", alias = "name")]
    pub name: String,
    #[serde(rename = "@release", alias = "release")]
    pub release: Option<String>,
    #[serde(rename = "@versionFrom", alias = "version_from")]
    pub version_from: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Source {
    #[serde(rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Homepage", alias = "homepage", default)]
    pub homepage: Option<String>,
    #[serde(rename = "Packager", alias = "packager", default)]
    pub packager: Option<Packager>,
    #[serde(rename = "License", alias = "license")]
    pub license: Vec<String>,
    #[serde(rename = "Summary", alias = "summary", default)]
    pub summary: Option<String>,
    #[serde(rename = "Description", alias = "description", default)]
    pub description: Option<String>,
    #[serde(rename = "Translation", alias = "translations", default)]
    pub translations: std::collections::HashMap<String, TranslationEntry>,
    #[serde(rename = "PartOf", alias = "part-of", default)]
    pub part_of: Option<String>,
    #[serde(rename = "Archive", alias = "archives", default)]
    pub archives: Vec<Archive>,
    #[serde(alias = "archive_url", skip_serializing)]
    pub archive_url_kdl: Option<String>,
    #[serde(alias = "archive_sha1", skip_serializing)]
    pub archive_sha1_kdl: Option<String>,

    #[serde(rename = "BuildDependencies", alias = "build-dependencies", default)]
    pub build_dependencies: Option<BuildDeps>,
    #[serde(rename = "BuildFlags", alias = "build-flags", default)]
    pub build_flags: Option<BuildFlagsWrapper>,
    #[serde(rename = "Patches", alias = "patches", default)]
    pub patches: Option<PatchesWrapper>,
    #[serde(rename = "AdditionalFiles", alias = "additional-files", default)]
    pub additional_files: Option<AdditionalFilesWrapper>,
    #[serde(rename = "History", default)]
    pub history: Option<History>,
    #[serde(alias = "history", skip_serializing)]
    pub history_kdl: Option<Vec<Update>>,
    #[serde(rename = "Icon", alias = "icon", default)]
    pub icon: Option<String>,
    #[serde(rename = "ScreenShot", alias = "screenshot", default)]
    pub screenshot: Option<String>,
    #[serde(rename = "IsA", alias = "provides", default)]
    pub provides: Vec<String>,
    #[serde(rename = "Architecture", alias = "architecture", default)]
    pub architecture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BuildDeps {
    #[serde(rename = "Dependency", default)]
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PathDef {
    #[serde(rename = "$value")]
    pub path: String,
    #[serde(alias = "@fileType")]
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Files {
    #[serde(rename = "Path", default)]
    pub paths: Vec<PathDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Dependencies {
    #[serde(default)]
    pub runtime: Vec<String>,
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Comar {
    #[serde(rename = "$value", alias = "provide")]
    pub provide: String,
    #[serde(rename = "@script", alias = "script")]
    pub script: String,
    #[serde(rename = "@name", alias = "name", default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ProvidesBlock {
    #[serde(rename = "COMAR", alias = "comar", default)]
    pub comar: Vec<Comar>,
    #[serde(rename = "IsA", alias = "isa", default)]
    pub isa: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageActions {
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub pre_install: Option<String>,
    #[serde(default)]
    pub post_install: Option<String>,
    #[serde(default)]
    pub pre_upgrade: Option<String>,
    #[serde(default)]
    pub post_upgrade: Option<String>,
    #[serde(default)]
    pub pre_remove: Option<String>,
    #[serde(default)]
    pub post_remove: Option<String>,
    #[serde(default)]
    pub install_filters: Vec<String>,
    #[serde(rename = "NoStrip", default)]
    pub no_strip: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct User {
    #[serde(rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "@uid", alias = "uid", default)]
    pub uid: Option<u32>,
    #[serde(rename = "@gid", alias = "gid", default)]
    pub gid: Option<u32>,
    #[serde(rename = "@home", alias = "home", default)]
    pub home: Option<String>,
    #[serde(rename = "@shell", alias = "shell", default)]
    pub shell: Option<String>,
    #[serde(rename = "@system", alias = "system", default)]
    pub system: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UsersWrapper {
    #[serde(rename = "User", alias = "user", default)]
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Group {
    #[serde(rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "@gid", alias = "gid", default)]
    pub gid: Option<u32>,
    #[serde(rename = "@system", alias = "system", default)]
    pub system: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GroupsWrapper {
    #[serde(rename = "Group", alias = "group", default)]
    pub groups: Vec<Group>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageDefinition {
    #[serde(rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Summary", alias = "summary", default)]
    pub summary: String,
    #[serde(rename = "Description", alias = "description", default)]
    pub description: String,
    #[serde(rename = "Translation", alias = "translations", default)]
    pub translations: std::collections::HashMap<String, TranslationEntry>,
    #[serde(
        rename = "RuntimeDependencies",
        alias = "runtime-dependencies",
        default
    )]
    pub runtime_dependencies: Option<RuntimeDeps>,
    #[serde(
        rename = "BuildDependencies",
        alias = "build-dependencies",
        default
    )]
    pub build_dependencies: Option<BuildDeps>,
    #[serde(rename = "Files", alias = "files", default)]
    pub files: Files,

    #[serde(rename = "Homepage", alias = "homepage", default)]
    pub homepage: Option<String>,
    #[serde(rename = "Icon", alias = "icon", default)]
    pub icon: Option<String>,
    #[serde(rename = "ScreenShot", alias = "screenshot", default)]
    pub screenshot: Option<String>,
    #[serde(rename = "Provides", alias = "provides", default)]
    pub provides: Option<ProvidesBlock>,

    #[serde(rename = "AdditionalFiles", alias = "additional-files", default)]
    pub additional_files: Option<AdditionalFilesWrapper>,

    #[serde(rename = "PartOf", alias = "part-of", default)]
    pub part_of: Option<String>,
    #[serde(rename = "Replaces", alias = "replaces", default)]
    pub replaces: Option<ReplacesWrapper>,
    #[serde(rename = "Conflicts", alias = "conflicts", default)]
    pub conflicts: Option<ConflictsWrapper>,

    #[serde(rename = "BuildType", alias = "build-type", default)]
    pub build_type: Option<String>,

    #[serde(rename = "Users", alias = "users", default)]
    pub users: Option<UsersWrapper>,

    #[serde(rename = "Groups", alias = "groups", default)]
    pub groups: Option<GroupsWrapper>,

    #[serde(rename = "Mirrors", alias = "mirrors", default)]
    pub mirrors: Vec<String>,

    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub packager: Packager,
    #[serde(default)]
    pub deps: Dependencies,
    #[serde(default)]
    pub actions: PackageActions,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct RuntimeDeps {
    #[serde(rename = "Dependency", default)]
    pub dependencies: Vec<Dependency>,
    #[serde(rename = "AnyDependency", default)]
    pub any_dependency: Option<AnyDependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Update {
    #[serde(
        rename = "@release",
        alias = "release"
    )]
    pub release: u32,
    #[serde(rename = "Date", alias = "date")]
    pub date: String,
    #[serde(rename = "Version", alias = "version")]
    pub version: String,
    #[serde(rename = "Comment", alias = "comment")]
    pub comment: String,
    #[serde(rename = "Name", alias = "name", alias = "committer")]
    pub committer: String,
    #[serde(rename = "Email", alias = "email", default)]
    pub email: String,
    #[serde(rename = "Type", alias = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(rename = "Requires", default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<UpdateRequires>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateRequires {
    #[serde(rename = "Action", default)]
    pub actions: Vec<UpdateAction>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateAction {
    #[serde(rename = "@package", default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(rename = "$value", default)]
    pub action: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct History {
    #[serde(rename = "Update", default)]
    pub updates: Vec<Update>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "LUPPO")]
pub struct LuppoSpec {
    #[serde(rename = "Source", alias = "source-metadata")]
    pub source: Source,
    #[serde(rename = "Package", alias = "subpackages", default)]
    pub packages: Vec<PackageDefinition>,
    #[serde(rename = "History", default)]
    pub history: Option<History>,
    #[serde(alias = "history", skip_serializing)]
    pub history_kdl: Option<Vec<Update>>,
    #[serde(
        rename = "MainPackage",
        alias = "main-package",
        skip_serializing_if = "Option::is_none"
    )]
    pub main_package: Option<PackageDefinition>,
}

// COMAR models
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TriggerEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Suffix")]
    pub suffix: Option<String>,
    #[serde(rename = "Script")]
    pub script: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename = "Triggers")]
pub struct TriggersConfig {
    #[serde(rename = "Trigger", default)]
    pub triggers: Vec<TriggerEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryAction {
    pub trace_id: u64,
    pub operation: String,
    pub timestamp: String,
    pub details: String,
}
