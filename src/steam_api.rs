use reqwest::blocking::Client; // Use blocking client for synchronous requests
use serde_json::Value; // Import Value from serde_json
use std::error::Error;

/// Fetches the list of owned game names for a given Steam user
pub fn get_steam_library(api_key: &str, steam_id: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::new();
    let url = "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/";

    // Send request to Steam API
    let res = client
        .get(url)
        .query(&[
            ("key", api_key),
            ("steamid", steam_id),
            ("include_appinfo", "true"),           // Include game names
            ("include_played_free_games", "true"), // Include free games
        ])
        .send()?;

    // Parse the JSON response
    let json_data: Value = serde_json::from_str(&res.text()?)?;

    // Initialize a vector to hold game names
    let mut game_names = Vec::new();

    // Check if the "games" field exists and is an array
    if let Some(games_array) = json_data["response"]["games"].as_array() {
        // Iterate through the games array and extract game names
        for game in games_array {
            if let Some(name) = game["name"].as_str() {
                game_names.push(name.to_string()); // Add the game name to the vector
            }
        }
    } else {
        eprintln!("No games found or the response structure is unexpected.");
    }

    Ok(game_names)
}
