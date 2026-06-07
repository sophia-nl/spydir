mod cli;
mod item;
mod result;
mod walk;

use cli::Args;
use result::diff::DiffResult::{Changed, NoChange};
use result::last::GetLastResult;
use result::new::{CreateNewResult, NewResult};

use clap::Parser;

fn main() {
    let args: Args = Args::parse();
    match args.target_directory.get_last_result() {
        Some(last_result) => {
            let new_result: NewResult = if args.full {
                args.target_directory.create_new_result(None)
            } else {
                args.target_directory.create_new_result(Some(&last_result))
            };
            match last_result.diff(&new_result) {
                Changed(changed_part) => {
                    new_result.write_to_file(&args.target_directory);
                    println!("{}", changed_part)
                }
                NoChange => println!("\x1B[1mNo change.\x1B[0m"),
            }
        }
        None => {
            args.target_directory
                .create_new_result(None)
                .write_to_file(&args.target_directory);
            println!("\x1B[1mThe first check is done.\x1B[0m");
        }
    }
}
