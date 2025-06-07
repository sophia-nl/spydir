use super::last::LastResult;
use crate::info::{CommonFileInfo, EmptyDirInfo};
use crate::walk::ClassifiedDirEntry::{self, CommonFile, EmptyDir};
use crate::walk::Walk;

use std::fs::{self, DirEntry, File};
use std::io::{self, Error, Write};
use std::ops::Not;
use std::path::PathBuf;
use std::sync::mpsc;

use chrono::{DateTime, Local};
use md5::{Digest, Md5};
use threadpool::ThreadPool;

pub struct NewResult {
    pub common_files: Vec<CommonFileInfo>,
    pub empty_dirs: Vec<EmptyDirInfo>,
}

pub trait CreateNewResult {
    fn create_new_result(&self, last_result_option: Option<&LastResult>) -> NewResult;
}

impl CreateNewResult for PathBuf {
    fn create_new_result(&self, last_result_option: Option<&LastResult>) -> NewResult {
        let commonpath_len: usize = self.to_str().unwrap().len();
        let mut common_files: Vec<CommonFileInfo> = vec![];
        let mut empty_dirs: Vec<EmptyDirInfo> = vec![];
        let mut temp_files: Vec<DirEntry> = vec![];
        self.walk()
            .into_iter()
            .for_each(
                |classified_dir_entry: ClassifiedDirEntry| match classified_dir_entry {
                    CommonFile(common_file) => temp_files.push(common_file),
                    EmptyDir(empty_dir) => {
                        if let Some(full_path) = empty_dir.path().to_str() {
                            if let Some(sub_path) = full_path.get(commonpath_len..) {
                                empty_dirs.push(EmptyDirInfo {
                                    relpath: format!(".{}", sub_path),
                                });
                            }
                        }
                    }
                },
            );
        match last_result_option {
            Some(last_result) => {
                let total_num: usize = temp_files.len();
                let mut temporary_num: u64 = 0;
                print!("Calculating MD5...\t[{temporary_num}/{total_num}]\r");
                io::stdout().flush().unwrap();
                let thread_pool: ThreadPool = ThreadPool::new(num_cpus::get());
                let (tx, rx) = mpsc::channel();
                for entry in temp_files {
                    let tx_: mpsc::Sender<CommonFileInfo> = tx.clone();
                    thread_pool.execute(move || {
                        tx_.send(entry.generate(commonpath_len).unwrap()).unwrap();
                    });
                }
                drop(tx);
                for item in &rx {
                    common_files.push(item);
                    temporary_num += 1;
                    print!("Calculating MD5...\t[{temporary_num}/{total_num}]\r");
                    io::stdout().flush().unwrap();
                }
                println!("\n");
            }
            None => {
                let total_num: usize = temp_files.len();
                let mut temporary_num: u64 = 0;
                print!("Calculating MD5...\t[{temporary_num}/{total_num}]\r");
                io::stdout().flush().unwrap();
                let thread_pool: ThreadPool = ThreadPool::new(num_cpus::get());
                let (tx, rx) = mpsc::channel();
                for entry in temp_files {
                    let tx_: mpsc::Sender<CommonFileInfo> = tx.clone();
                    thread_pool.execute(move || {
                        tx_.send(entry.generate(commonpath_len).unwrap()).unwrap();
                    });
                }
                drop(tx);
                for item in &rx {
                    common_files.push(item);
                    temporary_num += 1;
                    print!("Calculating MD5...\t[{temporary_num}/{total_num}]\r");
                    io::stdout().flush().unwrap();
                }
                println!("\n");
            }
        }
        NewResult {
            common_files,
            empty_dirs,
        }
    }
}

trait Generate {
    fn generate(&self, commonpath_len: usize) -> Result<CommonFileInfo, Error>;
}

impl Generate for DirEntry {
    fn generate(&self, commonpath_len: usize) -> Result<CommonFileInfo, Error> {
        let last_modification_time: DateTime<Local> =
            self.metadata().unwrap().modified().unwrap().into();
        let mtime: String = last_modification_time.format("%y%m%d%H%M%S").to_string();
        let mut hasher: md5::digest::core_api::CoreWrapper<md5::Md5Core> = Md5::new();
        let file_path: PathBuf = self.path();
        io::copy(&mut File::open(&file_path)?, &mut hasher)?;
        Ok(CommonFileInfo {
            md5: format!("{:x}", hasher.finalize()),
            mtime,
            relpath: format!(
                ".{}",
                file_path.to_str().unwrap().get(commonpath_len..).unwrap()
            ),
        })
    }
}

pub trait WriteToFile {
    fn write_to_file(&self, path: &PathBuf);
}

impl WriteToFile for NewResult {
    fn write_to_file(&self, path: &PathBuf) {
        let result_dir: PathBuf = path.join(".spydir");
        if result_dir.exists().not() {
            fs::create_dir(&result_dir).unwrap();
        } else if result_dir.is_dir().not() {
            fs::remove_file(&result_dir).unwrap();
            fs::create_dir(&result_dir).unwrap();
        }
        let mut text: String = String::new();
        for common_file_info in &self.common_files {
            text.push_str(format!("{}\n", common_file_info.to_string()).as_str());
        }
        for empty_dir_info in &self.empty_dirs {
            text.push_str(format!("{}\n", empty_dir_info.to_string()).as_str());
        }
        let mut new_result_file: File =
            File::create(result_dir.join(format!("{}.txt", Local::now().format("%y%m%d%H%M%S"))))
                .unwrap();
        new_result_file.write_all(text.as_bytes()).unwrap();
    }
}
