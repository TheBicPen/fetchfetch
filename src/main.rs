use std::collections::{HashSet, HashMap};
use std::env::VarError;
use std::io::BufRead;
use std::{env, io};
use std::path::{Path};
use std::fs::{self, File};

use dirs::home_dir;

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

fn get_fetch_history() -> Result<HashMap<String, i32>, io::Error> {
    let home_dir_path = home_dir().ok_or(
        io::Error::new(io::ErrorKind::NotFound, "Unable to find home directory")
    );
    let bash_history_path = home_dir_path?.join(".bash_history");
    let history_file = File::open(bash_history_path)?;
    let mut history_fetch : HashMap<String, i32> = HashMap::new();
    for maybe_line in io::BufReader::new(history_file).lines() {
        let line = maybe_line?;
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() > 0 {
            if words[0].ends_with("fetch") {
                history_fetch.entry(words[0].to_string())
                    .and_modify(|x| *x += 1 )
                    .or_insert(1);
            }
        }
    }
    Ok(history_fetch)
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
    let fetch_history = get_fetch_history().expect("failed to read fetch history");
    for (name, count) in fetch_history {
        println!("{} {}", name, count);
    }

}
