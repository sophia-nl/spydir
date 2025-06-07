use super::last::LastResult;
use super::new::NewResult;
use std::fmt::{Display, Formatter, Result};
use std::ops::Not;

pub struct ChangedParts {
    added: Vec<String>,
    deleted: Vec<String>,
}

pub enum DiffResult {
    Changed(ChangedParts),
    NoChange,
}

pub trait Diff {
    fn diff(&self, new_result: &NewResult) -> DiffResult;
}

impl Diff for LastResult {
    fn diff(&self, new_result: &NewResult) -> DiffResult {
        let mut last_inners: Vec<String> = vec![];
        self.common_files.iter().for_each(|common_file_info| {
            last_inners.push(common_file_info.to_string());
        });
        self.empty_dirs.iter().for_each(|empty_dir_info| {
            last_inners.push(empty_dir_info.to_string());
        });
        let mut new_inners: Vec<String> = vec![];
        new_result.common_files.iter().for_each(|common_file_info| {
            new_inners.push(common_file_info.to_string());
        });
        new_result.empty_dirs.iter().for_each(|empty_dir_info| {
            new_inners.push(empty_dir_info.to_string());
        });
        let mut added: Vec<String> = vec![];
        let mut deleted: Vec<String> = vec![];
        let mut same: Vec<String> = vec![];
        last_inners.iter().for_each(|last_inner: &String| {
            if new_inners.contains(last_inner) {
                same.push(last_inner.to_string());
            } else {
                deleted.push(last_inner.to_string());
            }
        });
        new_inners
            .iter()
            .filter(|new_inner: &&String| same.contains(new_inner).not())
            .for_each(|new_inner: &String| added.push(new_inner.to_string()));
        if added.is_empty() && deleted.is_empty() {
            DiffResult::NoChange
        } else {
            DiffResult::Changed(ChangedParts { added, deleted })
        }
    }
}

impl Display for ChangedParts {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let mut text: String = String::new();
        if self.added.is_empty().not() {
            text.push_str("\x1B[92;1mNewly Added:\x1B[0m\n");
        }
        self.added
            .iter()
            .for_each(|added_item: &String| text.push_str(&format!("{}\n", added_item)));
        if self.added.is_empty().not() && self.deleted.is_empty().not() {
            text.push('\n');
        }
        if self.deleted.is_empty().not() {
            text.push_str("\x1B[96;1mRemoved:\x1B[0m\n");
        }
        self.deleted
            .iter()
            .for_each(|deleted_item: &String| text.push_str(&format!("{}\n", deleted_item)));
        write!(formatter, "{text}")
    }
}
