use std::{collections::HashMap, fs};

use fs::FileType;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod cli;

fn main() {
    let matches = cli::Opt::from_args();
    let mut map: HashMap<DirEntry, Vec<String>> = HashMap::new();
    for entry in WalkDir::new(matches.config.clone())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_hidden(e))
        .filter(|e| is_python(e))
        .map(|e| (e.clone(), fs::read_to_string(e.path()).unwrap()))
        .map(|(dir, content)| {
            (
                dir,
                content
                    .split('\n')
                    .filter_map(|e| extract_import(e))
                    .collect::<Vec<String>>(),
            )
        })
    {
        // TODO insert into the hashmap
        println!("{:?}", &entry);
    }

    // TODO Strip common path for shorter filenames
    // let base_path = matches.config.clone();
    // let base_path = base_path.to_str().unwrap();
    // let realt_path: Vec<&str> = entry.path().to_str().unwrap().split(base_path).collect();
    // let realt_path = realt_path.get(1).unwrap();
    println!("Hello, world!");
}

fn extract_import(line: &str) -> Option<String> {
    match line {
        l if l.contains("from ") && l.contains(" import ") => {
            let first: Vec<&str> = l.split("from ").nth(1).unwrap().split(" import ").collect();
            let first = first.join(".");
            Some(first)
        }
        l if l.contains("import ") => Some(l.split("import ").nth(1).unwrap().to_string()),
        _ => None,
    }
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
