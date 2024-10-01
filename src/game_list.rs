use std::collections::HashSet;
// src/game_list.rs
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Result, Write};

/// Reads the game list from the specified file
pub fn read_game_list(file_path: &str) -> Result<HashSet<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut games = HashSet::new();
    for line in reader.lines() {
        games.insert(line?);
    }

    Ok(games)
}

pub fn write_over_game_list(file_path: &str, games_set: HashSet<String>) {
    // Open the file and truncate its contents
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)
        .unwrap();

    for game_name in games_set {
        writeln!(&mut file, "{}", game_name.as_str()).unwrap();
    }
}
