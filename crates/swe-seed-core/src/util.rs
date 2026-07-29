//! Small shared helpers ported from the Python harness so route/trace output
//! stays format-compatible (timestamps, slugs, redaction).

use chrono::Utc;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Write a harness-managed file below `root` without following symlinked path
/// components. On Unix the directory descriptor remains authoritative through
/// the final atomic rename, closing pathname replacement races.
pub fn secure_write(
    root: &Path,
    components: &[&str],
    name: &str,
    bytes: &[u8],
    mode: u32,
) -> std::io::Result<PathBuf> {
    #[cfg(unix)]
    {
        secure_write_unix(root, components, name, bytes, mode)
    }
    #[cfg(not(unix))]
    {
        let dir = components
            .iter()
            .fold(root.to_path_buf(), |path, component| path.join(component));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(name);
        std::fs::write(&path, bytes)?;
        Ok(path)
    }
}

#[cfg(unix)]
fn secure_write_unix(
    root: &Path,
    components: &[&str],
    name: &str,
    bytes: &[u8],
    mode: u32,
) -> std::io::Result<PathBuf> {
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;

    struct DirFd(i32);
    impl Drop for DirFd {
        fn drop(&mut self) {
            unsafe { libc::close(self.0) };
        }
    }
    fn cstring(bytes: &[u8]) -> std::io::Result<CString> {
        CString::new(bytes).map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))
    }
    fn open_dir_at(fd: i32, name: &CString) -> std::io::Result<i32> {
        let opened = unsafe {
            libc::openat(
                fd,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if opened < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(opened)
        }
    }

    let root = root.canonicalize()?;
    let root_c = cstring(root.as_os_str().as_bytes())?;
    let opened = unsafe {
        libc::open(
            root_c.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if opened < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut dir = DirFd(opened);
    let mut path = root;
    for component in components {
        let component_c = cstring(component.as_bytes())?;
        let child = match open_dir_at(dir.0, &component_c) {
            Ok(fd) => fd,
            Err(error) if error.raw_os_error() == Some(libc::ENOENT) => {
                if unsafe { libc::mkdirat(dir.0, component_c.as_ptr(), 0o755) } != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                open_dir_at(dir.0, &component_c)?
            }
            Err(error) => return Err(error),
        };
        dir = DirFd(child);
        path.push(component);
    }
    let name_c = cstring(name.as_bytes())?;
    let mut stat = unsafe { std::mem::zeroed::<libc::stat>() };
    let stat_result =
        unsafe { libc::fstatat(dir.0, name_c.as_ptr(), &mut stat, libc::AT_SYMLINK_NOFOLLOW) };
    if stat_result == 0 {
        if stat.st_mode & libc::S_IFMT == libc::S_IFLNK {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "refusing to replace symlinked harness file",
            ));
        }
    } else if std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT) {
        return Err(std::io::Error::last_os_error());
    }
    let temporary = cstring(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()).as_bytes())?;
    let file_fd = unsafe {
        libc::openat(
            dir.0,
            temporary.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            mode as libc::mode_t,
        )
    };
    if file_fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_fd(file_fd) };
    let write_result = file.write_all(bytes).and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = write_result {
        unsafe { libc::unlinkat(dir.0, temporary.as_ptr(), 0) };
        return Err(error);
    }
    if unsafe { libc::renameat(dir.0, temporary.as_ptr(), dir.0, name_c.as_ptr()) } != 0 {
        unsafe { libc::unlinkat(dir.0, temporary.as_ptr(), 0) };
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::fsync(dir.0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(path.join(name))
}

/// `2026-06-17T00:23:16+00:00` — `datetime.now(utc).replace(microsecond=0).isoformat()`.
pub fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
}

/// `20260617T002316Z` — `strftime('%Y%m%dT%H%M%SZ')`, used in trace ids.
pub fn utc_stamp() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

/// Lowercase, non-[a-z0-9] runs → `-`, trimmed, capped at 48 chars, default `trace`.
pub fn slugify(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut out = String::new();
    let mut last = false;
    for ch in lower.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            last = false;
        } else if !last {
            out.push('-');
            last = true;
        }
    }
    let trimmed = out.trim_matches('-');
    let capped = if trimmed.len() > 48 {
        &trimmed[..48]
    } else {
        trimmed
    };
    if capped.is_empty() {
        "trace".to_string()
    } else {
        capped.to_string()
    }
}

/// Replace known secret patterns with `[REDACTED]` (mirrors harness.redact_secrets).
pub fn redact_secrets(text: &str) -> String {
    const PATTERNS: &[&str] = &[
        r"sk-[a-zA-Z0-9]{20,}",
        r"ghp_[a-zA-Z0-9]{36}",
        r"gho_[a-zA-Z0-9]{36}",
        r"xox[bpras]-[a-zA-Z0-9-]+",
        r"AKIA[0-9A-Z]{16}",
        r"-----BEGIN (?:RSA |EC |DSA )?PRIVATE KEY-----",
    ];
    let mut out = text.to_string();
    for pat in PATTERNS {
        let re = Regex::new(pat).expect("static regex");
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    out
}
