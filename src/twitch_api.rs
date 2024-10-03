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

use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
    thread,
};

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

pub fn fetch_viewercounts(
    games: HashSet<String>,
    token: &str,
    client_id: &str,
) -> HashMap<String, u64> {
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel();

    for game_id in games.clone() {
        let tx = tx.clone();
        let token = token.to_string();
        let client_id = client_id.to_string();
        let bambam = game_id.clone();
        let handle = thread::spawn(move || {
            let viewers_count = fetch_viewers_count_per_game(game_id, &token, &client_id);
            tx.send((bambam, viewers_count)).unwrap();
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Collect results from the channel
    let mut results = HashMap::new();
    for _ in 0..games.len() {
        if let Ok(result) = rx.recv() {
            println!("{:?}{:?}", result.0, result.1);
            results.insert(result.0, result.1);
        }
    }
    results
}

/// Fetches the total viewer count for a game on Twitch by summing up the viewer counts of all live streams.
pub fn fetch_viewers_count_per_game(game_id: String, token: &str, client_id: &str) -> u64 {
    let client = Client::new();
    let mut total_viewers = 0;
    let mut pagination_cursor: Option<String> = None;

    loop {
        let response = fetch_streams(&client, &game_id, token, client_id, &pagination_cursor);
        println!("{:?}", response);
        match response {
            Ok(res) => {
                total_viewers += extract_viewer_counts(&res);
                pagination_cursor = res["pagination"]["cursor"].as_str().map(|s| s.to_string());

                if pagination_cursor.is_none() {
                    println!("{:?}", res);
                    break;
                }
            }
            Err(err) => {
                println!("Error fetching streams: {}", err);
                break;
            }
        }
    }

    total_viewers
}
/// Fetch streams from Twitch API.
fn fetch_streams(
    client: &Client,
    game_id: &str,
    token: &str,
    client_id: &str,
    pagination_cursor: &Option<String>,
) -> Result<Value, reqwest::Error> {
    let mut request = client
        .get("https://api.twitch.tv/helix/streams")
        .query(&[("game_id", game_id)])
        .header("Authorization", format!("Bearer {}", token))
        .header("Client-Id", client_id);

    // Include pagination cursor if present
    if let Some(cursor) = pagination_cursor {
        request = request.query(&[("after", cursor)]);
    }

    // Send the request and parse the response
    request.send()?.json()
}

/// Extract viewer counts from the response.
fn extract_viewer_counts(res: &Value) -> u64 {
    match res["data"].as_array() {
        Some(streams) => streams
            .iter()
            .map(|stream| stream["viewer_count"].as_u64().unwrap_or(0))
            .sum(),
        None => {
            println!("Failed to retrieve streams. Response structure: {:?}", res);
            0
        }
    }
}
