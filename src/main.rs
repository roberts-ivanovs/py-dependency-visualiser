use std::{collections::HashMap, fs, io::Write};

#[macro_use]
extern crate lazy_static;

use fs::{File, FileType};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod cli;

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
                            .map(|e| sanity_cleanup(&e))
                            .filter(|e| e.starts_with(&name_filter))
                            .collect::<Vec<String>>()
                    })
                    .filter(|e| e.len() > 0)
                    .fold(vec![], |mut acc, it| {
                        acc.extend(it);
                        acc
                    }),
            )
        })
        .filter(|(_, content)| content.len() > 0)
        .collect::<Vec<(DirEntry, Vec<String>)>>();
    for (entry, content) in imports_per_dir {
        let base_path = matches.config.clone();
        let base_path = base_path.to_str().unwrap();
        let realt_path: Vec<&str> = entry.path().to_str().unwrap().split(base_path).collect();
        let realt_path = name_filter.clone().to_owned()
            + realt_path
                .get(1)
                .unwrap()
                .replace(".py", "")
                .replace("/", ".")
                .as_ref();
        println!("\n{:?} \n{:#?}", realt_path, &content);
        map.insert(realt_path, content);
    }
    write_mermaid(&map);
}

fn write_mermaid(map: &HashMap<String, Vec<String>>) {
    let mut file = File::create("output.md").unwrap();
    file.write(
        br###"
```mermaid
graph RL
"###,
    )
    .unwrap();
    for (key, val) in map.iter() {
        let items = val
            .iter()
            .map(|e| e.to_owned() + " --> " + key.clone().as_ref())
            .collect::<Vec<String>>();
        let items = items.join("\n") + "\n";
        file.write(items.as_bytes()).unwrap();
    }
    file.write(
        br###"
```
"###,
    )
    .unwrap();
}

fn sanity_cleanup(line: &str) -> String {
    line.split("#")
        .nth(0)
        .unwrap()
        .to_string()
        .split(" as")
        .nth(0)
        .unwrap()
        .to_string()
        .replace("class", "classs")
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
