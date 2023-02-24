use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};
use std::os::unix::prelude::PermissionsExt;

pub struct FilePermissions;

impl Rule for FilePermissions {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        let mode = entry.meta.permissions().mode() & 0o777;
        let is_dir = entry.meta.is_dir();

        match (is_dir, mode) {
            (true, 0o755) | (false, 0o644) => None,
            (true, _) => Some(Diagnostic {
                message: format!(
                    "Directory has incorrect permissions; should be 755 (is {:o})",
                    mode
                ),
                path: entry.entry.path(),
                hints: vec![],
            }),
            (false, _) => Some(Diagnostic {
                message: format!(
                    "File has incorrect permissions; should be 644 (is {:o})",
                    mode
                ),
                path: entry.entry.path(),
                hints: vec![],
            }),
        }
    }
}
