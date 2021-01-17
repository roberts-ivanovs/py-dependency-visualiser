use std::{collections::HashMap, fs, io::Write};

use fs::{File};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod cli;

fn main() {
    let matches = cli::Opt::from_args();
    let name_filter = matches.name_filter;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let imports_per_dir = WalkDir::new(matches.config.clone())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_hidden(e))
        .filter(|e| is_python(e))
        .map(|e| (e.clone(), fs::read_to_string(e.path()).unwrap()))
        .map(|(dir, content)| {
            (
                dir,
                extract_import(&content)
                    .iter()
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
        .replace("(", "")
        .replace(".*", "")
        .replace(")", "") // Init files point to the named module anyway
        .replace("class", "classs") // Necessary because mermaid does has `class` as keyword
}

#[derive(Debug, PartialEq)]
enum LineParsingState {
    READ,
    COCNAT,
}

fn extract_import(content: &str) -> Vec<Vec<String>> {
    let mut state = LineParsingState::READ;
    let mut concat_state = String::new();
    let mut import_lines = vec![];
    for line in content.split("\n") {
        match line {
            l if state == LineParsingState::READ => {
                match l {
                    l if l.starts_with("from ") && l.contains(" import ") => {
                        // Handle case `from xsd.asd import (ccc, aaa)`
                        if l.contains("(") && l.contains(")") {
                            let l = sanity_cleanup(l);
                            import_lines.push(l);
                        } else {
                            match l.contains("(") {
                                true => {
                                    state = LineParsingState::COCNAT;
                                    concat_state = sanity_cleanup(l);
                                }
                                false => import_lines.push(sanity_cleanup(l)),
                            }
                        }
                    }
                    l if l.starts_with("import ") => import_lines.push(l.to_owned()),
                    _ => (),
                }
            }
            l if l.contains(")") && (state == LineParsingState::COCNAT) => {
                state = LineParsingState::READ;
                concat_state += l;
                import_lines.push(concat_state.to_owned());
                concat_state = String::new();
            }
            l if (state == LineParsingState::COCNAT) => {
                concat_state += sanity_cleanup(l).as_ref();
            }
            _ => (),
        }
    }

    println!("{:#?}", concat_state);

    import_lines
        .iter()
        .filter_map(|e| extract_single_import(e))
        .collect::<Vec<Vec<String>>>()
}

fn extract_single_import(line: &str) -> Option<Vec<String>> {
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
                .map(|e| base.to_string() + "." + e.replace(" ", "").replace(",", "").as_ref())
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
        let res = extract_single_import(line).unwrap();
        let first = res.get(0).unwrap();
        assert_eq!(1, res.len());
        assert_eq!("zeep.xsd", first);
    }

    #[test]
    fn test_extract_import_multiple() {
        let line = "from zeep.xsd import asd, dasd";
        let res = extract_single_import(line).unwrap();
        let first = res.get(0).unwrap();
        let second = res.get(1).unwrap();
        assert_eq!(2, res.len());
        assert_eq!("zeep.xsd.asd", first);
        assert_eq!("zeep.xsd.dasd", second);
    }

    #[test]
    fn test_extract_import_state_machine_f_1() {
        let line = "from zeep.xsd import asd";
        let res = extract_import(line);
        assert_eq!(1, res.len());
        let res = res.get(0).unwrap();
        assert_eq!(1, res.len());
        let first = res.get(0).unwrap();
        assert_eq!("zeep.xsd.asd", first);
    }

    #[test]
    fn test_extract_import_state_machine_f_2() {
        let line = "import zeep.xsd";
        let res = extract_import(line);
        assert_eq!(1, res.len());
        let res = res.get(0).unwrap();
        assert_eq!(1, res.len());
        let first = res.get(0).unwrap();
        assert_eq!("zeep.xsd", first);
    }

    #[test]
    fn test_extract_import_state_machine_f_3() {
        let line = "from zeep.xsd import asd, dasd";
        let res = extract_import(line);
        assert_eq!(1, res.len());
        let res = res.get(0).unwrap();
        assert_eq!(2, res.len());
        let first = res.get(0).unwrap();
        assert_eq!("zeep.xsd.asd", first);
        let second = res.get(1).unwrap();
        assert_eq!("zeep.xsd.dasd", second);
    }

    #[test]
    fn test_extract_import_state_machine_f_4() {
        let line = "from zeep.xsd import (asd, dasd)";
        let res = extract_import(line);
        assert_eq!(1, res.len());
        let res = res.get(0).unwrap();
        assert_eq!(2, res.len());
        let first = res.get(0).unwrap();
        assert_eq!("zeep.xsd.asd", first);
        let second = res.get(1).unwrap();
        assert_eq!("zeep.xsd.dasd", second);
    }

    #[test]
    fn test_extract_import_state_machine_f_5() {
        let line = r###"
from zeep.xsd import (
    asd,
    dasd,
)"###;
        let res = extract_import(line);
        assert_eq!(1, res.len());
        let res = res.get(0).unwrap();
        assert_eq!(2, res.len());
        let first = res.get(0).unwrap();
        assert_eq!("zeep.xsd.asd", first);
        let second = res.get(1).unwrap();
        assert_eq!("zeep.xsd.dasd", second);
    }

    #[test]
    fn test_extract_import_state_machine_f_6() {
        let line = r###"
from zeep.loader import (
    absolute_location,
    is_relative_path,
    load_external,
    load_external_async,
)"###;
        let res = extract_import(line);
        assert_eq!(1, res.len());
        let res = res.get(0).unwrap();
        assert_eq!(4, res.len());
        let item = res.get(0).unwrap();
        assert_eq!("zeep.loader.absolute_location", item);
        let item = res.get(1).unwrap();
        assert_eq!("zeep.loader.is_relative_path", item);
        let item = res.get(2).unwrap();
        assert_eq!("zeep.loader.load_external", item);
        let item = res.get(3).unwrap();
        assert_eq!("zeep.loader.load_external_async", item);
    }
}
