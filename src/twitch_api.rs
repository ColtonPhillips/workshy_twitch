// src/twitch_api.rs
use reqwest::{blocking::Client, Error};
use serde_json::Value;

/// Retrieves an OAuth token from the Twitch API using client credentials
pub fn get_twitch_token(client_id: &str, client_secret: &str) -> String {
    let client = Client::new();
    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .query(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .unwrap()
        .json::<Value>()
        .unwrap();

    res["access_token"].as_str().unwrap().to_string()
}

/// Fetches the popularity (viewer count) of a game from the Twitch API
pub fn get_game_data(game_name: &str, token: &str, client_id: &str) -> Result<Value, Error> {
    let client = Client::new();
    client
        .get("https://api.twitch.tv/helix/games")
        .query(&[("name", game_name)])
        .header("Authorization", format!("Bearer {}", token))
        .header("Client-Id", client_id)
        .send()
        .unwrap()
        .json::<Value>()
    // .unwrap()
}

use std::collections::{HashMap, HashSet};

// Assuming get_game_data returns a Result<HashMap<String, String>, Error>
// or something similar where game data is returned as a map (e.g., game name, viewer count)
pub fn fetch_all_game_data(
    games: HashSet<String>,
    token: &str,
    client_id: &str,
) -> Vec<HashMap<String, String>> {
    let mut game_data_list: Vec<HashMap<String, String>> = Vec::new();

    // Iterate over each game and fetch its data on Twitch
    for game in games {
        match get_game_data(&game, &token, &client_id) {
            Ok(game_data) => {
                let mut game_info = HashMap::new();
                game_info.insert("name".to_string(), game.to_string());

                game_info.insert(
                    "id".to_string(),
                    game_data["data"][0]["id"].to_string().replace("\"", ""),
                );

                game_info.insert(
                    "igdb_id".to_string(),
                    game_data["data"][0]["igdb_id"]
                        .to_string()
                        .replace("\"", ""),
                );

                game_data_list.push(game_info); // Add the game data to the list
            }
            _ => {
                println!("Game: {} not found or has no viewers", game);
            }
        }
    }

    game_data_list // Return the list of all the game data
}

/// Fetches the total viewer count for a game on Twitch by summing up the viewer counts of all live streams
pub fn get_total_viewers(game_id: String, token: &str, client_id: &str) -> u64 {
    let client = Client::new();
    let mut total_viewers: u64 = 0;
    let mut pagination_cursor: Option<String> = None;

    loop {
        // Build the request
        let mut request = client
            .get("https://api.twitch.tv/helix/streams")
            .query(&[("game_id", &game_id)])
            .header("Authorization", format!("Bearer {}", token))
            .header("Client-Id", client_id);

        // If there's a pagination cursor, include it in the query
        if let Some(cursor) = &pagination_cursor {
            request = request.query(&[("after", cursor)]);
        }

        // Send the request and parse the response
        let res = request.send().unwrap().json::<Value>().unwrap();

        // Extract streams data
        let streams = res["data"].as_array().unwrap();

        // Add up viewer counts for the current batch of streams
        total_viewers += streams
            .iter()
            .map(|stream| stream["viewer_count"].as_u64().unwrap_or(0))
            .sum::<u64>();

        // Check if there's more data to paginate through
        if let Some(cursor) = res["pagination"]["cursor"].as_str() {
            pagination_cursor = Some(cursor.to_string());
        } else {
            break;
        }
    }

    total_viewers
}
