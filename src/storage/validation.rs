//! DirectoryValidator validates raw link targets without walking through links.
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForbiddenSymlinkInfo {
    pub link: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentValidationError {
    pub directory: PathBuf,
    pub entries: Vec<ForbiddenSymlinkInfo>,
}

impl fmt::Display for ContentValidationError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "Failed to validate '{}'. Found forbidden symlinks: ",
            self.directory.display()
        )?;
        for (index, entry) in self.entries.iter().enumerate() {
            if index != 0 {
                write!(output, ", ")?;
            }
            write!(
                output,
                "{}->{}",
                entry.link.display(),
                entry.target.display()
            )?;
        }
        Ok(())
    }
}
impl std::error::Error for ContentValidationError {}

pub struct DirectoryValidator<F> {
    allows_target: F,
}

impl DirectoryValidator<fn(&Path) -> bool> {
    pub fn deny_all() -> Self {
        Self::new(|_| false)
    }
}

impl<F: Fn(&Path) -> bool> DirectoryValidator<F> {
    pub fn new(allows_target: F) -> Self {
        Self { allows_target }
    }

    pub fn validate_symlink(&self, path: &Path) -> io::Result<Vec<ForbiddenSymlinkInfo>> {
        let mut issues = Vec::new();
        self.check_link(path, &mut issues)?;
        Ok(issues)
    }

    fn check_link(&self, path: &Path, issues: &mut Vec<ForbiddenSymlinkInfo>) -> io::Result<()> {
        // Java passes readSymbolicLink's result directly to the matcher: do not
        // canonicalize it or resolve a relative target against the link's parent.
        let target = fs::read_link(path)?;
        if !(self.allows_target)(&target) {
            issues.push(ForbiddenSymlinkInfo {
                link: path.to_owned(),
                target,
            });
        }
        Ok(())
    }

    pub fn validate_directory(
        &self,
        directory: &Path,
        allow_top_symlink: bool,
    ) -> io::Result<Vec<ForbiddenSymlinkInfo>> {
        let metadata = match fs::symlink_metadata(directory) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
            Err(error) => return Err(error),
        };
        if metadata.is_file() {
            return Err(io::Error::other(format!(
                "Path {} is not a directory",
                directory.display()
            )));
        }
        let mut issues = Vec::new();
        if metadata.is_symlink() {
            if !allow_top_symlink {
                return self.validate_symlink(directory);
            }
            // This is deliberately the raw target, matching validateDirectory.
            self.validate_known_directory(&fs::read_link(directory)?, &mut issues)?;
        } else {
            self.validate_known_directory(directory, &mut issues)?;
        }
        Ok(issues)
    }

    pub fn validate_known_directory(
        &self,
        directory: &Path,
        issues: &mut Vec<ForbiddenSymlinkInfo>,
    ) -> io::Result<()> {
        let metadata = fs::symlink_metadata(directory)?;
        if metadata.is_symlink() {
            self.check_link(directory, issues)?;
        } else if metadata.is_dir() {
            for entry in fs::read_dir(directory)? {
                self.validate_known_directory(&entry?.path(), issues)?;
            }
        }
        Ok(())
    }

    pub fn require_valid_directory(
        &self,
        directory: &Path,
        allow_top_symlink: bool,
    ) -> io::Result<()> {
        let entries = self.validate_directory(directory, allow_top_symlink)?;
        if entries.is_empty() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                ContentValidationError {
                    directory: directory.to_owned(),
                    entries,
                },
            ))
        }
    }
}

#[cfg(test)]
mod tests;
