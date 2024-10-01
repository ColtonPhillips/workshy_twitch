// src/main.rs

// Import modules
mod game_list;
mod steam_api;
mod twitch_api;

use dotenvy::dotenv;
use std::{collections::HashSet, env};
use steam_api::get_steam_library;
use twitch_api::{fetch_all_game_data, get_total_viewers};

fn main() {
    // mise en scène
    // Load environment variables from .env file
    // Retrieve credentials from environment variables
    // Get all of the custom games and steam games
    // and create a "main list" of the games we shall search for
    // Get OAuth token for Twitch
    dotenv().ok();
    let Secrets {
        steam_api_key,
        steam_id,
        twitch_client_id,
        twitch_client_secret,
    } = get_secrets();

    let custom_games = crate::game_list::read_game_list("custom_games.txt").unwrap_or_default();
    let steam_games_library = get_steam_library(&steam_api_key, &steam_id).unwrap_or_default();
    let mut searchable_games = vec![];
    searchable_games.extend(steam_games_library);

    let twitch_token = twitch_api::get_twitch_token(&twitch_client_id, &twitch_client_secret);

    // Search and Destroy
    // then fetch and store a hashmap of for each game's stats
    // Sort the game data by total viewers in descending order
    // Read the games (from a file) that don't get views into a hash-set,
    // Add in any new games that don't get views, and then save over the file
    // And print out the statistics

    let mut no_viewers_games: HashSet<String> = game_list::read_game_list("no_viewers_games.txt")
        .unwrap_or_default()
        .into_iter()
        .collect();

    if cfg!(feature = "quiet") {
        searchable_games.retain(|game| !no_viewers_games.contains(game));
    }

    searchable_games.extend(custom_games);

    let mut game_datas = fetch_all_game_data(searchable_games, &twitch_token, &twitch_client_id);
    for game_data in game_datas.iter_mut() {
        let total_viewers = get_total_viewers(
            game_data["id"].to_string(),
            &twitch_token,
            &twitch_client_id,
        );

        game_data.insert("viewers".to_string(), total_viewers.to_string());
        if total_viewers == 0 {
            no_viewers_games.insert(game_data["name"].clone());
        }
    }

    game_datas.sort_by(|a, b| {
        // Convert viewer counts from strings to u32
        let b32: u32 = b["viewers"].parse().unwrap_or(0);
        let a32: u32 = a["viewers"].parse().unwrap_or(0);
        b32.cmp(&a32) // Sort in descending order
    });

    game_list::write_over_game_list("no_viewers_games.txt", no_viewers_games);

    println!("Title | Live Viewers");
    for game_data in game_datas {
        let total_viewers = get_total_viewers(
            game_data["id"].to_string(),
            &twitch_token,
            &twitch_client_id,
        );
        if total_viewers > 0 {
            println!("{} | {}", game_data["name"], total_viewers);
        }
    }
}

struct Secrets {
    steam_api_key: String,
    steam_id: String,
    twitch_client_id: String,
    twitch_client_secret: String,
}

fn get_secrets() -> Secrets {
    let steam_api_key =
        env::var("STEAM_API_KEY").expect("Missing STEAM_API_KEY environment variable");

    let steam_id = env::var("STEAM_ID").expect("Missing STEAM_ID environment variable");

    let twitch_client_id =
        env::var("TWITCH_CLIENT_ID").expect("Missing TWITCH_CLIENT_ID environment variable");

    let twitch_client_secret = env::var("TWITCH_CLIENT_SECRET")
        .expect("Missing TWITCH_CLIENT_SECRET environment variable");

    Secrets {
        steam_api_key,
        steam_id,
        twitch_client_id,
        twitch_client_secret,
    }
}
