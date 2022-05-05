use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use rspotify::{
    model::{FullTrack, PlayableItem, PlaylistItem},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    env_logger::init();

    let creds = Credentials::from_env().unwrap();

    let oauth = OAuth::from_env(scopes!("user-read-playback-state")).unwrap();

    let mut spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();
    let user = spotify.me();
    let user_id = user.await.unwrap().id;
    let stream = spotify.user_playlists(&user_id);
    pin_mut!(stream);
    let mut playlists = HashMap::new();

    while let Some(item) = stream.try_next().await.unwrap() {
        let playlist_id = item.id;
        let playlist_name = item.name;
        playlists.insert(playlist_name, playlist_id);
    }

    let mut track_stream = spotify.playlist_items(
        &playlists
            .get("Japanese songs that i am extremely fond of")
            .unwrap(),
        None,
        None,
    );

    while let Some(item) = track_stream.try_next().await.unwrap() {
        let playable_item = item.track.unwrap();
        match playable_item {
            PlayableItem::Track(track) => println!("* {}", track.name),
            PlayableItem::Episode(episode) => println!("* {}", episode.name),
        }
    }
}
