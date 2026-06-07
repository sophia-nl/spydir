use crate::item::{CommonFile, EmptyDir};
use crate::result::last::LastResult;
use crate::walk::Walk;

use std::fs::{self, DirEntry, File, Metadata};
use std::io::{self, Write};
use std::num::NonZero;
use std::ops::Not;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use digest_io::IoWrapper;
use md5::{Digest, Md5};
use threadpool::ThreadPool;

pub struct NewResult {
    pub common_files: Vec<CommonFile>,
    pub empty_dirs: Vec<EmptyDir>,
}

struct CommonFileIntermediate {
    mtime: String,
    path: PathBuf,
    relpath: String,
}

pub trait CreateNewResult {
    fn create_new_result(&self, maybe_last_result: Option<&LastResult>) -> NewResult;
}

impl CreateNewResult for PathBuf {
    fn create_new_result(&self, maybe_last_result: Option<&LastResult>) -> NewResult {
        let walk_result = self.walk();
        let mut common_files: Vec<CommonFile> = vec![];
        let mut empty_dirs: Vec<EmptyDir> = walk_result
            .empty_dirs
            .iter()
            .filter_map(|dir_entry: &DirEntry| {
                dir_entry
                    .path()
                    .to_string_lossy()
                    .get(self.to_string_lossy().len()..)
                    .and_then(|str: &str| {
                        Some(EmptyDir {
                            relpath: format!(".{}", str),
                        })
                    })
            })
            .collect();
        let mut common_file_intermediates: Vec<CommonFileIntermediate> = walk_result
            .common_files
            .iter()
            .filter_map(|dir_entry: &DirEntry| {
                dir_entry
                    .path()
                    .to_string_lossy()
                    .get(self.to_string_lossy().len()..)
                    .and_then(|str: &str| Some((dir_entry, format!(".{}", str))))
            })
            .filter_map(|(dir_entry, relpath)| {
                dir_entry.metadata().ok().and_then(|metadata: Metadata| {
                    metadata
                        .modified()
                        .ok()
                        .and_then(|modified_time: SystemTime| {
                            Some(CommonFileIntermediate {
                                mtime: DateTime::<Local>::from(modified_time)
                                    .format("%y%m%d%H%M%S")
                                    .to_string(),
                                path: dir_entry.path(),
                                relpath,
                            })
                        })
                })
            })
            .collect();
        if let Some(last_result) = maybe_last_result {
            common_file_intermediates.retain(
                |common_file_intermediate: &CommonFileIntermediate| {
                    last_result
                        .common_files
                        .iter()
                        .all(|last_result_common_file: &CommonFile| {
                            let is_md5_available: bool = last_result_common_file.relpath
                                == common_file_intermediate.relpath
                                && last_result_common_file.mtime == common_file_intermediate.mtime;
                            if is_md5_available {
                                common_files.push(CommonFile {
                                    md5: last_result_common_file.md5.to_owned(),
                                    mtime: common_file_intermediate.mtime.to_owned(),
                                    relpath: common_file_intermediate.relpath.to_owned(),
                                });
                            }
                            is_md5_available.not()
                        })
                },
            );
        }
        if common_file_intermediates.is_empty().not() {
            let total_num: usize = common_file_intermediates.len();
            let mut completed_num: usize = 0;
            print!("Calculating MD5...\t[{completed_num}/{total_num}]\r");
            io::stdout().flush().unwrap();
            let thread_pool: ThreadPool = ThreadPool::new(
                thread::available_parallelism()
                    .and_then(|parallelism: NonZero<usize>| Ok(parallelism.get()))
                    .unwrap_or(1),
            );
            let (tx, rx) = mpsc::channel();
            common_file_intermediates.into_iter().for_each(
                |common_file_intermediate: CommonFileIntermediate| {
                    let tx_clone: mpsc::Sender<CommonFile> = tx.clone();
                    thread_pool.execute(move || {
                        tx_clone
                            .send({
                                let mut hasher: IoWrapper<Md5> = IoWrapper(Md5::new());
                                io::copy(
                                    &mut File::open(common_file_intermediate.path).unwrap(),
                                    &mut hasher,
                                )
                                .unwrap();
                                CommonFile {
                                    md5: hasher
                                        .0
                                        .finalize()
                                        .iter()
                                        .map(|byte: &u8| format!("{:02x}", byte))
                                        .collect::<String>(),
                                    mtime: common_file_intermediate.mtime,
                                    relpath: common_file_intermediate.relpath,
                                }
                            })
                            .unwrap();
                    });
                },
            );
            drop(tx);
            rx.iter().for_each(|common_file: CommonFile| {
                common_files.push(common_file);
                completed_num += 1;
                print!("Calculating MD5...\t[{completed_num}/{total_num}]\r");
                io::stdout().flush().unwrap();
            });
            println!("\n");
        }
        common_files.sort_by(|a: &CommonFile, b: &CommonFile| a.relpath.cmp(&b.relpath));
        empty_dirs.sort_by(|a: &EmptyDir, b: &EmptyDir| a.relpath.cmp(&b.relpath));
        NewResult {
            common_files,
            empty_dirs,
        }
    }
}

impl NewResult {
    pub fn write_to_file(&self, path: &PathBuf) {
        let result_dir: PathBuf = path.join(".spydir");
        if result_dir.exists().not() {
            fs::create_dir(&result_dir).unwrap();
        }
        let mut text: String = String::new();
        self.empty_dirs.iter().for_each(|empty_dir: &EmptyDir| {
            text.push_str(format!("{}\n", empty_dir.to_string().as_str()).as_str());
        });
        self.common_files
            .iter()
            .for_each(|common_file: &CommonFile| {
                text.push_str(format!("{}\n", common_file.to_string().as_str()).as_str());
            });
        let mut new_result_file: File =
            File::create(result_dir.join(format!("{}.txt", Local::now().format("%y%m%d%H%M%S"))))
                .unwrap();
        new_result_file.write_all(text.as_bytes()).unwrap();
    }
}
