use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long, short)]
    pub full: bool,
    #[arg(value_parser = parse_path)]
    pub target_directory: PathBuf,
}

fn parse_path(input: &str) -> Result<PathBuf, String> {
    let path: PathBuf = PathBuf::from(input).components().collect();
    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!("'{}' was not a valid directory path.", input))
    }
}
