use rust_i18n::t;
use sha1::Digest;
use std::fs;
use std::io::{self, Read, Write};

/// İndirilen kaynak arşivin doğruluğunu kontrol eder.
pub fn verify_archive(
    archive_path: &str,
    expected_hash: &str,
    algorithm: &str,
) -> Result<(), String> {
    let algo = if algorithm == "unknown" || algorithm.is_empty() {
        "sha1"
    } else {
        algorithm
    };

    println!(
        "{}",
        t!("api_verify_archive", algo = algo, path = archive_path)
    );

    let mut file = fs::File::open(archive_path)
        .map_err(|e| t!("api_err_archive_open", error = e).to_string())?;

    let calculated_hash = match algo.to_lowercase().as_str() {
        "sha1" => {
            let mut hasher = sha1::Sha1::new();
            io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
            format!("{:x}", hasher.finalize())
        }
        "sha256" => {
            let mut hasher = sha2::Sha256::new();
            io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
            format!("{:x}", hasher.finalize())
        }
        "md5" => {
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(|e| e.to_string())?;
            format!("{:x}", md5::compute(&content))
        }
        _ => return Err(t!("api_err_hash_algo", algo = algo).to_string()),
    };

    if expected_hash.is_empty() || algorithm == "unknown" {
        println!("{}: {}", t!("api_archive_hash_calculated"), calculated_hash);
        return Ok(());
    }

    if calculated_hash == expected_hash {
        println!("{}", t!("api_archive_success"));
        Ok(())
    } else {
        Err(t!(
            "api_err_hash_mismatch",
            path = archive_path,
            expected = expected_hash,
            calculated = calculated_hash
        )
        .to_string())
    }
}

/// Verilen arşiv dosyasını (tar.gz, tar.xz, zip) mevcut dizine açar.
pub fn unpack_archive(archive_path: &str) -> Result<(), String> {
    println!("{}", t!("api_unpack", path = archive_path));

    let path = std::path::Path::new(archive_path);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // Sıkıştırılmış patch/diff dosyası mı? (.patch.gz, .diff.xz vb.)
    let is_compressed_patch = stem.ends_with(".patch") || stem.ends_with(".diff");

    if is_compressed_patch {
        let decompress_cmd: &[&str] = match ext {
            "xz" => &["unxz", "-f", archive_path],
            "bz2" => &["bunzip2", "-f", archive_path],
            "zst" => &["unzstd", "-f", archive_path],
            _ => &["gunzip", "-f", archive_path],
        };
        super::core::run_command(decompress_cmd[0], &decompress_cmd[1..])
    } else {
        match ext {
            "gz" | "tgz" => super::core::run_command("tar", &["-xvf", archive_path]),
            "xz" => super::core::run_command("tar", &["-xJvf", archive_path]),
            "zip" => super::core::run_command("unzip", &[archive_path]),
            _ => super::core::run_command("tar", &["-xvf", archive_path]),
        }
    }
}

/// Sıkıştırılmış patch dosyasını açar ve geçici dosyanın yolunu döndürür.
/// Sıkıştırılmamışsa orijinal yolu döndürür. Arayan, dönen `Some` path'i silmelidir.
fn detect_compression_from_header(data: &[u8]) -> Option<&'static str> {
    if data.len() < 4 {
        return None;
    }
    // xz: fd 37 7a 58 5a 00
    if data.starts_with(&[0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00]) {
        return Some("xz");
    }
    // gz: 1f 8b
    if data.starts_with(&[0x1f, 0x8b]) {
        return Some("gz");
    }
    // bz2: 42 5a 68
    if data.starts_with(&[0x42, 0x5a, 0x68]) {
        return Some("bz2");
    }
    // zst: 28 b5 2f fd
    if data.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
        return Some("zst");
    }
    None
}

pub fn decompress_patch(patch_file: &str) -> Result<(String, bool), String> {
    use std::path::Path;
    let path = Path::new(patch_file);
    let ext = path.extension().and_then(|e| e.to_str());

    // Tüm dosyayı belleğe oku
    let raw = fs::read(patch_file)
        .map_err(|e| format!("Cannot open {}: {}", patch_file, e))?;
    if raw.is_empty() {
        return Ok((patch_file.to_string(), false));
    }

    // Önce magic bytes, sonra uzantı ile format tespiti
    let detected_ext = detect_compression_from_header(&raw).or_else(|| ext.and_then(|e| {
        match e {
            "xz" | "gz" | "bz2" | "zst" => Some(e),
            _ => None,
        }
    }));

    let decompressed: Vec<u8> = match detected_ext {
        Some("xz") => {
            let mut d = xz2::read::XzDecoder::new(raw.as_slice());
            let mut buf = Vec::new();
            d.read_to_end(&mut buf).map_err(|e| format!("xz decompress {}: {}", patch_file, e))?;
            buf
        }
        Some("gz") => {
            let mut d = flate2::read::GzDecoder::new(raw.as_slice());
            let mut buf = Vec::new();
            d.read_to_end(&mut buf).map_err(|e| format!("gz decompress {}: {}", patch_file, e))?;
            buf
        }
        Some("bz2") => {
            let mut d = bzip2::read::BzDecoder::new(raw.as_slice());
            let mut buf = Vec::new();
            d.read_to_end(&mut buf).map_err(|e| format!("bz2 decompress {}: {}", patch_file, e))?;
            buf
        }
        Some("zst") => {
            let mut d = zstd::Decoder::new(raw.as_slice()).map_err(|e| format!("zstd init {}: {}", patch_file, e))?;
            let mut buf = Vec::new();
            d.read_to_end(&mut buf).map_err(|e| format!("zstd decompress {}: {}", patch_file, e))?;
            buf
        }
        _ => return Ok((patch_file.to_string(), false)),
    };

    let tmp = format!("{}.decompressed", patch_file);
    let mut out = fs::File::create(&tmp).map_err(|e| format!("Cannot create {}: {}", tmp, e))?;
    out.write_all(&decompressed).map_err(|e| format!("Cannot write {}: {}", tmp, e))?;
    Ok((tmp, true))
}

/// Kaynak dosya içindeki bir yamayı (patch) uygular.
pub fn do_patch(patch_file: &str, strip_level: u8) -> Result<(), String> {
    let strip = format!("-p{}", strip_level);
    let (patch_path, is_temp) = decompress_patch(patch_file)?;
    let result = super::core::run_command(
        "patch",
        &["--remove-empty-files", "--no-backup-if-mismatch", "-i", &patch_path, &strip],
    );
    if is_temp {
        let _ = std::fs::remove_file(&patch_path);
    }
    result
}
