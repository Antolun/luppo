pub mod models;

use chrono::NaiveDate;
use models::PisiSpec;
use regex::Regex;
use roxmltree::Document;
use rust_i18n::t;
use std::fs;
use std::path::Path;

/// Belirtilen XML dosyasını ayrıştırır ve bir PisiSpec yapısı döndürür.
pub fn strip_doctype(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(pos) = remaining.find("<!DOCTYPE") {
        result.push_str(&remaining[..pos]);
        let rest = &remaining[pos..];
        let bytes = rest.as_bytes();
        let mut depth: i32 = 0;
        let mut in_quote = false;
        let mut quote = b'"';
        let mut i = 9;
        while i < bytes.len() {
            if in_quote {
                if bytes[i] == quote {
                    in_quote = false;
                }
            } else {
                match bytes[i] {
                    b'"' | b'\'' => { in_quote = true; quote = bytes[i]; }
                    b'[' => depth += 1,
                    b']' => depth = depth.saturating_sub(1),
                    b'>' if depth <= 0 => {
                        remaining = &rest[i + 1..];
                        break;
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        if i >= bytes.len() {
            result.push_str(rest);
            remaining = "";
            break;
        }
    }
    result.push_str(remaining);
    result
}

pub fn strip_xml_declaration(content: &str) -> String {
    if content.starts_with("<?xml") {
        if let Some(end) = content.find("?>") {
            // ?>'den sonraki kısmı al, baştaki boşlukları koru (trim yok)
            content[end + 2..].to_string()
        } else {
            content.to_string()
        }
    } else {
        content.to_string()
    }
}

pub fn strip_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(pos) = remaining.find("<!--") {
        result.push_str(&remaining[..pos]);
        if let Some(end) = remaining[pos..].find("-->") {
            remaining = &remaining[pos + end + 3..];
        } else {
            result.push_str(&remaining[pos..]);
            remaining = "";
            break;
        }
    }
    result.push_str(remaining);
    result
}

pub fn strip_bom_and_meta(content: &str) -> String {
    let mut s: &str = content.trim();
    if let Some(stripped) = s.strip_prefix('\u{FEFF}') {
        s = stripped.trim();
    }
    let s = strip_doctype(s);
    let s = strip_comments(&s);

    // Strip PIs (including <?xml?>) from the beginning only
    let mut s: &str = &s;
    loop {
        let before = s.len();
        s = s.trim_start();
        if s.starts_with("<?") {
            if let Some(end) = s.find("?>") {
                s = &s[end + 2..];
                continue;
            }
        }
        if s.len() == before {
            break;
        }
        break;
    }
    s.trim().to_string()
}

/// Removes all <?...?> processing instructions from the content (global).
/// Unlike `strip_xml_declaration` which only removes from the start,
/// this removes PIs everywhere. Use for complete sanitization.
pub fn strip_all_pis(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(pos) = remaining.find("<?") {
        result.push_str(&remaining[..pos]);
        if let Some(end) = remaining[pos..].find("?>") {
            remaining = &remaining[pos + end + 2..];
        } else {
            result.push_str(&remaining[pos..]);
            remaining = "";
            break;
        }
    }
    result.push_str(remaining);
    result
}

/// quick-xml serde deserializer'ı elementler arasındaki entity
/// referanslarından (ör. `&gt;`, `&amp;`) kaynaklanan metin düğümlerini
/// kaldırmaz. Bu fonksiyon, elementler arasındaki (`>` ile `<` arasındaki)
/// entity referanslarını temizler.
fn strip_inter_element_entities(content: &str) -> String {
    // Regex: > ile < arasında kalan entity referanslarını bul ve kaldır
    let re = regex::Regex::new(r">\s*&(?:amp|lt|gt|quot|apos|#\d+|#x[0-9a-fA-F]+);\s*<")
        .expect("valid regex");
    re.replace_all(content, "><").to_string()
}

pub fn sanitize_xml_for_serde(content: &str) -> String {
    strip_inter_element_entities(&strip_bom_and_meta(content))
}

pub fn parse_xml_spec<P: AsRef<Path>>(path: P) -> Result<PisiSpec, String> {
    let path_ref = path.as_ref();
    let content = fs::read_to_string(path_ref).map_err(|e| e.to_string())?;

    let content = sanitize_xml_for_serde(&content);

    validate_pspec(&content)?;

    let spec: PisiSpec = quick_xml::de::from_str(&content)
        .map_err(|e| {
            // quick-xml hatasını debug için ilk 200 karakteri göster
            let preview: String = content.chars().take(200).collect();
            format!(
                "{} [preview: {}]",
                rust_i18n::t!("models_error_xml", error = e.to_string()),
                preview.replace('\n', "\\n")
            )
        })?;

    Ok(spec)
}

pub fn validate_pspec(content: &str) -> Result<(), String> {
    // roxmltree DTD'leri desteklemez ve DtdDetected hatası verir.
    // <!DOCTYPE zaten parse_xml_spec'te temizlendi, ama buraya doğrudan
    // çağrılırsa diye yine de temizle.
    let cleaned_content = strip_doctype(content);

    let doc = Document::parse(&cleaned_content)
        .map_err(|e| t!("val_xml_parse_error", error = format!("{}", e)).to_string())?;

    // Kurallar için Regex tanımlamaları
    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    let archive_hash_re = Regex::new(r"^[a-fA-F0-9]{40}|[a-fA-F0-9]{64}$").unwrap();
    let release_re = Regex::new(r"^\d+$").unwrap();

    let mut errors = Vec::new();

    // Ağacı gez ve kuralları doğrula
    for node in doc.descendants() {
        let node: roxmltree::Node = node;
        if !node.is_element() {
            continue;
        }

        let tag_name = node.tag_name().name();

        match tag_name {
            "Date" => {
                if let Some(text) = node.text() {
                    let trimmed: &str = text.trim();
                    if NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").is_err() {
                        let pos = doc.text_pos_at(node.range().start);
                        errors.push(
                            t!(
                                "val_invalid_date",
                                row = pos.row,
                                col = pos.col,
                                value = trimmed
                            )
                            .to_string(),
                        );
                    }
                }
            }
            "Email" => {
                if let Some(text) = node.text() {
                    let trimmed: &str = text.trim();
                    if !email_re.is_match(trimmed) {
                        let pos = doc.text_pos_at(node.range().start);
                        errors.push(
                            t!(
                                "val_invalid_email",
                                row = pos.row,
                                col = pos.col,
                                value = trimmed
                            )
                            .to_string(),
                        );
                    }
                }
            }
            "Archive" => {
                if let Some(hash) = node.attribute("sha1sum").or_else(|| node.attribute("hash")) {
                    let trimmed: &str = hash.trim();
                    if !archive_hash_re.is_match(trimmed) {
                        let pos = doc.text_pos_at(node.range().start);
                        errors.push(
                            t!(
                                "val_invalid_hash",
                                row = pos.row,
                                col = pos.col,
                                value = trimmed
                            )
                            .to_string(),
                        );
                    }
                }
            }
            "Update" => {
                if let Some(release) = node.attribute("release") {
                    let trimmed: &str = release.trim();
                    if !release_re.is_match(trimmed) {
                        let pos = doc.text_pos_at(node.range().start);
                        errors.push(
                            t!(
                                "val_invalid_release",
                                row = pos.row,
                                col = pos.col,
                                value = trimmed
                            )
                            .to_string(),
                        );
                    }
                }
            }
            // Gerekli Alanların boş olmaması kontrolü
            "Name" | "Summary" | "Description" | "License" | "Version" => {
                if let Some(text) = node.text() {
                    let trimmed: &str = text.trim();
                    if trimmed.is_empty() {
                        let pos = doc.text_pos_at(node.range().start);
                        errors.push(
                            t!(
                                "val_field_empty",
                                row = pos.row,
                                col = pos.col,
                                field = tag_name
                            )
                            .to_string(),
                        );
                    }
                } else {
                    let pos = doc.text_pos_at(node.range().start);
                    errors.push(
                        t!(
                            "val_field_missing",
                            row = pos.row,
                            col = pos.col,
                            field = tag_name
                        )
                        .to_string(),
                    );
                }
            }
            _ => {}
        }
    }

    if !errors.is_empty() {
        let err_msg = format!("{}:\n{}", t!("val_errors_title"), errors.join("\n"));
        return Err(err_msg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_pspec() {
        let xml = r#"
        <PISI>
            <Source>
                <Name>test-package</Name>
                <Version>1.0</Version>
                <Summary>Test summary</Summary>
                <Description>Test description</Description>
                <License>GPLv2</License>
                <Archive sha1sum="1234567890123456789012345678901234567890">http://test.com/test.tar.gz</Archive>
            </Source>
            <History>
                <Update release="1">
                    <Date>2026-04-20</Date>
                    <Version>1.0</Version>
                    <Comment>First release</Comment>
                    <Name>John Doe</Name>
                    <Email>john@doe.com</Email>
                </Update>
            </History>
        </PISI>
        "#;
        assert!(validate_pspec(xml).is_ok());
    }

    #[test]
    fn test_validate_invalid_date() {
        let xml = r#"
        <PISI>
            <History>
                <Update release="1">
                    <Date>20-04-2026</Date>
                </Update>
            </History>
        </PISI>
        "#;
        let result = validate_pspec(xml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("VALIDATION ERROR: {}", err);
        assert!(err.contains("Satır:") || err.contains("Line:"));
    }

    #[test]
    fn test_validate_invalid_month() {
        let xml = r#"
        <PISI>
            <History>
                <Update release="1">
                    <Date>2025-21-14</Date>
                </Update>
            </History>
        </PISI>
        "#;
        let result = validate_pspec(xml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("VALIDATION ERROR: {}", err);
        assert!(err.contains("Satır:") || err.contains("Line:"));
    }

    #[test]
    fn test_parse_libreoffice_pspec() {
        let path = "/media/pisicik/DEPO/PISILINUX/PisiLinux_docker/main/office/libreoffice/libreoffice/pspec.xml";
        // parse_xml_spec now sanitizes inter-element entity references
        match parse_xml_spec(path) {
            Ok(spec) => {
                assert_eq!(spec.source.name, "libreoffice");
                assert!(!spec.packages.is_empty());
                assert_eq!(spec.packages[0].name, "libreoffice");
                println!("LibreOffice pspec.xml parsed successfully: {} packages, {} history entries",
                    spec.packages.len(),
                    spec.history.as_ref().map(|h| h.updates.len()).unwrap_or(0));
                for pkg in &spec.packages {
                    if pkg.files.paths.is_empty() {
                        println!("Package Empty: {}", pkg.name);
                    } else {
                        println!("Package: {} -> {} paths (first: {})", pkg.name, pkg.files.paths.len(), pkg.files.paths[0].path);
                    }
                }
            }
            Err(e) => panic!("parse_xml_spec failed: {}", e),
        }
    }

    #[test]
    fn test_inter_element_entity_stripping() {
        let xml = r#"<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
    <Package>
        <Name>test</Name>
        <Files>
            <Path fileType="data">/a</Path>&gt;<Path fileType="data">/b</Path>
        </Files>
    </Package>
</PISI>"#;
        // raw fails
        assert!(quick_xml::de::from_str::<models::PisiSpec>(xml).is_err());
        // sanitized works
        let sanitized = sanitize_xml_for_serde(xml);
        assert!(quick_xml::de::from_str::<models::PisiSpec>(&sanitized).is_ok());
    }

    #[test]
    fn test_strip_doctype_and_decl() {
        let xml = r#"<?xml version="1.0" ?>
<!DOCTYPE PISI SYSTEM "https://example.dtd">
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#;
        // Simulate full cleanup
        let cleaned = sanitize_xml_for_serde(xml);
        assert!(!cleaned.contains("<!DOCTYPE"));
        assert!(!cleaned.contains("<?xml"));
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(20)]);

        // Verify it parses
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?}", spec.err());
    }

    #[test]
    fn test_doctype_on_same_line_as_decl() {
        let xml = r#"<?xml version="1.0"?> <!DOCTYPE PISI SYSTEM "pisi.dtd">
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#;
        let cleaned = sanitize_xml_for_serde(xml);
        assert!(!cleaned.contains("<!DOCTYPE"),
            "DOCTYPE should be stripped but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_doctype_only_on_same_line() {
        let xml = r#"<!DOCTYPE PISI SYSTEM "pisi.dtd">
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#;
        let cleaned = sanitize_xml_for_serde(xml);
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_comment_between_doctype_and_root() {
        let xml = r#"<!DOCTYPE PISI SYSTEM "pisi.dtd">
<!-- comment between doctype and root -->
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#;
        let cleaned = sanitize_xml_for_serde(xml);
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_bom_with_doctype_and_comment() {
        let xml = format!("\u{FEFF}{}", r#"<?xml version="1.0"?>
<!DOCTYPE PISI SYSTEM "pisi.dtd">
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#);
        let cleaned = sanitize_xml_for_serde(&xml);
        assert!(!cleaned.contains("<!DOCTYPE"));
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_bom_and_doctype_same_line_with_comment() {
        let xml = format!("\u{FEFF}{}", r#"<?xml version="1.0"?> <!DOCTYPE PISI SYSTEM "pisi.dtd">
<!-- eh -->
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#);
        let cleaned = sanitize_xml_for_serde(&xml);
        assert!(!cleaned.contains("<!DOCTYPE"),
            "DOCTYPE should be stripped but got: {:?}", &cleaned[..cleaned.len().min(80)]);
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_pi_before_root() {
        let xml = r#"<?somepi data="value"?>
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#;
        let cleaned = sanitize_xml_for_serde(xml);
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_multiple_comments_before_root() {
        let xml = r#"<!-- first comment -->
<!-- second comment -->
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#;
        let cleaned = sanitize_xml_for_serde(xml);
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(60)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_all_prolog_constructs_combined() {
        let xml = format!("\u{FEFF}{}", r#"<?xml version="1.0"?>
<!DOCTYPE PISI SYSTEM "pisi.dtd">
<!-- important note -->
<?somepi?>
<PISI>
    <Source>
        <Name>test</Name>
        <Version>1.0</Version>
        <Summary>test</Summary>
        <Description>test</Description>
        <License>MIT</License>
    </Source>
</PISI>"#);
        let cleaned = sanitize_xml_for_serde(&xml);
        assert!(!cleaned.contains("<!DOCTYPE"));
        assert!(!cleaned.contains("<?xml"));
        assert!(!cleaned.contains("<!--"));
        assert!(cleaned.starts_with("<PISI>"), "Expected <PISI> but got: {:?}", &cleaned[..cleaned.len().min(80)]);
        let spec: Result<PisiSpec, _> = quick_xml::de::from_str(&cleaned);
        assert!(spec.is_ok(), "Parse failed: {:?} [preview: {:?}]", spec.err(), &cleaned[..cleaned.len().min(200)]);
    }

    #[test]
    fn test_new_fields_parsing() {
        let xml = r#"
        <PISI>
            <Source>
                <Name>test-pkg</Name>
                <Homepage>https://example.com</Homepage>
                <Icon>test-icon</Icon>
                <ScreenShot>https://example.com/ss.png</ScreenShot>
                <IsA>app:gui</IsA>
                <IsA>library</IsA>
                <License>GPLv3</License>
                <Archive type="targz">http://test.com/test.tar.gz</Archive>
            </Source>
            <Package>
                <Name>test-pkg</Name>
                <Files>
                    <Path>/usr/bin</Path>
                </Files>
            </Package>
            <History>
                <Update release="1">
                    <Date>2026-05-11</Date>
                    <Version>1.0</Version>
                    <Comment>Init</Comment>
                    <Name>Tester</Name>
                </Update>
            </History>
        </PISI>
        "#;

        let spec: PisiSpec = quick_xml::de::from_str(xml).unwrap();

        assert_eq!(spec.source.icon, Some("test-icon".to_string()));
        assert_eq!(
            spec.source.screenshot,
            Some("https://example.com/ss.png".to_string())
        );
        assert_eq!(
            spec.source.provides,
            vec!["app:gui".to_string(), "library".to_string()]
        );
    }
}
