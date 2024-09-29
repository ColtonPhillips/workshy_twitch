// src/main.rs

// Import modules
mod game_list;
mod steam_api;
mod twitch_api;

use dotenvy::dotenv;
use std::env;
use steam_api::get_owned_games;
use twitch_api::{fetch_all_game_data, get_total_viewers};

fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Retrieve credentials from environment variables
    let steam_api_key =
        env::var("STEAM_API_KEY").expect("Missing STEAM_API_KEY environment variable");
    let steam_id = env::var("STEAM_ID").expect("Missing STEAM_ID environment variable");

    // Attempt to get the owned games
    let games_library = match get_owned_games(&steam_api_key, &steam_id) {
        Ok(steam_library) => {
            // Print the list of games
            for game in steam_library.clone() {
                println!("{}", game);
            }
            steam_library
        }
        Err(e) => {
            eprintln!("Failed to retrieve games: {}", e);
            Vec::new()
        }
    };

    let mut searchable_games = crate::game_list::read_game_list("games.txt").unwrap_or_default();
    searchable_games.extend(games_library);

    // Retrieve credentials from environment variables
    let twitch_client_id =
        env::var("TWITCH_CLIENT_ID").expect("Missing TWITCH_CLIENT_ID environment variable");
    let twitch_client_secret = env::var("TWITCH_CLIENT_SECRET")
        .expect("Missing TWITCH_CLIENT_SECRET environment variable");

    // Get OAuth token for Twitch
    let token = twitch_api::get_twitch_token(&twitch_client_id, &twitch_client_secret);

    let mut game_datas = fetch_all_game_data(searchable_games, &token, &twitch_client_id);

    for game_data in game_datas.iter_mut() {
        let total_viewers =
            get_total_viewers(game_data["id"].to_string(), &token, &twitch_client_id);
        game_data.insert("viewers".to_string(), total_viewers.to_string());
    }

    // Sort the game data by total viewers in descending order
    game_datas.sort_by(|a, b| {
        // Convert viewer counts from strings to u32
        let b32: u32 = b["viewers"].parse().unwrap_or(0);
        let a32: u32 = a["viewers"].parse().unwrap_or(0);
        b32.cmp(&a32) // Sort in descending order
    });

    println!("Title | Live Viewers");
    for game_data in game_datas {
        let total_viewers =
            get_total_viewers(game_data["id"].to_string(), &token, &twitch_client_id);
        if total_viewers > 0 {
            println!("{} | {}", game_data["name"], total_viewers);
        }
    }
}
