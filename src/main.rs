// src/main.rs

// Import modules
mod game_list;
mod steam_api;
mod twitch_api;

use dotenvy::dotenv;
use std::{collections::HashSet, env};
use steam_api::get_steam_library;
use twitch_api::fetch_all_game_data;

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
    let mut searchable_games = HashSet::new();
    // searchable_games.extend(steam_games_library);

    let twitch_token = twitch_api::get_twitch_token(&twitch_client_id, &twitch_client_secret);

    // Search and Destroy
    // Find games tagged as being no viewer Losers
    // retain loser games from searchable if feature = "retain"
    // then extend to add my custom games
    // Fetch game data from Twitch, and save no_viewer games if feature = "losers"
    // Print searchable games, with zero-view games if feature = "verbose"

    let mut no_viewers_games: HashSet<String> = game_list::read_game_list("no_viewers_games.txt")
        .unwrap_or_default()
        .into_iter()
        .collect();

    if cfg!(feature = "retain") {
        searchable_games.retain(|game| !no_viewers_games.contains(game));
    }

    searchable_games.extend(custom_games);

    let mut game_datas =
        fetch_all_game_data(searchable_games.clone(), &twitch_token, &twitch_client_id);

    let x = crate::twitch_api::fetch_viewercounts(
        searchable_games.clone(),
        &twitch_token,
        &twitch_client_id,
    );
    println!("{:?}", x);

    for game_data in game_datas.iter_mut() {
        let total_viewers: u64 = x[&game_data["id"]];
        game_data.insert("viewers".to_string(), total_viewers.to_string());
        if total_viewers == 0 {
            no_viewers_games.insert(game_data["name"].clone());
        }
    }

    if cfg!(feature = "losers") {
        game_list::write_over_game_list("no_viewers_games.txt", no_viewers_games);
    }

    game_datas.sort_by(|a, b| {
        // Convert viewer counts from strings to u32
        let b32: u32 = b["viewers"].parse().unwrap_or(0);
        let a32: u32 = a["viewers"].parse().unwrap_or(0);
        b32.cmp(&a32) // Sort in descending order
    });

    println!("Title | Live Viewers");
    println!("====================");
    for game_data in game_datas {
        let total_viewers: u64 = game_data["viewers"].parse().unwrap_or(0);
        if total_viewers > 0 || cfg!(feature = "verbose") {
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
