use std::ffi::OsString;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::process;

pub struct WalkResult {
    pub common_files: Vec<DirEntry>,
    pub empty_dirs: Vec<DirEntry>,
}

trait Merge {
    fn merge(&mut self, other: &mut Self);
}

impl Merge for WalkResult {
    fn merge(&mut self, other: &mut WalkResult) {
        self.common_files.append(&mut other.common_files);
        self.empty_dirs.append(&mut other.empty_dirs);
    }
}

trait Traverse: Iterator {
    fn traverse(self) -> WalkResult;
}

impl<T: Iterator<Item = DirEntry> + Sized> Traverse for T {
    fn traverse(self) -> WalkResult {
        let mut walk_result: WalkResult = WalkResult {
            common_files: vec![],
            empty_dirs: vec![],
        };
        self.for_each(|dir_entry: DirEntry| {
            if let Ok(file_type) = dir_entry.file_type() {
                if file_type.is_dir() && dir_entry.file_name() != OsString::from(".spydir") {
                    let path: PathBuf = dir_entry.path();
                    if let Ok(read_dir) = path.read_dir() {
                        let subdir_entries: Vec<DirEntry> = read_dir.flatten().collect();
                        match subdir_entries.len() {
                            0 => {
                                walk_result.empty_dirs.push(dir_entry);
                            }
                            1 => {
                                if path.join(".DS_Store").is_file() || path.join(".spydir").is_dir()
                                {
                                    walk_result.empty_dirs.push(dir_entry);
                                } else {
                                    walk_result.merge(&mut subdir_entries.into_iter().traverse());
                                }
                            }
                            2 => {
                                if path.join(".DS_Store").is_file() && path.join(".spydir").is_dir()
                                {
                                    walk_result.empty_dirs.push(dir_entry);
                                } else {
                                    walk_result.merge(&mut subdir_entries.into_iter().traverse());
                                }
                            }
                            _ => {
                                walk_result.merge(&mut subdir_entries.into_iter().traverse());
                            }
                        }
                    };
                } else if file_type.is_file()
                    && dir_entry.file_name() != OsString::from(".DS_Store")
                {
                    walk_result.common_files.push(dir_entry);
                }
            }
        });
        walk_result
    }
}

pub trait Walk {
    fn walk(&self) -> WalkResult;
}

impl Walk for PathBuf {
    fn walk(&self) -> WalkResult {
        self.read_dir()
            .unwrap_or_else(|_| {
                eprintln!(
                    "\x1B[91;1merror:\x1B[0m '{}' was unreadable.",
                    self.display()
                );
                process::exit(exitcode::IOERR)
            })
            .flatten()
            .traverse()
    }
}
