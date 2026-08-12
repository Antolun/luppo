use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[cfg(target_os = "linux")]
pub fn lchown_path(path: &Path, uid: u32, gid: u32) -> std::io::Result<()> {
    use std::ffi::CString;
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "path contains null bytes"))?;
    let ret = unsafe { libc::lchown(c_path.as_ptr(), uid as libc::uid_t, gid as libc::gid_t) };
    if ret == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(not(target_os = "linux"))]
pub fn lchown_path(path: &Path, uid: u32, gid: u32) -> std::io::Result<()> {
    std::os::unix::fs::chown(path, Some(uid), Some(gid))
}
