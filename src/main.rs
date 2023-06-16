use std::collections::{HashSet, HashMap};
use std::env::VarError;
use std::io::BufRead;
use std::process::Command;
use std::{env, io};
use std::path::Path;
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

fn get_uptime() -> Result<String, io::Error>  {
    let output = Command::new("uptime").arg("-p").output()?.stdout;
    Ok(String::from_utf8_lossy(&output).trim_end().to_string())
}

fn get_distro() -> Result<String, io::Error> {
    let os_file_path = Path::new("/etc/os-release");
    let file = File::open(os_file_path)?;
    for maybe_line in io::BufReader::new(file).lines() {
        let line = maybe_line?;
        if line.starts_with("NAME") {
            return Ok(line[6..line.len()-1].to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "Failed to read os-release"))
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

fn print_output(print_logo: bool, least_used: (&String, &i32), most_used: (&String, &i32), fetches_installed: String, distro: String, uptime: String) {
    let logo = "\
    ######## ######## ########  ######  ##     ##\n\
    ##       ##          ##    ##    ## ##     ##\n\
    ##       ##          ##    ##       ##     ##\n\
    ######   ######      ##    ##       #########\n\
    ##       ##          ##    ##       ##     ##\n\
    ##       ##          ##    ##    ## ##     ##\n\
    ##       ########    ##     ######  ##     ##\n";
    let mut logo_lines = logo.lines();
    macro_rules! maybe_logo {
        () => {
            if print_logo {logo_lines.next().unwrap()} else {""}
        };
    }
    println!("{}         Least used fetch: {} - used {} times", maybe_logo!(), least_used.0, least_used.1);
    println!("{}         Most used fetch: {} - used {} times", maybe_logo!(), most_used.0, most_used.1);
    println!("{}         Fetches installed: {}", maybe_logo!(), fetches_installed);
    println!("{}         System has been {}", maybe_logo!(), uptime);
    println!("{}         ", maybe_logo!());
    println!("{}         I use {} btw", maybe_logo!(), distro);
    println!("{}         ", maybe_logo!());
    println!();
    println!("    `echo '{}' >> ~/.bashrc`", most_used.0);
    println!();
}

fn main() {
    let paths = get_paths().expect("Failed to read PATH variable.");
    let found_fetches = find_fetches(paths).expect("Failed to read path {} : {}");
    let mut fetch_history = get_fetch_history().expect("Failed to read fetch history");
    for fetch in &found_fetches {
        if !fetch_history.contains_key(fetch) {
            fetch_history.insert(fetch.to_string(), 0);
        }
    }
    let least_used_fetch = fetch_history.iter().min_by(|(_k1, v1), (_k2, v2)| v1.cmp(v2));
    let most_used_fetch = fetch_history.iter().max_by(|(_k1, v1), (_k2, v2)| v1.cmp(v2));
    let distro = get_distro().expect("Failed to determine distro");
    let uptime = get_uptime().expect("Failed to get uptime");
    let num_fetches_installed = found_fetches.len();
    let fetches_installed_str = if num_fetches_installed > 4 {num_fetches_installed.to_string()} else {Vec::from_iter(found_fetches).join(", ")};
    print_output(true, least_used_fetch.unwrap(), most_used_fetch.unwrap(), fetches_installed_str, distro, uptime);

}
