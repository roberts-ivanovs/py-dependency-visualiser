use std::fs;

use structopt::StructOpt;
use walkdir::WalkDir;

mod cli;

fn main() {
    let matches = cli::Opt::from_args();
    for entry in WalkDir::new(matches.config).into_iter().filter_map(|e| e.ok()) {
        println!("{}", entry.path().display());
    }

    println!("Hello, world!");
}
