// src/game_list.rs
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

/// Reads the game list from the specified file
pub fn read_game_list(file_path: &str) -> Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut games = Vec::new();
    for line in reader.lines() {
        games.push(line?);
    }

    Ok(games)
}
