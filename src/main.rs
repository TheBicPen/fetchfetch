use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::fs;
use std::process::exit;

fn main() {
    let paths = env::var("PATH").expect("PATH is undefined.");
    let mut found_fetches: HashSet<String> = HashSet::new();
    for path in paths.split(':') {
        let dir = Path::new(path);
        let entries_result = fs::read_dir(dir);
        match entries_result {
            Ok(entries) => {
                found_fetches.extend(entries
                    .filter_map(|x| x.ok())
                    .filter(|entry| entry.file_type().is_ok() && entry.file_type().unwrap().is_file())
                    .filter_map(|entry| {
                        let name = entry.file_name();
                        let name_str = name.to_str();
                        if name_str.is_some() && name_str.unwrap().ends_with("fetch") {
                            Some(String::from(name_str.unwrap()))
                        } else {
                            None
                        }
                    }));
            },
            Err(err) => {
                eprintln!("Failed to read path {} : {}", dir.display(), err);
                exit(1);
            }
        };
    }
    for fetch in found_fetches {
        println!("{}", fetch);
    }
}
