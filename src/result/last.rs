use crate::info::{CommonFileInfo, EmptyDirInfo};

use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct LastResult {
    pub common_files: Vec<CommonFileInfo>,
    pub empty_dirs: Vec<EmptyDirInfo>,
}

pub trait GetLastResult {
    fn get_last_result(&self) -> Option<LastResult>;
}

impl GetLastResult for PathBuf {
    fn get_last_result(&self) -> Option<LastResult> {
        match self.join(".spydir").read_dir() {
            Ok(read_dir) => {
                let mut last_result: Option<LastResult> = None;
                let mut last_result_file: PathBuf = PathBuf::new();
                let mut temp_num: u64 = 0;
                read_dir.flatten().for_each(|dir_entry: DirEntry| {
                    if let Some(file_name) = dir_entry.file_name().to_str() {
                        if file_name.len().eq(&16) && file_name.ends_with(".txt") {
                            let file_name_num: &str = &file_name[..12];
                            if let Ok(num) = file_name_num.to_owned().parse::<u64>() {
                                if num > temp_num {
                                    temp_num = num;
                                    last_result_file = dir_entry.path();
                                }
                            }
                        }
                    }
                });
                if let Ok(file) = File::open(last_result_file) {
                    let mut common_files: Vec<CommonFileInfo> = vec![];
                    let mut empty_dirs: Vec<EmptyDirInfo> = vec![];
                    for line in BufReader::new(file).lines().map_while(Result::ok) {
                        if line.starts_with("               empty_directory                ") {
                            empty_dirs.push(EmptyDirInfo {
                                relpath: line.get(46..).unwrap().to_string(),
                            });
                        } else {
                            common_files.push(CommonFileInfo {
                                md5: line.get(13..45).unwrap().to_string(),
                                mtime: line.get(..12).unwrap().to_string(),
                                relpath: line.get(46..).unwrap().to_string(),
                            });
                        }
                    }
                    last_result = Some(LastResult {
                        common_files,
                        empty_dirs,
                    });
                };
                last_result
            }
            Err(_) => None,
        }
    }
}
