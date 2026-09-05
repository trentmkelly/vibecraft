//! Java LevelStorageAccess deletion: no-follow traversal and lock-aware retries.
use super::*;
use std::io;
use std::time::Duration;

impl LevelStorageAccess {
    /// Delete a world while retaining session ownership until its contents are
    /// gone. Failure on a child leaves the lock held; failure removing the root
    /// occurs after lock release, matching Java's root postvisit callback.
    pub fn delete_level(&mut self) -> io::Result<()> {
        self.delete_with(remove_if_exists, std::thread::sleep)
    }

    fn delete_with(
        &mut self,
        mut remove: impl FnMut(&Path, bool) -> io::Result<()>,
        mut wait: impl FnMut(Duration),
    ) -> io::Result<()> {
        self.check_lock()?;
        for attempt in 1..=5 {
            let result = delete_tree(
                self.level_directory.path(),
                self.level_directory.path(),
                &self.level_directory.lock_file(),
                &mut self.lock,
                &mut remove,
            );
            match result {
                Ok(()) => return Ok(()),
                Err(error) if attempt == 5 => return Err(error),
                Err(error) => {
                    eprintln!(
                        "Failed to delete {}: {error}",
                        self.level_directory.path().display()
                    );
                    wait(Duration::from_millis(500));
                }
            }
        }
        unreachable!("the fifth deletion attempt always returns")
    }
}

fn delete_tree(
    path: &Path,
    root: &Path,
    lock_path: &Path,
    lock: &mut SessionLock,
    remove: &mut impl FnMut(&Path, bool) -> io::Result<()>,
) -> io::Result<()> {
    // Files.walkFileTree does not follow symbolic links. In particular, never
    // recurse through a link into another world's or an operator's directory.
    if fs::symlink_metadata(path)?.is_dir() {
        for entry in fs::read_dir(path)? {
            delete_tree(&entry?.path(), root, lock_path, lock, remove)?;
        }
        if path == root {
            // Java closes only in the root's postVisitDirectory callback, after
            // all descendants succeeded. Errors after this point leave it closed.
            lock.close()?;
            remove(lock_path, false)?;
        }
        remove(path, true)
    } else if path != lock_path {
        remove(path, false)
    } else {
        Ok(())
    }
}

fn remove_if_exists(path: &Path, directory: bool) -> io::Result<()> {
    let result = if directory {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    };
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests;
