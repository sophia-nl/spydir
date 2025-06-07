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
    let mut path: PathBuf = PathBuf::from(input);
    if path.is_dir() {
        if input.ends_with('/') {
            let mut input_string: String = input.to_owned();
            input_string.pop();
            path = PathBuf::from(input_string);
        }
        Ok(path)
    } else {
        Err(format!("'{}' was not a valid directory path.", input))
    }
}
