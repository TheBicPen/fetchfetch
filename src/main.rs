use std::collections::HashSet;
use std::env::VarError;
use std::{env, io};
use std::path::Path;
use std::fs;

fn find_fetches(paths: Vec<String>) -> Result<HashSet<String>, io::Error> {
    let mut found_fetches: HashSet<String> = HashSet::new();
    for path in paths {
        let dir = Path::new(&path);
        let entries_result = fs::read_dir(dir);
        match entries_result {
            Ok(entries) => {
                found_fetches.extend(entries
                    .filter_map(|x| x.ok())
                    .filter(|entry| entry.file_type().is_ok() && entry.file_type().unwrap().is_file())
                    .filter_map(|entry| {
                        let name = entry.file_name();
                        let name_str = name.to_str();
                        // ignore programs that don't end with "fetch", e.g. fetchyahoo
                        if name_str.is_some() && name_str.unwrap().ends_with("fetch") {
                            Some(String::from(name_str.unwrap()))
                        } else {
                            None
                        }
                    }));
            },
            Err(err) => {
                return Err(err);
            }
        };
    }
    Ok(found_fetches)
}

fn get_paths() -> Result<Vec<String>, VarError> {
    let paths = env::var("PATH")?;
    Ok(paths.split(":").map(str::to_owned).collect())
}

fn main() {
    let paths = get_paths().expect("Failed to read PATH variable.");
    let found_fetches = find_fetches(paths).expect("Failed to read path {} : {}");
    for fetch in found_fetches {
        println!("{}", fetch);
    }
}
