use crate::item::{CommonFile, EmptyDir, ToCommonFile, ToEmptyDir};

use std::fs::{DirEntry, File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct LastResult {
    pub common_files: Vec<CommonFile>,
    pub empty_dirs: Vec<EmptyDir>,
}

pub trait GetLastResult {
    fn get_last_result(&self) -> Option<LastResult>;
}

impl GetLastResult for PathBuf {
    fn get_last_result(&self) -> Option<LastResult> {
        let result_dir: PathBuf = self.join(".spydir");
        result_dir.read_dir().ok().and_then(|read_dir: ReadDir| {
            read_dir
                .flatten()
                .filter_map(|dir_entry: DirEntry| dir_entry.file_name().into_string().ok())
                .filter(|file_name: &String| file_name.len().eq(&16) && file_name.ends_with(".txt"))
                .filter_map(|file_name: String| file_name[..12].parse::<u64>().ok())
                .max()
                .and_then(|last_num: u64| {
                    File::open(result_dir.join(format!("{}.txt", last_num)))
                        .ok()
                        .and_then(|file: File| {
                            let mut common_files: Vec<CommonFile> = vec![];
                            let mut empty_dirs: Vec<EmptyDir> = vec![];
                            BufReader::new(file)
                                .lines()
                                .filter_map(Result::ok)
                                .for_each(|line: String| {
                                    if line.starts_with(
                                        "               empty_directory                ",
                                    ) {
                                        empty_dirs.push(line.to_empty_dir());
                                    } else {
                                        common_files.push(line.to_common_file());
                                    }
                                });
                            Some(LastResult {
                                common_files,
                                empty_dirs,
                            })
                        })
                })
        })
    }
}
