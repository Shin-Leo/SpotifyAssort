extern crate reqwest;
extern crate tokio;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use std::fs;
extern crate env_logger;
use std::{collections::HashMap, env, path::PathBuf};

#[tokio::main]
async fn main() {
    env_logger::init();

    let creds = Credentials::from_env().unwrap();

    let scopes = scopes!(
        "user-top-read",
        "user-read-recently-played",
        "user-library-read",
        "user-read-currently-playing",
        "user-read-playback-state",
        "user-read-playback-position",
        "playlist-read-private",
        "user-library-modify",
        "user-modify-playback-state",
        "playlist-modify-public",
        "playlist-modify-private"
    );

    let oauth = OAuth::from_env(scopes).unwrap();

    let mut spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();

    let token = spotify.token.lock().await.unwrap();
    println!("Access token: {}", &token.as_ref().unwrap().access_token);
    println!(
        "Refresh token: {}",
        token.as_ref().unwrap().refresh_token.as_ref().unwrap()
    );
}
