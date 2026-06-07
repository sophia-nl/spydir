use std::string::ToString;

#[derive(Clone)]
pub struct CommonFile {
    pub md5: String,
    pub mtime: String,
    pub relpath: String,
}

#[derive(Clone)]
pub struct EmptyDir {
    pub relpath: String,
}

impl ToString for CommonFile {
    fn to_string(&self) -> String {
        format!("{} {} {}", self.mtime, self.md5, self.relpath)
    }
}

impl ToString for EmptyDir {
    fn to_string(&self) -> String {
        format!(
            "               empty_directory                {}",
            self.relpath
        )
    }
}

pub trait ToCommonFile {
    fn to_common_file(&self) -> CommonFile;
}

impl ToCommonFile for String {
    fn to_common_file(&self) -> CommonFile {
        CommonFile {
            md5: self.get(13..45).unwrap().to_string(),
            mtime: self.get(..12).unwrap().to_string(),
            relpath: self.get(46..).unwrap().to_string(),
        }
    }
}

pub trait ToEmptyDir {
    fn to_empty_dir(&self) -> EmptyDir;
}

impl ToEmptyDir for String {
    fn to_empty_dir(&self) -> EmptyDir {
        EmptyDir {
            relpath: self.get(46..).unwrap().to_string(),
        }
    }
}
