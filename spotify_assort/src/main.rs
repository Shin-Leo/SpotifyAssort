#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use serde_json::json;
use serde_json::Value;

use rocket::Build;
use rspotify::clients::mutex::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use getrandom::getrandom;
use rocket::http::Cookie;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Rocket;
use rocket_dyn_templates::Template;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};

use std::fs;
use std::{env, path::PathBuf};

#[derive(Debug, Responder)]
pub enum AppResponse {
    Template(Template),
    Redirect(Redirect),
    Json(Value),
}

#[derive(Serialize)]
struct PlaylistDetail {
    name: String,
    cover: String,
}

const CACHE_PATH: &str = ".spotify_cache/.spotify_token_cache.json";

fn generate_random_uuid(length: usize) -> String {
    let alphanum: &[u8] =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
    let mut buf = vec![0u8; length];
    getrandom(&mut buf).unwrap();
    let range = alphanum.len();

    buf.iter()
        .map(|byte| alphanum[*byte as usize % range] as char)
        .collect()
}

fn get_cache_path(cookies: &CookieJar) -> PathBuf {
    let project_dir_path = env::current_dir().unwrap();
    let mut cache_path = project_dir_path;
    cache_path.push(CACHE_PATH);
    // cache_path.push(cookies.get("uuid").unwrap().value());
    cache_path
}

fn create_cache_path_if_absent(cookies: &CookieJar) -> PathBuf {
    let cache_path = get_cache_path(cookies);
    if !cache_path.exists() {
        let mut path = cache_path.clone();
        path.pop();
        fs::create_dir_all(path).unwrap();
    }
    cache_path
}

fn remove_cache_path(cookies: &CookieJar<'_>) {
    let cache_path = get_cache_path(&cookies);
    if cache_path.exists() {
        fs::remove_file(cache_path).unwrap()
    }
    cookies.remove(Cookie::named("uuid"))
}

fn check_cache_path_exists(cookies: &CookieJar) -> bool {
    let cache_path = get_cache_path(cookies);
    cache_path.exists()
}

fn init_spotify(cookies: &CookieJar) -> AuthCodeSpotify {
    let config = Config {
        token_cached: true,
        cache_path: create_cache_path_if_absent(cookies),
        ..Default::default()
    };

    let creds = Credentials::from_env().unwrap();

    let oauth = OAuth::from_env(scopes!("user-read-playback-state")).unwrap();

    AuthCodeSpotify::with_config(creds, oauth, config)
}

#[get("/callback?<code>")]
fn callback(cookies: &CookieJar<'_>, code: String) -> AppResponse {
    let mut spotify = init_spotify(&cookies);
    match spotify.request_token(&code) {
        Ok(_) => {
            println!("Request user token successful");
            AppResponse::Redirect(Redirect::to("/"))
        }
        Err(err) => {
            println!("Failed to get user token {:?}", err.to_string());
            let mut context = HashMap::new();
            context.insert("err_msg", "Failed to get token!");
            AppResponse::Template(Template::render("error", context))
        }
    }
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> AppResponse {
    let mut context = HashMap::new();

    // The user is authenticated if their cookie is set and a cache exists for
    // them.

    let authenticated = cookies.get("uuid").is_some() || check_cache_path_exists(&cookies);
    if !authenticated {
        cookies.add(Cookie::new("uuid", generate_random_uuid(64)));

        let spotify = init_spotify(&cookies);
        let auth_url = spotify.get_authorize_url(true).unwrap();
        context.insert("auth_url", auth_url);
        return AppResponse::Template(Template::render("authorize", context));
    }
    let cache_path = get_cache_path(&cookies);
    let token = Token::from_cache(cache_path).unwrap();
    let spotify = AuthCodeSpotify::from_token(token);
    match spotify.me() {
        Ok(user_info) => {
            context.insert(
                "display_name",
                user_info
                    .display_name
                    .unwrap_or_else(|| String::from("Dear")),
            );
            AppResponse::Template(Template::render("index", context.clone()))
        }
        Err(err) => {
            context.insert("err_msg", format!("Failed for {}!", err));
            AppResponse::Template(Template::render("error", context))
        }
    }
}

#[get("/sign_out")]
fn sign_out(cookies: &CookieJar<'_>) -> AppResponse {
    remove_cache_path(cookies);
    AppResponse::Redirect(Redirect::to("/"))
}

#[get("/playlists")]
fn playlist(cookies: &CookieJar<'_>) -> AppResponse {
    let mut context = HashMap::new();
    let mut playlists_details = vec![];
    let mut spotify = init_spotify(&cookies);
    if !spotify.config.cache_path.exists() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    let token = spotify.read_token_cache(false).unwrap();

    spotify.token = Arc::new(Mutex::new(token));
    let playlists = spotify
        .current_user_playlists()
        .take(50)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    if playlists.is_empty() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    for playlist in playlists.into_iter() {
        let image = &playlist.images[0];
        let playlist_detail = PlaylistDetail {
            name: playlist.name,
            cover: image.url.clone(),
        };
        playlists_details.push(playlist_detail);
    }
    context.insert("playlists".to_string(), playlists_details);

    AppResponse::Template(Template::render("playlists", context))
}

#[get("/me")]
fn me(cookies: &CookieJar<'_>) -> AppResponse {
    let mut spotify = init_spotify(&cookies);
    if !spotify.config.cache_path.exists() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    spotify.token = Arc::new(Mutex::new(spotify.read_token_cache(false).unwrap()));
    match spotify.me() {
        Ok(user_info) => AppResponse::Json(json!(user_info)),
        Err(_) => AppResponse::Redirect(Redirect::to("/")),
    }
}
#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index, callback, sign_out, me, playlist])
        .attach(Template::fairing())
}

// #[tokio::main]
// async fn main() {
//     env_logger::init();

//     let creds = Credentials::from_env().unwrap();

//     let oauth = OAuth::from_env(scopes!("user-read-playback-state")).unwrap();

//     let mut spotify = AuthCodeSpotify::new(creds, oauth);

//     let url = spotify.get_authorize_url(false).unwrap();
//     spotify.prompt_for_token(&url).await.unwrap();
//     let user = spotify.me();
//     let user_id = user.await.unwrap().id;
//     let stream = spotify.user_playlists(&user_id);
//     pin_mut!(stream);
//     let mut playlists = HashMap::new();

//     while let Some(item) = stream.try_next().await.unwrap() {
//         let playlist_id = item.id;
//         let playlist_name = item.name;
//         playlists.insert(playlist_name, playlist_id);
//     }

//     let mut track_stream = spotify.playlist_items(
//         &playlists
//             .get("Japanese songs that i am extremely fond of")
//             .unwrap(),
//         None,
//         None,
//     );

//     while let Some(item) = track_stream.try_next().await.unwrap() {
//         let playable_item = item.track.unwrap();
//         match playable_item {
//             PlayableItem::Track(track) => println!("* {}", track.name),
//             PlayableItem::Episode(episode) => println!("* {}", episode.name),
//         }
//     }
// }
