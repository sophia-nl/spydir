use self::ClassifiedDirEntry::{CommonFile, EmptyDir};

use std::fs::DirEntry;
use std::path::PathBuf;
use std::process;

pub enum ClassifiedDirEntry {
    CommonFile(DirEntry),
    EmptyDir(DirEntry),
}

trait Classify {
    fn classify(self) -> Vec<ClassifiedDirEntry>;
}

impl Classify for Vec<DirEntry> {
    fn classify(self) -> Vec<ClassifiedDirEntry> {
        let mut classified_dir_entries: Vec<ClassifiedDirEntry> = vec![];
        self.into_iter().for_each(|dir_entry: DirEntry| {
            if let Some(file_name) = dir_entry.file_name().to_str() {
                if let Ok(file_type) = dir_entry.file_type() {
                    if file_type.is_dir() {
                        if ![".spydir"].contains(&file_name) {
                            if let Ok(read_dir) = dir_entry.path().read_dir() {
                                let dir_entries: Vec<DirEntry> = read_dir.flatten().collect();
                                if dir_entries.is_empty() {
                                    classified_dir_entries.push(EmptyDir(dir_entry));
                                } else {
                                    classified_dir_entries.append(&mut dir_entries.classify());
                                }
                            }
                        }
                    } else if file_type.is_file() {
                        if ![".DS_Store"].contains(&file_name) {
                            classified_dir_entries.push(CommonFile(dir_entry));
                        }
                    }
                }
            }
        });
        classified_dir_entries
    }
}

pub trait Walk {
    fn walk(&self) -> Vec<ClassifiedDirEntry>;
}

impl Walk for PathBuf {
    fn walk(&self) -> Vec<ClassifiedDirEntry> {
        self.read_dir()
            .unwrap_or_else(|_| {
                eprintln!(
                    "\x1B[91;1merror:\x1B[0m '{}' was unreadable.",
                    self.display()
                );
                process::exit(exitcode::IOERR)
            })
            .flatten()
            .collect::<Vec<DirEntry>>()
            .classify()
    }
}
