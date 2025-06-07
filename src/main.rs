mod cli;
mod info;
mod result;
mod walk;

use cli::Args;
use result::{Changed, CreateNewResult, Diff, GetLastResult, NewResult, NoChange, WriteToFile};

use clap::Parser;

fn main() {
    let args: Args = Args::parse();
    match args.target_directory.get_last_result() {
        Some(last_result) => {
            let new_result: NewResult = if args.full {
                args.target_directory.create_new_result(Some(&last_result))
            } else {
                args.target_directory.create_new_result(None)
            };
            match last_result.diff(&new_result) {
                Changed(changed_parts) => {
                    new_result.write_to_file(&args.target_directory);
                    println!("{}", changed_parts)
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
