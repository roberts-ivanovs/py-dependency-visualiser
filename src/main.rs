use std::fs;

use fs::FileType;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod cli;

fn main() {
    let matches = cli::Opt::from_args();
    for entry in WalkDir::new(matches.config)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_hidden(e))
        .filter(|e| is_python(e))
    {
        println!("{} ", entry.path().display());
    }

    println!("Hello, world!");
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
fn is_python(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with("py"))
        .unwrap_or(false)
}
