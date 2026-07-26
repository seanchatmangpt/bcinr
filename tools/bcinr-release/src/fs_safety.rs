use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct EvidenceLock {
    path: PathBuf,
}

impl EvidenceLock {
    pub fn acquire(
        output_directory: &Path,
        head_sha: &str,
        started_unix_ms: u128,
    ) -> io::Result<Self> {
        let path = output_directory.join("admission.lock");
        let mut file = create_new_evidence_file(&path)?;
        writeln!(file, "pid={}", std::process::id())?;
        writeln!(file, "head={head_sha}")?;
        writeln!(file, "started_unix_ms={started_unix_ms}")?;
        file.sync_all()?;
        Ok(Self { path })
    }
}

impl Drop for EvidenceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn prepare_output_directory(repository_root: &Path, relative: &Path) -> io::Result<PathBuf> {
    ensure_no_symlink_components(repository_root, relative, true)?;
    let output = repository_root.join(relative);
    fs::create_dir_all(&output)?;
    set_private_directory_permissions(&output)?;

    let canonical_root = repository_root.canonicalize()?;
    let canonical_output = output.canonicalize()?;
    if !canonical_output.starts_with(&canonical_root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "evidence directory escaped repository root",
        ));
    }

    let mut entries = fs::read_dir(&canonical_output)?;
    if entries.next().transpose()?.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "evidence directory must be empty",
        ));
    }
    Ok(canonical_output)
}

pub fn resolve_existing_file(repository_root: &Path, relative: &Path) -> io::Result<PathBuf> {
    ensure_no_symlink_components(repository_root, relative, false)?;
    let path = repository_root.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a regular non-symlink file", path.display()),
        ));
    }
    ensure_canonical_within_root(repository_root, &path)
}

pub fn resolve_existing_directory(repository_root: &Path, relative: &Path) -> io::Result<PathBuf> {
    ensure_no_symlink_components(repository_root, relative, false)?;
    let path = repository_root.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a non-symlink directory", path.display()),
        ));
    }
    ensure_canonical_within_root(repository_root, &path)
}

pub fn create_new_evidence_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic-write path has no parent",
        )
    })?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic-write path has no file name",
        )
    })?;
    let temporary = parent.join(format!(
        ".{}.{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id(),
        sequence
    ));

    let result = (|| {
        let mut file = create_new_evidence_file(&temporary)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        sync_directory(parent)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn ensure_no_symlink_components(
    repository_root: &Path,
    relative: &Path,
    allow_missing_tail: bool,
) -> io::Result<()> {
    let mut current = repository_root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("symlink component refused: {}", current.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound && allow_missing_tail => {
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn ensure_canonical_within_root(repository_root: &Path, path: &Path) -> io::Result<PathBuf> {
    let canonical_root = repository_root.canonicalize()?;
    let canonical_path = path.canonicalize()?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{} escaped repository root", path.display()),
        ));
    }
    Ok(canonical_path)
}

fn sync_directory(path: &Path) -> io::Result<()> {
    match File::open(path).and_then(|directory| directory.sync_all()) {
        Ok(()) => Ok(()),
        Err(error) if cfg!(windows) && error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}

fn set_private_directory_permissions(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_root(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "bcinr-release-{name}-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let created = fs::create_dir_all(&path);
        assert!(created.is_ok());
        path
    }

    #[test]
    fn atomic_write_replaces_no_partial_file() {
        let root = temporary_root("atomic");
        let path = root.join("receipt.json");
        assert!(atomic_write(&path, b"receipt").is_ok());
        assert_eq!(fs::read(&path).ok().as_deref(), Some(b"receipt".as_slice()));
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlink_components() {
        use std::os::unix::fs::symlink;

        let root = temporary_root("symlink");
        let outside = temporary_root("outside");
        assert!(symlink(&outside, root.join("target")).is_ok());
        let result = prepare_output_directory(&root, Path::new("target/evidence"));
        assert!(result.is_err());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
    }
}
