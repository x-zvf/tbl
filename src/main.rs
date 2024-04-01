use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub mod arguments;
pub mod output;
pub mod process;

fn main() {
    let args = arguments::Args::parse();
    let input: Box<dyn BufRead> = match args.file {
        Some(ref f) => Box::new(BufReader::new(File::open(f).expect("Failed to open file"))),
        None => Box::new(BufReader::new(io::stdin())),
    };
    let columns = process::process(&args, input);
    output::display(args, &columns);
}
