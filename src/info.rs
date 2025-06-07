use std::string::ToString;

pub struct CommonFileInfo {
    pub md5: String,
    pub mtime: String,
    pub relpath: String,
}

pub struct EmptyDirInfo {
    pub relpath: String,
}

impl ToString for CommonFileInfo {
    fn to_string(&self) -> String {
        format!("{} {} {}", self.mtime, self.md5, self.relpath)
    }
}

impl ToString for EmptyDirInfo {
    fn to_string(&self) -> String {
        format!(
            "               empty_directory                {}",
            self.relpath
        )
    }
}
