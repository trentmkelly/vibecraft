//! Java DirectoryLock file ownership and lifecycle.
use super::*;

#[derive(Debug)]
pub struct SessionLock {
    // `None` corresponds to a Java `DirectoryLock` after `close()`: the file
    // lock has been released and the channel is closed.
    file: Option<File>,
}

impl SessionLock {
    pub fn acquire(dir: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(dir)?;
        let lock_path = dir.join(SESSION_LOCK_FILE);
        let mut file = OpenOptions::new()
            .create(true)
            // Java opens with CREATE + WRITE, without TRUNCATE_EXISTING.
            .truncate(false)
            .write(true)
            .open(&lock_path)?;
        file.write_all(SESSION_LOCK_MARKER)?;
        file.sync_all()?;

        lock_file_exclusive_nonblocking(&file, &lock_path)?;
        Ok(Self { file: Some(file) })
    }

    /// Mirrors `DirectoryLock.isValid()`.
    ///
    /// Rust's advisory-lock API does not expose a separate validity query, so
    /// ownership of the still-open locked file is the equivalent state.
    pub fn is_valid(&self) -> bool {
        self.file.is_some()
    }

    /// Releases the lock and closes its file, matching `DirectoryLock.close()`.
    /// Calling `close` repeatedly is harmless, just as Java skips releasing an
    /// already-invalid lock and an already-closed channel.
    pub fn close(&mut self) -> std::io::Result<()> {
        let Some(file) = self.file.take() else {
            return Ok(());
        };

        unlock_file(&file)
    }

    pub fn is_locked(dir: &Path) -> std::io::Result<bool> {
        let lock_path = dir.join(SESSION_LOCK_FILE);
        let file = match OpenOptions::new().write(true).open(&lock_path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return Ok(true),
            Err(err) => return Err(err),
        };

        match try_lock_file_exclusive_nonblocking(&file) {
            Ok(()) => {
                unlock_file(&file)?;
                Ok(false)
            }
            Err(err) if is_would_block_lock_error(&err) => Ok(true),
            Err(err) => Err(err),
        }
    }
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = unlock_file(&file);
        }
    }
}
