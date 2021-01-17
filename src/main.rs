use std::{collections::HashMap, fs};

#[macro_use]
extern crate lazy_static;

use fs::FileType;
use regex::Regex;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod cli;

lazy_static! {
    static ref SINGLE_IMPORT: Regex = Regex::new("").unwrap();
}

fn main() {
    let matches = cli::Opt::from_args();
    let name_filter = matches.name_filter;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    // TODO: Fix paring of imports like:
    //  ```
    // from xsd.asd import (
    //                      first,
    //                      second,
    //                    )
    let imports_per_dir = WalkDir::new(matches.config.clone())
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
                    .filter_map(|e| extract_import(&e))
                    .map(|e| {
                        e.iter()
                            .map(|e| remove_comments(&e))
                            .filter(|e| e.starts_with(&name_filter))
                            .collect::<Vec<String>>()
                    })
                    .filter(|e| e.len() > 0)
                    .collect::<Vec<Vec<String>>>(),
            )
        })
        .collect::<Vec<(DirEntry, Vec<Vec<String>>)>>();
    for (entry, content) in imports_per_dir {
        // TODO insert into the hashmap
        println!("\n{:?} \n{:#?}", &entry, &content);
        // map.insert(entry.path().to_str().unwrap().to_owned(), content);
    }

    // TODO Strip common path for shorter filenames
    // let base_path = matches.config.clone();
    // let base_path = base_path.to_str().unwrap();
    // let realt_path: Vec<&str> = entry.path().to_str().unwrap().split(base_path).collect();
    // let realt_path = realt_path.get(1).unwrap();
}

fn remove_comments(line: &str) -> String {
    line.split("#").nth(0).unwrap().to_string()
}

fn extract_import(line: &str) -> Option<Vec<String>> {
    match line {
        l if l.starts_with("from ") && l.contains(" import ") => {
            let first: Vec<&str> = l.split("from ").nth(1).unwrap().split(" import ").collect();
            let base = first.first().unwrap();
            let first = first
                .iter()
                .skip(1)
                .map(|e| e.split(", ").collect::<Vec<&str>>())
                .fold(vec![], |mut acc, item| {
                    acc.extend(item);
                    acc
                })
                .iter()
                .map(|e| base.to_string() + "." + e)
                .collect::<Vec<String>>();
            Some(first)
        }
        l if l.starts_with("import ") => Some(vec![l.split("import ").nth(1).unwrap().to_string()]),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_import_single() {
        let line = "import zeep.xsd";
        let res = extract_import(line).unwrap();
        let first = res.get(0).unwrap();
        assert_eq!(1, res.len());
        assert_eq!("zeep.xsd", first);
    }

    #[test]
    fn test_extract_import_multiple() {
        let line = "from zeep.xsd import asd, dasd";
        let res = extract_import(line).unwrap();
        let first = res.get(0).unwrap();
        let second = res.get(1).unwrap();
        assert_eq!(2, res.len());
        assert_eq!("zeep.xsd.asd", first);
        assert_eq!("zeep.xsd.dasd", second);
    }
}
