use crate::item::{CommonFile, EmptyDir};
use crate::result::last::LastResult;
use crate::result::new::NewResult;

use std::fmt::{Display, Formatter, Result};
use std::ops::Not;

struct Items {
    common_files: Vec<CommonFile>,
    empty_dirs: Vec<EmptyDir>,
}

pub struct ChangedPart {
    added: Items,
    removed: Items,
}

pub enum DiffResult {
    Changed(ChangedPart),
    NoChange,
}

impl LastResult {
    pub fn diff(&self, new_result: &NewResult) -> DiffResult {
        let (unchanged_empty_dirs, removed_empty_dirs): (Vec<EmptyDir>, Vec<EmptyDir>) = self
            .empty_dirs
            .clone()
            .into_iter()
            .partition(|last_result_empty_dir: &EmptyDir| {
                new_result
                    .empty_dirs
                    .iter()
                    .any(|new_result_empty_dir: &EmptyDir| {
                        new_result_empty_dir.relpath == last_result_empty_dir.relpath
                    })
            });
        let added_empty_dirs: Vec<EmptyDir> = new_result
            .empty_dirs
            .clone()
            .into_iter()
            .filter(|new_result_empty_dir: &EmptyDir| {
                unchanged_empty_dirs
                    .iter()
                    .any(|unchanged_empty_dir: &EmptyDir| {
                        unchanged_empty_dir.relpath == new_result_empty_dir.relpath
                    })
                    .not()
            })
            .collect();
        let (unchanged_common_files, removed_common_files): (Vec<CommonFile>, Vec<CommonFile>) =
            self.common_files.clone().into_iter().partition(
                |last_result_common_file: &CommonFile| {
                    new_result
                        .common_files
                        .iter()
                        .any(|new_result_common_file: &CommonFile| {
                            new_result_common_file.relpath == last_result_common_file.relpath
                                && new_result_common_file.md5 == last_result_common_file.md5
                        })
                },
            );
        let added_common_files: Vec<CommonFile> = new_result
            .common_files
            .clone()
            .into_iter()
            .filter(|new_result_common_file: &CommonFile| {
                unchanged_common_files
                    .iter()
                    .any(|unchanged_common_file: &CommonFile| {
                        unchanged_common_file.relpath == new_result_common_file.relpath
                            && unchanged_common_file.md5 == new_result_common_file.md5
                    })
                    .not()
            })
            .collect();
        if added_common_files.is_empty()
            && added_empty_dirs.is_empty()
            && removed_common_files.is_empty()
            && removed_empty_dirs.is_empty()
        {
            DiffResult::NoChange
        } else {
            DiffResult::Changed(ChangedPart {
                added: Items {
                    common_files: added_common_files,
                    empty_dirs: added_empty_dirs,
                },
                removed: Items {
                    common_files: removed_common_files,
                    empty_dirs: removed_empty_dirs,
                },
            })
        }
    }
}

impl Display for ChangedPart {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let mut content: String = String::new();
        let is_added_empty: bool =
            self.added.common_files.is_empty() && self.added.empty_dirs.is_empty();
        let is_removed_empty: bool =
            self.removed.common_files.is_empty() && self.removed.empty_dirs.is_empty();
        if is_added_empty.not() {
            content.push_str("\x1B[92;1mNewly Added:\x1B[0m\n");
        }
        self.added
            .empty_dirs
            .iter()
            .for_each(|added_empty_dir: &EmptyDir| {
                content.push_str(&format!("{}\n", added_empty_dir.to_string()))
            });
        self.added
            .common_files
            .iter()
            .for_each(|added_common_file: &CommonFile| {
                content.push_str(&format!("{}\n", added_common_file.to_string()))
            });
        if is_added_empty.not() && is_removed_empty.not() {
            content.push('\n');
        }
        if is_removed_empty.not() {
            content.push_str("\x1B[96;1mRemoved:\x1B[0m\n");
        }
        self.removed
            .empty_dirs
            .iter()
            .for_each(|removed_empty_dir: &EmptyDir| {
                content.push_str(&format!("{}\n", removed_empty_dir.to_string()))
            });
        self.removed
            .common_files
            .iter()
            .for_each(|removed_common_file: &CommonFile| {
                content.push_str(&format!("{}\n", removed_common_file.to_string()))
            });
        write!(formatter, "{content}")
    }
}
