use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- XML / Paket Yapısı ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "PISI")]
pub struct PisiRoot {
    #[serde(rename = "Source", default)]
    pub source: Option<SourceInfo>,
    #[serde(rename = "Package")]
    pub package: Package,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Homepage", default)]
    pub homepage: String,
    #[serde(rename = "Packager", default)]
    pub packager: Option<Packager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Summary")]
    pub summaries: Vec<LocalizedText>,

    #[serde(rename = "Description")]
    pub descriptions: Vec<LocalizedText>,

    #[serde(rename = "History")]
    pub history: History,

    #[serde(rename = "Architecture")]
    pub architecture: String,

    #[serde(rename = "RuntimeDependencies", default)] // 👈 Büyük R ve büyük D olduğundan emin olun
    pub runtime_dependencies: Option<RuntimeDependencies>,

    #[serde(rename = "Conflicts", default)]
    pub conflicts: Option<Conflicts>,

    #[serde(rename = "BuildDependencies", default)]
    pub build_dependencies: Option<BuildDependencies>,

    // Repo işlemlerinde URL oluşturmak için kullanılır
    #[serde(rename = "Archive")]
    pub archive: Option<String>,

    // XML'deki PackageURI alanını yakalamak için
    #[serde(rename = "PackageURI", alias = "package_uri", default)]
    pub package_uri: String,

    // Paketin hangi depodan geldiğini saklamak için
    #[serde(default)]
    pub repo_url: String,

    // Alternatif yansıma (mirror) URL'leri
    #[serde(default)]
    pub mirrors: Vec<String>,

    #[serde(rename = "PartOf", default)]
    pub partof: String,

    #[serde(rename = "License", default)]
    pub licenses: Vec<String>,

    #[serde(rename = "IsA", default)]
    pub provides: Vec<String>,

    #[serde(rename = "Provides", default)]
    pub provides_block: Option<ProvidesBlock>,

    // Hem nitelik (@release) hem de eleman (Release) olarak gelme ihtimaline karşı:
    #[serde(rename = "@release", alias = "Release", default)]
    pub release: u32,

    #[serde(rename = "PackageHash", default)]
    pub package_hash: String, // İndeks dosyasından gelen hash

    #[serde(rename = "InstalledSize")]
    pub installed_size: u64,

    #[serde(rename = "PackageSize", default)]
    pub package_size: u64,

    #[serde(rename = "DistributionRelease", default)]
    pub distribution_release: String,

    #[serde(default)]
    pub repo_name: String,

    #[serde(rename = "PreInstall", default)]
    pub pre_install: Option<String>,

    #[serde(rename = "PostInstall", default)]
    pub post_install: Option<String>,

    #[serde(rename = "PreUpgrade", default)]
    pub pre_upgrade: Option<String>,

    #[serde(rename = "PostUpgrade", default)]
    pub post_upgrade: Option<String>,

    #[serde(rename = "PostRemove", default)]
    pub post_remove: Option<String>,
    #[serde(rename = "PreRemove", default)]
    pub pre_remove: Option<String>,

    #[serde(rename = "Users", default)]
    pub users: Option<pisi_spec::models::UsersWrapper>,

    #[serde(rename = "Groups", default)]
    pub groups: Option<pisi_spec::models::GroupsWrapper>,

    #[serde(rename = "Homepage", default)]
    pub homepage: Option<String>,

    #[serde(rename = "Icon", default)]
    pub icon: Option<String>,

    #[serde(rename = "ScreenShot", default)]
    pub screenshot: Option<String>,

    #[serde(rename = "Packager", default)]
    pub packager: Option<Packager>,

    #[serde(rename = "Source", default)]
    pub source: Option<PackageSource>,

    #[serde(rename = "InstallTarHash", default)]
    pub install_tar_hash: Option<String>,

    #[serde(rename = "PackageFormat", default)]
    pub package_format: Option<String>,

    #[serde(rename = "BuildHost", default)]
    pub build_host: Option<String>,

    #[serde(rename = "Distribution", default)]
    pub distribution: Option<String>,

    #[serde(skip)]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvidesBlock {
    #[serde(rename = "COMAR", default)]
    pub comar: Vec<ComarProvide>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComarProvide {
    #[serde(rename = "$value")]
    pub provide: String,
    #[serde(rename = "@script", default)]
    pub script: Option<String>,
    #[serde(rename = "@name", default)]
    pub name: Option<String>,
}

impl Package {
    pub fn effective_homepage(&self) -> Option<String> {
        self.homepage
            .clone()
            .or_else(|| self.source.as_ref().and_then(|s| s.homepage.clone()))
    }

    pub fn effective_packager(&self) -> Option<Packager> {
        self.packager
            .clone()
            .or_else(|| self.source.as_ref().and_then(|s| s.packager.clone()))
    }

    pub fn effective_screenshot(&self) -> Option<String> {
        self.screenshot
            .clone()
            .or_else(|| self.source.as_ref().and_then(|s| s.screenshot.clone()))
    }

    /// Veritabanı boyutunu küçültmek için geçmişi son 3 kayıtla sınırlandırır.
    pub fn prune(&mut self) {
        if self.history.updates.len() > 3 {
            self.history.updates.truncate(3);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageSource {
    #[serde(rename = "Name", default)]
    pub name: Option<String>,
    #[serde(rename = "Homepage", default)]
    pub homepage: Option<String>,
    #[serde(rename = "Packager", default)]
    pub packager: Option<Packager>,
    #[serde(rename = "ScreenShot", default)]
    pub screenshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Packager {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Email", default)]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalizedText {
    #[serde(rename = "$value")]
    pub text: String,

    #[serde(
        rename = "@xml:lang",
        alias = "@lang",
        alias = "xml:lang",
        alias = "lang"
    )]
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDependencies {
    #[serde(rename = "Dependency", default)] // XML'de <Dependency> ise büyük D olmalı
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDependencies {
    #[serde(rename = "Dependency", default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflicts {
    #[serde(rename = "Package", default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct History {
    // XML'deki her bir <Update> etiketini bu vektöre doldurur.
    // Rename yapılmazsa, serde 'updates' isminde bir etiket arar ve bulamaz.
    #[serde(rename = "Update", default)]
    pub updates: Vec<Update>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    // Hem @release hem release ismini kabul et ve eksikse hata verme (default: 0)
    #[serde(rename = "@release", alias = "release", default)]
    pub release: u32,

    #[serde(rename = "Version")]
    pub version: String,

    #[serde(rename = "Date")]
    pub date: String,

    #[serde(rename = "Comment")]
    #[serde(default)] // Bazı güncellemelerde yorum olmayabilir
    pub comment: String,

    #[serde(rename = "Name", default)]
    pub name: String,

    #[serde(rename = "Email", default)]
    pub email: Option<String>,

    #[serde(rename = "Type", alias = "type", default)]
    pub type_: Option<String>,

    #[serde(rename = "Requires", default)]
    pub requires: Option<UpdateRequires>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRequires {
    #[serde(rename = "Action", default)]
    pub actions: Vec<UpdateAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAction {
    #[serde(rename = "@package", default)]
    pub package: Option<String>,
    #[serde(rename = "$value", default)]
    pub action: String,
}

// --- İndeks / Veritabanı Yapıları ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIndexEntry {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "SourceURI", default)]
    pub source_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIndexEntry {
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFileIndexEntry {
    #[serde(rename = "Source")]
    pub source: SourceIndexEntry,

    #[serde(rename = "Package", default)]
    pub packages: Vec<PackageIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Distribution {
    #[serde(rename = "SourceName")]
    pub source_name: String,

    #[serde(rename = "Description", default)]
    pub descriptions: Vec<LocalizedText>,

    #[serde(rename = "Version")]
    pub version: String,

    #[serde(rename = "Type")]
    pub distro_type: String,

    #[serde(rename = "BinaryName")]
    pub binary_name: String,

    #[serde(rename = "Obsoletes", default)]
    pub obsoletes: Option<Obsoletes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Obsoletes {
    #[serde(rename = "Package", default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "LocalName", default)]
    pub local_names: Vec<LocalizedText>,

    #[serde(rename = "Icon", default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Maintainer {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Email")]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename = "PISI")]
pub struct PisiIndex {
    #[serde(rename = "Distribution", default)]
    pub distribution: Option<Distribution>,

    #[serde(rename = "Package", default)]
    pub packages: Vec<Package>,

    #[serde(rename = "SpecFile", default)]
    pub spec_files: Vec<SpecFileIndexEntry>,

    #[serde(rename = "Component", default)]
    pub components: Vec<Component>,

    #[serde(rename = "Group", default)]
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Component {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "LocalName", default)]
    pub local_names: Vec<LocalizedText>,

    #[serde(rename = "Summary", default)]
    pub summaries: Vec<LocalizedText>,

    #[serde(rename = "Description", default)]
    pub descriptions: Vec<LocalizedText>,

    #[serde(rename = "Group", default)]
    pub group: Option<String>,

    #[serde(rename = "Maintainer", default)]
    pub maintainer: Option<Maintainer>,

    #[serde(rename = "Dependencies", default)]
    pub dependencies: Option<ComponentDependencies>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentDependencies {
    #[serde(rename = "Dependency", default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "PISI")]
pub struct ComponentsRoot {
    #[serde(rename = "Components", default)]
    pub components: ComponentList,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentList {
    #[serde(rename = "Component", default)]
    pub items: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "PISI")]
pub struct GroupsRoot {
    #[serde(rename = "Groups", default)]
    pub groups: GroupList,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GroupList {
    #[serde(rename = "Group", default)]
    pub items: Vec<Group>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename = "PISI")]
pub struct DistributionRoot {
    #[serde(rename = "SourceName", default)]
    pub source_name: String,

    #[serde(rename = "Version", default)]
    pub version: String,

    #[serde(rename = "Description", default)]
    pub descriptions: Vec<LocalizedText>,

    #[serde(rename = "Type", default)]
    pub distro_type: String,

    #[serde(rename = "BinaryName", default)]
    pub binary_name: String,

    #[serde(rename = "Obsoletes", default)]
    pub obsoletes: Option<Obsoletes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub install_date: String,
    pub installed_files: HashMap<String, FileMetadata>,
    pub total_size: u64,
    pub package_hash: String,
    #[serde(default)]
    pub release: u32,
    pub distribution_release: String,
    #[serde(default)]
    pub licenses: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub post_remove: Option<String>,
    #[serde(default)]
    pub pre_remove: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub screenshot: Option<String>,
    #[serde(default)]
    pub packager: Option<Packager>,
    #[serde(default)]
    pub install_tar_hash: Option<String>,
    #[serde(default)]
    pub package_format: Option<String>,
    #[serde(default)]
    pub build_host: Option<String>,
    #[serde(default)]
    pub distribution: Option<String>,
    #[serde(default = "default_configured")]
    pub configured: bool,
    #[serde(default)]
    pub signature_verified: bool,
}

impl InstalledPackage {
    /// Veritabanı boyutunu küçültmek için (eğer varsa) geçmişi veya gereksiz alanları temizler.
    pub fn prune(&mut self) {
        // InstalledPackage içinde şu an tam History yok ama eğer eklenirse burası kullanılabilir.
        // Şimdilik sadece metod imzası olarak dursun.
    }
}

fn default_configured() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub mode: u32,
    pub uid: u64,
    pub gid: u64,
    pub size: u64,
}

// --- Yardımcı Yapılar ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesXmlRoot {
    #[serde(rename = "File", default)]
    pub files: Vec<FileXmlEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileXmlEntry {
    #[serde(rename = "Path")]
    pub path: String,

    #[serde(rename = "Type")]
    pub file_type: String,

    #[serde(rename = "Size")]
    pub size: u64,

    #[serde(rename = "Uid")]
    pub uid: u64,

    #[serde(rename = "Gid")]
    pub gid: u64,

    #[serde(rename = "Mode")]
    pub mode: String,

    #[serde(rename = "Hash")]
    pub hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PisiPackageData {
    pub metadata: Package,
    pub files: Vec<FileData>,
    pub files_xml: Option<Vec<FileXmlEntry>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileData {
    pub content: Vec<u8>,
    pub path: String,
    pub size: u64,
    #[serde(default)]
    pub mode: u32,
    #[serde(default)]
    pub uid: u64,
    #[serde(default)]
    pub gid: u64,
    pub symlink_target: Option<String>,
}

// --- Implementasyonlar ---

impl Package {
    /// Paketin en son (en güncel) sürüm numarasını döndürür.
    pub fn latest_version(&self) -> String {
        self.history
            .updates
            .first()
            .map(|u| u.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string())
    }

    pub fn get_summary(&self) -> String {
        self.summaries
            .iter()
            .find(|s| s.lang.as_deref() == Some("tr")) // Önce Türkçe
            .or_else(|| {
                self.summaries
                    .iter()
                    .find(|s| s.lang.as_deref() == Some("en"))
            }) // Sonra İngilizce
            .map(|s| s.text.clone())
            .or_else(|| self.summaries.first().map(|s| s.text.clone())) // En son ilk eleman
            .unwrap_or_default()
    }

    pub fn get_description(&self) -> String {
        self.descriptions
            .iter()
            .find(|d| d.lang.as_deref() == Some("tr")) // Önce Türkçe
            .or_else(|| {
                self.descriptions
                    .iter()
                    .find(|d| d.lang.as_deref() == Some("en"))
            }) // Sonra İngilizce
            .map(|d| d.text.clone())
            .or_else(|| self.descriptions.first().map(|d| d.text.clone())) // En son ilk eleman
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_source_nesting() {
        let xml = r#"
        <Package>
            <Name>test-pkg</Name>
            <Summary>Test Summary</Summary>
            <Description>Test Description</Description>
            <Source>
                <Homepage>https://nest.org</Homepage>
                <Packager>
                    <Name>Nestor</Name>
                    <Email>nest@pisi.org</Email>
                </Packager>
            </Source>
            <Icon>nest-icon</Icon>
            <History>
                <Update release="1">
                    <Version>1.0</Version>
                    <Date>2023-01-01</Date>
                    <Comment>Init</Comment>
                </Update>
            </History>
            <Architecture>x86_64</Architecture>
            <InstalledSize>100</InstalledSize>
            <PackageSize>50</PackageSize>
            <PackageHash>hash123</PackageHash>
            <PackageURI>t/test-pkg.pisi</PackageURI>
        </Package>
        "#;

        let pkg: Package = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(pkg.name, "test-pkg");
        assert_eq!(pkg.effective_homepage().unwrap(), "https://nest.org");
        assert_eq!(pkg.effective_packager().unwrap().name, "Nestor");
        assert_eq!(pkg.icon.unwrap(), "nest-icon");
    }
}
